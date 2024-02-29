use chrono::{self, DateTime, Local};
use diesel::dsl::not;
use diesel::{debug_query, prelude::*};
use dotenvy::dotenv;
use std::env;

use crate::models::{DamageAction, HitOrMiss, Summary};
use crate::parser_model::*;
use crate::schema::damage_action::{line_number, source_name, summary_key};
use crate::schema::summary::{first_line_number, last_line_number};
use crate::schema::{damage_action, hit_or_miss, summary};

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn write_to_database(file_name: &String, data_points: &Vec<FileDataPoint>) {
    let mut conn = establish_connection();
    let key = 42;
    let mut summaries: Vec<Summary> = Vec::new();
    let mut damage_actions: Vec<DamageAction> = Vec::new();
    let mut hits_misses: Vec<HitOrMiss> = Vec::new();

    // Create placeholder summary
    let placeholder = Summary {
        summary_key: key,
        player_name: String::from("NO NAME"),
        log_date: String::from("PLACEHOLDER"),
        first_line_number: 1,
        last_line_number: i32::MAX,
        log_file_name: file_name.clone(),
    };
    summaries.push(placeholder);

    for dp in data_points {
        match dp {
            FileDataPoint::SessionMarker {
                data_position,
                player_name,
            } => {
                summaries.push(Summary {
                    summary_key: data_position.date.timestamp() as i32,
                    player_name: player_name.clone(),
                    log_date: data_position.date.to_rfc2822(),
                    first_line_number: data_position.line_number as i32,
                    last_line_number: i32::MAX,
                    log_file_name: file_name.clone(),
                });
            }
            FileDataPoint::PlayerDirectDamage {
                data_position,
                damage_dealt,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc2822(),
                    target: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("Direct"),
                    source_type: String::from("Player"),
                    source_name: String::from("Player"),
                });
            }
            FileDataPoint::PlayerDamageDoT {
                data_position,
                damage_dealt,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc2822(),
                    target: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("DoT"),
                    source_type: String::from("Player"),
                    source_name: String::from("Player"),
                });
            }
            FileDataPoint::PlayerCriticalDamage {
                data_position,
                damage_dealt,
                critical_type: _,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc2822(),
                    target: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("Critical"),
                    source_type: String::from("Player"),
                    source_name: String::from("Player"),
                });
            }
            FileDataPoint::PseudoPetDirectDamage {
                data_position,
                damage_dealt,
                pet_name,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc2822(),
                    target: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("Direct"),
                    source_type: String::from("PlayerPet"),
                    source_name: String::from(pet_name),
                });
            }
            FileDataPoint::PsuedoPetDamageDoT {
                data_position,
                damage_dealt,
                pet_name,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc2822(),
                    target: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("DoT"),
                    source_type: String::from("PlayerPet"),
                    source_name: String::from(pet_name),
                });
            }
            FileDataPoint::PsuedoPetCriticalDamage {
                data_position,
                damage_dealt,
                pet_name,
                critical_type: _,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc2822(),
                    target: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("Critical"),
                    source_type: String::from("PlayerPet"),
                    source_name: String::from(pet_name),
                });
            }
            FileDataPoint::PlayerHit {
                data_position,
                action_result,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc2822(),
                    hit: 1,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("Player"),
                    source_name: String::from("Player"),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 0,
                });
            }
            FileDataPoint::PlayerStreakbreakerHit {
                data_position,
                action_result,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc2822(),
                    hit: 1,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("Player"),
                    source_name: String::from("Player"),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 1,
                });
            }
            FileDataPoint::PlayerMiss {
                data_position,
                action_result,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc2822(),
                    hit: 0,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("Player"),
                    source_name: String::from("Player"),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 0,
                });
            }
            _ => (),
        }
    }

    if !summaries.is_empty() {
        insert_summaries(&mut conn, &summaries);

        if !hits_misses.is_empty() {
            insert_hits_misses(&mut conn, &hits_misses);
        }

        if !damage_actions.is_empty() {
            insert_damage(&mut conn, &damage_actions);
        }
        let final_summaries = finalize_summaries(&mut conn, &summaries[..]);
        finalize_data(&mut conn, &final_summaries[..]);
    }
}

pub fn insert_summaries(conn: &mut SqliteConnection, summaries: &Vec<Summary>) {
    diesel::insert_into(summary::table)
        .values(summaries)
        .execute(conn)
        .expect("Error saving new summary");
}

fn insert_hits_misses(conn: &mut SqliteConnection, hits_misses: &Vec<HitOrMiss>) {
    diesel::insert_into(hit_or_miss::table)
        .values(hits_misses)
        .execute(conn)
        .expect("Error saving new hit or miss");
}

fn insert_damage(conn: &mut SqliteConnection, actions: &Vec<DamageAction>) {
    diesel::insert_into(damage_action::table)
        .values(actions)
        .execute(conn)
        .expect("Error saving new damage action");
}

fn finalize_summaries(conn: &mut SqliteConnection, summaries: &[Summary]) -> Vec<Summary> {
    let mut start_lines: Vec<i32> = Vec::new();
    for s in summaries {
        start_lines.push(s.first_line_number.clone());
    }

    let mut end_lines: Vec<i32> = start_lines.iter().map(|i| i - 1).collect();

    end_lines.remove(0);
    end_lines.push(i32::MAX);

    for (i, _) in summaries.iter().enumerate() {
        let query = diesel::update(summary)
            .filter(first_line_number.eq(start_lines.get(i).unwrap()))
            .set(last_line_number.eq(end_lines.get(i).unwrap()));
        query.execute(conn).expect("Unable to update summary row");
    }

    use crate::schema::summary::dsl::*;
    summary.select(Summary::as_select()).load(conn).unwrap()
}

fn finalize_data(conn: &mut SqliteConnection, summaries: &[Summary]) {
    for s in summaries {
        finalize_damage_action(conn, s);
        finalize_hits_misses(conn, s);
    }
}

fn finalize_damage_action(conn: &mut SqliteConnection, s: &Summary) {
    let gt_ln = line_number.gt(s.first_line_number);
    let lt_ln = line_number.lt(s.last_line_number);

    use crate::schema::damage_action::dsl::*;
    let player_damage_pre = crate::schema::damage_action::source_type.eq("Player");
    diesel::update(damage_action)
        .filter(gt_ln.and(lt_ln).and(player_damage_pre))
        .set((
            summary_key.eq(s.summary_key),
            source_name.eq(s.player_name.clone()),
        ))
        .execute(conn)
        .expect("Unable to update damage_action");

    diesel::update(damage_action)
        .filter(gt_ln.and(lt_ln))
        .filter(not(player_damage_pre))
        .set(summary_key.eq(s.summary_key))
        .execute(conn)
        .expect("Unable to update damage_action");
}

fn finalize_hits_misses(conn: &mut SqliteConnection, s: &Summary) {
    let gt_ln = line_number.gt(s.first_line_number);
    let lt_ln = line_number.lt(s.last_line_number);

    use crate::schema::hit_or_miss::dsl::*;
    let player_hit_miss_pre = crate::schema::hit_or_miss::source_type.eq("Player");
    diesel::update(hit_or_miss)
        .filter(gt_ln.and(lt_ln).and(player_hit_miss_pre))
        .set((
            summary_key.eq(s.summary_key),
            source_name.eq(s.player_name.clone()),
        ))
        .execute(conn)
        .expect("Unable to update damage_action");

    diesel::update(hit_or_miss)
        .filter(gt_ln.and(lt_ln))
        .filter(not(player_hit_miss_pre))
        .set(summary_key.eq(s.summary_key))
        .execute(conn)
        .expect("Unable to update damage_action");
}
