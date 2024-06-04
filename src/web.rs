use std::{borrow::Borrow, path::PathBuf};

use actix_files as fs;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use tera::Context;

use crate::{
    find_all_summaries, generate_index, get_last_modified_file_in_dir,
    log_processing::{self, ParserJob, ProcessingError},
    read_log_file_dir, AppContext,
};

#[derive(Deserialize)]
struct FileFormData {
    log_path: String,
}

fn create_parser_job(path_buf: PathBuf) -> Result<ParserJob, ParserJob> {
    let mut parser_job = ParserJob {
        files: Vec::new(),
        processed: 0,
        run_time: 0,
        errors: Vec::new(),
    };

    let local_path = path_buf.to_owned();
    match log_processing::verify_file(local_path) {
        Ok(path) => {
            if path.is_file() {
                parser_job.files.push(path);
            } else if path.is_dir() {
               let mut files = read_log_file_dir(path);
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

fn create_job_response(context: &AppContext, job: ParserJob) -> impl Responder {
    let mut result_context = Context::new();
    result_context.insert("result", &job);
    result_context.insert("error_count", &job.errors.len());
    let result = context.tera.render("job_result.html", &result_context);
    match result {
        Ok(data) => {
            HttpResponse::Ok()
        .insert_header(("refresh", "5;url=http://localhost:11227"))
        .insert_header(("no-cache", "no-cache"))
        .body(data)
        }
        Err(e) => panic!("Could not render {}:{:?}", "index.html", e),
    }
}

#[get("/process_file")]
async fn process_file(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let form: web::Query<FileFormData> = web::Query::from_query(req.query_string()).unwrap();
    println!("Latest Request: {:?}", form.log_path);

    let stripped_file = form.log_path.replace("\"", "");
    match create_parser_job(stripped_file.into()) {
        Ok(job) => {
            let result = job.process_logs(&context);

            let indexes = find_all_summaries(&context.output_dir);
            generate_index(&context, &indexes);
            create_job_response(&context, result)
        }
        Err(job) => {
            create_job_response(&context, job)
        },
    }
}

#[get("/process_latest")]
async fn process_latest(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let form: web::Query<FileFormData> = web::Query::from_query(req.query_string()).unwrap();
    println!("Latest Request: {:?}", form.log_path);

    let stripped_file = form.log_path.replace("\"", "");
    match create_parser_job(get_last_modified_file_in_dir(stripped_file.into())) {
        Ok(job) => {
            let result = job.process_logs(&context);

            let indexes = find_all_summaries(&context.output_dir);
            generate_index(&context, &indexes);
            create_job_response(&context, result)
        }
        Err(job) => {
            create_job_response(&context, job)
        },
    }

}

#[get("/process_dir")]
async fn process_dir(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    process_all_files(req, context)
}

#[get("/parse_dir")]
async fn parse_dir(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    process_all_files(req, context)
}

fn process_all_files(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let form: web::Query<FileFormData> = web::Query::from_query(req.query_string()).unwrap();
    println!("Request: {:?}", form.log_path);

    let stripped_file = form.log_path.replace("\"", "");
    match create_parser_job(stripped_file.into()) {
        Ok(job) => {
            let result = job.process_logs(&context);

            let indexes = find_all_summaries(&context.output_dir);
            generate_index(&context, &indexes);
            create_job_response(&context, result)
        }
        Err(job) => {
            create_job_response(&context, job)
        },
    }
}

#[actix_web::main]
pub async fn start(context: AppContext) -> std::io::Result<()> {
    let address = context.web_address.to_string();
    let port = context.web_port;
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(context.clone()))
            .service(process_file)
            .service(parse_dir)
            .service(process_dir)
            .service(process_latest)
            .service(
                fs::Files::new("/", context.output_dir.to_owned())
                    .index_file("index.html")
                    .show_files_listing(),
            )
    });
    server.bind((address, port))?.run().await
}
