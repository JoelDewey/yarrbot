use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ArrHealthCheckResult {
    Ok,
    Notice,
    Warning,
    Error,
    #[serde(other)]
    Unknown,
}
