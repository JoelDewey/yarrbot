use anyhow::Error;
use base64::{CharacterSet, Config};
use uuid::Uuid;

/// Enables converting some UUID struct ([uuid::Uuid]) to and from a "short ID", which is a base64 and URL-safe
/// representation of said UUID struct.
pub trait ShortId {
    /// Convert [Self] to a short ID as a [String].
    fn to_short_id(&self) -> String;

    /// Convert some [&str] back into a [Self].
    fn from_short_id(short_id: &str) -> Result<Box<Self>, Error>;
}

const SHORT_ID_CONFIG: Config = Config::new(CharacterSet::UrlSafe, false);

impl ShortId for Uuid {
    fn to_short_id(&self) -> String {
        base64::encode_config(self.as_bytes(), SHORT_ID_CONFIG)
    }

    fn from_short_id(short_id: &str) -> Result<Box<Self>, Error> {
        let mut buf: [u8; 16] = [0; 16];
        base64::decode_config_slice(short_id, SHORT_ID_CONFIG, &mut buf)?;
        Ok(Box::new(Uuid::from_slice(&buf)?))
    }
}

#[cfg(test)]
mod test {
    use crate::short_id::ShortId;
    use uuid::Uuid;

    #[test]
    fn shortid_trait_converts_uuid_to_and_from_short_id() {
        // Arrange
        let expected = "e3f4e475-d503-49f2-b6ba-907a96ac4d8d";
        let expected_uuid = Uuid::parse_str(expected).unwrap();
        let expected_short_id = "4_TkddUDSfK2upB6lqxNjQ";

        // Act
        let result_short_id = expected_uuid.to_short_id();
        let actual_short_id = result_short_id.as_str();
        let actual_uuid = Uuid::from_short_id(actual_short_id);

        // Assert
        assert_eq!(expected_short_id, actual_short_id);
        assert!(actual_uuid.is_ok());
        assert_eq!(&expected_uuid, actual_uuid.unwrap().as_ref());
    }

    #[test]
    fn from_short_id_returns_error_given_invalid_short_id() {
        // Arrange
        let input = "Totally not a short ID";

        // Act
        let actual = Uuid::from_short_id(input);

        // Assert
        assert!(actual.is_err());
    }
}
