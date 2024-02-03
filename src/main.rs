use std::fs::File;
use std::io::Lines;
use std::io::{BufRead, BufReader};
use std::path::*;
use std::{env, fs};

mod parsers;
use crate::parser_model::FileDataPoint;
use crate::parsers::*;

mod parser_model;

mod reports;
use crate::reports::*;

const OUTPUT_DIR: &str = "output";

fn main() {
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

        write_reports(&report_dir, reports.0, reports.1);
    }
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
    parsed_lines: Vec<FileDataPoint>,
    damage_report: DamageReport,
) -> bool {
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

fn process_lines(lines: Lines<BufReader<File>>) -> (Vec<FileDataPoint>, DamageReport) {
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

    let damage_report: DamageReport = total_player_damage(&data_points);
    println!(
        "Total damage: {}, Normal damage {}, Critical damage {}, Critical damage percentage: {:.1}%",
        damage_report.total_damage,
        damage_report.normal_damage,
        damage_report.critical_damage,
        (damage_report.critical_damage / damage_report.total_damage) * 100.0
    );
    (data_points, damage_report)
}
