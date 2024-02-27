// Generated by diesel_ext
use super::schema::*;
use diesel::prelude::*;
use diesel::prelude::*;

//#![allow(unused)]
//#![allow(clippy::all)]

#[derive(Queryable, Debug, Identifiable, Insertable, Selectable)]
#[diesel(primary_key(summary_key, line_number, log_date))]
#[diesel(table_name = damage_action)]
pub struct DamageAction {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: i32,
    pub target: String,
    pub power_name: String,
    pub damage: i32,
    pub damage_type: String,
    pub source_type: String,
    pub source_name: Option<String>,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(summary_key, line_number, log_date))]
#[diesel(table_name = debuff_action)]
pub struct DebuffAction {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: String,
    pub source_type: Option<String>,
    pub source_name: String,
    pub power_name: Option<String>,
    pub target_name: Option<String>,
    pub debuff_type: Option<String>,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(summary_key, line_number, log_date))]
#[diesel(table_name = defeated_targets)]
pub struct DefeatedTarget {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: String,
    pub source_name: String,
    pub target_name: String,
}

#[derive(Queryable, Debug, Identifiable, Insertable)]
#[diesel(primary_key(summary_key, line_number, log_date))]
#[diesel(table_name = hit_or_miss)]
pub struct HitOrMis {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: String,
    pub hit: i32,
    pub chance_to_hit: i32,
    pub hit_roll: i32,
    pub source_type: String,
    pub source_name: String,
    pub target_name: String,
    pub power_name: String,
    pub streakbreaker: i32,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(summary_key, line_number, log_date))]
#[diesel(table_name = player_activation)]
pub struct PlayerActivation {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: i32,
    pub power_name: Option<String>,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(session_key, line_number, log_date))]
#[diesel(table_name = reward)]
pub struct Reward {
    pub session_key: i32,
    pub line_number: i32,
    pub log_date: String,
    pub experience: Option<i32>,
    pub influence: Option<i32>,
    pub item_drop: Option<String>,
    pub reward_type: Option<String>,
}

#[derive(Queryable, Debug, Identifiable, Insertable, Selectable)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = summary)]
pub struct Summary {
    pub player_name: String,
    pub log_date: String,
    pub line_number: i32,
    pub log_file_name: String,
    pub summary_key: i32,
}
