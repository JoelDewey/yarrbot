//! Utilities for building [MessageData] structs for delivery of Matrix messages.

use crate::message::MessageData;

const DEFAULT_PLAIN_BREAK: &str = "|";
const DEFAULT_HTML_BREAK: &str = "<br>";

/// Implementors may implement this trait to allow a struct to format its data for display in Matrix.
pub trait MatrixMessageDataPart {
    /// Print a message to send to clients that do not support HTML messages.
    ///
    /// [break_character] represents some separator to place between this
    /// [MatrixMessageDataPart] and the next message data part. It is expected
    /// that the implementor add leading whitespace before appending the [break_character].
    fn to_plain(&self, break_character: &str) -> String;

    /// Print a message to send to clients that do support HTML messages.
    /// Will take precedence over plain messages if so.
    ///
    /// The [break_character] represents some character to separate this [MatrixMessageDataPart]
    /// from the next message data part. This is likely a line break element, `<br>`, but isn't
    /// guaranteed.
    fn to_html(&self, break_character: &str) -> String;
}

/// Builds [MessageData] structs with special formatting. Forward-only.
pub struct MessageDataBuilder {
    plain_parts: String,
    html_parts: String,
}

impl MessageDataBuilder {
    pub(crate) fn new() -> Self {
        MessageDataBuilder {
            plain_parts: String::new(),
            html_parts: String::new(),
        }
    }

    /// Adds some character or set of characters that "break" or "separate" message data parts.
    /// For HTML text messages, a line break (`<br>`) after the last part added to the builder.
    /// For plain text messages, the default separator `|` is added instead.
    pub(crate) fn break_character(mut self) -> Self {
        self.plain_parts.push(' ');
        self.plain_parts.push_str(DEFAULT_PLAIN_BREAK);

        self.html_parts.push(' ');
        self.html_parts.push_str(DEFAULT_HTML_BREAK);
        self
    }

    /// Adds a key-value data pair to the message with a line break (or the default plain text separator).
    ///
    /// # Examples
    /// Plain: `key: value`
    /// Rich: `<strong>key</strong>: value`
    pub(crate) fn add_key_value(mut self, key: &str, value: &str) -> Self {
        self.plain_parts.push(' ');
        self.plain_parts.push_str(key);
        self.plain_parts.push_str(": ");
        self.plain_parts.push_str(value);

        self.html_parts.push_str("<strong>");
        self.html_parts.push_str(key);
        self.html_parts.push_str("</strong>: ");
        self.html_parts.push_str(value);

        self.break_character()
    }

    /// Adds a key-value data pair to the message with a line break (or the default plain text separator).
    /// The value is wrapped in `<code>` tags for the HTML message.
    ///
    /// # Examples
    /// Plain: `key: value`
    /// Rich: `<strong>key</strong>: <code>value</code>`
    pub(crate) fn add_key_value_with_code(mut self, key: &str, value: &str) -> Self {
        self.plain_parts.push(' ');
        self.plain_parts.push_str(key);
        self.plain_parts.push_str(": ");
        self.plain_parts.push_str(value);

        self.html_parts.push_str("<strong>");
        self.html_parts.push_str(key);
        self.html_parts.push_str("</strong>: <code>");
        self.html_parts.push_str(value);
        self.html_parts.push_str("</code>");

        self.break_character()
    }

    /// Adds the data from some implementor of [MatrixMessageDataPart] to the end of the resulting [MessageData].
    /// This function _does not_ add any "break characters" after appending the [MatrixMessageDataPart] and
    /// requires that either the [MatrixMessageDataPart] supply the separators/line breaks or call [Self::break_character()].
    pub(crate) fn add_matrix_message_part(mut self, part: impl MatrixMessageDataPart) -> Self {
        self.plain_parts
            .push_str(&part.to_plain(DEFAULT_PLAIN_BREAK));
        self.html_parts.push_str(&part.to_html(DEFAULT_HTML_BREAK));
        self
    }

    /// Adds a line of text followed by [Self::break_character()].
    pub(crate) fn add_line(mut self, line: &str) -> Self {
        self.plain_parts.push_str(line);
        self.html_parts.push_str(line);
        self.break_character()
    }

    /// Copy the contents of this builder to a new [MessageData].
    ///
    /// Note that for the [MessageData.plain], preceding and trailing whitespace is trimmed. For both [MessageData.plain]  
    /// and [MessageData.html], trailing "break characters" (as inserted by [Self::break_character()]) are trimmed.
    pub(crate) fn to_message_data(&self) -> MessageData {
        MessageData::new(
            self.plain_parts
                .trim_end_matches(DEFAULT_PLAIN_BREAK)
                .trim(),
            self.html_parts.trim_end_matches(DEFAULT_HTML_BREAK).trim(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::message::message_data_builder::{MatrixMessageDataPart, MessageDataBuilder};

    #[test]
    pub fn key_value_returns_result_without_trailing_break_character_given_single_use() {
        // Arrange
        let expected_plain = "Test: Of KeyValue";
        let expected_html = "<strong>Test</strong>: Of KeyValue";
        let builder = MessageDataBuilder::new().add_key_value("Test", "Of KeyValue");

        // Act
        let actual = builder.to_message_data();

        // Assert
        assert_eq!(expected_plain, actual.plain);
        assert_eq!(expected_html, actual.html);
    }

    #[test]
    pub fn key_value_returns_result_with_separating_break_character_given_multiple_uses() {
        // Arrange
        let expected_plain = "Test: Of KeyValue | Test2: Of KeyValue2";
        let expected_html =
            "<strong>Test</strong>: Of KeyValue <br><strong>Test2</strong>: Of KeyValue2";
        let builder = MessageDataBuilder::new()
            .add_key_value("Test", "Of KeyValue")
            .add_key_value("Test2", "Of KeyValue2");

        // Act
        let actual = builder.to_message_data();

        // Assert
        assert_eq!(expected_plain, actual.plain);
        assert_eq!(expected_html, actual.html);
    }

    struct TestMessageDataPart;

    impl MatrixMessageDataPart for TestMessageDataPart {
        fn to_plain(&self, break_character: &str) -> String {
            format!("Testing test {}", break_character)
        }

        fn to_html(&self, break_character: &str) -> String {
            format!("<h1>Testing</h1><br>Test! {}", break_character)
        }
    }

    #[test]
    pub fn add_matrix_message_part_inserts_as_is() {
        // Arrange
        let expected_plain = "Testing test | 1: 2";
        let expected_html = "<h1>Testing</h1><br>Test! <br><strong>1</strong>: 2";
        let builder = MessageDataBuilder::new()
            .add_matrix_message_part(TestMessageDataPart)
            .add_key_value("1", "2");

        // Act
        let actual = builder.to_message_data();

        // Assert
        assert_eq!(expected_plain, actual.plain);
        assert_eq!(expected_html, actual.html);
    }
}
