use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Lines, Write};
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

#[derive(Parser)]
#[command(name = "Cyclops")]
#[command(author = "Ben Hill <benhill70@yahoo.com")]
#[command(version = ".01")]
#[command(about = "Application to parse City Of Heroes log files", long_about = None)]
struct Args {
    #[arg(short, long, value_name = "DIR")]
    logdir: Option<PathBuf>,
    #[arg(short, long)]
    interval: Option<usize>,
    #[arg(short, long, value_name = "DIR")]
    outputdir: Option<PathBuf>,
    #[arg(short, long, value_name = "FILES")]
    pub files: Option<Vec<PathBuf>>,
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
    counts: Vec<u32>,
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

    if !create_output_dir(&output_path) {
        std::process::exit(-1);
    }

    for file in log_file_names {
        let result = verify_file(&file);
        let file_name = result.0.file_name().unwrap().to_str().unwrap();

        let reader = open_log_file(result.0);

        let lines = reader.lines();

        let reports: (Vec<FileDataPoint>, Vec<SummaryReport>) = process_lines(lines);

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
    }

    println!("Total run time took: {} second.", start.elapsed().as_secs());
}

fn read_log_file_dir(dir: PathBuf) -> Vec<String> {
    match fs::canonicalize(&dir) {
        Ok(path) => {
            if path.exists() && path.is_dir() {
                let mut file_list: Vec<String> = Vec::new();
                for file in fs::read_dir(path).unwrap() {
                    let file_name = file.unwrap().path().into_os_string().into_string().unwrap();
                    file_list.push(file_name);
                }
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
    parsed_lines: &Vec<FileDataPoint>,
    index: usize,
    summary: &SummaryReport,
    dps_interval: usize,
) {
    // files to write
    // original data file
    // parsed log
    // error log todo
    // summary files for each session
    if let Err(e) = std::fs::copy(data_file, report_dir.join(file_name)) {
        println!("Copying data file return zero bytes: {}", e);
    }

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

        dps_reports.push(vec![
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

    let inital_points = summary.get_targets_effected_by_power();

    // for p in &inital_points {
    //     println!("{:?}", p);
    // }
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
        // println!(
        // "Inital: {}:{}:{}",
        // power_name, initial_activation_line_number, next_activation_line_number
        // );
        for np in inital_points.clone() {
            if np.activation
                && np.line_number > initial_activation_line_number
                && np.power_name == power_name
            {
                next_activation_line_number = np.line_number;
                //println!("NP: {}:{}", next_activation_line_number, power_name);
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

        if next_activation_line_number == u32::MAX {
            break;
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

    for r in effected_reports.values_mut() {
        r.average_hits = (r.total_hits as f32 / r.activations as f32);
        r.median = median(&mut r.counts.clone());
        r.mode = mode(&mut r.counts.clone());
        println!("Final: {}", serde_json::to_string(r).unwrap());
    }
    let effected: Vec<&EffectedReport> = effected_reports.values().collect();

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

fn create_output_dir(output_path: &PathBuf) -> bool {
    let result: bool = match fs::canonicalize(&output_path) {
        Ok(path) => {
            println!("Output dir: {:?}", path);
            true
        }
        Err(_) => match fs::create_dir(&output_path) {
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
    println!(
        "File opened for processing: {}",
        path.as_os_str()
            .to_str()
            .expect("Could not create a file name from os string.")
    );
    BufReader::new(file)
}

fn process_lines(lines: Lines<BufReader<File>>) -> (Vec<FileDataPoint>, Vec<SummaryReport>) {
    let mut line_count: u32 = 0;
    let parsers = initialize_matcher();
    let mut data_points: Vec<FileDataPoint> = Vec::with_capacity(50000);

    let start = Instant::now();
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
    println!(
        "Matching and conversion: {} second.",
        start.elapsed().as_secs()
    );

    let start = Instant::now();
    let summaries: Vec<SummaryReport> = total_player_attacks(&data_points);
    println!(
        "Generating summaries took: {} second.",
        start.elapsed().as_secs()
    );

    data_points.shrink_to_fit();

    (data_points, summaries)
}
