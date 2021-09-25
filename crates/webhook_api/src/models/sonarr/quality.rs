use serde::{Deserialize, Serialize};

/// Metadata about the quality of an episode.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrQuality {
    pub id: u64,
    pub name: String,
    pub source: Option<String>,
    pub resolution: u32,
}
