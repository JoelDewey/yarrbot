use serde::{Deserialize, Serialize};

/// Metadata around a renamed episode file.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrRenamedEpisodeFile {
    pub relative_path: Option<String>,
    pub path: Option<String>,
    pub previous_relative_path: Option<String>,
    pub previous_path: Option<String>,
}
