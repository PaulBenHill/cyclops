pub mod event_processing;
pub mod queries;

use std::fs;
use std::path::PathBuf;

use diesel::{Connection, RunQueryDsl, SqliteConnection};
use diesel_migrations::FileBasedMigrations;
use diesel_migrations::MigrationHarness;

pub fn get_file_conn(path: PathBuf) -> SqliteConnection {
    let mut conn = SqliteConnection::establish(path.to_str().unwrap())
        .unwrap_or_else(|_| panic!("Unable to connect to database"));

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