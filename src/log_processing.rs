use std::{
    fmt,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, LineWriter, Lines, Write},
    path::PathBuf,
    time::Instant,
};

use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::{db_actions, models::Summary, parser_model::FileDataPoint, parsers, web::TableNames, AppContext};

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
            let file_path = match verify_file(&file) {
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
                    &file,
                    &summaries.first().unwrap().player_name.replace(" ", "_"),
                );
                db_actions::copy_db(conn, report_dir.join("summary.db"));
                Self::write_data_files(
                    conn,
                    &report_dir,
                   &file,
                    &file_path,
                    &data_points,
                    &summaries,
                );
                // let first_summary = summaries.get(0).unwrap();
                // let db_path = context.working_dir.
                // join(context.output_dir.
                // join(
                //     format!( "{}_{}", first_summary.player_name.
                //     replace(" ", "_"), first_summary.log_date[0..10].
                //     replace(" ", "_").replace("-", "_")))).join("summary.db");
                // for (i, s) in summaries.iter().enumerate() {
                //     let page_content = Self::generate_summary(
                //         conn,
                //         &context.tera,
                //         s,
                //         context.dps_interval,
                //         &db_path,
                //     );
                //     let page_name = format!("{}_{}.html", s.player_name, i);
                //     let mut report_file = match File::create(report_dir.join(page_name.clone())) {
                //         Ok(f) => f,
                //         Err(e) => panic!("Cannot create report page file: {:?}", e),
                //     };
                //     report_file
                //         .write_all(page_content.as_bytes())
                //         .expect(&format!("Unable to write file: {}", page_name));
                // }
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

    pub fn open_log_file(path_buf: PathBuf) -> Result<BufReader<File>, ProcessingError> {
        let file_name = path_buf
            .clone()
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

    fn write_summary_chunk(summaries: &Vec<Summary>, report_dir: &PathBuf, log_path: &PathBuf) {
        let result = File::open(log_path);
        let mut buf = String::new();
        let mut lines: Vec<String> = Vec::new();

        match result {
            Ok(file) => {
                let mut reader = BufReader::new(file);

                while reader
                    .read_line(&mut buf)
                    .expect(&format!("Unable to read line from {:?}", log_path))
                    > 0
                {
                    lines.push(buf.clone());
                    buf.clear();
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
                .write_all(format!("{:?}\n,", data_point).as_bytes())
                .expect("Unable to write parsed.txt")
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

pub fn verify_file(path_buf: &PathBuf) -> Result<&PathBuf, ProcessingError> {
    if path_buf.is_file() || path_buf.is_dir() {
        Ok(path_buf)
    } else {
        Err(ProcessingError {
            file_name: path_buf.clone(),
            message: "Unable to verify file existence. Might be invalid file name or permissions"
                .to_owned(),
        })
    }
}
