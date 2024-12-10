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
    pub matchers: Vec<Matcher>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Matcher {
    #[serde(rename = "input_text")]
    pub input_text: String,
    pub actions: Vec<Action>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(rename = "trigger_type")]
    pub trigger_type: TriggerType,
    #[serde(rename = "output_text")]
    pub output_text: String,
    #[serde(rename = "delay_secs")]
    pub delay_secs: u64,
    #[serde(rename = "linger_secs")]
    pub linger_secs: u64,
}