use std::{fs, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};
use tera::Context;
use walkdir::WalkDir;

use crate::{
    db_actions, log_processing::{self, ParserJob, ProcessingError}, read_log_file_dir, AppContext
};

#[derive(Serialize, Deserialize)]
pub struct SummaryEntry {
    pub log_file: String,
    pub log_date: String,
    pub links: Vec<String>,
    pub data_points: Vec<String>
}

pub fn create_parser_job(path_buf: PathBuf) -> Result<ParserJob, ParserJob> {
    let mut parser_job = ParserJob {
        files: Vec::new(),
        processed: 0,
        run_time: 0,
        errors: Vec::new(),
    };

    let local_path = path_buf.to_owned();
    match log_processing::verify_file(&local_path) {
        Ok(path) => {
            if path.is_file() {
                parser_job.files.push(path.to_owned());
            } else if path.is_dir() {
                let mut files = read_log_file_dir(&path);
                parser_job.files.append(&mut files);
            }
        }
        Err(e) => {
            parser_job.errors.push(ProcessingError {
                file_name: path_buf.to_owned(),
                message: e.to_string(),
            });
        }
    }

    Ok(parser_job)
}

pub fn load_summaries(context: &AppContext) -> String {
    let indexes = find_all_summaries(&context.output_dir);
    generate_index(&context, &indexes)
}

pub fn find_all_summaries(output_path: &Path) -> Vec<SummaryEntry> {
    let mut entries: Vec<SummaryEntry> = Vec::new();
    let walker = WalkDir::new(output_path).into_iter();
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.path().ends_with("summary.db") {
            let db_path = fs::canonicalize(entry.path()).unwrap().to_path_buf();
            let mut conn = db_actions::get_file_conn(db_path.clone());
            let details = db_actions::index_details(&mut conn);

            let mut links: Vec<String> = Vec::new();
            let mut data_points: Vec<String> = Vec::new();
            for d in &details {
                links.push(format!(
                    "<a href=\"/summary?key={}&db_path={}\" target=\"_blank\">{}</a>", d.summary_key, db_path.clone().display(), d.player_name));
                data_points.push(d.data_points.clone());
            }

            let date = details.get(0).unwrap().log_date.to_owned();
            let file = details.get(0).unwrap().file.to_owned();
            let entry = SummaryEntry {
                log_file: file,
                log_date: date,
                links: links,
                data_points: data_points,
            };
            entries.push(entry);
        }
    }

    entries.sort_by(|a, b| b.log_date.cmp(&a.log_date));
    entries
}

pub fn generate_index(context: &AppContext, indexes: &Vec<SummaryEntry>) -> String {
    let mut index_content = Context::new();
    index_content.insert("indexes", &indexes);
    let result = context.tera.render("index_table.html", &index_content);
    match result {
        Ok(data) => data,
        Err(e) => panic!("Could not render {}:{:?}", "index_table.html", e),
    }
}