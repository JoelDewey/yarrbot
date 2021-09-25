use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Metadata about a movie that Radarr is tracking.
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
