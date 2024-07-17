use std::path::PathBuf;

use actix_files as fs;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{
    damage_by_power_table, damage_dealt_by_type_table, damage_taken_by_mob_power_table,
    damage_taken_by_mob_table, damage_taken_by_type_table,
    db_actions::{self},
    dps_interval_table, find_all_summaries, generate_index, get_last_modified_file_in_dir,
    log_processing::{self, ParserJob, ProcessingError},
    powers_and_mobs_table::{self},
    read_log_file_dir, AppContext,
};

#[derive(Deserialize)]
struct FileFormData {
    log_path: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SortDirection {
    ASC,
    DESC,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TableNames {
    DamageDealtByType,
    DamageTakenByType,
    DamageTakenByMob,
    DamageTakenByMobPower,
    DPSIntervals,
}

#[derive(Deserialize, Debug)]
pub struct TableQuery {
    pub key: i32,
    pub db_path: String,
    pub table_name: Option<TableNames>,
    pub sort_field: Option<String>,
    pub sort_dir: Option<SortDirection>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PowerTableActions {
    Revert,
    RemoveNonDamaging,
    Merge,
    Delete,
}

#[derive(Deserialize, Debug)]
pub struct DamageByPowerQuery {
    pub key: i32,
    pub db_path: String,
    pub sort_field: Option<String>,
    pub sort_dir: Option<SortDirection>,
    pub action: Option<PowerTableActions>,
    pub power_row: Option<Vec<u8>>,
}

#[derive(Deserialize, Debug)]
pub struct PowersMobsData {
    pub key: i32,
    pub db_path: String,
    pub table_name: Option<TableNames>,
    pub power_name: Option<String>,
    pub mob_name: Option<String>,
    pub sort_field: Option<String>,
    pub sort_dir: Option<SortDirection>,
}

fn create_parser_job(path_buf: PathBuf) -> Result<ParserJob, ParserJob> {
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

fn create_job_response(context: &AppContext, job: ParserJob) -> impl Responder {
    let mut result_context = Context::new();
    result_context.insert("result", &job);
    result_context.insert("error_count", &job.errors.len());
    let result = context.tera.render("job_result.html", &result_context);
    match result {
        Ok(data) => HttpResponse::Ok()
            .insert_header((
                "refresh",
                format!("5;url=http://{}:{}", context.web_address, context.web_port),
            ))
            .insert_header(("no-cache", "no-cache"))
            .body(data),
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
        Err(job) => create_job_response(&context, job),
    }
}

#[get("/process_latest")]
async fn process_latest(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let form: web::Query<FileFormData> = web::Query::from_query(req.query_string()).unwrap();
    println!("Latest Request: {:?}", form.log_path);

    let stripped_file = form.log_path.replace("\"", "");
    match create_parser_job(get_last_modified_file_in_dir(&stripped_file.into())) {
        Ok(job) => {
            let result = job.process_logs(&context);

            let indexes = find_all_summaries(&context.output_dir);
            generate_index(&context, &indexes);
            create_job_response(&context, result)
        }
        Err(job) => create_job_response(&context, job),
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
        Err(job) => create_job_response(&context, job),
    }
}
#[get("/damage_by_power")]
async fn damage_by_power(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
     let qs_non_strict = serde_qs::Config::new(5, false);
     let query: DamageByPowerQuery = qs_non_strict.deserialize_str(&req.query_string()).unwrap();
    println!("{:?}", query);

    let mut table_context = Context::new();
    damage_by_power_table::process(&mut table_context, &query);
    let result = context.tera.render("damage_by_power.html", &table_context);
    match result {
        Ok(data) => {
            println!("=================");
            println!("{}", data.len());
            HttpResponse::Ok().body(data)
        }
        Err(e) => {
            println!("Could not render {}:{:?}", "damage_by_power.html", e);
            HttpResponse::Ok().body("NO DATA")
        }
    }
}

#[get("/damage_table")]
async fn damage_table(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let query: web::Query<TableQuery> = web::Query::from_query(req.query_string()).unwrap();

    match &query.table_name {
        Some(table_name) => {
            let mut table_context = Context::new();
            match table_name {
                TableNames::DamageDealtByType => {
                    damage_dealt_by_type_table::process(&mut table_context, &query);
                }
                TableNames::DamageTakenByType => {
                    damage_taken_by_type_table::process(&mut table_context, &query);
                }
                TableNames::DamageTakenByMob => {
                    damage_taken_by_mob_table::process(&mut table_context, &query);
                }
                TableNames::DamageTakenByMobPower => {
                    damage_taken_by_mob_power_table::process(&mut table_context, &query);
                }
                TableNames::DPSIntervals => {
                    dps_interval_table::process(&context, &mut table_context, &query);
                }
            }
            let result = context.tera.render("simple_table.html", &table_context);
            match result {
                Ok(data) => HttpResponse::Ok().body(data),
                Err(e) => {
                    println!("Could not render {}:{:?}", "simple_table.html", e);
                    HttpResponse::Ok().body("NO DATA")
                }
            }
        }
        None => HttpResponse::Ok().body("NO DATA"),
    }
}

#[get("/powers_and_mobs")]
async fn powers_and_mobs_query(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let selected: web::Query<PowersMobsData> = web::Query::from_query(req.query_string()).unwrap();

    let mut table_context = Context::new();
    table_context.insert(
        "damaging_powers",
        &db_actions::get_damaging_powers(&selected),
    );
    table_context.insert("mobs_damaged", &db_actions::get_mobs_damaged(&selected));
    table_context.insert("headers", &powers_and_mobs_table::headers());

    match db_actions::get_damage_dealt_by_power_or_mob(&selected) {
        Some(mut data) => {
            if selected.sort_field.is_some() {
                powers_and_mobs_table::sort(
                    selected.sort_field.clone().unwrap(),
                    selected.sort_dir.clone().unwrap(),
                    &mut data,
                );
            }
            if selected.power_name.is_some() {
                table_context.insert("power_name", &selected.power_name);
            } else if selected.mob_name.is_some() {
                table_context.insert("mob_name", &selected.mob_name);
            }
            let rows = powers_and_mobs_table::flatten(data);
            table_context.insert("table_rows", &rows);
        }
        None => (),
    }

    match &selected.sort_dir {
        Some(dir) => match dir {
            SortDirection::ASC => table_context.insert("sort_dir", &SortDirection::DESC),
            SortDirection::DESC => table_context.insert("sort_dir", &SortDirection::ASC),
        },
        None => table_context.insert("sort_dir", &SortDirection::DESC),
    }

    let result = context
        .tera
        .render("powers_and_mobs_table.html", &table_context);
    match result {
        Ok(data) => HttpResponse::Ok().body(data),
        Err(e) => panic!("Could not render {}:{:?}", "powers_and_mobs_table.html", e),
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
            .service(damage_by_power)
            .service(damage_table)
            .service(powers_and_mobs_query)
            .service(
                fs::Files::new("/", context.output_dir.to_owned())
                    .index_file("index.html")
                    .show_files_listing(),
            )
    });
    server.bind((address, port))?.run().await
}
