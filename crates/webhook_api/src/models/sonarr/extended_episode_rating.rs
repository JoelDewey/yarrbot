use serde::{Deserialize, Serialize};

/// A rating of a particular episode of a show.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrExtendedEpisodeRating {
    pub votes: u32,
    pub value: serde_json::Number,
}
