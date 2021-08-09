use crate::yarrbot_api_error::YarrbotApiError;
use actix_web::dev::Payload;
use actix_web::http::HeaderValue;
use actix_web::web::Data;
use actix_web::{Error, FromRequest, HttpRequest};
use futures_util::future::{err, ok, Ready};
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

fn get_webhook_auth(req: &HttpRequest) -> Option<WebhookAuth> {
    let header: &HeaderValue = match req.headers().get("Authorization") {
        Some(h) => h,
        _ => return None,
    };
    let value = match header.to_str() {
        Ok(v) => v,
        _ => return None,
    };
    let mut pieces = value.split_ascii_whitespace();
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
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let webhook_id = match req.match_info().get("webhook_id") {
            Some(w) => w,
            _ => {
                error!("The Webhook extractor was called on an endpoint that doesn't have a webhook_id.");
                return err(YarrbotApiError::internal_server_error().into());
            }
        };
        let webhook_auth = match get_webhook_auth(req) {
            Some(a) => a,
            _ => return err(YarrbotApiError::unauthorized().into()),
        };
        let uuid = match Uuid::from_short_id(webhook_id) {
            Ok(u) => u,
            _ => return err(YarrbotApiError::not_found().into()),
        };

        let pool = match req.app_data::<Data<DbPool>>() {
            Some(p) => p,
            None => {
                error!("Connection pool was missing while processing Webhook extractor");
                return err(YarrbotApiError::internal_server_error().into());
            }
        };
        let conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                error!("Encountered an error while retrieving connection: {}", e);
                return err(YarrbotApiError::internal_server_error().into());
            }
        };

        let optional_webhook = match Webhook::try_get(&conn, &uuid) {
            Ok(w) => w,
            _ => {
                error!(
                    "Failed to retrieve webhook with ID \"{}\" from the database.",
                    uuid
                );
                return err(YarrbotApiError::internal_server_error().into());
            }
        };
        let webhook = match optional_webhook {
            Some(w) => w,
            _ => return err(YarrbotApiError::not_found().into()),
        };

        if is_authorized_for_webhook(webhook_auth, &webhook) {
            ok(WebhookInfo { webhook })
        } else {
            err(YarrbotApiError::unauthorized().into())
        }
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
        let actual_wrapped = get_webhook_auth(&req);

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_none_given_value_not_str() {
        // Arrange
        let input: u64 = 42;
        let req = test::TestRequest::with_header("authorization", input).to_http_request();

        // Act
        let actual_wrapped = get_webhook_auth(&req);

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_none_given_malformed_auth_header() {
        // Arrange
        let req = test::TestRequest::with_header("authorization", "deadbeef").to_http_request();

        // Act
        let actual_wrapped = get_webhook_auth(&req);

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_none_given_not_basic_auth() {
        // Arrange
        let req = test::TestRequest::with_header("authorization", "Digest dXNlcjpwYXNzd29yZA==")
            .to_http_request();

        // Act
        let actual_wrapped = get_webhook_auth(&req);

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_none_given_not_base64() {
        // Arrange
        let req = test::TestRequest::with_header("authorization", "Digest dXNlcjpwYXNzd29yZA==")
            .to_http_request();

        // Act
        let actual_wrapped = get_webhook_auth(&req);

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_none_given_str_without_colon_delimiter() {
        // Arrange
        let input_b64 = "dXNlciBwYXNzd29yZA=="; // user password
        let req = test::TestRequest::with_header("authorization", format!("Basic {}", input_b64))
            .to_http_request();

        // Act
        let actual_wrapped = get_webhook_auth(&req);

        // Assert
        assert!(actual_wrapped.is_none());
    }

    #[test]
    fn get_webhook_auth_returns_webhookauth_given_valid_auth_header() {
        // Arrange
        let expected_user = "user";
        let expected_password = "password";
        let input_b64 = "dXNlcjpwYXNzd29yZA=="; // user:password
        let req = test::TestRequest::with_header("authorization", format!("Basic {}", input_b64))
            .to_http_request();

        // Act
        let actual_wrapped = get_webhook_auth(&req);

        // Assert
        assert!(actual_wrapped.is_some());
        let actual = actual_wrapped.unwrap();
        assert_eq!(expected_user, actual.user);
        assert_eq!(expected_password, actual.password);
    }
}
