use serde::{Deserialize, Serialize};

/// Additional movie metadata from sources remote to Radarr (e.g. IMDB).
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RadarrRemoteMovie {
    pub title: String,
    pub year: Option<u32>,
    pub tmdb_id: Option<u32>,
    pub imdb_id: Option<String>,
}
