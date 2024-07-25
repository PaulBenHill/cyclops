use std::path::PathBuf;

use crate::{
    find_all_summaries, generate_index,
    log_processing::{self, ParserJob, ProcessingError},
    read_log_file_dir,
    AppContext
};

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
