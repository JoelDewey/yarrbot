use crate::message::MessageData;
use tracing::info;

const SOURCE_URL: &str = "https://github.com/JoelDewey/yarrbot";

pub fn get_message() -> MessageData {
    info!("Received sourcecode command.");
    MessageData::from(
        format!(
            "The source code for this bot is available at: {}",
            SOURCE_URL
        )
        .as_str(),
    )
}
