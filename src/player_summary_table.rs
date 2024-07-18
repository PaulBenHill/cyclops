use std::path::PathBuf;

use serde::Deserialize;
use tera::Context;

use crate::{db_actions, web::TableNames, AppContext};

#[derive(Deserialize, Debug)]
pub struct SummaryQuery {
    pub key: i32,
    pub db_path: String,
}

pub fn process(app_context: &AppContext, report_context: &mut Context, query: &SummaryQuery) {
    let db_path: PathBuf = query.db_path.clone().into();
    let mut conn = db_actions::get_file_conn(db_path);

    let binding = db_actions::get_summary(&mut conn, query.key);
    let summary = binding.first().unwrap();
    report_context.insert("db_path", &query.db_path);

    report_context.insert("summary", &summary);
    report_context.insert(
        "rewards_defeats",
        &db_actions::get_rewards_defeats(&mut conn, summary.summary_key, &summary.player_name),
    );
    report_context.insert(
        "total_damage",
        &db_actions::get_total_damage_report(&mut conn, summary.summary_key),
    );
    if let Some(damage_taken) = db_actions::get_damage_taken_report(&mut conn, summary.summary_key) {
        report_context.insert("damage_taken", &damage_taken);
    }
    report_context.insert("dps_interval", &app_context.dps_interval);
    report_context.insert("dps_report", &TableNames::DPSIntervals);
    report_context.insert("damage_dealt_by_type", &TableNames::DamageDealtByType);
    report_context.insert("damage_taken_by_type", &TableNames::DamageTakenByType);
    report_context.insert("damage_taken_by_mob", &TableNames::DamageTakenByMob);
    report_context.insert(
        "damage_taken_by_mob_power",
        &TableNames::DamageTakenByMobPower,
    );

}
