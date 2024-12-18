use std::hash::Hash;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Default, Hash)]
pub enum TriggerType {
    #[default]
    ACTIVATION,
    RECHARGE,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct EventKey {
    pub log_date: DateTime<Local>,
    pub line_number: i32,
    pub trigger_type: TriggerType,
    pub power_name: String,
}

#[derive(Debug)]
pub struct MessageDetails {
    pub trigger_type: TriggerType,
    pub power_name: String,
    pub output_text: String,
    pub escalation_one_time: DateTime<Local>,
    pub escalation_one_font_size: u8,
    pub escalation_one_color: String,
    pub escalation_two_time: DateTime<Local>,
    pub escalation_two_font_size: u8,
    pub escalation_two_color: String,
    pub escalation_three_time: DateTime<Local>,
    pub escalation_three_font_size: u8,
    pub escalation_three_color: String,
    pub end_time: DateTime<Local>,
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
    #[serde(rename = "display_colors")]
    pub display_colors: Vec<String>,
    #[serde(rename = "font_sizes")]
    pub font_size: Vec<u8>,
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
    pub delay_secs: u16,
    #[serde(rename = "display_secs")]
    pub display_secs: u16,
}
