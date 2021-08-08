use crate::models::{MatrixRoom, NewMatrixRoom};
use crate::schema::matrix_rooms::dsl::*;
use crate::DbPoolConnection;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{delete, insert_into};

pub trait MatrixRoomActions {
    /// Create a new [MatrixRoom] in the database and return the result.
    ///
    /// # Remarks
    ///
    /// This function takes ownership of the [NewMatrixRoom].
    fn create_room(
        connection: &DbPoolConnection,
        new_room: NewMatrixRoom,
    ) -> Result<MatrixRoom, diesel::result::Error>;

    /// Delete a [MatrixRoom].
    fn delete(&self, connection: &DbPoolConnection) -> Result<(), diesel::result::Error>;
}

impl MatrixRoomActions for MatrixRoom {
    fn create_room(
        connection: &DbPoolConnection,
        new_room: NewMatrixRoom,
    ) -> Result<MatrixRoom, Error> {
        insert_into(matrix_rooms)
            .values(&new_room)
            .execute(connection)?;
        Ok(MatrixRoom::from(new_room))
    }

    fn delete(&self, connection: &DbPoolConnection) -> Result<(), Error> {
        delete(matrix_rooms)
            .filter(id.eq(self.id))
            .execute(connection)?;
        Ok(())
    }
}
