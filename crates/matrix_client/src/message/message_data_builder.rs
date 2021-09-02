//! Utilities for building [MessageData] structs for delivery of Matrix messages.

use crate::message::MessageData;
use std::fmt::Write;

const DEFAULT_PLAIN_BREAK: &str = "\n";
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

/// Represents the level of section heading; otherwise known as `<h1>` through `<h6>`.
#[allow(dead_code)]
pub enum SectionHeadingLevel {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

/// Builds [MessageData] structs with special formatting. Forward-only.
pub struct MessageDataBuilder {
    plain_parts: String,
    html_parts: String,
}

impl MessageDataBuilder {
    pub fn new() -> Self {
        MessageDataBuilder {
            plain_parts: String::new(),
            html_parts: String::new(),
        }
    }

    /// Adds some character or set of characters that "break" or "separate" message data parts.
    /// For HTML text messages, a line break (`<br>`) after the last part added to the builder.
    /// For plain text messages, the default separator `\n` is added instead.
    pub fn break_character(&mut self) {
        self.plain_parts.push(' ');
        self.plain_parts.push_str(DEFAULT_PLAIN_BREAK);

        self.html_parts.push(' ');
        self.html_parts.push_str(DEFAULT_HTML_BREAK);
    }

    /// Adds a key-value data pair to the message with a line break (or the default plain text separator).
    ///
    /// # Examples
    /// Plain: `key: value`
    /// Rich: `<strong>key</strong>: value`
    pub fn add_key_value(&mut self, key: &str, value: &str) {
        self.plain_parts.push_str(" **");
        self.plain_parts.push_str(key);
        self.plain_parts.push_str("**: ");
        self.plain_parts.push_str(value);

        self.html_parts.push_str("<strong>");
        self.html_parts.push_str(key);
        self.html_parts.push_str("</strong>: ");
        self.html_parts.push_str(value);

        self.break_character();
    }

    /// Adds a key-value data pair to the message with a line break (or the default plain text separator).
    /// The value is wrapped in `<code>` tags for the HTML message.
    ///
    /// # Examples
    /// Plain: `key: value`
    /// Rich: `<strong>key</strong>: <code>value</code>`
    pub fn add_key_value_with_code(&mut self, key: &str, value: &str) {
        self.plain_parts.push_str(" **");
        self.plain_parts.push_str(key);
        self.plain_parts.push_str("**: ");
        self.plain_parts.push_str(value);

        self.html_parts.push_str("<strong>");
        self.html_parts.push_str(key);
        self.html_parts.push_str("</strong>: <code>");
        self.html_parts.push_str(value);
        self.html_parts.push_str("</code>");

        self.break_character();
    }

    /// Adds the data from some implementor of [MatrixMessageDataPart] to the end of the resulting [MessageData].
    /// This function _does not_ add any "break characters" after appending the [MatrixMessageDataPart] and
    /// requires that either the [MatrixMessageDataPart] supply the separators/line breaks or call [Self::break_character()].
    pub fn add_matrix_message_part(&mut self, part: impl MatrixMessageDataPart) {
        self.plain_parts
            .push_str(&part.to_plain(DEFAULT_PLAIN_BREAK));
        self.html_parts.push_str(&part.to_html(DEFAULT_HTML_BREAK));
    }

    /// Adds a line of text followed by [Self::break_character()].
    pub fn add_line(&mut self, line: &str) {
        self.plain_parts.push_str(line);
        write!(self.html_parts, "<p>{}</p>", line).expect("Failed to write HTML String.");
        self.break_character();
    }

    /// Adds a heading to the message. For HTML messages, this will be some `<h1>` through `<h6>` element surrounded by
    /// a `<div>` with a  `<br>`. For plain messages, this will be some text prefixed with a Markdown header character
    /// and two line breaks.
    pub fn add_heading(&mut self, heading: &SectionHeadingLevel, text: &str) {
        let html_heading = match heading {
            SectionHeadingLevel::One => "h1",
            SectionHeadingLevel::Two => "h2",
            SectionHeadingLevel::Three => "h3",
            SectionHeadingLevel::Four => "h4",
            SectionHeadingLevel::Five => "h5",
            SectionHeadingLevel::Six => "h6",
        };
        write!(
            self.html_parts,
            "<div><{}><i>{}</i></{}></div><br>",
            html_heading, text, html_heading
        )
        .expect("Failed to write to underlying HTML String.");

        let plain_heading = match heading {
            SectionHeadingLevel::One => "#",
            SectionHeadingLevel::Two => "##",
            SectionHeadingLevel::Three => "###",
            SectionHeadingLevel::Four => "####",
            SectionHeadingLevel::Five => "#####",
            SectionHeadingLevel::Six => "######",
        };

        write!(
            self.plain_parts,
            "{} {} {}{}",
            plain_heading, text, DEFAULT_PLAIN_BREAK, DEFAULT_PLAIN_BREAK
        )
        .expect("Failed to write to underlying plain String.");
    }

    /// Copy the contents of this builder to a new [MessageData].
    ///
    /// Note that:
    ///  * Preceding and trailing whitespace is trimmed.
    ///  * Trailing "break characters" (as inserted by [Self::break_character()]) are trimmed.
    ///  * For HTML messages only, a single "break character" is inserted after trimming for formatting.
    pub fn to_message_data(&self) -> MessageData {
        let mut plain = String::from(
            self.plain_parts
                .trim_end_matches(DEFAULT_PLAIN_BREAK)
                .trim(),
        );
        plain.push(' ');
        plain.push_str(DEFAULT_PLAIN_BREAK);
        let mut html = String::from(self.html_parts.trim_end_matches(DEFAULT_HTML_BREAK).trim());
        html.push(' ');
        html.push_str(DEFAULT_HTML_BREAK);

        MessageData::new(&plain, &html)
    }
}

impl Default for MessageDataBuilder {
    fn default() -> Self {
        MessageDataBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::message::message_data_builder::{
        MatrixMessageDataPart, MessageDataBuilder, SectionHeadingLevel,
    };

    #[test]
    pub fn key_value_returns_result_without_trailing_break_character_given_single_use() {
        // Arrange
        let expected_plain = "**Test**: Of KeyValue \n";
        let expected_html = "<strong>Test</strong>: Of KeyValue <br>";
        let mut builder = MessageDataBuilder::new();
        builder.add_key_value("Test", "Of KeyValue");

        // Act
        let actual = builder.to_message_data();

        // Assert
        assert_eq!(expected_plain, actual.plain);
        assert_eq!(expected_html, actual.html);
    }

    #[test]
    pub fn key_value_returns_result_with_separating_break_character_given_multiple_uses() {
        // Arrange
        let expected_plain = "**Test**: Of KeyValue \n **Test2**: Of KeyValue2 \n";
        let expected_html =
            "<strong>Test</strong>: Of KeyValue <br><strong>Test2</strong>: Of KeyValue2 <br>";
        let mut builder = MessageDataBuilder::new();
        builder.add_key_value("Test", "Of KeyValue");
        builder.add_key_value("Test2", "Of KeyValue2");

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
        let expected_plain = "Testing test \n **1**: 2 \n";
        let expected_html = "<h1>Testing</h1><br>Test! <br><strong>1</strong>: 2 <br>";
        let mut builder = MessageDataBuilder::new();
        builder.add_matrix_message_part(TestMessageDataPart);
        builder.add_key_value("1", "2");

        // Act
        let actual = builder.to_message_data();

        // Assert
        assert_eq!(expected_plain, actual.plain);
        assert_eq!(expected_html, actual.html);
    }

    #[test]
    pub fn add_heading_inserts_expected() {
        // Arrange
        let expected_plain = "## Test 123 \n\n **1**: 2 \n";
        let expected_html = "<div><h2><i>Test 123</i></h2></div><br><strong>1</strong>: 2 <br>";
        let mut builder = MessageDataBuilder::new();
        builder.add_heading(&SectionHeadingLevel::Two, "Test 123");
        builder.add_key_value("1", "2");

        // Act
        let actual = builder.to_message_data();

        // Assert
        assert_eq!(expected_plain, actual.plain);
        assert_eq!(expected_html, actual.html);
    }
}
