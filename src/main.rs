use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{Lines, Write};
use std::path::*;
use std::time::Instant;
use std::{env, fs};

mod parsers;
use tera::{Context, Tera};

use crate::parser_model::FileDataPoint;
use crate::parsers::*;

mod parser_model;

mod reports;
use crate::reports::*;

const OUTPUT_DIR: &str = "output";
const TEMPLATES: &str = "templates";
const PLAYER_ATTACK_REPORT_TEMPLATE: &str = "player_attack_report.html";

fn main() {
    let start = Instant::now();
    let working_dir = env::current_dir().unwrap().clone();
    println!("The current directory is {}", working_dir.display());

    if !create_output_dir() {
        std::process::exit(-1);
    }

    let args: Vec<String> = env::args().skip(1).collect();

    for file in args {
        let result = verify_file(&file);
        let file_name = result.0.file_name().unwrap().to_str().unwrap();

        let reader = open_log_file(result.0);

        let lines = reader.lines();

        let reports = process_lines(lines);

        let report_dir = create_report_dir(&working_dir, file_name, result.1);

        write_reports(&report_dir, result.0, file_name, reports.0, reports.1);
    }

    println!("Run time took: {} second.", start.elapsed().as_secs());
}

fn create_report_dir(working_dir: &PathBuf, filename: &str, file_size: u64) -> PathBuf {
    let mut report_dir_name = String::new();
    report_dir_name.push_str(filename);
    report_dir_name.push_str(&file_size.to_string());
    report_dir_name = report_dir_name
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    let report_dir: PathBuf = [
        working_dir,
        Path::new(OUTPUT_DIR),
        Path::new(&report_dir_name),
    ]
    .iter()
    .collect();

    if !report_dir.exists() {
        match fs::create_dir(&report_dir) {
            Ok(_) => (),
            Err(e) => panic!("Unable to create report dir: {:?}:{:?}", report_dir, e),
        }
    }

    println!("Report directory: {:?}", report_dir);
    report_dir.clone()
}

fn write_reports(
    report_dir: &PathBuf,
    data_file: &Path,
    file_name: &str,
    parsed_lines: Vec<FileDataPoint>,
    summary_report: SummaryReport,
) -> bool {
    // four files to write
    // original data file
    // summary (damage (criticals), damage by power (criticals), damage by type)
    // parsed log
    // error log
    if let Err(e) = std::fs::copy(data_file, report_dir.join(file_name)) {
        println!("Copying data file return zero bytes: {}", e);
    }

    let mut parsed_file = match File::create(report_dir.join("parsed.txt")) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create parser.txt file: {:?}", e),
    };

    for data_point in parsed_lines {
        match write!(parsed_file, "{:?}\n", data_point) {
            Ok(_) => (),
            Err(e) => panic!("Cannot write to parsed.txt file: {:?}", e),
        }
    }

    let mut summary_file = match File::create(report_dir.join("summary.html")) {
        Ok(f) => f,
        Err(e) => panic!("Cannot create summary.txt file: {:?}", e),
    };

    let tera = match Tera::new(&format!("{}{}*.html", TEMPLATES, std::path::MAIN_SEPARATOR)) {
        Ok(t) => t,
        Err(e) => panic!("Unable to load templates: {:?}", e),
    };

    let mut report_context = Context::new();
    report_context.insert("report", &summary_report);
    report_context.insert("powers", &summary_report.sort_powers_by_total_damage());

    let result = tera.render(PLAYER_ATTACK_REPORT_TEMPLATE, &report_context);
    match result {
        Ok(data) => {
            summary_file
                .write_all(data.as_bytes())
                .expect("Unable to write file.");
        }
        Err(e) => panic!("Could not render {}:{:?}", PLAYER_ATTACK_REPORT_TEMPLATE, e),
    }
    /*
        for power in summary_report.sort_powers_by_total_damage() {
            match write!(summary_file, "{}\n", power) {
                Ok(_) => (),
                Err(e) => panic!("Cannot write to summary.txt file: {:?}", e),
            }
        }
    */
    true
}

fn create_output_dir() -> bool {
    let result: bool = match fs::canonicalize(OUTPUT_DIR) {
        Ok(path) => {
            println!("Output dir: {:?}", path);
            true
        }
        Err(e) => match fs::create_dir(OUTPUT_DIR) {
            Ok(output_dir) => {
                println!(
                    "Output directory did not exist. Creating dir: {:?}",
                    output_dir
                );
                true
            }
            Err(e) => {
                eprintln!("Unable to create output directory: {:?}", e);
                false
            }
        },
    };
    result
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
        panic!("Cannot find file");
    }
}

fn open_log_file(path: &Path) -> BufReader<File> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => panic!("Unable to open file {:?}", e),
    };
    println!("File opened for processing: {:?}", file);
    BufReader::new(file)
}

fn process_lines(lines: Lines<BufReader<File>>) -> (Vec<FileDataPoint>, SummaryReport) {
    let mut line_count: u32 = 0;
    let parsers = initialize_matcher();
    let mut data_points: Vec<FileDataPoint> = Vec::new();

    for line in lines.flatten() {
        line_count += 1;
        for p in &parsers {
            if let Some(data) = p(line_count, &line) {
                //println!("{:?}", data);
                data_points.push(data);
                break;
            }
        }
    }
    println!(
        "Line count: {}, Data point count: {}",
        line_count,
        data_points.len()
    );

    let damage_report: SummaryReport = total_player_attacks(&data_points);
    println!(
        "Total damage: {}, Normal damage {}, Critical damage {}, Critical damage percentage: {:.1}%",
        damage_report.total_damage,
        damage_report.total_direct_damage,
        damage_report.total_critical_damage,
        (damage_report.total_critical_damage / damage_report.total_damage) * 100
    );
    (data_points, damage_report)
}
