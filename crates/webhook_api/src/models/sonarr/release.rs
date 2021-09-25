use serde::{Deserialize, Serialize};

/// A representation of a file that Sonarr has grabbed but potentially not downloaded yet.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrRelease {
    pub quality: Option<String>,
    pub quality_version: Option<u32>,
    pub release_group: Option<String>,
    pub release_title: Option<String>,
    pub indexer: Option<String>,
    pub size: Option<u64>,
}
