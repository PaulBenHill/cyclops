use clap::Parser;
use current_platform::{COMPILED_ON, CURRENT_PLATFORM};
use models::IndexDetails;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::*;
use std::time::Instant;
use std::{env, fs};
use tera::{Result, Value};
use walkdir::WalkDir;

mod parsers;
use tera::{Context, Tera};

use crate::schema::summary::log_file_name;

mod args;
mod db_actions;
mod log_processing;
mod models;
mod parser_model;
mod schema;
mod web;

const OUTPUT_DIR: &str = "output";
const TEMPLATES: &str = "templates";

#[derive(Clone)]
struct AppContext {
    working_dir: PathBuf,
    output_dir: PathBuf,
    dps_interval: usize,
    web_address: String,
    web_port: u16,
    tera: Tera,
}

// Todos
// File processing in index page
//  status updates
// charts
// windows dialog projects exist
// logdir multiple
// file or log dir not required
fn main() {
    let start = Instant::now();

    let (app_context, log_file_names) = initialize(); 
    
    log_processing::process_logs(&app_context, log_file_names);

    println!(
        "File(s) processing time took: {} second.",
        start.elapsed().as_secs()
    );

    let indexes = find_all_summaries(&app_context.output_dir);
    generate_index(&app_context, &indexes);

    println!(
        "Starting web server at http://{}:{}",
        app_context.web_address, app_context.web_port
    );
    if let Err(e) = web::start(app_context) {
        panic!("Unable to start web server {:?}", e);
    }

    println!("Total run time took: {} second.", start.elapsed().as_secs());
}

fn generate_index(context: &AppContext, indexes: &Vec<SummaryEntry>) {
    let mut index_file = match File::create(context.output_dir.join("index.html")) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create summary.txt file: {:?}", e),
    };

    let mut log_dirs: HashSet<PathBuf> = HashSet::new();
    for i in indexes {
        let f = Path::new(&i.details.file);
        if f.is_dir() {
            log_dirs.insert(f.to_path_buf());
        } else {
            log_dirs.insert(f.parent().unwrap().to_path_buf());
        }
    }
    let mut dir_list: Vec<PathBuf> = log_dirs.into_iter().collect();
    dir_list.sort();

    let mut index_content = Context::new();
    index_content.insert("indexes", &indexes);
    index_content.insert("log_dirs", &dir_list);
    let result = context.tera.render("index.html", &index_content);
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

fn get_last_modified_file_in_dir(dir: PathBuf) -> String {
    std::fs::read_dir(dir)
        .expect("Couldn't access local directory")
        .flatten() // Remove failed
        .filter(|f| f.metadata().unwrap().is_file()) // Filter out directories (only consider files)
        .max_by_key(|x| x.metadata().unwrap().modified().unwrap())
        .map(|r| {
            dunce::canonicalize(r.path())
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
        })
        .unwrap()
}

fn read_log_file_dir(dir: PathBuf) -> Vec<String> {
    match fs::canonicalize(&dir) {
        Ok(path) => {
            if path.exists() && path.is_dir() {
                let file_list: Vec<String> = fs::read_dir(path)
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

fn initialize() -> (AppContext, Vec<String>) {
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
        println!("No chat logs found. Continuing to web server.");
    } else {
        println!("Processing {} log files", log_file_names.len());
    }

    let mut output_dir = PathBuf::new().join(OUTPUT_DIR);
    if let Some(outputdir) = args.outputdir {
        println!("Value for output dir: {:?}", outputdir);
        output_dir = outputdir.to_path_buf();
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

    log_processing::create_dir(&output_dir);
    println!("Output directory: {}", output_dir.display());

    let tera = setup_tera();

    (AppContext {
        working_dir,
        output_dir,
        dps_interval,
        web_address: String::from(webserver_address),
        web_port: webserver_port as u16,
        tera,
    },
    log_file_names)
}