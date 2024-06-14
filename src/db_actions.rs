use diesel::connection::SimpleConnection;
use diesel::dsl::not;
use diesel::prelude::*;
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use std::fs;
use std::path::*;

use crate::models::DamageDealtByType;
use crate::models::DamageDealtToMobByPower;
use crate::models::DamageTaken;
use crate::models::DamageTakenByMob;
use crate::models::DamageTakenByMobPower;
use crate::models::DamageTakenByType;
use crate::models::IndexDetails;
use crate::models::RewardsDefeats;
use crate::models::{
    DamageAction, DamageIntervals, DamageReportByPower, DefeatedTarget, HitOrMiss,
    PlayerActivation, Reward, Summary, TotalDamageReport,
};
use crate::parser_model::*;
use crate::schema::{
    damage_action, defeated_targets, hit_or_miss, player_activation, reward, summary,
};
pub fn get_file_conn(path: PathBuf) -> SqliteConnection {
    let mut conn = SqliteConnection::establish(path.to_str().unwrap())
        .unwrap_or_else(|_| panic!("Unable to create in memory database"));

    conn
}

pub fn establish_connection() -> SqliteConnection {
    //let database_url = "summary.db";
    //let mut conn = SqliteConnection::establish(&database_url)
    //    .unwrap_or_else(|_| panic!("Unable to create in memory database"));
    let mut conn = SqliteConnection::establish(":memory:")
        .unwrap_or_else(|_| panic!("Unable to create in memory database"));

    let migrations_harness =
        FileBasedMigrations::find_migrations_directory().expect("Unable to find migrations dir");
    conn.run_pending_migrations(migrations_harness)
        .expect("Unable to migrate db");

    diesel::sql_query("pragma foreign_keys=ON")
        .execute(&mut conn)
        .expect("Could not turn on foreign keys");

    conn
}

pub fn copy_db(conn: &mut SqliteConnection, path: PathBuf) {
    if path.exists() {
        fs::remove_file(&path).expect("Unable to remove old db file");
    }
    let command = format!(
        "VACUUM main INTO '{}'",
        &path.into_os_string().into_string().unwrap()
    );
    println!("{}", command);
    if let Err(e) = diesel::sql_query(command).execute(conn) {
        println!("Unable to copy db: {:?}", e);
    }
}

pub fn write_to_database(
    conn: &mut SqliteConnection,
    file_name: String,
    data_points: &Vec<FileDataPoint>,
) {
    let key = (chrono::offset::Local::now().timestamp() % 1000) as i32;
    let mut summaries: Vec<Summary> = Vec::new();
    let mut activations: Vec<PlayerActivation> = Vec::new();
    let mut hits_misses: Vec<HitOrMiss> = Vec::new();
    let mut damage_actions: Vec<DamageAction> = Vec::new();
    let mut defeats: Vec<DefeatedTarget> = Vec::new();
    let mut rewards: Vec<Reward> = Vec::new();

    // Create placeholder summary
    let placeholder = Summary {
        summary_key: key,
        player_name: String::from("NO NAME"),
        log_date: String::from("PLACEHOLDER"),
        first_line_number: 1,
        last_line_number: data_points.len() as i32,
        log_file_name: String::from(&file_name),
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
                    log_date: data_position.date.to_rfc3339(),
                    first_line_number: data_position.line_number as i32,
                    last_line_number: data_points.len() as i32,
                    log_file_name: String::from(&file_name),
                });
            }
            FileDataPoint::PlayerPowerActivation {
                data_position,
                power_name,
            } => activations.push(PlayerActivation {
                summary_key: key,
                line_number: data_position.line_number as i32,
                log_date: data_position.date.to_rfc3339(),
                power_name: power_name.clone(),
            }),
            FileDataPoint::AutohitPower {
                data_position,
                source: _,
                target: _,
                power_name,
            } => {
                if power_name.contains("Force Feedback") {
                    activations.push(PlayerActivation {
                        summary_key: key,
                        line_number: data_position.line_number as i32,
                        log_date: data_position.date.to_rfc3339(),
                        power_name: power_name.clone(),
                    })
                }
            }
            FileDataPoint::PlayerHit {
                data_position,
                action_result,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
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
                    log_date: data_position.date.to_rfc3339(),
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
                    log_date: data_position.date.to_rfc3339(),
                    hit: 0,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("Player"),
                    source_name: String::from("Player"),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 0,
                });
            }
            FileDataPoint::PseudoPetHit {
                data_position,
                action_result,
                name,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 1,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("PlayerPet"),
                    source_name: name.clone(),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 0,
                });
            }
            FileDataPoint::PseudoPetStreakbreakerHit {
                data_position,
                action_result,
                name,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 1,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("PlayerPet"),
                    source_name: name.clone(),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 1,
                });
            }
            FileDataPoint::PsuedoPetMiss {
                data_position,
                action_result,
                name,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 0,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("PlayerPet"),
                    source_name: name.clone(),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 0,
                });
            }
            FileDataPoint::MobHit {
                data_position,
                action_result,
                name,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 1,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("Mob"),
                    source_name: String::from(name),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 0,
                });
            }
            FileDataPoint::MobAutoHit {
                data_position,
                action_result,
                name,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 1,
                    chance_to_hit: 100,
                    source_type: String::from("Mob"),
                    source_name: String::from(name),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 0,
                });
            }
            FileDataPoint::MobMiss {
                data_position,
                action_result,
                name,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 0,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("Mob"),
                    source_name: String::from(name),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 0,
                });
            }
            FileDataPoint::MobPseudoPetHit {
                data_position,
                action_result,
                name,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 1,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("Mob"),
                    source_name: String::from(name),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 0,
                });
            }
            FileDataPoint::MobPseudoPetMiss {
                data_position,
                action_result,
                name,
            } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 0,
                    chance_to_hit: action_result.chance_to_hit.round() as i32,
                    source_type: String::from("Mob"),
                    source_name: String::from(name),
                    target_name: action_result.target.clone(),
                    power_name: action_result.power_name.clone(),
                    streakbreaker: 0,
                });
            }
            FileDataPoint::PlayerDirectDamage {
                data_position,
                damage_dealt,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("Direct"),
                    source_type: String::from("Player"),
                    source_name: String::from("Player"),
                });

                if damage_dealt.power_name.contains("Chance for")
                    || damage_dealt.power_name.contains("Spider's Bite")
                {
                    activations.push(PlayerActivation {
                        summary_key: key,
                        line_number: data_position.line_number as i32,
                        log_date: data_position.date.to_rfc3339(),
                        power_name: damage_dealt.power_name.clone(),
                    });
                }
            }
            FileDataPoint::PlayerDamageDoT {
                data_position,
                damage_dealt,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("DoT"),
                    source_type: String::from("Player"),
                    source_name: String::from("Player"),
                });
                if damage_dealt.power_name.contains("Interface")
                {
                    activations.push(PlayerActivation {
                        summary_key: key,
                        line_number: data_position.line_number as i32,
                        log_date: data_position.date.to_rfc3339(),
                        power_name: damage_dealt.power_name.clone(),
                    });
                }
            }
            FileDataPoint::PlayerCriticalDamage {
                data_position,
                damage_dealt,
                critical_type: _,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    target_name: damage_dealt.target.clone(),
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
                    log_date: data_position.date.to_rfc3339(),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("Direct"),
                    source_type: String::from("PlayerPet"),
                    source_name: String::from(pet_name),
                });
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 1,
                    chance_to_hit: 100,
                    source_type: String::from("PlayerPet"),
                    source_name: String::from(pet_name),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    streakbreaker: 0,
                });
                if damage_dealt.power_name.contains("Chance for")
                    || damage_dealt.power_name.contains("Spider's Bite")
                {
                    activations.push(PlayerActivation {
                        summary_key: key,
                        line_number: data_position.line_number as i32,
                        log_date: data_position.date.to_rfc3339(),
                        power_name: damage_dealt.power_name.clone(),
                    });
                }
            }
            FileDataPoint::PsuedoPetDamageDoT {
                data_position,
                damage_dealt,
                pet_name,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("DoT"),
                    source_type: String::from("PlayerPet"),
                    source_name: String::from(pet_name),
                });
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 1,
                    chance_to_hit: 100,
                    source_type: String::from("PlayerPet"),
                    source_name: String::from(pet_name),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    streakbreaker: 0,
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
                    log_date: data_position.date.to_rfc3339(),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("Critical"),
                    source_type: String::from("PlayerPet"),
                    source_name: String::from(pet_name),
                });
            }
            FileDataPoint::MobDamage {
                data_position,
                damage_dealt,
                name,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("Direct"),
                    source_type: String::from("Mob"),
                    source_name: String::from(name),
                });
            }
            FileDataPoint::MobDamageDoT {
                data_position,
                damage_dealt,
                name,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("DoT"),
                    source_type: String::from("Mob"),
                    source_name: String::from(name),
                });
            }
            FileDataPoint::MobPseudoPetDamage {
                data_position,
                damage_dealt,
                name,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("Direct"),
                    source_type: String::from("MobPet"),
                    source_name: String::from(name),
                });
            }
            FileDataPoint::MobPseudoPetDamageDoT {
                data_position,
                damage_dealt,
                name,
            } => {
                damage_actions.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    target_name: damage_dealt.target.clone(),
                    power_name: damage_dealt.power_name.clone(),
                    damage: damage_dealt.damage.round() as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    damage_mode: String::from("DoT"),
                    source_type: String::from("MobPet"),
                    source_name: String::from(name),
                });
            }
            FileDataPoint::PlayerFulcrumShift { data_position } => {
                hits_misses.push(crate::models::HitOrMiss {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    hit: 1,
                    chance_to_hit: 0,
                    source_type: String::from("Player"),
                    source_name: String::from("Player"),
                    target_name: String::from("NA"),
                    power_name: String::from("Fulcrum Shift"),
                    streakbreaker: 0,
                });
            }
            FileDataPoint::PlayerVictory {
                data_position,
                target,
            } => {
                defeats.push(DefeatedTarget {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    source_name: String::from("Player"),
                    target_name: String::from(target),
                });
            }
            FileDataPoint::OtherVictory {
                data_position,
                source,
                target,
            } => {
                defeats.push(DefeatedTarget {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.to_rfc3339(),
                    source_name: String::from(source),
                    target_name: String::from(target),
                });
            }
            FileDataPoint::ExpAndInfGain {
                data_position,
                exp,
                inf,
            } => rewards.push(Reward {
                summary_key: key,
                line_number: data_position.line_number as i32,
                log_date: data_position.date.to_rfc3339(),
                experience: Some(*exp as i32),
                influence: Some(*inf as i32),
                item_drop: None,
                reward_type: String::from("ExpAndInf"),
            }),
            _ => (),
        }
    }

    if !summaries.is_empty() {
        insert_summaries(conn, &summaries);

        if !activations.is_empty() {
            insert_activations(conn, &activations);
        }

        if !hits_misses.is_empty() {
            insert_hits_misses(conn, &hits_misses);
        }

        if !damage_actions.is_empty() {
            insert_damage(conn, &damage_actions);
        }

        if !defeats.is_empty() {
            insert_defeats(conn, &defeats);
        }

        if !rewards.is_empty() {
            insert_rewards(conn, &rewards);
        }
        let final_summaries = finalize_summaries(conn, data_points.len(), &summaries[..]);
        finalize_data(conn, &final_summaries[..]);
        cleanup_summaries(conn);
    }
}

pub fn insert_summaries(conn: &mut SqliteConnection, summaries: &Vec<Summary>) {
    diesel::insert_into(summary::table)
        .values(summaries)
        .execute(conn)
        .expect("Error saving new summary");
}

fn insert_activations(conn: &mut SqliteConnection, activations: &Vec<PlayerActivation>) {
    diesel::insert_into(player_activation::table)
        .values(activations)
        .execute(conn)
        .expect("Error saving new activation");
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

fn insert_defeats(conn: &mut SqliteConnection, actions: &Vec<DefeatedTarget>) {
    diesel::insert_into(defeated_targets::table)
        .values(actions)
        .execute(conn)
        .expect("Error saving new damage action");
}

fn insert_rewards(conn: &mut SqliteConnection, rewards: &Vec<Reward>) {
    diesel::insert_into(reward::table)
        .values(rewards)
        .execute(conn)
        .expect("Error saving new damage action");
}

fn finalize_summaries(
    conn: &mut SqliteConnection,
    end_line: usize,
    summaries: &[Summary],
) -> Vec<Summary> {
    use crate::schema::summary::dsl::*;
    let mut start_lines: Vec<i32> = Vec::new();
    for s in summaries {
        start_lines.push(s.first_line_number);
    }

    let mut end_lines: Vec<i32> = start_lines.iter().map(|i| i - 1).collect();

    end_lines.remove(0);
    end_lines.push(end_line as i32);

    for (i, _) in summaries.iter().enumerate() {
        let query = diesel::update(summary)
            .filter(first_line_number.eq(start_lines.get(i).unwrap()))
            .set(last_line_number.eq(end_lines.get(i).unwrap()));
        query.execute(conn).expect("Unable to update summary row");
    }

    conn.batch_execute("update summary set log_date = (select pa.log_date from player_activation pa, summary s where s.summary_key = pa.summary_key AND s.log_date = 'PLACEHOLDER' group by s.summary_key)
    where log_date = 'PLACEHOLDER'").expect("Unable to update date for placeholder summary");

    summary.select(Summary::as_select()).load(conn).unwrap()
}

pub fn get_summaries(conn: &mut SqliteConnection) -> Vec<Summary> {
    use crate::schema::summary::dsl::*;
    summary
        .select(Summary::as_select())
        .load(conn)
        .expect("Unable to load summaries")
}

fn finalize_data(conn: &mut SqliteConnection, summaries: &[Summary]) {
    for s in summaries {
        finalize_activations(conn, s);
        finalize_hits_misses(conn, s);
        finalize_damage_action(conn, s);
        finalize_defeats(conn, s);
        finalize_rewards(conn, s);
    }
}

fn finalize_activations(conn: &mut SqliteConnection, s: &Summary) {
    let gt_ln = line_number.gt(s.first_line_number);
    let lt_ln = line_number.lt(s.last_line_number);

    use crate::schema::player_activation::dsl::*;
    diesel::update(player_activation)
        .filter(gt_ln.and(lt_ln))
        .set((summary_key.eq(s.summary_key),))
        .execute(conn)
        .expect("Unable to update activations");
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
        .expect("Unable to update hit or miss");

    diesel::update(hit_or_miss)
        .filter(gt_ln.and(lt_ln))
        .filter(not(player_hit_miss_pre))
        .set(summary_key.eq(s.summary_key))
        .execute(conn)
        .expect("Unable to update hit or miss");
}

fn finalize_damage_action(conn: &mut SqliteConnection, s: &Summary) {
    let gt_ln = line_number.gt(s.first_line_number);
    let lt_ln = line_number.lt(s.last_line_number);

    use crate::schema::damage_action::dsl::*;
    let player_damage_pre = crate::schema::damage_action::source_type.eq("Player");

    diesel::update(damage_action)
        .filter(gt_ln.and(lt_ln))
        .filter(not(player_damage_pre))
        .set(summary_key.eq(s.summary_key))
        .execute(conn)
        .expect("Unable to update other damage action");

    diesel::update(damage_action)
        .filter(gt_ln.and(lt_ln).and(player_damage_pre))
        .set((
            summary_key.eq(s.summary_key),
            source_name.eq(s.player_name.clone()),
        ))
        .execute(conn)
        .expect("Unable to update player damage action");
}

fn finalize_defeats(conn: &mut SqliteConnection, s: &Summary) {
    let gt_ln = line_number.gt(s.first_line_number);
    let lt_ln = line_number.lt(s.last_line_number);

    use crate::schema::defeated_targets::dsl::*;
    let player_damage_pre = crate::schema::defeated_targets::source_name.eq("Player");

    diesel::update(defeated_targets)
        .filter(gt_ln.and(lt_ln))
        .filter(not(player_damage_pre))
        .set(summary_key.eq(s.summary_key))
        .execute(conn)
        .expect("Unable to update other defeats");

    diesel::update(defeated_targets)
        .filter(gt_ln.and(lt_ln).and(player_damage_pre))
        .set((
            summary_key.eq(s.summary_key),
            source_name.eq(s.player_name.clone()),
        ))
        .execute(conn)
        .expect("Unable to update player defeats");
}

fn finalize_rewards(conn: &mut SqliteConnection, s: &Summary) {
    let gt_ln = line_number.gt(s.first_line_number);
    let lt_ln = line_number.lt(s.last_line_number);

    use crate::schema::reward::dsl::*;

    diesel::update(reward)
        .filter(gt_ln.and(lt_ln))
        .set(summary_key.eq(s.summary_key))
        .execute(conn)
        .expect("Unable to update rewards");
}

fn cleanup_summaries(conn: &mut SqliteConnection) {
    diesel::sql_query("delete from summary as s WHERE summary_key NOT IN (select summary_key from damage_action a where s.summary_key = a.summary_key)")
        .execute(conn)
        .expect("An error has occured");
}

pub fn index_details(conn: &mut SqliteConnection) -> Vec<IndexDetails> {
    use crate::schema::index_details::dsl::*;
    index_details
        .select(IndexDetails::as_select())
        .load(conn)
        .expect("Unable to load index details")
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

pub fn get_damage_dealt_by_type_report(
    conn: &mut SqliteConnection,
    key: i32,
) -> Option<Vec<DamageDealtByType>> {
    use crate::schema::damage_dealt_by_type::dsl::*;
    match damage_dealt_by_type
        .filter(summary_key.eq(key))
        .load::<DamageDealtByType>(conn)
    {
                Ok(data) => {
            if data.is_empty()  {
                None
            } else {
                Some(data)
            }
        }
        Err(_) => None,

    }
}

pub fn get_damage_taken_by_type_report(
    conn: &mut SqliteConnection,
    key: i32,
) -> Option<Vec<DamageTakenByType>> {
    use crate::schema::damage_taken_by_type::dsl::*;
    match damage_taken_by_type
        .filter(summary_key.eq(key))
        .load::<DamageTakenByType>(conn)
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

pub fn get_damage_taken_by_mob_report(
    conn: &mut SqliteConnection,
    key: i32,
) -> Option<Vec<DamageTakenByMob>> {
    use crate::schema::damage_taken_by_mob::dsl::*;
    match damage_taken_by_mob
        .filter(summary_key.eq(key))
        .load::<DamageTakenByMob>(conn)
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

pub fn get_damage_taken_by_mob_power_report(
    conn: &mut SqliteConnection,
    key: i32,
) -> Option<Vec<DamageTakenByMobPower>> {
    use crate::schema::damage_taken_by_mob_power::dsl::*;
    match damage_taken_by_mob_power
        .filter(summary_key.eq(key))
        .load::<DamageTakenByMobPower>(conn)
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

pub fn get_damage_dealt_to_mob_by_power_report(
    conn: &mut SqliteConnection,
    key: i32,
) -> Option<Vec<DamageDealtToMobByPower>> {
    use crate::schema::damage_dealt_to_mob_by_power::dsl::*;
    match damage_dealt_to_mob_by_power
        .filter(summary_key.eq(key))
        .load::<DamageDealtToMobByPower>(conn)
    {
        Ok(data) => Some(data),
        Err(_) => None,
    }
}

pub fn get_damage_by_power_report(
    conn: &mut SqliteConnection,
    key: i32,
) -> Vec<DamageReportByPower> {
    use crate::schema::damage_report_by_power::dsl::*;

    damage_report_by_power
        .filter(summary_key.eq(key))
        .load(conn)
        .expect("Unable to load damage report by power")
}

pub fn select_damage_intervals(conn: &mut SqliteConnection) -> Vec<DamageIntervals> {
    use crate::schema::damage_intervals::dsl::*;
    damage_intervals
        .select(DamageIntervals::as_select())
        .load(conn)
        .expect("Unable to load damage report by power")
}

pub fn get_damage_intervals(
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
