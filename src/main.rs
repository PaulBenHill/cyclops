use std::env;
use std::fs::File;
use std::io::Lines;
use std::io::{BufRead, BufReader};
use std::path::*;

mod parsers;
use crate::parser_model::FileDataPoint;
use crate::parsers::*;

mod parser_model;

mod reports;
use crate::reports::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    for file in args {
        let path = verify_file(&file);

        let reader = open_log_file(path);

        let lines = reader.lines();

        let reports = process_lines(lines);
    }
}

fn verify_file(filename: &String) -> &Path {
    let path = Path::new(filename);

    if path.exists() {
        path
    } else {
        panic!("Cannot find file");
    }
}

fn open_log_file(path: &Path) -> BufReader<File> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => panic!("Unable to open file {:?}", e),
    };
    println!("{:?}", file);
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
                println!("{:?}", data);
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
