use crate::message::{MessageData, MessageDataBuilder};
use tracing::info;

/// Returns information on using Yarrbot.
pub fn get_message() -> MessageData {
    info!("Received help command.");
    // TODO: Can this be simplified with macros?
    let mut builder = MessageDataBuilder::new();
    builder.add_key_value_with_code("Check that Yarrbot is online", "!yarrbot ping");
    builder.add_key_value_with_code("View this help message", "!yarrbot help");
    builder.add_key_value_with_code("Get the sourcecode for Yarrbot", "!yarrbot sourcecode");
    builder.add_key_value_with_code(
        "Add a new webhook",
        "!yarrbot webhook add roomOrAliasId username [password]",
    );
    builder.add_key_value_with_code("List configured webhooks", "!yarrbot webhook list");
    builder.add_key_value_with_code("Remove a webhook", "!yarrbot webhook remove webhookId");

    builder.to_message_data()
}
