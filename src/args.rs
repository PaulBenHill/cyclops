use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "Cyclops")]
#[command(author = "Ben Hill <benhill70@yahoo.com")]
#[command(version = ".04")]
#[command(about = "Application to parse City Of Heroes log files", long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        required = false,
        value_delimiter = ',',
        conflicts_with = "files",
        value_name = "Directory where you game chat files are stored. All files in the directory will be processed."
    )]
    pub logdir: Option<Vec<PathBuf>>,
    #[arg(
        short,
        long,
        required = false,
        value_delimiter = ',',
        conflicts_with = "logdir",
        value_name = "List of game log files comma separated."
    )]
    pub files: Option<Vec<PathBuf>>,
    #[arg(
        short,
        long,
        required = false,
        value_name = "Time in seconds between combat sessions for DPS reports"
    )]
    pub interval: Option<usize>,
    #[arg(
        short,
        long,
        required = false,
        value_name = "Directory where you want the reports written. Defaults to \"output\""
    )]
    pub outputdir: Option<PathBuf>,
    #[arg(short, long, required = false, value_name = "Web server IP address")]
    pub address: Option<String>,
    #[arg(short, long, required = false, value_name = "Web server port")]
    pub port: Option<usize>,
}
