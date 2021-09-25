//! Models intended to be used when deserializing webhook bodies from Sonarr.
//! Source: https://github.com/Sonarr/Sonarr/tree/3c45349404f59064d1c8db0549401189c456e4c0/src/NzbDrone.Core/Notifications/Webhook

mod episode;
mod episode_deleted_file;
mod episode_file;
mod episode_list;
mod extended_episode;
mod extended_episode_rating;
mod quality;
mod quality_model;
mod release;
mod renamed_episode_file;
mod series;

pub use crate::models::common::ArrHealthCheckResult;
pub use episode::SonarrEpisode;
pub use episode_deleted_file::SonarrEpisodeDeletedFile;
pub use episode_file::SonarrEpisodeFile;
pub use episode_list::SonarrEpisodeList;
pub use extended_episode::SonarrExtendedEpisode;
pub use extended_episode_rating::SonarrExtendedEpisodeRating;
pub use quality::SonarrQuality;
pub use quality_model::SonarrQualityModel;
pub use release::SonarrRelease;
pub use renamed_episode_file::SonarrRenamedEpisodeFile;
use serde::{Deserialize, Serialize};
pub use series::{SonarrSeries, SonarrSeriesType};

/// Represents the various webhooks that Sonarr can send. The type of webhook is determined by
/// the webhook's `eventType` property in the body.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "eventType")]
pub enum SonarrWebhook {
    #[serde(rename_all = "camelCase")]
    Grab {
        series: SonarrSeries,
        episodes: Vec<SonarrEpisode>,
        release: SonarrRelease,
        download_client: Option<String>,
        download_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Download {
        series: SonarrSeries,
        episodes: Vec<SonarrEpisode>,
        episode_file: SonarrEpisodeFile,
        is_upgrade: bool,
        download_client: Option<String>,
        download_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Rename {
        series: SonarrSeries,
        renamed_episode_files: Vec<SonarrRenamedEpisodeFile>,
    },
    #[serde(rename_all = "camelCase")]
    SeriesDelete {
        series: SonarrSeries,
        deleted_files: bool,
    },
    #[serde(rename_all = "camelCase")]
    EpisodeFileDelete {
        series: SonarrSeries,
        episodes: Vec<SonarrEpisode>,
        episode_file: SonarrEpisodeDeletedFile,
        delete_reason: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Test {
        series: SonarrSeries,
        episodes: Vec<SonarrEpisode>,
    },
    #[serde(rename_all = "camelCase")]
    Health {
        level: Option<ArrHealthCheckResult>,
        message: Option<String>,
        #[serde(rename = "type")]
        health_type: Option<String>,
        wiki_url: Option<String>,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{DateTime, NaiveDate};

    const GRAB_BODY: &str = "{
    \"eventType\": \"Grab\",
    \"series\": {
        \"id\": 2,
        \"title\": \"Gravity Falls\",
        \"path\": \"C:\\\\Temp\\\\sonarr\\\\Gravity Falls\",
        \"tvdbId\": 259972,
        \"type\": \"standard\"
    },
    \"episodes\": [
        {
            \"id\": 67,
            \"episodeNumber\": 14,
            \"seasonNumber\": 2,
            \"title\": \"The Stanchurian Candidate\",
            \"airDate\": \"2015-08-24\",
            \"airDateUtc\": \"2015-08-25T01:30:00Z\",
            \"quality\": \"HDTV-720p\",
            \"qualityVersion\": 1
        }
    ],
    \"release\": {
        \"quality\": \"HDTV-720p\",
        \"qualityVersion\": 1,
        \"size\": 0
    }
}
";
    #[test]
    fn serde_deserialize_grab_body() {
        // Arrange
        let expected = SonarrWebhook::Grab {
            series: SonarrSeries {
                id: 2,
                title: String::from("Gravity Falls"),
                path: String::from("C:\\Temp\\sonarr\\Gravity Falls"),
                tvdb_id: Some(259972),
                tv_maze_id: None,
                imdb_id: None,
                series_type: SonarrSeriesType::Standard,
            },
            episodes: vec![SonarrEpisode {
                id: 67,
                episode_number: 14,
                season_number: 2,
                title: String::from("The Stanchurian Candidate"),
                air_date: Some(NaiveDate::from_ymd(2015, 8, 24)),
                air_date_utc: Some(DateTime::from(
                    DateTime::parse_from_rfc3339("2015-08-25T01:30:00Z").unwrap(),
                )),
            }],
            release: SonarrRelease {
                quality: Some(String::from("HDTV-720p")),
                quality_version: Some(1),
                release_group: None,
                release_title: None,
                indexer: None,
                size: Some(0),
            },
            download_client: None,
            download_id: None,
        };

        // Act
        let actual: SonarrWebhook = serde_json::from_str(GRAB_BODY).unwrap();

        // Assert
        assert_eq!(expected, actual)
    }

    const DOWNLOAD_BODY: &str = "{
    \"eventType\": \"Download\",
    \"series\": {
        \"id\": 2,
        \"title\": \"Gravity Falls\",
        \"path\": \"C:\\\\Temp\\\\sonarr\\\\Gravity Falls\",
        \"tvdbId\": 259972,
        \"type\": \"standard\"
    },
    \"episodes\": [
        {
            \"id\": 67,
            \"episodeNumber\": 14,
            \"seasonNumber\": 2,
            \"title\": \"The Stanchurian Candidate\",
            \"airDate\": \"2015-08-24\",
            \"airDateUtc\": \"2015-08-25T01:30:00Z\",
            \"quality\": \"HDTV-720p\",
            \"qualityVersion\": 1
        }
    ],
    \"episodeFile\": {
        \"id\": 1181,
        \"relativePath\": \"Season 02\\\\Gravity Falls - s02e14.mkv\",
        \"path\": \"C:\\\\path\\\\to\\\\file\\\\GravityFalls - s02e14.mkv\",
        \"quality\": \"HDTV-720p\",
        \"qualityVersion\": 1
    },
    \"isUpgrade\": false
}";
    #[test]
    fn serde_deserialize_download_body() {
        // Arrange
        let expected = SonarrWebhook::Download {
            series: SonarrSeries {
                id: 2,
                title: String::from("Gravity Falls"),
                path: String::from("C:\\Temp\\sonarr\\Gravity Falls"),
                tvdb_id: Some(259972),
                tv_maze_id: None,
                imdb_id: None,
                series_type: SonarrSeriesType::Standard,
            },
            episodes: vec![SonarrEpisode {
                id: 67,
                episode_number: 14,
                season_number: 2,
                title: String::from("The Stanchurian Candidate"),
                air_date: Some(NaiveDate::from_ymd(2015, 8, 24)),
                air_date_utc: Some(DateTime::from(
                    DateTime::parse_from_rfc3339("2015-08-25T01:30:00Z").unwrap(),
                )),
            }],
            episode_file: SonarrEpisodeFile {
                id: 1181,
                relative_path: String::from("Season 02\\Gravity Falls - s02e14.mkv"),
                path: String::from("C:\\path\\to\\file\\GravityFalls - s02e14.mkv"),
                release_group: None,
                scene_name: None,
                size: None,
                quality: Some(String::from("HDTV-720p")),
                quality_version: Some(1),
            },
            is_upgrade: false,
            download_client: None,
            download_id: None,
        };

        // Act
        let actual: SonarrWebhook = serde_json::from_str(DOWNLOAD_BODY).unwrap();

        // Assert
        assert_eq!(expected, actual)
    }

    const RENAME_BODY: &str = "{
    \"eventType\": \"Rename\",
    \"series\": {
        \"id\": 2,
        \"title\": \"Gravity Falls\",
        \"path\": \"C:\\\\Temp\\\\sonarr\\\\Gravity Falls\",
        \"tvdbId\": 259972,
        \"type\": \"standard\"
    },
    \"renamedEpisodeFiles\": []
}
";
    #[test]
    fn serde_deserialize_rename_body() {
        // Arrange
        let expected = SonarrWebhook::Rename {
            series: SonarrSeries {
                id: 2,
                title: String::from("Gravity Falls"),
                path: String::from("C:\\Temp\\sonarr\\Gravity Falls"),
                tvdb_id: Some(259972),
                tv_maze_id: None,
                imdb_id: None,
                series_type: SonarrSeriesType::Standard,
            },
            renamed_episode_files: vec![],
        };

        // Act
        let actual: SonarrWebhook = serde_json::from_str(RENAME_BODY).unwrap();

        // Assert
        assert_eq!(expected, actual)
    }

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
    fn serde_deserialize_test_body() {
        // Arrange
        let expected = SonarrWebhook::Test {
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
        };

        // Act
        let actual: SonarrWebhook = serde_json::from_str(TEST_BODY).unwrap();

        // Assert
        assert_eq!(expected, actual)
    }
}
