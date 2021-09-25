use crate::models::sonarr::episode_list::SonarrEpisodeList;
use crate::models::sonarr::quality_model::SonarrQualityModel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A record of an episode that Sonarr deleted.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrEpisodeDeletedFile {
    pub id: u64,
    pub relative_path: String,
    pub path: String,
    pub quality: Option<SonarrQualityModel>,
    pub release_group: Option<String>,
    pub scene_name: Option<String>,
    pub size: Option<u64>,
    pub date_added: Option<DateTime<Utc>>,
    pub episodes: Option<SonarrEpisodeList>,
    // mediaInfo omitted
}
