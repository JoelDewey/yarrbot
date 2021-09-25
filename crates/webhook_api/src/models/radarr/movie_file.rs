use serde::{Deserialize, Serialize};

/// Metadata regarding the movie file Radarr is managing.
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
