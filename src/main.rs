use chrono::DateTime;
use clap::Parser;
use current_platform::{COMPILED_ON, CURRENT_PLATFORM};
use diesel::dsl::sum;
use models::Summary;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Lines, Write};
use std::path::*;
use std::time::Instant;
use std::{env, fs};
use tera::{Result, Value};
use walkdir::{DirEntry, WalkDir};

use diesel::SqliteConnection;

mod parsers;
use tera::{Context, Tera};

use crate::parser_model::FileDataPoint;
use crate::parsers::*;

mod db_actions;
mod models;
mod parser_model;
mod schema;
mod web;

const OUTPUT_DIR: &str = "output";
const TEMPLATES: &str = "templates";

#[derive(Parser)]
#[command(name = "Cyclops")]
#[command(author = "Ben Hill <benhill70@yahoo.com")]
#[command(version = ".04")]
#[command(about = "Application to parse City Of Heroes log files", long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        required = false,
        conflicts_with = "files",
        value_name = "Directory where you game chat files are stored. All files in the directory will be processed."
    )]
    logdir: Option<PathBuf>,
    #[arg(
        short,
        long,
        required = false,
        value_delimiter = ',',
        conflicts_with = "logdir",
        value_name = "List of game log files comma separated."
    )]
    pub files: Option<Vec<PathBuf>>,
    #[arg(
        short,
        long,
        required = false,
        value_name = "Time in seconds between combat sessions for DPS reports"
    )]
    interval: Option<usize>,
    #[arg(
        short,
        long,
        required = false,
        value_name = "Directory where you want the reports written. Defaults to \"output\""
    )]
    outputdir: Option<PathBuf>,
    #[arg(short, long, required = false, value_name = "Web server IP address")]
    address: Option<String>,
    #[arg(short, long, required = false, value_name = "Web server port")]
    port: Option<usize>,
}

fn main() {
    let start = Instant::now();

    let working_dir = env::current_dir().unwrap().clone();
    println!(
        "Cyclops was compiled on {}:{}.",
        CURRENT_PLATFORM, COMPILED_ON
    );
    println!("The current directory is {}", working_dir.display());

    let args = Args::parse();

    let mut log_file_names: Vec<String> = Vec::new();
    if let Some(data_dir) = args.logdir {
        println!("Value for log dir: {:?}", data_dir);
        log_file_names = read_log_file_dir(data_dir.to_path_buf());
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

    create_dir(&output_path);
    println!("Output directory: {}", output_path.display());

    let mut tera = setup_tera();
    tera.autoescape_on(vec![]);
    tera.register_function("calc_percentage", calc_percentage);

    for file in log_file_names {
        let conn = &mut db_actions::establish_connection(); // In memory db, fresh db on each call
        let result = verify_file(&file);
        let file_name = result.0.file_name().unwrap().to_str().unwrap();

        let reader = open_log_file(result.0);

        let lines = reader.lines();

        let processing_result = process_lines(conn, &file, lines);

        if processing_result.0 {
            let data_points = processing_result.1;
            let summaries = db_actions::get_summaries(conn);

            let report_dir = create_report_dir(
                &working_dir,
                &output_path,
                file_name,
                &summaries.first().unwrap().player_name,
                result.1,
            );
            db_actions::copy_db(conn, report_dir.join("summary.db"));

            let mut summary_renders: Vec<String> = Vec::new();
            write_data_files(conn, &report_dir, file_name, result.0, &data_points);
            for (i, s) in summaries.iter().enumerate() {
                summary_renders.push(generate_summary(
                    conn,
                    &tera,
                    i,
                    s,
                    data_points.len(),
                    dps_interval,
                ));
            }

            generate_top_level(&tera, &report_dir, file_name, summaries, summary_renders);
        } else {
            println!("No valid data found in {}.", file_name);
        }
    }
    println!(
        "File(s) processing time took: {} second.",
        start.elapsed().as_secs()
    );

    find_all_summaries(&output_path);

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

fn find_all_summaries(output_path: &Path) {
    let walker = WalkDir::new(output_path).into_iter();
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.path().ends_with("summary.html") {
            //println!("{}", entry.path().display());
        }
    }
}

fn write_data_files(
    conn: &mut SqliteConnection,
    report_dir: &Path,
    file_name: &str,
    data_file: &Path,
    parsed_lines: &Vec<FileDataPoint>,
) {
    if let Err(e) = std::fs::copy(data_file, report_dir.join(file_name)) {
        println!("Copying data file return zero bytes: {}", e);
    }

    // write parsed logs for troubleshooting
    write_parsed_files(report_dir, parsed_lines);

    let dps_file = match File::create(report_dir.join("dps.csv")) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create dps.csv file: {:?}", e),
    };

    let dps_intervals = db_actions::select_damage_intervals(conn);
    let mut wtr = csv::Writer::from_writer(dps_file);
    for dp in dps_intervals {
        if let Err(e) = wtr.serialize(&dp) {
            panic!("Unable to write dps data. {:?}:{}", dp, e);
        }
    }
}

fn generate_top_level(
    tera: &Tera,
    report_dir: &Path,
    file_name: &str,
    summaries: Vec<Summary>,
    renders: Vec<String>,
) {
    let mut summary_file = match File::create(report_dir.join("summary.html")) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create summary.txt file: {:?}", e),
    };

    let mut top_level_context = Context::new();
    top_level_context.insert("data_file_name", file_name);
    top_level_context.insert("summaries", &summaries);
    top_level_context.insert("renders", &renders);
    let result = tera.render("summary.html", &top_level_context);
    match result {
        Ok(data) => {
            summary_file
                .write_all(data.as_bytes())
                .expect("Unable to write file.");
        }
        Err(e) => panic!("Could not render {}:{:?}", "summary.html", e),
    }
}

fn generate_summary(
    conn: &mut SqliteConnection,
    tera: &Tera,
    index: usize,
    summary: &Summary,
    line_count: usize,
    dps_interval: usize,
) -> String {
    let rewards_defeats =
        db_actions::get_rewards_defeats(conn, summary.summary_key, &summary.player_name);
    let total_damage = db_actions::get_total_damage_report(conn, summary.summary_key).unwrap();
    let damage_by_power = db_actions::get_damage_by_power_report(conn, summary.summary_key);

    let mut dps_reports: Vec<Vec<String>> = Vec::new();
    let damage_intervals =
        db_actions::get_damage_intervals(conn, summary.summary_key, dps_interval as i32);

    for intervals in damage_intervals {
        let first_interval = intervals.first().unwrap();
        let last_interval = intervals.last().unwrap();

        let mut end_line: usize = 0;
        if end_line < line_count {
            end_line = last_interval.line_number as usize;
        } else {
            end_line = line_count;
        }

        let total_damage: i32 = intervals.iter().map(|i| i.damage).sum();

        let elapsed_seconds = DateTime::parse_from_rfc3339(last_interval.log_date.as_str())
            .unwrap()
            .timestamp()
            - DateTime::parse_from_rfc3339(first_interval.log_date.as_str())
                .unwrap()
                .timestamp();

        let mut dps: i64 = 1;
        if elapsed_seconds == 0 {
            dps = total_damage as i64;
        } else {
            dps = (total_damage as i64) / elapsed_seconds;
        }

        dps_reports.push(vec![
            first_interval.line_number.to_string(),
            end_line.to_string(),
            intervals.len().to_string(),
            elapsed_seconds.to_string(),
            total_damage.to_string(),
            dps.to_string(),
        ]);
    }
    let mut report_context = Context::new();

    report_context.insert("index", &format!("player{}", index + 1));
    report_context.insert("summary", &summary);
    report_context.insert("rewards_defeats", &rewards_defeats);
    report_context.insert("total_damage", &total_damage);
    report_context.insert("powers", &damage_by_power);
    report_context.insert("dps_interval", &dps_interval);
    report_context.insert("dps_reports", &dps_reports);

    let result = tera.render("player_attack_report.html", &report_context);
    match result {
        Ok(data) => data,
        Err(e) => panic!("Could not render {}:{:?}", "player_attack_report.html", e),
    }
}

fn setup_tera() -> Tera {
    match Tera::new(&format!("{}{}*.html", TEMPLATES, std::path::MAIN_SEPARATOR)) {
        Ok(t) => t,
        Err(e) => panic!("Unable to load templates: {:?}", e),
    }
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
                    .map(|r| r.into_os_string().into_string().unwrap())
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

fn create_report_dir(
    working_dir: &PathBuf,
    output_dir: &PathBuf,
    filename: &str,
    player_name: &str,
    file_size: u64,
) -> PathBuf {
    let mut report_dir_name = format!(
        "{}_{}",
        filename
            .replace('-', "_")
            .replace(".txt", "")
            .replace("chatlog", player_name),
        file_size
    );
    report_dir_name = report_dir_name.replace(' ', "_");

    let report_dir: PathBuf = [
        working_dir,
        Path::new(output_dir),
        Path::new(&report_dir_name),
    ]
    .iter()
    .collect();

    create_dir(&report_dir);

    println!("Report directory: {:?}", report_dir);
    report_dir.clone()
}

fn write_parsed_files(report_dir: &Path, parsed_lines: &Vec<FileDataPoint>) {
    let parsed_text_file = match File::create(report_dir.join("parsed.txt")) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create parser.txt file: {:?}", e),
    };
    let mut buf_text_writer = BufWriter::new(parsed_text_file);
    for data_point in parsed_lines {
        buf_text_writer
            .write_all(format!("{:?}\n,", data_point).as_bytes())
            .expect("Unable to write parsed.txt")
    }
}

fn create_dir(dir_path: &PathBuf) {
    if !dir_path.exists() {
        match fs::create_dir_all(dir_path) {
            Ok(_) => (),
            Err(err) => panic!("Unable to create directory: {:?}", err),
        }
    }
}

fn verify_file(filename: &String) -> (&Path, u64) {
    let path = Path::new(filename);

    if path.exists() {
        match fs::metadata(path) {
            Ok(meta) => (path, meta.len()),
            Err(e) => panic!(
                "Cannot retrieve metadata. Probably permissions issue: {:?}",
                e
            ),
        }
    } else {
        panic!("Cannot find file: {}", filename);
    }
}

fn open_log_file(path: &Path) -> BufReader<File> {
    if path.exists() && path.is_file() {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => panic!("Unable to open file {:?}:{:?}", e, path),
        };
        println!(
            "File opened for processing: {}",
            path.as_os_str()
                .to_str()
                .expect("Could not create a file name from os string.")
        );
        BufReader::new(file)
    } else {
        panic!("Path provided is not a file: {:?}", path);
    }
}

fn process_lines(
    conn: &mut SqliteConnection,
    file: &str,
    lines: Lines<BufReader<File>>,
) -> (bool, Vec<FileDataPoint>) {
    let mut line_count: u32 = 0;
    let parsers = initialize_matchers();
    let mut data_points: Vec<FileDataPoint> = Vec::with_capacity(50000);

    let start = Instant::now();
    for line in lines.flatten() {
        //let start_match = Instant::now();
        line_count += 1;
        //let mut loop_count = 0;
        for p in &parsers {
            // loop_count += 1;
            if let Some(data) = p(line_count, &line) {
                data_points.push(data);
                break;
            }
        }
        // println!(
        //     "Matching:  {} micros. Loops: {}",
        //     start_match.elapsed().as_micros(),
        //     loop_count
        // );
    }

    println!(
        "Line count: {}, Data point count: {}",
        line_count,
        data_points.len()
    );
    println!(
        "Matching and conversion: {} second.",
        start.elapsed().as_secs()
    );

    let mut has_data = false;
    for dp in &data_points {
        match dp {
            FileDataPoint::PlayerDirectDamage {
                data_position: _,
                damage_dealt: _,
            } => {
                has_data = true;
                break;
            }
            _ => (),
        }
    }

    if has_data {
        // write to database
        let start = Instant::now();
        db_actions::write_to_database(conn, file, &data_points);
        println!(
            "Generating summaries took: {} second.",
            start.elapsed().as_secs()
        );
    }

    data_points.shrink_to_fit();

    (has_data, data_points)
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
