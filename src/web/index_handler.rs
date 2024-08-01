use std::{
    collections::HashSet, fs, path::{Path, PathBuf}, sync::Mutex
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tera::Context;
use walkdir::WalkDir;

use crate::{
    db,
    log_processing::{self, ParserJob, ProcessingError},
    models::IndexDetails,
    read_log_file_dir, AppContext,
};

lazy_static! {
    static ref INDEX_CACHE: Mutex<IndexCache> = Mutex::new(IndexCache::new());
}

#[derive(Clone)]
pub struct IndexCache {
    pub log_dirs: Vec<PathBuf>,
    pub player_names: Vec<String>,
    pub summaries: Vec<SummaryEntry>,
}

impl IndexCache {
    pub fn new() -> Self {
        IndexCache {
            log_dirs: Vec::<PathBuf>::new(),
            player_names: Vec::<String>::new(),
            summaries: Vec::<SummaryEntry>::new(),
        }
    }

    pub fn update(
        &mut self,
        dirs: HashSet<PathBuf>,
        names: HashSet<String>,
        mut summaries: Vec<SummaryEntry>,
    ) -> &mut Self {
        let mut sorted_summaries: Vec<SummaryEntry> = summaries.into_iter().filter(|s| s.indexes.len() > 0).collect();
        sorted_summaries.sort_by(|a, b| b.log_date.cmp(&a.log_date));

        let mut player_vec: Vec<String> = names.into_iter().collect();
        player_vec.sort();

        let mut path_vec: Vec<PathBuf> = dirs.into_iter().collect();
        path_vec.sort();

        self.log_dirs.clear();
        self.player_names.clear();
        self.summaries.clear();

        self.log_dirs.append(&mut path_vec);
        self.player_names.append(&mut player_vec);
        self.summaries.append(&mut sorted_summaries);

        self
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SummaryEntry {
    pub log_file: String,
    pub log_date: String,
    pub db_path: PathBuf,
    pub indexes: Vec<IndexDetails>,
}

pub fn create_parser_job(path_buf: &PathBuf) -> Result<ParserJob, ParserJob> {
    let mut parser_job = ParserJob {
        files: Vec::new(),
        processed: 0,
        run_time: 0,
        errors: Vec::new(),
        completion_date: "".to_string(),
        last_file: "".to_string(),
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
            Ok(parser_job)
        }
        Err(e) => {
            parser_job.errors.push(ProcessingError {
                file_name: path_buf.to_owned(),
                message: e.to_string(),
            });
            Err(parser_job)
        }
    }
}

pub fn load_summaries(context: &AppContext) -> String {
    let cache = find_all_summaries(&context.output_dir);
    generate_index(&context, None, None, cache)
}

pub fn search_by_player_name(player_name: &String, context: &AppContext) -> String {
    let cache = find_summaries_by_player_name(&player_name, &context.output_dir);
    generate_index(&context, Some(player_name), None, cache)
}

pub fn search_by_directory(log_dir: &PathBuf, context: &AppContext) -> String {
    let cache = find_summaries_by_directory(&log_dir, &context.output_dir);
    generate_index(&context, None, Some(log_dir), cache)
}

pub fn search_by_log_file(log_file: &PathBuf, context: &AppContext) -> String {
    let cache = find_summaries_by_log_file(&log_file, &context.output_dir);
    generate_index(&context, None, None, cache)
}

pub fn find_summaries_by_log_file(file_name: &PathBuf, output_path: &Path) -> IndexCache {
    let mut cache = INDEX_CACHE.lock().unwrap();

    let mut player_set = HashSet::<String>::new();
    let mut log_dirs: HashSet<PathBuf> = HashSet::new();
    let mut entries = Vec::<SummaryEntry>::new();
    let walker = WalkDir::new(output_path).into_iter();

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.path().ends_with("summary.db") {
            let db_path = fs::canonicalize(entry.path()).unwrap().to_path_buf();
            let mut conn = db::get_file_conn(db_path.clone());
            let details = db::queries::index_details(&mut conn);

            let mut entry = SummaryEntry {
                log_file: details.get(0).unwrap().file.to_owned(),
                log_date: details.get(0).unwrap().log_date.to_owned(),
                db_path: entry.path().to_path_buf(),
                indexes: Vec::new(),
            };
            for d in &details {
                player_set.insert(d.player_name.clone());
                let index_file = Path::new(&d.file).to_path_buf();
                let parent = index_file.parent().unwrap().to_path_buf();
                log_dirs.insert(parent.clone());

                if index_file.eq(file_name) {
                    entry.indexes.push(d.clone());
                }
            }

            entries.push(entry);
        }
    }

    cache.update(log_dirs, player_set, entries).clone()
}

pub fn find_summaries_by_player_name(name: &String, output_path: &Path) -> IndexCache {
    let mut cache = INDEX_CACHE.lock().unwrap();

    let mut player_set = HashSet::<String>::new();
    let mut log_dirs: HashSet<PathBuf> = HashSet::new();
    let mut entries = Vec::<SummaryEntry>::new();
    let walker = WalkDir::new(output_path).into_iter();

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.path().ends_with("summary.db") {
            let db_path = fs::canonicalize(entry.path()).unwrap().to_path_buf();
            let mut conn = db::get_file_conn(db_path.clone());
            let details = db::queries::index_details(&mut conn);

            let mut entry = SummaryEntry {
                log_file: details.get(0).unwrap().file.to_owned(),
                log_date: details.get(0).unwrap().log_date.to_owned(),
                db_path: entry.path().to_path_buf(),
                indexes: Vec::new(),
            };
            for d in &details {
                player_set.insert(d.player_name.clone());
                let f = Path::new(&d.file);
                log_dirs.insert(f.parent().unwrap().to_path_buf());

                if d.player_name.eq(name) {
                    entry.indexes.push(d.clone());
                }
            }

            entries.push(entry);
        }
    } 

    cache.update(log_dirs, player_set, entries).clone()
}

pub fn find_summaries_by_directory(log_dir: &PathBuf, output_path: &Path) -> IndexCache {
    let mut cache = INDEX_CACHE.lock().unwrap();

    let mut player_set = HashSet::<String>::new();
    let mut log_dirs: HashSet<PathBuf> = HashSet::new();
    let mut entries = Vec::<SummaryEntry>::new();
    let walker = WalkDir::new(output_path).into_iter();

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.path().ends_with("summary.db") {
            let db_path = fs::canonicalize(entry.path()).unwrap().to_path_buf();
            let mut conn = db::get_file_conn(db_path.clone());
            let details = db::queries::index_details(&mut conn);

            let mut entry = SummaryEntry {
                log_file: details.get(0).unwrap().file.to_owned(),
                log_date: details.get(0).unwrap().log_date.to_owned(),
                db_path: entry.path().to_path_buf(),
                indexes: Vec::new(),
            };
            for d in &details {
                player_set.insert(d.player_name.clone());
                let parent = Path::new(&d.file).parent().unwrap().to_path_buf();
                log_dirs.insert(parent.clone());

                if parent.eq(log_dir) {
                    entry.indexes.push(d.clone());
                }
            }

            entries.push(entry);
        }
    }

    cache.update(log_dirs, player_set, entries).clone()
}

pub fn find_all_summaries(output_path: &Path) -> IndexCache {
    let mut cache = INDEX_CACHE.lock().unwrap();

    let mut player_set = HashSet::<String>::new();
    let mut log_dirs: HashSet<PathBuf> = HashSet::new();
    let mut entries: Vec<SummaryEntry> = Vec::new();

    let walker = WalkDir::new(output_path).into_iter();
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.path().ends_with("summary.db") {
            let db_path = fs::canonicalize(entry.path()).unwrap().to_path_buf();
            let mut conn = db::get_file_conn(db_path.clone());
            let details = db::queries::index_details(&mut conn);

            let mut entry = SummaryEntry {
                log_file: details.get(0).unwrap().file.to_owned(),
                log_date: details.get(0).unwrap().log_date.to_owned(),
                db_path: entry.path().to_path_buf(),
                indexes: Vec::new(),
            };
            for d in &details {
                player_set.insert(d.player_name.clone());
                let f = Path::new(&d.file);
                log_dirs.insert(f.parent().unwrap().to_path_buf());
                entry.indexes.push(d.clone());
            }

            entries.push(entry);
        }
    }
    cache.update(log_dirs, player_set, entries).clone()
}

pub fn generate_index(
    context: &AppContext,
    last_player_name: Option<&String>,
    last_log_dir: Option<&PathBuf>,
    cache: IndexCache,
) -> String {
    let mut index_content = Context::new();
    if last_player_name.is_some() {
        index_content.insert("last_player_name", last_player_name.unwrap());
    }
    if last_log_dir.is_some() {
        index_content.insert("last_log_dir", last_log_dir.unwrap());
    }
    index_content.insert("players", &cache.player_names);
    index_content.insert("log_dirs", &cache.log_dirs);
    index_content.insert("summaries", &cache.summaries);
    let result = context.tera.render("index_table.html", &index_content);
    match result {
        Ok(data) => data,
        Err(e) => panic!("Could not render {}:{:?}", "index_table.html", e),
    }
}
