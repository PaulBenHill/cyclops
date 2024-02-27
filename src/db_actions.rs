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

pub fn write_to_database(data_points: Vec<FileDataPoint>) {
    let mut conn = establish_connection();
}

//fn insert_damage(
//    key: i32,
//    line_number: u32,
//    date: DateTime<Local>,
//    target: &str,
//    power_name: &str,
//    damage_str: &str,
//    damage_type: &str,
//    source_type: &str,
//    source_name: &str,
//) {
//    let mut conn = establish_connection();
//    // todo!("Critical")
//    let action = DamageAction {
//        summary_key: key,
//        line_number: line_number as i32,
//        log_date: date.timestamp() as i32,
//        target: String::from(target),
//        power_name: String::from(power_name),
//        damage: damage_str.parse().unwrap(),
//        damage_type: String::from(damage_type),
//        source_type: String::from("Player"),
//        source_name: Some(String::from(source_name)),
//    };
//
//    create_damage(&mut conn, action);
//}
//
//fn insert_summary(session_marker: &FileDataPoint) {
//    let mut conn = establish_connection();
//    match session_marker {
//        FileDataPoint::SessionMarker {
//            data_position,
//            player_name,
//        } => {
//            let summary_row = Summary {
//                summary_key: data_position.date.timestamp() as i32,
//                player_name: String::from(player_name),
//                log_date: data_position.date.to_rfc2822(),
//                line_number: data_position.line_number as i32,
//                log_file_name: "NA".to_string(),
//            };
//            create_summary(&mut conn, summary_row);
//        }
//        _ => (),
//    }
//}
