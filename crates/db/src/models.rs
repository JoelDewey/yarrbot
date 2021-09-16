//! Defines the models that represent database objects belonging to Yarrbot.

use crate::enums::*;
use crate::schema::*;
use diesel::Queryable;
use uuid::Uuid;

/// Some chat room user that can manage [Webhook] endpoints for one of the *arr services to push to.
#[derive(Queryable, Identifiable, Debug, Clone)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,

    /// The user's unique ID on Matrix.
    pub service_username: String,

    /// The user's role with the bot (see [UserRole]).
    pub user_role: UserRole,
}

/// Model specifically for creating a new [User].
#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    id: Uuid,

    /// The user's unique ID on Matrix.
    pub service_username: String,

    /// The user's role with the bot (see [UserRole]).
    pub user_role: UserRole,
}

impl NewUser {
    pub fn new(username: &str, user_role: Option<UserRole>) -> NewUser {
        let role = user_role.unwrap_or(UserRole::Administrator);
        NewUser {
            id: Uuid::new_v4(),
            service_username: String::from(username),
            user_role: role,
        }
    }
}

impl From<NewUser> for User {
    fn from(user: NewUser) -> Self {
        User {
            id: user.id,
            service_username: user.service_username,
            user_role: user.user_role,
        }
    }
}

/// A definition of a webhook endpoint URL for one of the *arr services to push messages to.
#[derive(Queryable, Identifiable, Associations, Clone)]
#[belongs_to(User)]
#[table_name = "webhooks"]
pub struct Webhook {
    pub id: Uuid,

    /// The username required to access the webhook endpoint.
    pub username: String,

    /// The Argon2id hash+salt representing the password required to access the webhook endpoint.
    pub password: Vec<u8>,

    /// The [User] that owns this [Webhook].
    pub user_id: Uuid,
}

#[derive(Insertable, Associations)]
#[belongs_to(User)]
#[table_name = "webhooks"]
pub struct NewWebhook {
    id: Uuid,

    /// The username required to access the webhook endpoint.
    pub username: String,

    /// The Argon2id hash+salt representing the password required to access the webhook endpoint.
    pub password: Vec<u8>,

    /// The [User] that owns this [Webhook].
    user_id: Uuid,
}

impl NewWebhook {
    pub fn new(username: &str, password: Vec<u8>, user: &User) -> NewWebhook {
        NewWebhook {
            id: Uuid::new_v4(),
            username: String::from(username),
            password,
            user_id: user.id,
        }
    }
}

impl From<NewWebhook> for Webhook {
    fn from(webhook: NewWebhook) -> Self {
        Webhook {
            id: webhook.id,
            username: webhook.username,
            password: webhook.password,
            user_id: webhook.user_id,
        }
    }
}

/// A definition of a room on Yarrbot's homeserver to post messages received from
/// a [Webhook] to.
#[derive(Queryable, Identifiable, Associations, Clone)]
#[belongs_to(Webhook)]
#[table_name = "matrix_rooms"]
pub struct MatrixRoom {
    pub id: Uuid,

    /// The room ID of the Matrix room (_not_ an alias).
    pub room_id: String,

    /// The webhook to get message data from for messages posted in this room.
    pub webhook_id: Uuid,
}

#[derive(Insertable)]
#[table_name = "matrix_rooms"]
pub struct NewMatrixRoom {
    id: Uuid,

    /// The room ID of the Matrix room (_not_ an alias).
    pub room_id: String,

    /// The webhook to get message data from for messages posted in this room.
    pub webhook_id: Uuid,
}

impl NewMatrixRoom {
    pub fn new(room_id: &str, webhook: &Webhook) -> NewMatrixRoom {
        NewMatrixRoom {
            id: Uuid::new_v4(),
            room_id: String::from(room_id),
            webhook_id: webhook.id,
        }
    }
}

impl From<NewMatrixRoom> for MatrixRoom {
    fn from(room: NewMatrixRoom) -> Self {
        MatrixRoom {
            id: room.id,
            room_id: room.room_id,
            webhook_id: room.webhook_id,
        }
    }
}
