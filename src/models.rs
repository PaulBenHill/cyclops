// Generated by diesel_ext
use super::schema::*;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable};
use serde::{Deserialize, Serialize};

//#![allow(unused)]
//#![allow(clippy::all)]

#[derive(Queryable, Debug, Identifiable, Insertable, Selectable)]
#[diesel(primary_key(summary_key, line_number, log_date))]
#[diesel(table_name = damage_action)]
pub struct DamageAction {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: String,
    pub target_name: String,
    pub power_name: String,
    pub damage: i32,
    pub damage_type: String,
    pub damage_mode: String,
    pub source_type: String,
    pub source_name: String,
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

#[derive(Queryable, Debug, Clone, Identifiable, Insertable, Selectable)]
#[diesel(primary_key(summary_key, line_number, log_date))]
#[diesel(table_name = defeated_targets)]
pub struct DefeatedTarget {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: String,
    pub source_name: String,
    pub target_name: String,
}

#[derive(Queryable, Debug, Clone, Identifiable, Insertable, Selectable)]
#[diesel(primary_key(summary_key, line_number, log_date))]
#[diesel(table_name = hit_or_miss)]
pub struct HitOrMiss {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: String,
    pub hit: i32,
    pub chance_to_hit: i32,
    pub source_type: String,
    pub source_name: String,
    pub target_name: String,
    pub power_name: String,
    pub streakbreaker: i32,
}

#[derive(Queryable, Debug, Clone, Identifiable, Insertable, Selectable)]
#[diesel(primary_key(summary_key, line_number, log_date))]
#[diesel(table_name = player_activation)]
pub struct PlayerActivation {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: String,
    pub power_name: String,
}

#[derive(Queryable, Debug, Clone, Identifiable, Insertable, Selectable, Serialize, Deserialize)]
#[diesel(primary_key(summary_key, line_number, log_date))]
#[diesel(table_name = reward)]
pub struct Reward {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: String,
    pub experience: Option<i32>,
    pub influence: Option<i32>,
    pub item_drop: Option<String>,
    pub reward_type: String,
}

#[derive(Queryable, Debug, Clone, Identifiable, Insertable, Selectable, Serialize, Deserialize)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = summary)]
pub struct Summary {
    pub summary_key: i32,
    pub first_line_number: i32,
    pub last_line_number: i32,
    pub log_date: String,
    pub player_name: String,
    pub log_file_name: String,
}

#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(log_date))]
#[diesel(table_name = index_details)]
pub struct IndexDetails {
    pub log_date: String,
    pub player_name: String,
    pub data_points: String,
    pub file: String,
}

#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = total_damage_report)]
pub struct TotalDamageReport {
    pub summary_key: i32,
    pub activations: i32,
    pub hits: i32,
    pub streak_breakers: i32,
    pub misses: i32,
    pub total_damage: i32,
    pub direct_damage: i32,
    pub dot_damage: i32,
    pub critical_damage: i32,
    pub critical_hits: i32,
    pub critical_hit_percentage: i32,
    pub critical_damage_percentage: i32,
}

#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = damage_intervals)]
pub struct DamageIntervals {
    pub summary_key: i32,
    pub line_number: i32,
    pub log_date: String,
    pub damage: i32,
    pub delta: i32,
}

#[derive(Queryable, Debug, Clone, Identifiable, Insertable, Selectable, QueryableByName)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = activations_per_power)]
pub struct ActivationsPerPower {
    pub summary_key: i32,
    pub power_name: String,
    pub activations: i32,
}

#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = damage_report_by_power)]
pub struct DamageReportByPower {
    pub summary_key: i32,
    pub power_name: String,
    pub activations: i32,
    pub hits: i32,
    pub streak_breakers: i32,
    pub misses: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    pub hit_percentage: Option<i32>,
    pub power_total_damage: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    pub dpa: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub ate: Option<i32>,
    pub direct_damage: i32,
    pub dot_damage: i32,
    pub critical_damage: i32,
    pub critical_hits: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    pub percent_hits_critical: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub percent_damage_critical: Option<i32>,
}

#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = rewards_defeats)]
pub struct RewardsDefeats {
    pub summary_key: i32,
    pub experience: i32,
    pub influence: i32,
    pub mobs_defeated: i32,
}

#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = damage_taken)]
pub struct DamageTaken {
    pub summary_key: i32,
    pub hits: i32,
    pub misses: i32,
    pub hit_percentage: i32,
    pub total_damage_taken: i32,
    pub damage_per_hit: i32,
}

#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = damage_dealt_by_type)]
pub struct DamageDealtByType {
    pub summary_key: i32,
    pub damage_type: String,
    pub total_damage: i32,
    pub damage_percent: i32,
}

#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = damage_taken_by_type)]
pub struct DamageTakenByType {
    pub summary_key: i32,
    pub damage_type: String,
    pub total_damage: i32,
    pub damage_percent: i32,
}
#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = damage_taken_by_mob)]
pub struct DamageTakenByMob {
    pub summary_key: i32,
    pub source_name: String,
    pub hits: i32,
    pub avg_hit_chance: i32,
    pub total_damage: i32,
    pub damage_per_hit: i32,
}

#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = damage_taken_by_mob_power)]
pub struct DamageTakenByMobPower {
    pub summary_key: i32,
    pub source_name: String,
    pub power_name: String,
    pub damage_type: String,
    pub hits: i32,
    pub avg_hit_chance: i32,
    pub total_damage: i32,
    pub damage_per_hit: i32,
}

#[derive(
    Queryable,
    Debug,
    Clone,
    Identifiable,
    Insertable,
    Selectable,
    QueryableByName,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(summary_key))]
#[diesel(table_name = damage_dealt_to_mob_by_power)]
pub struct DamageDealtToMobByPower {
    pub summary_key: i32,
    pub target_name: String,
    pub power_name: String,
    pub hits: i32,
    pub misses: i32,
    pub chance_to_hit: i32,
    pub hit_percent: i32,
    pub total_damage: i32,
    pub damage_per_hit: i32,
}