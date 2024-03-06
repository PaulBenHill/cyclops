use clap::Parser;
use current_platform::{COMPILED_ON, CURRENT_PLATFORM};
use models::{DamageAction, DamageReportByPower, TotalDamageReport};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Lines, Write};
use std::path::*;
use std::time::Instant;
use std::{env, fs};

use diesel::SqliteConnection;

mod parsers;
use tera::{Context, Tera};

use crate::parser_model::FileDataPoint;
use crate::parsers::*;

mod db_actions;
mod models;
mod parser_model;
mod schema;

mod reports;
use crate::models::Summary;
use crate::reports::*;

const OUTPUT_DIR: &str = "output";
const TEMPLATES: &str = "templates";
const PLAYER_ATTACK_REPORT_TEMPLATE: &str = "player_attack_report.html";

#[derive(Parser)]
#[command(name = "Cyclops")]
#[command(author = "Ben Hill <benhill70@yahoo.com")]
#[command(version = ".01")]
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
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EffectedReport {
    pub power_name: String,
    pub max_hits: u32,
    pub min_hits: u32,
    pub activations: u32,
    pub total_hits: u32,
    pub average_hits: f32,
    pub median: u32,
    pub mode: u32,
    #[serde(serialize_with = "counts_to_string")]
    counts: Vec<u32>,
}

fn counts_to_string<S>(counts: &Vec<u32>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{:?}", counts))
}

fn median(numbers: &mut Vec<u32>) -> u32 {
    numbers.sort();
    let mid = numbers.len() / 2;
    numbers[mid]
}

fn mode(numbers: &[u32]) -> u32 {
    let mut occurrences = HashMap::new();

    for &value in numbers {
        *occurrences.entry(value).or_insert(0) += 1;
    }

    occurrences
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(val, _)| val)
        .expect("Cannot compute the mode of zero numbers")
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

    let mut dps_interval = 20;
    if let Some(interval_arg) = args.interval {
        println!("Value for interval: {:?}", interval_arg);
        dps_interval = interval_arg;
    }

    create_dir(&output_path);
    println!("Output directory: {}", output_path.display());

    let mut conn = &mut db_actions::establish_connection();

    for file in log_file_names {
        db_actions::initialize_db(conn);
        let result = verify_file(&file);
        let file_name = result.0.file_name().unwrap().to_str().unwrap();

        let reader = open_log_file(result.0);

        let lines = reader.lines();

        let reports: (Vec<FileDataPoint>, Vec<SummaryReport>) = process_lines(conn, &file, lines);

        let report_dir = create_report_dir(&working_dir, &output_path, file_name, result.1);

        for (index, report) in reports.1.iter().enumerate() {
            write_reports(
                &report_dir,
                result.0,
                file_name,
                &reports.0,
                index,
                report,
                dps_interval,
            );
        }

        let summaries = db_actions::get_summaries(conn);
        for (i, s) in summaries.iter().enumerate() {
            let damage_report = db_actions::get_total_damage_report(conn, s.summary_key);
            let damage_report_by_power =
                db_actions::get_damage_by_power_report(conn, s.summary_key);

            write_report(
                &report_dir,
                file_name,
                i,
                s,
                damage_report.unwrap(),
                damage_report_by_power,
            )
        }

        db_actions::copy_db(conn, report_dir.join("summary.db"));
    }

    println!("Total run time took: {} second.", start.elapsed().as_secs());
}

fn write_report(
    report_dir: &PathBuf,
    file_name: &str,
    index: usize,
    summary: &Summary,
    total_damage: TotalDamageReport,
    damage_by_power: Vec<DamageReportByPower>,
) {
    let mut report_context = Context::new();
    report_context.insert("data_file_name", file_name);
    report_context.insert("summary", summary);
    report_context.insert("total_damage", &total_damage);
    report_context.insert("powers", &damage_by_power);
    //report_context.insert("dps_interval", &dps_interval);
    //report_context.insert("dps_reports", &dps_reports);
    //report_context.insert("targets_effected", &effected);
    //
    let summary_file_name = format!(
        "new_{}_{}_summary.html",
        summary.player_name.replace(" ", "_").to_lowercase(),
        (index + 1).to_string()
    );
    let mut summary_file = match File::create(report_dir.join(summary_file_name)) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create summary.txt file: {:?}", e),
    };

    let tera = match Tera::new(&format!("{}{}*.html", TEMPLATES, std::path::MAIN_SEPARATOR)) {
        Ok(t) => t,
        Err(e) => panic!("Unable to load templates: {:?}", e),
    };

    let result = tera.render("new_player_attack_report.html", &report_context);
    match result {
        Ok(data) => {
            summary_file
                .write_all(data.as_bytes())
                .expect("Unable to write file.");
        }
        Err(e) => panic!("Could not render {}:{:?}", PLAYER_ATTACK_REPORT_TEMPLATE, e),
    }
}

fn read_log_file_dir(dir: PathBuf) -> Vec<String> {
    match fs::canonicalize(&dir) {
        Ok(path) => {
            if path.exists() && path.is_dir() {
                let file_list: Vec<String> = fs::read_dir(path)
                    .unwrap()
                    .into_iter()
                    .filter(|r| r.is_ok())
                    .map(|r| r.unwrap().path())
                    .filter(|r| r.is_file())
                    .map(|r| r.into_os_string().into_string().unwrap())
                    .collect();

                return file_list;
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
    file_size: u64,
) -> PathBuf {
    let mut report_dir_name = format!("{}_{}", filename.replace("-", "_"), file_size.to_string());
    report_dir_name = report_dir_name.replace(" ", "_");

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

fn generate_effected_report(summary: &SummaryReport) -> Box<Vec<EffectedReport>> {
    let inital_points = summary.get_targets_effected_by_power();

    let mut activations: Vec<TargetsEffected> = Vec::new();
    let mut points_counted: Vec<(u32, String, u32)> = Vec::new();
    for point in inital_points.clone() {
        if point.activation {
            activations.push(point.clone());
        }
    }

    // for a in &activations {
    //     println!("{:?}", a);
    // }
    let mut next_activation_line_number: u32 = u32::MAX;
    for action in activations {
        let mut initial_activation_line_number = action.line_number;
        let power_name = action.power_name;
        //        println!(
        //           "Inital: {}:{}:{}",
        //          power_name, initial_activation_line_number, next_activation_line_number
        //     );
        for np in inital_points.clone() {
            if np.activation
                && np.line_number > initial_activation_line_number
                && np.power_name == power_name
            {
                next_activation_line_number = np.line_number;
                //                println!("NP: {}:{}", next_activation_line_number, power_name);
                break;
            }
        }
        let mut effected_count: u32 = 0;
        for point in inital_points.clone() {
            if !point.activation
                && point.line_number > initial_activation_line_number
                && point.line_number < next_activation_line_number
                && point.power_name == power_name
            {
                // println!(
                //     "Final: {}:{}:{}:{}:{}:{}",
                //     point.activation,
                //     point.line_number,
                //     initial_activation_line_number,
                //     next_activation_line_number,
                //     power_name,
                //     point.power_name
                // );
                effected_count += 1;
            }
        }
        if effected_count > 0 {
            points_counted.push((initial_activation_line_number, power_name, effected_count));
        }

        initial_activation_line_number = next_activation_line_number;
    }

    let mut effected_reports: HashMap<String, EffectedReport> = HashMap::new();
    for p in &points_counted {
        match effected_reports.get_mut(&p.1) {
            Some(r) => {
                if p.2 > r.max_hits {
                    r.max_hits = p.2;
                }
                if p.2 < r.min_hits {
                    r.min_hits = p.2;
                }
                r.activations += 1;
                r.total_hits += p.2;
                r.counts.push(p.2)
            }
            None => {
                let mut report = EffectedReport::default();
                report.power_name = p.1.clone();
                report.activations = 1;
                report.max_hits = p.2;
                report.min_hits = p.2;
                report.total_hits += p.2;
                report.counts = Vec::new();
                report.counts.push(p.2);

                effected_reports.insert(p.1.clone(), report);
            }
        }
    }

    let mut results: Vec<EffectedReport> = Vec::new();
    // let mut results: Vec<EffectedReport> = Vec::new();
    for r in effected_reports.values_mut() {
        r.average_hits = r.total_hits as f32 / r.activations as f32;
        r.median = median(&mut r.counts.clone());
        r.mode = mode(&mut r.counts.clone());
        results.push(r.to_owned());
    }

    Box::new(results)
}

fn write_parsed_files(report_dir: &PathBuf, parsed_lines: &Vec<FileDataPoint>) {
    let parsed_json_file = match File::create(report_dir.join("parsed.json")) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create parser.json file: {:?}", e),
    };
    let parsed_text_file = match File::create(report_dir.join("parsed.txt")) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create parser.txt file: {:?}", e),
    };
    let mut buf_json_writer = BufWriter::new(parsed_json_file);
    let mut buf_text_writer = BufWriter::new(parsed_text_file);
    for data_point in parsed_lines {
        serde_json::to_writer_pretty(&mut buf_json_writer, data_point)
            .expect("Unable to write parsed.json");
        buf_text_writer
            .write_all(format!("{:?}\n,", data_point).as_bytes())
            .expect("Unable to write parsed.txt")
    }
}

fn write_reports(
    report_dir: &PathBuf,
    data_file: &Path,
    file_name: &str,
    parsed_lines: &Vec<FileDataPoint>,
    index: usize,
    summary: &SummaryReport,
    dps_interval: usize,
) {
    // error log todo
    // summary files for each session
    // original data file
    if let Err(e) = std::fs::copy(data_file, report_dir.join(file_name)) {
        println!("Copying data file return zero bytes: {}", e);
    }

    // write parsed logs for troubleshooting
    write_parsed_files(report_dir, parsed_lines);

    let summary_file_name = format!(
        "{}_{}_summary.html",
        summary.player_name.replace(" ", "_").to_lowercase(),
        (index + 1).to_string()
    );
    let mut summary_file = match File::create(report_dir.join(summary_file_name)) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create summary.txt file: {:?}", e),
    };

    let tera = match Tera::new(&format!("{}{}*.html", TEMPLATES, std::path::MAIN_SEPARATOR)) {
        Ok(t) => t,
        Err(e) => panic!("Unable to load templates: {:?}", e),
    };

    let dps_file = match File::create(report_dir.join("dps.csv")) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create dps.csv file: {:?}", e),
    };

    let mut wtr = csv::Writer::from_writer(dps_file);
    let mut dps_reports: Vec<Vec<String>> = Vec::new();
    for interval in &summary.get_damage_points_by_interval(dps_interval) {
        let line_count = interval.len();
        let elapsed_second = DamagePoint::get_delta_in_seconds(interval);
        let total_damage = DamagePoint::get_total_damage(interval);

        let mut dps: u64 = 0;
        if total_damage > 0 && elapsed_second > 0 {
            dps = total_damage / elapsed_second;
        }

        let line_positions = DamagePoint::get_line_positions(interval);

        dps_reports.push(vec![
            line_positions.0.to_string(),
            line_positions.1.to_string(),
            line_count.to_string(),
            elapsed_second.to_string(),
            total_damage.to_string(),
            dps.to_string(),
        ]);

        for dp in interval {
            if let Err(e) = wtr.serialize(dp) {
                panic!("Unable to write dps data. {:?}:{}", dp, e);
            }
        }
    }

    let effected: Vec<EffectedReport> = *generate_effected_report(&summary);
    write_effected_report(report_dir, &effected);

    let mut report_context = Context::new();
    report_context.insert("data_file_name", file_name);
    report_context.insert("report", &summary);
    report_context.insert("powers", &summary.sort_powers_by_total_damage());
    report_context.insert("dps_interval", &dps_interval);
    report_context.insert("dps_reports", &dps_reports);
    report_context.insert("targets_effected", &effected);

    let result = tera.render(PLAYER_ATTACK_REPORT_TEMPLATE, &report_context);
    match result {
        Ok(data) => {
            summary_file
                .write_all(data.as_bytes())
                .expect("Unable to write file.");
        }
        Err(e) => panic!("Could not render {}:{:?}", PLAYER_ATTACK_REPORT_TEMPLATE, e),
    }
}

fn write_effected_report(report_dir: &PathBuf, effected_reports: &Vec<EffectedReport>) {
    let final_path = report_dir.join("effected_report.csv");

    let effected_file: File;
    if final_path.exists() {
        effected_file = File::options()
            .append(true)
            .open(final_path)
            .expect("Cannot open effected.csv for appending");
    } else {
        effected_file = File::create(final_path).expect("Cannot create effected.csv file");
    }

    let mut wtr = csv::Writer::from_writer(effected_file);
    for r in effected_reports {
        if let Err(e) = wtr.serialize(r) {
            panic!("Unable to write effected data. {:?}:{:?}", e, r);
        }
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
        return BufReader::new(file);
    } else {
        panic!("Path provided is not a file: {:?}", path);
    }
}

fn process_lines(
    conn: &mut SqliteConnection,
    file: &String,
    lines: Lines<BufReader<File>>,
) -> (Vec<FileDataPoint>, Vec<SummaryReport>) {
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

    // write to database
    db_actions::write_to_database(conn, &file, &data_points);

    let start = Instant::now();
    let summaries: Vec<SummaryReport> = total_player_attacks(&data_points);
    println!(
        "Generating summaries took: {} second.",
        start.elapsed().as_secs()
    );

    data_points.shrink_to_fit();

    (data_points, summaries)
}
