//! Traits and utilities for sending messages to a Matrix server.

use anyhow::Result;

/// Formats information for display in Matrix.
pub trait MatrixMessage {
    /// A [String] containing HTML to send to a Matrix room.
    ///
    /// # Remarks
    //
    // The output of this method should be equivalent to what is returned by
    // [get_plain].
    fn get_html(&self) -> Result<String>;

    /// A [String] containing a plain text message to send to a Matrix room.
    ///
    /// # Remarks
    ///
    /// The output of this method should be equivalent to what is returned by
    /// [get_html].
    fn get_plain(&self) -> Result<String>;
}
