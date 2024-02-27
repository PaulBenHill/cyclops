use chrono::{self, DateTime, Local};
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

use crate::models::{DamageAction, Summary};
use crate::parser_model::*;
use crate::schema::{damage_action, summary};

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_summary(conn: &mut SqliteConnection, summary: Summary) -> Summary {
    diesel::insert_into(summary::table)
        .values(&summary)
        .returning(Summary::as_returning())
        .get_result(conn)
        .expect("Error saving new summary")
}

pub fn create_damage(conn: &mut SqliteConnection, damage_action: DamageAction) -> DamageAction {
    diesel::insert_into(damage_action::table)
        .values(&damage_action)
        .returning(DamageAction::as_returning())
        .get_result(conn)
        .expect("Error saving new damage action")
}

pub fn write_to_database(file_name: &String, data_points: &Vec<FileDataPoint>) {
    let mut conn = establish_connection();
    let key = 42;
    let mut damage_action: Vec<DamageAction> = Vec::new();

    // Create placeholder summary
    let placeholder = Summary {
        summary_key: key,
        player_name: String::from("NO NAME"),
        log_date: String::from("PLACEHOLDER"),
        line_number: 42,
        log_file_name: "PLACEHOLDER".to_string(),
    };
    create_summary(&mut conn, placeholder);

    for dp in data_points {
        match dp {
            FileDataPoint::SessionMarker {
                data_position,
                player_name,
            } => {
                let summary_row = Summary {
                    summary_key: data_position.date.timestamp() as i32,
                    player_name: String::from(player_name),
                    log_date: data_position.date.to_rfc2822(),
                    line_number: data_position.line_number as i32,
                    log_file_name: "NA".to_string(),
                };
                create_summary(&mut conn, summary_row);
            }
            FileDataPoint::PlayerDirectDamage {
                data_position,
                damage_dealt,
            } => {
                damage_action.push(DamageAction {
                    summary_key: key,
                    line_number: data_position.line_number as i32,
                    log_date: data_position.date.timestamp() as i32,
                    target: String::from(&damage_dealt.target),
                    power_name: String::from(&damage_dealt.power_name),
                    damage: damage_dealt.damage as i32,
                    damage_type: damage_dealt.damage_type.to_string(),
                    source_type: String::from("Player"),
                    source_name: Some(String::from("No name")),
                });
            }
            _ => (),
        }
    }

    if !damage_action.is_empty() {
        insert_damage(&mut conn, damage_action);
    }
}

fn insert_damage(conn: &mut SqliteConnection, actions: Vec<DamageAction>) {
    // todo!("Critical")

    diesel::insert_into(damage_action::table)
        .values(&actions)
        .execute(conn)
        .expect("Error saving new damage action");
}
