use std::{
    fmt,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Lines, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use chrono::DateTime;
use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::{db_actions, models::Summary, parser_model::FileDataPoint, parsers, AppContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingError {
    pub file_name: PathBuf,
    pub message: String,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {:?}", self.message, self.file_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserJob {
    pub files: Vec<PathBuf>,
    pub processed: usize,
    pub run_time: u64,
    pub errors: Vec<ProcessingError>,
}

impl ParserJob {
    pub fn process_logs(mut self, context: &AppContext) -> Self {
        let start = Instant::now();

        for file in &self.files[..] {
            let conn = &mut db_actions::establish_connection(); // In memory db, fresh db on each call
            let file_path = match verify_file(file.to_path_buf()) {
                Ok(f) => f,
                Err(e) => {
                    self.errors.push(e);
                    continue;
                }
            };

            let reader = match Self::open_log_file(file_path.to_path_buf()) {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    continue;
                }
            };

            let lines = reader.lines();

            let processing_result = Self::process_lines(conn, file.to_path_buf(), lines);
            if processing_result.0 {
                let data_points = processing_result.1;
                let summaries = db_actions::get_summaries(conn);

                let report_dir = Self::create_report_dir(
                    &context.working_dir,
                    &context.output_dir,
                    file.to_path_buf(),
                    &summaries.first().unwrap().player_name,
                );
                db_actions::copy_db(conn, report_dir.join("summary.db"));
                let mut summary_renders: Vec<String> = Vec::new();
                Self::write_data_files(
                    conn,
                    report_dir.to_path_buf(),
                    file.to_path_buf(),
                    file_path.to_path_buf(),
                    &data_points,
                );
                for (i, s) in summaries.iter().enumerate() {
                    summary_renders.push(Self::generate_summary(
                        conn,
                        &context.tera,
                        i,
                        s,
                        data_points.len(),
                        context.dps_interval,
                    ));
                }

                Self::generate_top_level(
                    &context.tera,
                    report_dir.to_path_buf(),
                    file_path.to_path_buf(),
                    summaries,
                    summary_renders,
                );
            } else {
                println!(
                    "No valid data found in {}.",
                    file_path
                        .to_path_buf()
                        .into_os_string()
                        .into_string()
                        .unwrap()
                );
            }
            self.processed += 1;
        }
        self.run_time = start.elapsed().as_secs();
        println!("File(s) processing time took: {} second.", self.run_time);

        println!("Starting file count: {}", self.files.len());
        println!("Processed file count: {}", self.processed);
        println!("Processing time: {}", self.run_time);

        if !self.errors.is_empty() {
            println!("ERROR(S):");
            for e in &self.errors[..] {
                println!(
                    "{}:{}",
                    &e.message,
                    &e.file_name
                        .to_path_buf()
                        .into_os_string()
                        .into_string()
                        .unwrap()
                );
            }
        }

        self
    }

    fn create_report_dir(
        working_dir: &PathBuf,
        output_dir: &PathBuf,
        file_name: PathBuf,
        player_name: &str,
    ) -> PathBuf {
        let fn_as_str = file_name
            .file_name()
            .unwrap()
            .to_ascii_lowercase()
            .into_string()
            .unwrap();
        let report_dir: PathBuf = format!(
            "{}",
            fn_as_str
                .replace('-', "_")
                .replace(' ', "_")
                .replace(".txt", "")
                .replace("chatlog", player_name),
        )
        .into();

        let report_dir: PathBuf = [working_dir, output_dir, &report_dir].iter().collect();

        create_dir(&report_dir);

        println!("Report directory: {:?}", report_dir);
        report_dir.clone()
    }

    pub fn open_log_file(path_buf: PathBuf) -> Result<BufReader<File>, ProcessingError> {
        let file_name = path_buf
            .to_path_buf()
            .into_os_string()
            .into_string()
            .expect("Could not create a file name from os string.");
        if path_buf.is_file() {
            let result = File::open(path_buf.to_path_buf());

            match result {
                Ok(file) => {
                    println!("File opened for processing: {}", file_name,);
                    Ok(BufReader::new(file))
                }
                Err(e) => Err(ProcessingError {
                    file_name: path_buf,
                    message: format!("Unable to open file might not readble. {}", e),
                }),
            }
        } else {
            Err(ProcessingError {
                file_name: path_buf,
                message: "File is not file, might be a directory".to_string(),
            })
        }
    }

    fn process_lines(
        conn: &mut SqliteConnection,
        file: PathBuf,
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
            db_actions::write_to_database(
                conn,
                file.into_os_string().into_string().unwrap(),
                &data_points,
            );
            println!("Generating summaries done.");
        }

        data_points.shrink_to_fit();

        (has_data, data_points)
    }

    fn write_data_files(
        conn: &mut SqliteConnection,
        report_dir: PathBuf,
        file_name: PathBuf,
        data_file: PathBuf,
        parsed_lines: &Vec<FileDataPoint>,
    ) {
        if let Err(e) = std::fs::copy(data_file, report_dir.join(file_name.file_name().unwrap())) {
            println!("Copying data file return zero bytes: {}", e);
        }

        // write parsed logs for troubleshooting
        Self::write_parsed_files(report_dir.to_path_buf(), parsed_lines);

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

    fn write_parsed_files(report_dir: PathBuf, parsed_lines: &Vec<FileDataPoint>) {
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
        if let Some(damage_dealt_by_type) =
            db_actions::get_damage_dealt_by_type_report(conn, summary.summary_key)
        {
            report_context.insert("damage_dealt_by_type", &damage_dealt_by_type);
        }
        if let Some(damage_taken_by_type) =
            db_actions::get_damage_taken_by_type_report(conn, summary.summary_key)
        {
            report_context.insert("damage_taken_by_type", &damage_taken_by_type);
        }
        if let Some(damage_taken_by_mob) =
            db_actions::get_damage_taken_by_mob_report(conn, summary.summary_key)
        {
            report_context.insert("damage_taken_by_mob", &damage_taken_by_mob);
        }
        if let Some(damage_taken_by_mob_power) =
            db_actions::get_damage_taken_by_mob_power_report(conn, summary.summary_key)
        {
            report_context.insert("damage_taken_by_mob_power", &damage_taken_by_mob_power);
        }

        let result = tera.render("player_attack_report.html", &report_context);
        match result {
            Ok(data) => data,
            Err(e) => panic!("Could not render {}:{:?}", "player_attack_report.html", e),
        }
    }

    fn generate_top_level(
        tera: &Tera,
        report_dir: PathBuf,
        file_name: PathBuf,
        summaries: Vec<Summary>,
        renders: Vec<String>,
    ) {
        let mut summary_file = match File::create(report_dir.join("summary.html")) {
            Ok(f) => f,
            Err(e) => panic!("Cannot create summary.txt file: {:?}", e),
        };

        let mut top_level_context = Context::new();
        top_level_context.insert("data_file_name", &file_name);
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
}

pub fn create_dir(dir_path: &PathBuf) {
    if !dir_path.exists() {
        match fs::create_dir_all(dir_path) {
            Ok(_) => (),
            Err(err) => panic!("Unable to create directory: {:?}", err),
        }
    }
}

pub fn verify_file(path_buf: PathBuf) -> Result<PathBuf, ProcessingError> {
    if path_buf.is_file() || path_buf.is_dir() {
        Ok(path_buf)
    } else {
        Err(ProcessingError {
            file_name: path_buf,
            message: "Unable to verify file existence. Might be invalid file name or permissions"
                .to_owned(),
        })
    }
}
