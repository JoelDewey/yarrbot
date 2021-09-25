use crate::models::sonarr::extended_episode::SonarrExtendedEpisode;
use serde::{Deserialize, Serialize};

/// A list of episodes.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrEpisodeList {
    pub value: Vec<SonarrExtendedEpisode>,
}
