use std::env;
use std::fs::File;
use std::io::Lines;
use std::io::{BufRead, BufReader};
use std::path::*;

mod parsers;
use crate::parsers::*;

mod parser_model;

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = verify_file(args.get(1).unwrap());

    let reader = open_log_file(path);

    let lines = reader.lines();

    process_lines(lines);
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

fn process_lines(lines: Lines<BufReader<File>>) {
    let mut line_count: u32 = 0;
    let mut match_count: u32 = 0;
    let parsers = initialize_matcher();
    for line in lines.flatten() {
        line_count += 1;
        for p in &parsers {
            if let Some(data) = p(line_count, &line) {
                match_count += 1;
                println!("{:?}", data);
                break;
            }
        }
    }
    println!("Line count: {}, Match count: {}", line_count, match_count)
}
