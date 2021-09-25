use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// A record of an episode.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SonarrEpisode {
    pub id: u64,
    pub episode_number: u32,
    pub season_number: u32,
    pub title: String,
    pub air_date: Option<NaiveDate>,
    pub air_date_utc: Option<DateTime<Utc>>,
}
