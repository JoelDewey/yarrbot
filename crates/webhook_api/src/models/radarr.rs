//! Models to be used when deserializing Radarr webhook request bodies.
//! These models are based on the source on Github.
//! Source: https://github.com/Radarr/Radarr/tree/627ab64fd023269c8bedece61e529329600a3419/src/NzbDrone.Core/Notifications/Webhook

use crate::models::common::ArrHealthCheckResult;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RadarrMovie {
    pub id: u64,
    pub title: String,
    pub file_path: Option<String>,
    // yyyy-MM-dd as per https://github.com/Radarr/Radarr/blob/627ab64fd023269c8bedece61e529329600a3419/src/NzbDrone.Core/Notifications/Webhook/WebhookMovie.cs#L25
    pub release_date: Option<NaiveDate>,
    pub folder_path: Option<String>,
    pub tmdb_id: Option<u32>,
    pub imdb_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RadarrRemoteMovie {
    pub title: String,
    pub year: Option<u32>,
    pub tmdb_id: Option<u32>,
    pub imdb_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RadarrRelease {
    pub quality: Option<String>,
    pub quality_version: Option<u32>,
    pub release_group: Option<String>,
    pub release_title: Option<String>,
    pub indexer: Option<String>,
    pub size: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RadarrMovieFile {
    pub id: u64,
    pub relative_path: String,
    pub path: String,
    pub quality: Option<String>,
    pub quality_version: Option<u32>,
    pub release_group: Option<String>,
    pub scene_name: Option<String>,
    pub size: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "eventType")]
pub enum RadarrWebhook {
    #[serde(rename_all = "camelCase")]
    Test {
        movie: RadarrMovie,
        remote_movie: RadarrRemoteMovie,
        release: RadarrRelease,
    },
    #[serde(rename_all = "camelCase")]
    Grab {
        movie: RadarrMovie,
        remote_movie: RadarrRemoteMovie,
        release: RadarrRelease,
        download_client: Option<String>,
        download_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Download {
        movie: RadarrMovie,
        remote_movie: RadarrRemoteMovie,
        movie_file: RadarrMovieFile,
        is_upgrade: bool,
        download_client: Option<String>,
        download_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Rename { movie: RadarrMovie },
    #[serde(rename_all = "camelCase")]
    MovieDelete {
        movie: RadarrMovie,
        deleted_files: bool,
    },
    #[serde(rename_all = "camelCase")]
    MovieFileDelete {
        movie: RadarrMovie,
        movie_file: RadarrMovieFile,
        delete_reason: Option<String>,
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
