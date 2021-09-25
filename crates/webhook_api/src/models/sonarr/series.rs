use serde::{Deserialize, Serialize};

/// The type of series, usually indicating the method that new episodes are aired.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SonarrSeriesType {
    Standard,
    Daily,
    Anime,
}

/// General data about a series Sonarr is tracking.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrSeries {
    pub id: u64,
    pub title: String,
    pub path: String,
    pub tvdb_id: Option<u32>,
    pub tv_maze_id: Option<u32>,
    pub imdb_id: Option<String>,
    #[serde(rename = "type")]
    pub series_type: SonarrSeriesType,
}
