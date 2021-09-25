//! Models to be used when deserializing Radarr webhook request bodies.
//! These models are based on the source on Github.
//! Source: https://github.com/Radarr/Radarr/tree/627ab64fd023269c8bedece61e529329600a3419/src/NzbDrone.Core/Notifications/Webhook

mod movie;
mod movie_file;
mod release;
mod remote_movie;

use crate::models::common::ArrHealthCheckResult;
pub use movie::RadarrMovie;
pub use movie_file::RadarrMovieFile;
pub use release::RadarrRelease;
pub use remote_movie::RadarrRemoteMovie;
use serde::{Deserialize, Serialize};

/// Represents the various webhooks that Radarr can send. The type of webhook is determined by
/// the webhook's `eventType` property in the body.
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
