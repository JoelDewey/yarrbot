use crate::models::sonarr::quality::SonarrQuality;
use serde::{Deserialize, Serialize};

/// Wrapper around [SonarrQuality].
///
/// # Remarks
///
/// This model omits `revision`.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrQualityModel {
    pub quality: SonarrQuality,
    // revision omitted
}
