use std::{
    error::Error, fs::{self, File}, io::{BufRead, BufReader, BufWriter, Lines, Write}, path::{Path, PathBuf}, time::Duration
};

use chrono::DateTime;
use diesel::SqliteConnection;
use tera::{Context, Tera};

use crate::{db_actions, models::Summary, parser_model::FileDataPoint, parsers, AppContext};

pub fn process_logs(context: &AppContext, files: Vec<String>) {
    for file in files {
        let conn = &mut db_actions::establish_connection(); // In memory db, fresh db on each call
        let stripped_file = file.replace("\"", "");
        let file_path = verify_file(&stripped_file);
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        let reader = open_log_file(file_path);

        let lines = reader.lines();

        let processing_result = process_lines(conn, &file, lines);

        if processing_result.0 {
            let data_points = processing_result.1;
            let summaries = db_actions::get_summaries(conn);

            let report_dir = create_report_dir(
                &context.working_dir,
                &context.output_dir,
                file_name,
                &summaries.first().unwrap().player_name,
            );
            db_actions::copy_db(conn, report_dir.join("summary.db"));

            let mut summary_renders: Vec<String> = Vec::new();
            write_data_files(conn, &report_dir, file_name, file_path, &data_points);
            for (i, s) in summaries.iter().enumerate() {
                summary_renders.push(generate_summary(
                    conn,
                    &context.tera,
                    i,
                    s,
                    data_points.len(),
                    context.dps_interval,
                ));
            }

            generate_top_level(
                &context.tera,
                &report_dir,
                file_name,
                summaries,
                summary_renders,
            );
        } else {
            println!("No valid data found in {}.", file_name);
        }
    }
    println!("File(s) processing done.");
}

fn create_report_dir(
    working_dir: &Path,
    output_dir: &Path,
    filename: &str,
    player_name: &str,
) -> PathBuf {
    let mut report_dir_name = format!(
        "{}",
        filename
            .replace('-', "_")
            .replace(".txt", "")
            .replace("chatlog", player_name),
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

pub fn verify_file(filename: &String) -> &Path {
    let path = Path::new(filename);

    if path.exists() {
        match fs::metadata(path) {
            Ok(_) => path,
            Err(e) => panic!(
                "Cannot retrieve metadata. Probably permissions issue: {:?}",
                e
            ),
        }
    } else {
        panic!("Cannot find file: {}", filename);
    }
}

pub fn open_log_file(path: &Path) -> BufReader<File> {
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
    let parsers = parsers::initialize_matchers();
    let mut data_points: Vec<FileDataPoint> = Vec::with_capacity(50000);

    for line in lines.flatten() {
        line_count += 1;
        for p in &parsers {
            if let Some(data) = p(line_count, &line) {
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
    println!("Matching and conversion done.");

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
        db_actions::write_to_database(conn, file, &data_points);
        println!("Generating summaries done.");
    }

    data_points.shrink_to_fit();

    (has_data, data_points)
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

        let elapsed_duration = Duration::from_secs(elapsed_seconds as u64).as_secs();
        let pretty_elapsed = format!(
            "{} min(s) {} second(s)",
            elapsed_duration / 60,
            elapsed_duration % 60
        );

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
            pretty_elapsed,
            total_damage.to_string(),
            dps.to_string(),
        ]);
    }
    let mut report_context = Context::new();

    report_context.insert("index", &format!("player{}", index + 1));
    report_context.insert("summary", &summary);
    report_context.insert("rewards_defeats", &rewards_defeats);
    report_context.insert("total_damage", &total_damage);
    if let Some(damage_taken) = db_actions::get_damage_taken_report(conn, summary.summary_key) {
        report_context.insert("damage_taken", &damage_taken);
    }
    report_context.insert("powers", &damage_by_power);
    report_context.insert("dps_interval", &dps_interval);
    report_context.insert("dps_reports", &dps_reports);

    let result = tera.render("player_attack_report.html", &report_context);
    match result {
        Ok(data) => data,
        Err(e) => panic!("Could not render {}:{:?}", "player_attack_report.html", e),
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

pub fn create_dir(dir_path: &PathBuf) {
    if !dir_path.exists() {
        match fs::create_dir_all(dir_path) {
            Ok(_) => (),
            Err(err) => panic!("Unable to create directory: {:?}", err),
        }
    }
}
