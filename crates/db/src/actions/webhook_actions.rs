use crate::models::{MatrixRoom, NewWebhook, Webhook};
use crate::schema::webhooks;
use crate::schema::webhooks::dsl::id;
use crate::DbPoolConnection;
use diesel::prelude::*;
use diesel::{delete, insert_into, result::Error};
use uuid::Uuid;

pub trait WebhookActions {
    /// Create a new [Webhook] and return the result.
    ///
    /// # Remarks
    ///
    /// This function takes ownership of the [NewWebhook].
    fn create_webhook(
        connection: &DbPoolConnection,
        new_webhook: NewWebhook,
    ) -> Result<Webhook, diesel::result::Error>;

    /// Get a [Webhook} by its ID.
    fn try_get(
        connection: &DbPoolConnection,
        identifier: &Uuid,
    ) -> Result<Option<Webhook>, diesel::result::Error>;

    /// Delete a [Webhook].
    fn delete(&self, connection: &DbPoolConnection) -> Result<(), diesel::result::Error>;

    /// Retrieve this [Webhook]'s list of [MatrixRoom] rows.
    fn get_rooms(
        &self,
        connection: &DbPoolConnection,
    ) -> Result<Vec<MatrixRoom>, diesel::result::Error>;
}

impl WebhookActions for Webhook {
    fn create_webhook(
        connection: &DbPoolConnection,
        new_webhook: NewWebhook,
    ) -> Result<Webhook, Error> {
        insert_into(webhooks::table)
            .values(&new_webhook)
            .execute(connection)?;
        Ok(Webhook::from(new_webhook))
    }

    fn try_get(
        connection: &DbPoolConnection,
        identifier: &Uuid,
    ) -> Result<Option<Webhook>, diesel::result::Error> {
        webhooks::table
            .filter(id.eq(identifier))
            .first(connection)
            .optional()
    }

    fn delete(&self, connection: &DbPoolConnection) -> Result<(), diesel::result::Error> {
        delete(webhooks::table)
            .filter(id.eq(self.id))
            .execute(connection)?;
        Ok(())
    }

    fn get_rooms(&self, connection: &DbPoolConnection) -> Result<Vec<MatrixRoom>, Error> {
        let results = MatrixRoom::belonging_to(self).load::<MatrixRoom>(connection)?;
        Ok(results)
    }
}
