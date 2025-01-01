use chrono::{DateTime, Local};
use lazy_static::lazy_static;
use log_processing::ProcessingError;
use monitor_structs::{
    Action, EventKey, MessageDetails, MonitorConfig, MonitorMessage, TriggerType,
};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::Mutex;
use std::{
    cmp,
    fs::{File, OpenOptions},
    io::BufRead,
    path::PathBuf,
    thread,
    time::{self, Instant},
};
use tracing_subscriber::field::display;

use crate::log_processing::monitor_lines;
use crate::models::SessionStats;
use crate::{
    db::{self},
    get_last_modified_file_in_dir,
    log_processing::{self, open_log_file, process_lines, verify_file},
};

pub mod monitor_structs;

lazy_static! {
    static ref DISPLAY_MESSAGES: Mutex<HashMap<EventKey, MessageDetails>> =
        Mutex::new(HashMap::new());
    static ref SESSION_STATS: Mutex<SessionStats> = Mutex::new(SessionStats {
        summary_key: 0,
        total_dps: 0,
        dps_5: 0,
        total_exp: 0,
        exp_5: 0,
        total_inf: 0,
        inf_5: 0,
    });
}

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

            let (success, file_points) = monitor_lines(conn, file_path.to_path_buf(), lines);
            if success {
                // Handle session stats
                let key = db::queries::get_summaries(conn).last().unwrap().summary_key;
                match db::queries::get_session_stats(conn, key) {
                    Some(data) => {
                        let mut old_state = SESSION_STATS.lock().unwrap();
                        let _ = std::mem::replace(&mut *old_state, data);
                    }
                    _ => (),
                }

                let mut display_map = DISPLAY_MESSAGES.lock().unwrap();
                println!("Datapoints parsed: {}", file_points.len());

                let now: DateTime<Local> = Local::now();
                let last_second = now - chrono::Duration::seconds(5);

                for action in &self.config.actions {
                    match action.trigger_type {
                        monitor_structs::TriggerType::ACTIVATION => {
                            let activation_option = db::queries::get_last_activation(
                                conn,
                                &action.power_name,
                                last_second,
                            );
                            println!("Power activation: {:?}", activation_option);

                            match activation_option {
                                Some(activation) => {
                                    writeln!(
                                        self.log_file,
                                        "Power activation: {:?},{:?}",
                                        last_second, activation
                                    )
                                    .expect("Unable to write activation to monitor log");
                                    let log_date = activation.log_date.parse().unwrap();
                                    let key = create_event_key(
                                        log_date,
                                        activation.line_number,
                                        TriggerType::ACTIVATION,
                                        &action.power_name,
                                    );

                                    if !display_map.contains_key(&key) {
                                        let display_message = create_display_message(
                                            TriggerType::ACTIVATION,
                                            log_date,
                                            &self.config,
                                            &action,
                                        );
                                        display_map.insert(key, display_message);
                                    }
                                }
                                None => (),
                            }
                        }
                        monitor_structs::TriggerType::RECHARGE => {
                            let recharge_option = db::queries::get_last_recharge(
                                conn,
                                &action.power_name,
                                last_second,
                            );
                            match recharge_option {
                                Some(recharge) => {
                                    writeln!(
                                        self.log_file,
                                        "Power recharge: {:?},{:?}",
                                        last_second, recharge
                                    )
                                    .expect("Unable to write recharge to monitor log");
                                    let log_date = recharge.log_date.parse().unwrap();
                                    let key = create_event_key(
                                        log_date,
                                        recharge.line_number,
                                        TriggerType::RECHARGE,
                                        &action.power_name,
                                    );

                                    if !display_map.contains_key(&key) {
                                        let display_message = create_display_message(
                                            TriggerType::RECHARGE,
                                            log_date,
                                            &self.config,
                                            &action,
                                        );
                                        display_map.insert(key, display_message);
                                    }
                                }
                                None => (),
                            }
                        }
                    }
                }

                // Remove all expired keys
                display_map.retain(|k, _| k.log_date.timestamp() < now.timestamp());

                if !display_map.is_empty() {
                    // All message must be verify as active
                    // So, collect them into a hashset and remove the invalid ones
                    let mut active_keys: HashSet<EventKey> = display_map.keys().cloned().collect();
                    for (key, _) in display_map.iter() {

                        // Remove activation is there is a newer activation for the same power
                        if display_map.keys().any(|existing_key| {
                            existing_key.trigger_type == TriggerType::ACTIVATION
                                && existing_key.line_number > key.line_number
                                && existing_key.power_name == key.power_name
                        }) {
                            active_keys.remove(key);
                        }

                        // Remove recharge is there is a newer recharge or activation for the same power
                        if key.trigger_type == TriggerType::RECHARGE {
                            if display_map.keys().any(|existing_key| {
                                existing_key.line_number > key.line_number
                                    && existing_key.power_name == key.power_name
                            }) {
                                active_keys.remove(key);
                            }
                        }
                    }
                    display_map.retain(|k, _| active_keys.contains(k));
                }
                writeln!(self.log_file, "### Display Map ###")
                    .expect("Unable to write to monitor log.");
                for (key, value) in display_map.iter() {
                    writeln!(self.log_file, "{:?}: {:?}", key, value)
                        .expect("Unable to write to monitor log.");
                }
                writeln!(self.log_file, "###  ###");

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
            let sleep_time = cmp::max(10, 1000 - self.last_run_time as i128);
            println!(
                "Run time: {}, Sleep time: {}",
                self.last_run_time, sleep_time
            );
            thread::sleep(time::Duration::from_millis(sleep_time as u64));
        }

        self
    }
}

fn create_event_key(
    date: DateTime<Local>,
    line_number: i32,
    trigger_type: TriggerType,
    power_name: &String,
) -> EventKey {
    EventKey {
        log_date: date,
        line_number: line_number,
        trigger_type: trigger_type,
        power_name: power_name.clone(),
    }
}

fn create_display_message(
    trigger_type: TriggerType,
    log_date: DateTime<Local>,
    config: &MonitorConfig,
    action: &Action,
) -> MessageDetails {
    let display_start = log_date + chrono::Duration::seconds(action.delay_secs as i64);
    let time_step = action.display_secs / 3;
    MessageDetails {
        trigger_type: trigger_type,
        power_name: action.power_name.clone(),
        output_text: action.output_text.clone(),
        escalation_one_time: display_start,
        escalation_one_font_size: *config.font_size.get(0).unwrap(),
        escalation_one_color: config.display_colors.get(0).unwrap().to_string(),
        escalation_two_time: display_start + chrono::Duration::seconds(time_step as i64),
        escalation_two_font_size: *config.font_size.get(1).unwrap(),
        escalation_two_color: config.display_colors.get(1).unwrap().to_string(),
        escalation_three_time: display_start + chrono::Duration::seconds((time_step * 2) as i64),
        escalation_three_font_size: *config.font_size.get(2).unwrap(),
        escalation_three_color: config.display_colors.get(2).unwrap().to_string(),
        end_time: display_start + chrono::Duration::seconds(action.display_secs as i64),
    }
}

pub fn get_messages() -> (DateTime<Local>, SessionStats, Vec<MonitorMessage>) {
    let details = DISPLAY_MESSAGES.lock().unwrap();
    let now = Local::now();
    let time_point = now.timestamp();
    let mut messages: Vec<MonitorMessage> = Vec::new();
    for value in details.values() {
        if value.escalation_three_time.timestamp() <= time_point {
            messages.push(MonitorMessage {
                output_text: value.output_text.clone(),
                color: value.escalation_three_color.clone(),
                font_size: value.escalation_three_font_size,
            });
        } else if value.escalation_two_time.timestamp() <= time_point {
            messages.push(MonitorMessage {
                output_text: value.output_text.clone(),
                color: value.escalation_two_color.clone(),
                font_size: value.escalation_two_font_size,
            });
        } else if value.escalation_one_time.timestamp() <= time_point {
            messages.push(MonitorMessage {
                output_text: value.output_text.clone(),
                color: value.escalation_one_color.clone(),
                font_size: value.escalation_one_font_size,
            });
        }
    }

    (now, SESSION_STATS.lock().unwrap().clone(), messages)
}
