use serde::{Deserialize, Serialize};
use crate::models::sonarr::SonarrWebhook;
use crate::models::radarr::RadarrWebhook;

pub mod common;
pub mod radarr;
pub mod sonarr;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum ArrWebhook {
    Sonarr(SonarrWebhook),
    Radarr(RadarrWebhook),
}

#[cfg(test)]
mod tests {
    use crate::models::sonarr::{SonarrWebhook, SonarrSeries, SonarrSeriesType, SonarrEpisode};
    use crate::models::ArrWebhook;

    const TEST_BODY: &str = "{
    \"eventType\": \"Test\",
    \"series\": {
        \"id\": 1,
        \"title\": \"Test Title\",
        \"path\": \"C:\\\\testpath\",
        \"tvdbId\": 1234,
        \"type\": \"standard\"
    },
    \"episodes\": [
        {
            \"id\": 123,
            \"episodeNumber\": 1,
            \"seasonNumber\": 1,
            \"title\": \"Test title\",
            \"qualityVersion\": 0
        }
    ]
}";
    #[test]
    fn serde_deserialize_sonarr_webhook_body() {
        // Arrange
        let expected = ArrWebhook::Sonarr(SonarrWebhook::Test {
            series: SonarrSeries {
                id: 1,
                title: String::from("Test Title"),
                path: String::from("C:\\testpath"),
                tvdb_id: Some(1234),
                tv_maze_id: None,
                imdb_id: None,
                series_type: SonarrSeriesType::Standard,
            },
            episodes: vec![SonarrEpisode {
                id: 123,
                episode_number: 1,
                season_number: 1,
                title: String::from("Test title"),
                air_date: None,
                air_date_utc: None,
            }],
        });

        // Act
        let actual: ArrWebhook = serde_json::from_str(TEST_BODY).unwrap();

        // Assert
        assert_eq!(expected, actual)
    }
}