//! Commands that administrators can send to Yarrbot.

pub mod help_command;
pub mod ping_command;
pub mod sourcecode_command;
pub mod webhook;
pub mod webhook_command;

#[derive(Debug)]
pub struct CommandMetadata {
    pub user: String,
    pub is_direct_message: bool,
}
