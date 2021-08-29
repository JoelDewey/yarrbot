use crate::yarrbot_api_error::YarrbotApiError;
use actix_web::dev::Payload;
use actix_web::web::{block, Data};
use actix_web::{Error, FromRequest, HttpRequest};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;
use yarrbot_common::{crypto::verify, short_id::ShortId};
use yarrbot_db::actions::webhook_actions::WebhookActions;
use yarrbot_db::models::Webhook;
use yarrbot_db::DbPool;

pub struct WebhookInfo {
    pub webhook: Webhook,
}

struct WebhookAuth {
    pub user: String,
    pub password: String,
}

fn get_webhook_auth(header: &str) -> Option<WebhookAuth> {
    let mut pieces = header.split_ascii_whitespace();
    let precursor = pieces.next().unwrap_or("");
    if precursor.to_ascii_lowercase() != "basic" {
        return None;
    }

    let auth = match base64::decode(pieces.next().unwrap_or("")) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        _ => return None,
    };
    let split = match auth.split_once(':') {
        Some(s) => s,
        _ => return None,
    };
    Some(WebhookAuth {
        user: String::from(split.0),
        password: String::from(split.1),
    })
}

fn is_authorized_for_webhook(auth: WebhookAuth, webhook: &Webhook) -> bool {
    webhook.username == auth.user && verify(auth.password.as_str(), webhook.password.as_slice())
}

impl FromRequest for WebhookInfo {
    type Config = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let pool = req.app_data::<Data<DbPool>>().unwrap().clone();
        let webhook_id = String::from(req.match_info().get("webhook_id").unwrap());
        let auth_header = match req.headers().get("Authorization") {
            Some(h) => String::from(h.to_str().unwrap_or("")),
            None => String::from(""),
        };

        debug!("Processing webhook request with ID {}.", &webhook_id);
        Box::pin(async move {
            let conn = match pool.get_ref().get() {
                Ok(c) => c,
                Err(e) => {
                    error!("Encountered an error while retrieving connection: {}", e);
                    return Err(YarrbotApiError::internal_server_error().into());
                }
            };
            debug!(
                "Attempting to retrieve login information for webhook {}.",
                &webhook_id
            );
            let webhook_auth = match get_webhook_auth(&auth_header) {
                Some(a) => a,
                _ => return Err(YarrbotApiError::unauthorized().into()),
            };
            debug!("Converting webhook {} back into a UUID.", &webhook_id);
            let uuid = match Uuid::from_short_id(&webhook_id) {
                Ok(u) => u,
                _ => return Err(YarrbotApiError::not_found().into()),
            };

            debug!("Getting webhook {} from the database.", &webhook_id);
            let optional_webhook = match block(move || Webhook::try_get(&conn, &uuid)).await {
                Ok(Ok(w)) => w,
                Err(e) => {
                    error!(
                        "Failed to retrieve webhook with ID \"{}\" from the database: {:?}",
                        uuid, e
                    );
                    return Err(YarrbotApiError::internal_server_error().into());
                }
                Ok(Err(e)) => {
                    error!(
                        "Failed to retrieve webhook with ID \"{}\" from the database: {:?}",
                        uuid, e
                    );
                    return Err(YarrbotApiError::internal_server_error().into());
                }
            };
            let webhook = match optional_webhook {
                Some(w) => {
                    debug!("Webhook {} found in database.", &webhook_id);
                    w
                }
                _ => {
                    debug!("Failed to find webhook {} in the database.", &webhook_id);
                    return Err(YarrbotApiError::not_found().into());
                }
            };

            if is_authorized_for_webhook(webhook_auth, &webhook) {
                info!(
                    "Webhook {} ({}) retrieved and authorized.",
                    &webhook_id, &webhook.id
                );
                Ok(WebhookInfo { webhook })
            } else {
                debug!(
                    "Current request is not authorized for webhook {}.",
                    &webhook_id
                );
                Err(YarrbotApiError::unauthorized().into())
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::test;

    #[test]
    fn get_webhook_auth_returns_none_given_no_auth_header() {
        // Arrange
        let req = test::TestRequest::default().to_http_request();

        // Act
        let actual_wrapped = get_webhook_auth("");

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_none_given_value_not_str() {
        // Arrange
        let input = "42";

        // Act
        let actual_wrapped = get_webhook_auth(input);

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_none_given_malformed_auth_header() {
        // Act
        let actual_wrapped = get_webhook_auth("deadbeef");

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_none_given_not_basic_auth() {
        // Act
        let actual_wrapped = get_webhook_auth("Digest dXNlcjpwYXNzd29yZA==");

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_none_given_not_base64() {
        // Act
        let actual_wrapped = get_webhook_auth("Basic notBase64");

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_none_given_str_without_colon_delimiter() {
        // Arrange
        let input_b64 = "dXNlciBwYXNzd29yZA=="; // user password

        // Act
        let actual_wrapped = get_webhook_auth(&format!("Basic {}", input_b64));

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_webhookauth_given_valid_auth_header() {
        // Arrange
        let expected_user = "user";
        let expected_password = "password";
        let input_b64 = "dXNlcjpwYXNzd29yZA=="; // user:password
                                                // Act
        let actual_wrapped = get_webhook_auth(&format!("Basic {}", input_b64));

        // Assert
        assert!(actual_wrapped.is_some());
        let actual = actual_wrapped.unwrap();
        assert_eq!(expected_user, actual.user);
        assert_eq!(expected_password, actual.password);
    }
}
