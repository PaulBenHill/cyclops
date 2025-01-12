use std::path::PathBuf;

use crate::schema::last_interesting_date::log_date;
use chrono::DateTime;
use chrono::Local;
use diesel::prelude::*;
use diesel::SqliteConnection;

use crate::db::get_file_conn;
use crate::models::{
    DamageDealtByType, DamageDealtToMobByPower, DamageIntervals, DamageReportByPower, DamageTaken,
    DamageTakenByMob, DamageTakenByMobPower, DamageTakenByType, IndexDetails, RewardsDefeats,
    Summary, TotalDamageReport, PlayerPowerRecharged, PlayerActivation, SessionStats
};
use crate::web::web_structs_enums::DamageByPowerQuery;
use crate::web::web_structs_enums::PowersMobsData;
use crate::web::web_structs_enums::TableQuery;

pub fn get_summary(conn: &mut SqliteConnection, key: i32) -> Vec<Summary> {
    use crate::schema::summary::dsl::*;
    summary
        .filter(summary_key.eq(key))
        .load(conn)
        .expect("Unable to load single summary")
}

pub fn get_summaries(conn: &mut SqliteConnection) -> Vec<Summary> {
    use crate::schema::summary::dsl::*;
    summary
        .select(Summary::as_select())
        .order_by(log_date.asc())
        .load(conn)
        .expect("Unable to load summaries")
}

pub fn index_details(conn: &mut SqliteConnection) -> Vec<IndexDetails> {
    use crate::schema::index_details::dsl::*;
    index_details
        .select(IndexDetails::as_select())
        .load(conn)
        .expect("Unable to load index details")
}

pub fn get_total_damage(query: &DamageByPowerQuery) -> i32 {
    use crate::schema::total_damage_report::dsl::*;
    let db_path: PathBuf = query.db_path.clone().into();
    let mut conn = get_file_conn(db_path);

    let mut result: Vec<i32> = total_damage_report
        .select(total_damage)
        .filter(summary_key.eq(query.key))
        .load(&mut conn)
        .expect("Unable to load total damage report");

    result.pop().unwrap()
}

pub fn get_total_damage_report(conn: &mut SqliteConnection, key: i32) -> TotalDamageReport {
    use crate::schema::total_damage_report::dsl::*;
    let mut result: Vec<TotalDamageReport> = total_damage_report
        .filter(summary_key.eq(key))
        .load(conn)
        .expect("Unable to load total damage report");

    result.pop().unwrap()
}

pub fn get_damage_taken_report(conn: &mut SqliteConnection, key: i32) -> Option<DamageTaken> {
    use crate::schema::damage_taken::dsl::*;
    match damage_taken
        .filter(summary_key.eq(key))
        .load::<DamageTaken>(conn)
    {
        Ok(mut data) => data.pop(),
        Err(_) => None,
    }
}

pub fn get_damage_dealt_by_type_query(query: &TableQuery) -> Option<Vec<DamageDealtByType>> {
    use crate::schema::damage_dealt_by_type::dsl::*;
    let db_path: PathBuf = query.db_path.clone().into();
    let mut conn = get_file_conn(db_path);

    match damage_dealt_by_type
        .filter(summary_key.eq(query.key))
        .load::<DamageDealtByType>(&mut conn)
    {
        Ok(data) => {
            if data.is_empty() {
                None
            } else {
                Some(data)
            }
        }
        Err(_) => None,
    }
}

pub fn get_damage_taken_by_type_query(query: &TableQuery) -> Option<Vec<DamageTakenByType>> {
    use crate::schema::damage_taken_by_type::dsl::*;
    let db_path: PathBuf = query.db_path.clone().into();
    let mut conn = get_file_conn(db_path);

    match damage_taken_by_type
        .filter(summary_key.eq(query.key))
        .load::<DamageTakenByType>(&mut conn)
    {
        Ok(data) => {
            if data.is_empty() {
                None
            } else {
                Some(data)
            }
        }
        Err(_) => None,
    }
}

pub fn get_damage_taken_by_mob_query(query: &TableQuery) -> Option<Vec<DamageTakenByMob>> {
    use crate::schema::damage_taken_by_mob::dsl::*;
    let db_path: PathBuf = query.db_path.clone().into();
    let mut conn = get_file_conn(db_path);

    match damage_taken_by_mob
        .filter(summary_key.eq(query.key))
        .load::<DamageTakenByMob>(&mut conn)
    {
        Ok(data) => {
            if data.is_empty() {
                None
            } else {
                Some(data)
            }
        }
        Err(_) => None,
    }
}

pub fn get_damage_taken_by_mob_power_query(
    query: &TableQuery,
) -> Option<Vec<DamageTakenByMobPower>> {
    use crate::schema::damage_taken_by_mob_power::dsl::*;
    let db_path: PathBuf = query.db_path.clone().into();
    let mut conn = get_file_conn(db_path);

    match damage_taken_by_mob_power
        .filter(summary_key.eq(query.key))
        .load::<DamageTakenByMobPower>(&mut conn)
    {
        Ok(data) => {
            if data.is_empty() {
                None
            } else {
                Some(data)
            }
        }
        Err(_) => None,
    }
}

pub fn get_damage_dealt_by_power_or_mob(
    query: &PowersMobsData,
) -> Option<Vec<DamageDealtToMobByPower>> {
    use crate::schema::damage_dealt_to_mob_by_power::dsl::*;
    let db_path: PathBuf = query.db_path.clone().into();
    let mut conn = get_file_conn(db_path);

    if query.power_name.is_some() && !query.power_name.as_ref().unwrap().is_empty() {
        Some(
            damage_dealt_to_mob_by_power
                .filter(
                    summary_key
                        .eq(query.key)
                        .and(power_name.eq(query.power_name.clone().unwrap().replace("_", " "))),
                )
                .order(total_damage.desc())
                .load::<DamageDealtToMobByPower>(&mut conn)
                .expect("Unable to load damage report by power"),
        )
    } else if query.mob_name.is_some() && !query.mob_name.as_ref().unwrap().is_empty() {
        Some(
            damage_dealt_to_mob_by_power
                .filter(
                    summary_key
                        .eq(query.key)
                        .and(target_name.eq(query.mob_name.clone().unwrap().replace("_", " "))),
                )
                .order(total_damage.desc())
                .load::<DamageDealtToMobByPower>(&mut conn)
                .expect("Unable to load damage report by power"),
        )
    } else {
        None
    }
}

pub fn get_damage_by_power_report(query: &DamageByPowerQuery) -> Vec<DamageReportByPower> {
    use crate::schema::damage_report_by_power::dsl::*;
    let db_path: PathBuf = query.db_path.clone().into();
    let mut conn = get_file_conn(db_path);

    damage_report_by_power
        .filter(summary_key.eq(query.key))
        .load(&mut conn)
        .expect("Unable to load damage report by power")
}

pub fn select_damage_intervals(conn: &mut SqliteConnection) -> Vec<DamageIntervals> {
    use crate::schema::damage_intervals::dsl::*;
    damage_intervals
        .select(DamageIntervals::as_select())
        .load(conn)
        .expect("Unable to load damage report by power")
}

pub fn get_damage_intervals_query(
    conn: &mut SqliteConnection,
    key: i32,
    interval: i32,
) -> Vec<Vec<DamageIntervals>> {
    use crate::schema::damage_intervals::dsl::*;

    let intervals: Vec<DamageIntervals> = damage_intervals
        .filter(summary_key.eq(key))
        .load(conn)
        .expect("Unable to load damage intervals");

    let mut result: Vec<Vec<DamageIntervals>> = Vec::new();
    let mut current_interval: Vec<DamageIntervals> = Vec::new();

    for i in intervals {
        current_interval.push(i.to_owned());
        if i.delta >= interval {
            result.push(current_interval);
            current_interval = Vec::new();
        }
    }
    if !current_interval.is_empty() {
        result.push(current_interval);
    }

    result
}

pub fn get_rewards_defeats(
    conn: &mut SqliteConnection,
    key: i32,
    player_name: &str,
) -> RewardsDefeats {
    use diesel::sql_query;
    use diesel::sql_types::*;

    let reward_query = sql_query("select r.summary_key, sum(r.experience) as experience, sum( influence) as influence, mobs_defeated from reward r INNER JOIN (select count(dt.summary_key) as mobs_defeated from defeated_targets dt where dt.summary_key = ? AND dt.source_name = ?) where r.summary_key = ? group by summary_key");
    let result = reward_query
        .bind::<Integer, _>(key)
        .bind::<Text, _>(player_name)
        .bind::<Integer, _>(key)
        .get_result::<RewardsDefeats>(conn);

    match result {
        Ok(data) => data,
        Err(_) => RewardsDefeats {
            influence: 0,
            summary_key: 0,
            experience: 0,
            mobs_defeated: 0,
        },
    }
}

pub fn get_damaging_powers(query: &PowersMobsData) -> Vec<String> {
    use crate::schema::damage_action::dsl::*;

    let db_path: PathBuf = query.db_path.clone().into();
    let mut conn = get_file_conn(db_path);

    let source_types: Vec<&str> = vec!["Player", "PlayerPet"];
    let result = damage_action
        .select(power_name)
        .distinct()
        .filter(summary_key.eq(query.key))
        .filter(source_type.eq_any(source_types))
        .order(power_name.asc())
        .load::<String>(&mut conn);

    match result {
        Ok(names) => names,
        Err(_) => panic!("Unable to load power names"),
    }
}

pub fn get_mobs_damaged(query: &PowersMobsData) -> Vec<String> {
    use crate::schema::damage_action::dsl::*;

    let db_path: PathBuf = query.db_path.clone().into();
    let mut conn = get_file_conn(db_path);

    let source_types: Vec<&str> = vec!["Player", "PlayerPet"];
    let result = damage_action
        .select(target_name)
        .distinct()
        .filter(summary_key.eq(query.key))
        .filter(source_type.eq_any(source_types))
        .order(target_name.asc())
        .load::<String>(&mut conn);

    match result {
        Ok(names) => names,
        Err(_) => panic!("Unable to load power names"),
    }
}

pub fn get_last_interesting_date(conn: &mut SqliteConnection) -> DateTime<Local> {
    use crate::schema::last_interesting_date::dsl::last_interesting_date;

    let result = last_interesting_date.select(log_date).load::<String>(conn);

    match result {
        Ok(list) => {
            let date = list.first().unwrap();
            date.parse().unwrap()
        }
        Err(e) => {
            panic!("Unable to load last interesting date!: {:?}", e);
        }
    }
}

pub fn get_last_activation(
    conn: &mut SqliteConnection,
    power_text: &String,
    time_point: DateTime<Local>,
) -> Option<PlayerActivation> {
    use crate::schema::player_activation::dsl::*;

    let result = player_activation
        .select(PlayerActivation::as_select())
        .filter(power_name.eq(power_text))
        .order_by(log_date.desc())
        .limit(1)
        .load(conn)
        .expect("Unable to load last player activation");


    match result.first() {
        Some(activation) => {
            let activation_date: DateTime<Local> = activation.log_date.parse().unwrap();
            if activation_date.timestamp() > time_point.timestamp() {
                Some(activation.clone())
            } else {
                None
            }
        }
        None => None
    }
}

pub fn get_last_recharge(
    conn: &mut SqliteConnection,
    power_text: &String,
    time_point: DateTime<Local>,
) -> Option<PlayerPowerRecharged> {
    use crate::schema::player_power_recharged::dsl::*;

    let result = player_power_recharged
        .select(PlayerPowerRecharged::as_select())
        .filter(power_name.eq(power_text))
        .order_by(log_date.desc())
        .limit(1)
        .load(conn)
        .expect("Unable to load last player recharge");

    
    match result.first() {
        Some(recharge) => {
            let recharge_date: DateTime<Local> = recharge.log_date.parse().unwrap();
            if recharge_date.timestamp() > time_point.timestamp() {
                Some(recharge.clone())
            } else {
                None
            }
        }
        None => None
    }
}

pub fn get_session_stats(conn: &mut SqliteConnection, key: i32) -> Option<SessionStats> {
    use crate::schema::session_stats::dsl::*;

    match session_stats
        .filter(summary_key.eq(key))
        .load::<SessionStats>(conn)
    {
        Ok(result) => {
            if result.is_empty() {
                None
            } else {
                Some(result.first().unwrap().clone())
            }
        }
        Err(_) => None,
    }
}
