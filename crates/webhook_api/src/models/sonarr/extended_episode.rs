use crate::models::sonarr::episode::SonarrEpisode;
use crate::models::sonarr::extended_episode_rating::SonarrExtendedEpisodeRating;
use serde::{Deserialize, Serialize};

/// An extended version of [SonarrEpisode] that contains some additional metadata.
/// Typically used in lists of deleted episode files.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrExtendedEpisode {
    #[serde(flatten)]
    pub base: SonarrEpisode,
    pub overview: Option<String>,
    pub monitored: bool,
    pub ratings: SonarrExtendedEpisodeRating,
}
