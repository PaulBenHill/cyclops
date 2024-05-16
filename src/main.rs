use chrono::DateTime;
use clap::Parser;
use current_platform::{COMPILED_ON, CURRENT_PLATFORM};
use models::{IndexDetails, Summary};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Lines, Write};
use std::path::*;
use std::time::{Duration, Instant};
use std::{env, fs};
use tera::{Result, Value};
use walkdir::WalkDir;

use diesel::SqliteConnection;

mod parsers;
use tera::{Context, Tera};

use crate::parser_model::FileDataPoint;
use crate::parsers::*;

mod args;
mod db_actions;
mod log_processing;
mod models;
mod parser_model;
mod schema;
mod web;

const OUTPUT_DIR: &str = "output";
const TEMPLATES: &str = "templates";

// Todos
// File processing in index page
//  status updates
// charts
// windows dialog projects exist
// logdir multiple
// file or log dir not required
fn main() {
    let start = Instant::now();

    let working_dir = env::current_dir().unwrap().clone();
    println!(
        "Cyclops was compiled on {}:{}.",
        CURRENT_PLATFORM, COMPILED_ON
    );
    println!("The current directory is {}", working_dir.display());

    let args = args::Args::parse();

    let mut log_file_names: Vec<String> = Vec::new();
    if let Some(log_dirs) = args.logdir {
        println!("Value for log dir: {:?}", log_dirs);
        for dir in log_dirs {
            log_file_names.append(&mut read_log_file_dir(dir.to_path_buf()));
        }
    } else if let Some(files) = args.files {
        for path in files {
            log_file_names.push(path.into_os_string().into_string().unwrap());
        }
    }

    if log_file_names.is_empty() {
        println!("No chat logs found. Exiting");
        std::process::exit(1);
    }

    let mut output_path = PathBuf::new().join(OUTPUT_DIR);
    if let Some(outputdir) = args.outputdir {
        println!("Value for output dir: {:?}", outputdir);
        output_path = outputdir.to_path_buf();
    }

    let mut dps_interval = 60;
    if let Some(interval_arg) = args.interval {
        println!("Value for interval: {:?}", interval_arg);
        dps_interval = interval_arg;
    }

    let mut webserver_address: String = String::from("127.0.0.1");
    if let Some(address_arg) = args.address {
        println!("Value for web server address: {:?}", address_arg);
        webserver_address = address_arg;
    }

    let mut webserver_port: usize = 11227;
    if let Some(port_arg) = args.port {
        println!("Value for web server address: {:?}", port_arg);
        webserver_port = port_arg;
    }

    log_processing::create_dir(&output_path);
    println!("Output directory: {}", output_path.display());

    let tera = setup_tera();

    log_processing::process_logs(
        &tera,
        &working_dir,
        &output_path,
        dps_interval,
        log_file_names,
    );

    println!(
        "File(s) processing time took: {} second.",
        start.elapsed().as_secs()
    );

    let indexes = find_all_summaries(&output_path);
    generate_index(&tera, &output_path, &indexes);

    println!(
        "Starting web server at http://{}:{}",
        webserver_address, webserver_port
    );
    let output_dir: String = String::from(output_path.to_str().unwrap());
    if let Err(e) = web::start(webserver_address, webserver_port, output_dir.to_owned()) {
        panic!("Unable to start web server {:?}", e);
    }

    println!("Total run time took: {} second.", start.elapsed().as_secs());
}

fn generate_index(tera: &Tera, output_dir: &PathBuf, indexes: &Vec<SummaryEntry>) {
    let mut index_file = match File::create(output_dir.join("index.html")) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create summary.txt file: {:?}", e),
    };

    let mut index_content = Context::new();
    index_content.insert("indexes", &indexes);
    let result = tera.render("index.html", &index_content);
    match result {
        Ok(data) => {
            index_file
                .write_all(data.as_bytes())
                .expect("Unable to write file.");
        }
        Err(e) => panic!("Could not render {}:{:?}", "index.html", e),
    }
}

#[derive(Serialize, Deserialize)]
pub struct SummaryEntry {
    details: IndexDetails,
    path: PathBuf,
}

fn find_all_summaries(output_path: &Path) -> Vec<SummaryEntry> {
    let mut entries: Vec<SummaryEntry> = Vec::new();
    let walker = WalkDir::new(output_path).into_iter();
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.path().ends_with("summary.db") {
            let mut conn =
                db_actions::get_file_conn(fs::canonicalize(entry.path()).unwrap().to_path_buf());
            let i = db_actions::index_details(&mut conn)
                .get(0)
                .unwrap()
                .to_owned();

            let mut html_file = entry
                .path()
                .strip_prefix(output_path)
                .unwrap()
                .to_path_buf();
            html_file.set_file_name("summary.html");

            let entry = SummaryEntry {
                details: i,
                path: html_file,
            };
            entries.push(entry);
        }
    }

    entries.sort_by(|a, b| b.details.log_date.cmp(&a.details.log_date));
    entries
}

fn setup_tera() -> Tera {
    match Tera::new(&format!("{}{}*.html", TEMPLATES, std::path::MAIN_SEPARATOR)) {
        Ok(mut t) => {
            t.autoescape_on(vec![]);
            t.register_function("calc_percentage", calc_percentage);
            t
        }
        Err(e) => panic!("Unable to load templates: {:?}", e),
    }
}

fn read_log_file_dir(dir: PathBuf) -> Vec<String> {
    match fs::canonicalize(&dir) {
        Ok(path) => {
            if path.exists() && path.is_dir() {
                let mut file_list: Vec<String> = fs::read_dir(path)
                    .unwrap()
                    .filter(|r| r.is_ok())
                    .map(|r| r.unwrap().path())
                    .filter(|r| r.is_file())
                    .map(|r| {
                        dunce::canonicalize(r)
                            .unwrap()
                            .into_os_string()
                            .into_string()
                            .unwrap()
                    })
                    .filter(|r| r.ends_with("txt"))
                    .collect();

                file_list
            } else {
                panic!(
                    "Log file directory does not exist or is not a directory: {:?}",
                    dir
                );
            }
        }
        Err(e) => panic!("Cannot determine directory name: {:?}:{:?}", dir, e),
    }
}

pub fn calc_percentage(args: &HashMap<String, Value>) -> Result<Value> {
    let numerator = match args.get("numerator") {
        Some(value) => match value {
            Value::Number(n) => n.as_f64().expect("Unable to convert numerator to float"),
            _ => 0.0,
        },
        None => 0.0,
    };

    let denominator = match args.get("denominator") {
        Some(value) => match value {
            Value::Number(n) => n.as_f64().expect("Unable to convert denominator to float"),
            _ => 0.0,
        },
        None => 0.0,
    };

    if numerator != 0.0 && denominator != 0.0 {
        let result = ((numerator / denominator) * 100.0).round();
        return Ok(Value::from(result as i64));
    }

    Ok(Value::Null)
}
