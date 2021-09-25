use serde::{Deserialize, Serialize};

/// The record of a file for an episode.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrEpisodeFile {
    pub id: u64,
    pub relative_path: String,
    pub path: String,
    pub quality: Option<String>,
    pub quality_version: Option<u32>,
    pub release_group: Option<String>,
    pub scene_name: Option<String>,
    pub size: Option<u64>,
}
