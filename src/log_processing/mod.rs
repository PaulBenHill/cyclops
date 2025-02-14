use std::{
    fmt,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, LineWriter, Lines, Write},
    mem,
    path::{Path, PathBuf},
    sync::Mutex,
    thread::{self},
    time::Instant,
};

use chrono::Local;
use diesel::SqliteConnection;
use lazy_static::lazy_static;
use parser_model::FileDataPoint;
use serde::{Deserialize, Serialize};

use crate::{
    db::{
        self,
        event_processing::{write_to_database, write_to_monitor},
    },
    models::Summary,
    monitoring, AppContext,
};

pub mod parser_model;
mod parsers;

lazy_static! {
    static ref PARSER_JOB_QUEUE: Mutex<Option<ParserJob>> = Mutex::new(None);
}

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
    pub completion_date: String,
    pub files: Vec<PathBuf>,
    pub processed: usize,
    pub run_time: u64,
    pub last_file: String,
    pub errors: Vec<ProcessingError>,
}

pub fn add_job(job: ParserJob) {
    let mut queue = PARSER_JOB_QUEUE.lock().unwrap();

    let _ = mem::replace(&mut *queue, Some(job));
}

pub fn get_job() -> Option<ParserJob> {
    let mut queue = PARSER_JOB_QUEUE.lock().unwrap();

    let job_option = mem::replace(&mut *queue, None);

    job_option
}

impl ParserJob {
    pub fn process_logs(mut self, context: &AppContext) -> Self {
        let start = Instant::now();

        for file in &self.files[..] {
            let conn = &mut db::establish_connection(); // In memory db, fresh db on each call
            let file_path = match verify_file(&file) {
                Ok(f) => f,
                Err(e) => {
                    self.errors.push(e);
                    continue;
                }
            };

            let reader = match open_log_file(file_path.to_path_buf(), true) {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    continue;
                }
            };

            let lines = reader.lines();

            let (success, file_points) = process_lines(conn, file.to_path_buf(), lines);
            if success {
                let summaries = db::queries::get_summaries(conn);

                let report_dir = Self::create_report_dir(
                    &context.working_dir,
                    &context.output_dir,
                    &file,
                    &summaries.first().unwrap().player_name.replace(" ", "_"),
                );
                db::copy_db(conn, report_dir.join("summary.db"));
                Self::write_data_files(
                    conn,
                    &report_dir,
                    &file,
                    &file_path,
                    &file_points,
                    &summaries,
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
        let local_time = Local::now();
        self.completion_date = format!("{}", local_time.format("%a %b %e %T %Y"));
        let last_file = self.files.last().unwrap();
        self.last_file = String::from(last_file.as_os_str().to_str().unwrap());

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
        file_name: &PathBuf,
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

    fn write_data_files(
        conn: &mut SqliteConnection,
        report_dir: &PathBuf,
        file_name: &PathBuf,
        data_file: &PathBuf,
        parsed_lines: &Vec<FileDataPoint>,
        summaries: &Vec<Summary>,
    ) {
        let log_file_path = report_dir.join(file_name.file_name().unwrap());
        if let Err(e) = std::fs::copy(data_file, log_file_path.to_path_buf()) {
            println!("Copying data file return zero bytes: {}", e);
        }

        Self::write_summary_chunk(&summaries, report_dir, &data_file);

        //write parsed logs for troubleshooting
        Self::write_parsed_files(&report_dir, parsed_lines);
        Self::write_rp_file(&report_dir, parsed_lines);

        let dps_file = match File::create(report_dir.join("dps.csv")) {
            Ok(f) => f,
            Err(e) => panic!("Cannot create dps.csv file: {:?}", e),
        };

        let dps_intervals = db::queries::select_damage_intervals(conn);
        let mut wtr = csv::Writer::from_writer(dps_file);
        for dp in dps_intervals {
            if let Err(e) = wtr.serialize(&dp) {
                panic!("Unable to write dps data. {:?}:{}", dp, e);
            }
        }
    }

    fn write_summary_chunk(summaries: &Vec<Summary>, report_dir: &PathBuf, log_path: &PathBuf) {
        let result = File::open(log_path);
        let mut buf = String::new();
        let mut lines: Vec<String> = Vec::new();

        match result {
            Ok(file) => {
                let mut reader = BufReader::new(file);

                loop {
                    match reader.read_line(&mut buf) {
                        Ok(count) => {
                            if count > 0 {
                                lines.push(buf.clone());
                                buf.clear();
                            } else {
                                break;
                            }
                        }
                        Err(e) => println!(
                            "Not valid line, error: {}, line: {}. Ignoring line.",
                            e,
                            lines.len() + 1
                        ),
                    };
                }
            }
            Err(e) => panic!("Unable to copied log file: {:?}:{}", log_path, e),
        }

        for (i, s) in summaries.iter().enumerate() {
            let first: usize = s.first_line_number.try_into().unwrap();
            let last: usize = s.last_line_number.try_into().unwrap();
            let chunk_path = report_dir.join(&format!(
                "{}_{}_{}.txt",
                i,
                &s.player_name.replace(" ", "_"),
                s.first_line_number.to_string()
            ));
            let chunk_file = File::create(chunk_path.clone()).expect(&format!(
                "Unable to create chunk file: {:?}",
                chunk_path.clone()
            ));
            let mut writer = LineWriter::new(chunk_file);

            for l in lines[first..last].iter() {
                writer.write_all(l.as_bytes()).expect(&format!(
                    "Unable to write lines to {:?}",
                    chunk_path.clone()
                ));
            }
        }
    }

    fn write_parsed_files(report_dir: &PathBuf, parsed_lines: &Vec<FileDataPoint>) {
        let parsed_text_file = match File::create(report_dir.join("parsed.txt")) {
            Ok(f) => f,
            Err(e) => panic!("Cannot create parser.txt file: {:?}", e),
        };
        let mut buf_text_writer = BufWriter::new(parsed_text_file);
        for data_point in parsed_lines {
            buf_text_writer
                .write_all(format!("{:?}\r\n,", data_point).as_bytes())
                .expect("Unable to write parsed.txt")
        }
    }

    const IGNORE_LIST: &'static [&'static str] = &[
        "reduces the regeneration rate",
        "You are now Stealthy.",
        "You activate Sprint and can now run faster.",
        "knocks you off your feet with their",
        "You are now Tough, and are slightly resistant to Smashing and Lethal damage",
        "You start to Weave and are now harder to hit and Immobilize",
        "is still recharging",
        "Your henchmen protect you from",
        "You cower in terror",
        "Your Hasten has increased your rate of attack",
        "Your Hasten drains",
        "You are no longer afraid",
        "boosts the damage of your attacks",
    ];

    fn write_rp_file(report_dir: &PathBuf, parsed_lines: &Vec<FileDataPoint>) {
        let parsed_text_file = match File::create(report_dir.join("rp.txt")) {
            Ok(f) => f,
            Err(e) => panic!("Cannot create rp.txt file: {:?}", e),
        };
        let mut buf_text_writer = BufWriter::new(parsed_text_file);
        for data_point in parsed_lines {
            match data_point {
                FileDataPoint::ChatMessage {
                    data_position,
                    category,
                    message,
                } => buf_text_writer
                    .write_all(
                        format!(
                            "[CHAT]#+!{}#+!{}#+!{}#+!{}\r\n",
                            data_position.line_number,
                            data_position.date.format("%Y/%m/%d %H:%M"),
                            category,
                            message
                        )
                        .as_bytes(),
                    )
                    .expect("Unable to write chat message to rp.txt"),
                FileDataPoint::Unparsed {
                    data_position,
                    content,
                } => {
                    if !Self::IGNORE_LIST.iter().any(|s| content.contains(s)) {
                        buf_text_writer
                            .write_all(
                                format!(
                                    "UNPARSED#+!{}#+!{}#+!{}\r\n",
                                    data_position.line_number,
                                    data_position.date.format("%Y/%m/%d %H:%M"),
                                    content
                                )
                                .as_bytes(),
                            )
                            .expect("Unable to write unparsed to rp.txt")
                    }
                }
                _ => (),
            }
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

pub fn verify_file<P: AsRef<Path>>(path_buf: P) -> Result<PathBuf, ProcessingError> {
    let path = path_buf.as_ref();
    if path.is_file() || path.is_dir() {
        Ok(path.to_path_buf())
    } else {
        Err(ProcessingError {
            file_name: path.to_path_buf(),
            message: "Unable to verify file existence. Might be invalid file name or permissions"
                .to_owned(),
        })
    }
}

pub fn process_lines(
    conn: &mut SqliteConnection,
    file: PathBuf,
    lines: Lines<BufReader<File>>,
) -> (bool, Vec<FileDataPoint>) {
    let mut line_count: u32 = 0;
    let parsers = parsers::MATCHER_FUNCS;
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
        write_to_database(
            conn,
            file.into_os_string().into_string().unwrap(),
            &data_points,
        );
        println!("Generating summaries done.");
    }

    data_points.shrink_to_fit();

    (has_data, data_points)
}

pub fn monitor_lines(
    debug_log: &mut File,
    conn: &mut SqliteConnection,
    file: PathBuf,
    lines: Lines<BufReader<File>>,
) -> (bool, Vec<FileDataPoint>) {
    let mut line_count: u32 = 0;
    let parsers = parsers::MONITOR_MATCHER_FUNCS;
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

    writeln!(
        debug_log,
        "Line count: {}, Data point count: {}",
        line_count,
        data_points.len()
    )
    .expect("Unable to write to debug log.");

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
        write_to_monitor(
            conn,
            file.into_os_string().into_string().unwrap(),
            &data_points,
            line_count,
        );
    }

    data_points.shrink_to_fit();

    (has_data, data_points)
}

pub fn open_log_file(path_buf: PathBuf, verbose: bool) -> Result<BufReader<File>, ProcessingError> {
    let file_name = path_buf
        .clone()
        .into_os_string()
        .into_string()
        .expect("Could not create a file name from os string.");
    if path_buf.is_file() {
        let result = File::open(path_buf.to_path_buf());

        match result {
            Ok(file) => {
                if verbose {
                    println!("File opened for processing: {}", file_name,);
                }
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
