use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ArrHealthCheckResult {
    Ok,
    Notice,
    Warning,
    Error,
    #[serde(other)]
    Unknown,
}
