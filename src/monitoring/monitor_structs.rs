use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub enum TriggerType {
    #[default]
    ACTIVATION,
    RECHARGE,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorConfig {
    pub dir: PathBuf,
    pub dps: bool,
    pub experience: bool,
    pub influence: bool,
    #[serde(rename = "session_totals")]
    pub session_totals: bool,
    pub actions: Vec<Action>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub dir: String,
    pub dps: bool,
    pub experience: bool,
    pub influence: bool,
    #[serde(rename = "session_totals")]
    pub session_totals: bool,
    pub actions: Vec<Action>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(rename = "trigger_type")]
    pub trigger_type: TriggerType,
    #[serde(rename = "power_name")]
    pub power_name: String,
    #[serde(rename = "output_text")]
    pub output_text: String,
    #[serde(rename = "delay_secs")]
    pub delay_secs: i64,
    #[serde(rename = "linger_secs")]
    pub linger_secs: i64,
}
