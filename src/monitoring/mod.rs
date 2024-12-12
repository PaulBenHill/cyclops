use chrono::Local;
use log_processing::ProcessingError;
use monitor_structs::MonitorConfig;
use std::io::Write;
use std::{
    cmp,
    fs::{File, OpenOptions},
    io::BufRead,
    path::PathBuf,
    thread,
    time::{self, Instant},
};

use crate::{
    db::{self},
    get_last_modified_file_in_dir,
    log_processing::{self, open_log_file, process_lines, verify_file},
};

pub mod monitor_structs;

pub struct MonitorJob {
    pub log_file: File,
    pub config: MonitorConfig,
    pub start_time: String,
    pub last_run_time: u128,
    pub errors: Vec<ProcessingError>,
} // configuration

impl MonitorJob {
    pub fn new(output_dir: &PathBuf, config: MonitorConfig) -> Self {
        let now = Local::now();
        let log_path = output_dir.join(format!("monitor.{}.log", now.format("%Y_%m_%d")));
        let log_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_path)
            .expect("Unable to create new monitor log");
        Self {
            log_file: log_file,
            config: config,
            start_time: "no started".to_owned(),
            last_run_time: 0,
            errors: Vec::<ProcessingError>::new(),
        }
    }

    // find file updated in the last min
    // open file for reading
    // read lines
    // process lines
    // write to db - mutex
    // sleep
    pub fn monitor_dir(mut self) -> Self {
        loop {
            let log_file = get_last_modified_file_in_dir(self.config.dir.clone());
            let file_path = match verify_file(&log_file) {
                Ok(f) => f,
                Err(e) => {
                    self.errors.push(e);
                    return self;
                }
            };

            let start = Instant::now();
            let conn = &mut db::establish_connection(); // In memory db, fresh db on each call

            let reader = match open_log_file(file_path.to_path_buf()) {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    continue;
                }
            };

            let lines = reader.lines();

            let (success, file_points) = process_lines(conn, file_path.to_path_buf(), lines);
            if success {
                println!("Datapoints parsed: {}", file_points.len());
                let end_date = db::queries::get_last_interesting_date(conn);
                let last_second = end_date - chrono::Duration::seconds(1);

                for a in &self.config.actions {
                    match a.trigger_type {
                        monitor_structs::TriggerType::ACTIVATION => {
                            let activation_option =
                                db::queries::get_last_activation(conn, &a.power_name, last_second);
                            println!("Power activation: {:?}", activation_option);

                            match activation_option {
                                Some(a) => {
                                   writeln!(self.log_file, "Power activation: {:?},{:?}", last_second, a).expect("Unable to write activation to monitor log");
                                }
                                None => (),
                            }
                        }
                        monitor_structs::TriggerType::RECHARGE => {
                            let recharge_option =
                                db::queries::get_last_recharge(conn, &a.power_name, last_second);
                            match recharge_option {
                                Some(r) => {
                                   writeln!(self.log_file, "Power recharge: {:?},{:?}", last_second, r).expect("Unable to write recharge to monitor log");
                                }
                                None => (),
                            }
                        }
                    }
                }

                self.last_run_time = start.elapsed().as_millis();
                println!("Processing time in milliseconds: {}", self.last_run_time);
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
                break;
            }

            let sleep_time = cmp::max(10, 1000 - self.last_run_time);
            thread::sleep(time::Duration::from_millis(sleep_time as u64));
        }

        self
    }
}
