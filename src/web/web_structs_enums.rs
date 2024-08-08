use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SortDirection {
    ASC,
    DESC,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TableNames {
    DamageDealtByType,
    DamageTakenByType,
    DamageTakenByMob,
    DamageTakenByMobPower,
    DPSIntervals,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ParseLog {
    ParsePath,
    LatestFile,
}

#[derive(Deserialize, Debug)]
pub struct ParseLogRequest {
    pub action: ParseLog,
    pub log_path: String,
}

#[derive(Deserialize, Debug)]
pub struct TableQuery {
    pub key: i32,
    pub db_path: String,
    pub table_name: Option<TableNames>,
    pub sort_field: Option<String>,
    pub sort_dir: Option<SortDirection>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PowerTableActions {
    Revert,
    RemoveNonDamaging,
    Merge,
    Delete,
}

#[derive(Deserialize, Debug)]
pub struct DamageByPowerQuery {
    pub key: i32,
    pub db_path: String,
    pub sort_field: Option<String>,
    pub sort_dir: Option<SortDirection>,
    pub action: Option<PowerTableActions>,
    pub power_row: Option<Vec<u8>>,
    pub mob_level: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PowersMobsData {
    pub key: i32,
    pub db_path: String,
    pub table_name: Option<TableNames>,
    pub power_name: Option<String>,
    pub mob_name: Option<String>,
    pub mob_level: Option<String>,
    pub sort_field: Option<String>,
    pub sort_dir: Option<SortDirection>,
}
