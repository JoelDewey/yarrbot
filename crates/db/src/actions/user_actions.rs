use crate::models::{NewUser, User};
use crate::schema::users::dsl::*;
use crate::DbPoolConnection;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{delete, insert_into};

pub trait UserActions {
    /// Create a new [User] in the database and returns the result.
    ///
    /// # Remarks
    ///
    /// This function takes ownership of the [NewUser].
    fn create_user(
        connection: &DbPoolConnection,
        new_user: NewUser,
    ) -> Result<User, diesel::result::Error>;

    /// Retrieve a [User] by their username if they exist.
    fn try_get_by_username(
        connection: &DbPoolConnection,
        username: &str,
    ) -> Result<Option<User>, diesel::result::Error>;

    /// Delete a [User].
    fn delete(&self, connection: &DbPoolConnection) -> Result<(), diesel::result::Error>;

    /// Check if any [User] records exist in the database.
    fn any(connection: &DbPoolConnection) -> Result<bool, diesel::result::Error>;
}

impl UserActions for User {
    fn create_user(
        connection: &DbPoolConnection,
        new_user: NewUser,
    ) -> Result<User, diesel::result::Error> {
        insert_into(users).values(&new_user).execute(connection)?;
        Ok(User::from(new_user))
    }

    fn try_get_by_username(
        connection: &DbPoolConnection,
        username: &str,
    ) -> Result<Option<User>, diesel::result::Error> {
        users
            .filter(service_username.eq(username))
            .first(connection)
            .optional()
    }

    fn delete(&self, connection: &DbPoolConnection) -> Result<(), diesel::result::Error> {
        delete(users).filter(id.eq(self.id)).execute(connection)?;
        Ok(())
    }

    fn any(connection: &DbPoolConnection) -> Result<bool, Error> {
        let count: i64 = users.count().get_result(connection)?;
        Ok(count > 0)
    }
}
