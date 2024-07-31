use clap::Parser;
use current_platform::{COMPILED_ON, CURRENT_PLATFORM};
use std::path::*;
use std::time::Instant;
use std::{env, fs};

use tera::Tera;

use crate::log_processing::ParserJob;

mod args;
mod db_actions;
mod log_processing;
mod models;
mod schema;
mod web;

const OUTPUT_DIR: &str = "output";
const TEMPLATES: &str = "templates";
const VERSION: &str = "1.0";

#[derive(Clone, Debug)]
struct AppContext {
    working_dir: PathBuf,
    resources_dir: PathBuf,
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
    println!("################################");
    println!("Cyclops CoX Log Parsing tool {}", VERSION);
    println!("################################");
    let start = Instant::now();

    let (app_context, log_file_names) = initialize(); 
    
    let parser_job = ParserJob {
        files: log_file_names,
        processed: 0,
        run_time: 0,
        errors: Vec::new(),
        completion_date: "".to_string(),
        last_file: "".to_string(),
    };
    if !parser_job.files.is_empty() {
       parser_job.process_logs(&app_context);
    }
    
    println!(
        "Starting web server at http://{}:{}",
        app_context.web_address, app_context.web_port
    );
    if let Err(e) = web::start(app_context) {
        panic!("Unable to start web server {:?}", e);
    }

    println!("Total run time took: {} second.", start.elapsed().as_secs());
}

fn setup_tera() -> Tera {
    match Tera::new(&format!("{}{}*.html", TEMPLATES, std::path::MAIN_SEPARATOR)) {
        Ok(mut t) => {
            t.autoescape_on(vec![]);
            t
        }
        Err(e) => panic!("Unable to load templates: {:?}", e),
    }
}

fn get_last_modified_file_in_dir(dir: &PathBuf) -> PathBuf {
    std::fs::read_dir(dir)
        .expect("Couldn't access local directory")
        .flatten() // Remove failed
        .filter(|f| f.metadata().unwrap().is_file()) // Filter out directories (only consider files)
        .max_by_key(|x| x.metadata().unwrap().modified().unwrap())
        .map(|r| {
            dunce::canonicalize(r.path())
                .unwrap()
        })
        .unwrap()
}

fn read_log_file_dir(dir: &PathBuf) -> Vec<PathBuf> {
    match fs::canonicalize(&dir) {
        Ok(path) => {
            if path.exists() && path.is_dir() {
                let file_list: Vec<PathBuf> = fs::read_dir(path)
                    .unwrap()
                    .filter(|r| r.is_ok())
                    .map(|r| r.unwrap().path())
                    .filter(|r| r.is_file())
                    .map(|r| {
                        dunce::canonicalize(r)
                            .unwrap()
                    })
                    .filter(|r| r.extension().unwrap() == "txt")
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

fn initialize() -> (AppContext, Vec<PathBuf>) {
    let working_dir = env::current_dir().unwrap().clone();
    println!(
        "Cyclops was compiled on {}:{}.",
        CURRENT_PLATFORM, COMPILED_ON
    );
    println!("The current directory is {}", working_dir.display());


    let args = args::Args::parse();

    let mut log_file_names: Vec<PathBuf> = Vec::new();
    if let Some(log_dirs) = args.logdir {
        println!("Value for log dir: {:?}", log_dirs);
        for dir in log_dirs {
            log_file_names.append(&mut read_log_file_dir(&dir));
        }
    } else if let Some(files) = args.files {
        for path_buf in files {
            log_file_names.push(path_buf);
        }
    }

    if log_file_names.is_empty() {
        println!("No logs found. Continuing to web server.");
    } 

    let mut output_dir = PathBuf::new().join(OUTPUT_DIR);
    if let Some(outputdir) = args.outputdir {
        println!("Value for output dir: {:?}", outputdir);
        output_dir = outputdir.clone();
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

    let res_dir = working_dir.clone().join("resources");

    (AppContext {
        working_dir,
        resources_dir: res_dir,
        output_dir,
        dps_interval,
        web_address: String::from(webserver_address),
        web_port: webserver_port as u16,
        tera,
    },
    log_file_names)
}