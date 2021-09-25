use serde::{Deserialize, Serialize};

/// Release metadata describing details about the released movie file that was obtained (e.g. quality).
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RadarrRelease {
    pub quality: Option<String>,
    pub quality_version: Option<u32>,
    pub release_group: Option<String>,
    pub release_title: Option<String>,
    pub indexer: Option<String>,
    pub size: Option<u64>,
}
