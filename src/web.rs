use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use actix_files as fs;
use actix_web::{
    get,
    web::{self},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{
    damage_by_power_table, damage_dealt_by_type_table, damage_taken_by_mob_power_table,
    damage_taken_by_mob_table, damage_taken_by_type_table,
    db_actions::{self},
    dps_interval_table, get_last_modified_file_in_dir,
    index_handler,
    log_processing::ParserJob,
    player_summary_table::{self, SummaryQuery},
    powers_and_mobs_table::{self},
    AppContext,
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

#[derive(Serialize, Deserialize, Debug)]
pub enum ParseLog {
    EntireDir,
    LatestFile,
    SingleFile,
    Directory,
}

#[derive(Deserialize, Debug)]
pub struct ParseLogRequest {
    pub action: ParseLog,
    pub log_path: String,
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

fn create_job_response(context: &AppContext, job: ParserJob) -> impl Responder {
    let mut result_context = Context::new();
    result_context.insert("result", &job);
    result_context.insert("error_count", &job.errors.len());
    let result = context.tera.render("job_result.html", &result_context);
    match result {
        Ok(data) => HttpResponse::Ok()
            .insert_header(("HX-Trigger", "{\"newSummary\": \"fire\"}"))
            .body(data),
        Err(e) => HttpResponse::Ok().body(format!("ERROR CHECK LOGS: {:?}", e)),
    }
}

#[get("parse_request")]
async fn parse_request(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let form: web::Query<ParseLogRequest> = web::Query::from_query(req.query_string()).unwrap();
    println!("Latest Request: {:?}", form.log_path);
    let stripped_path = PathBuf::from(form.log_path.replace("\"", ""));

    match form.action {
        ParseLog::EntireDir => process_all_files(&stripped_path, context),
        ParseLog::LatestFile => {
            let latest_file = get_last_modified_file_in_dir(&stripped_path.into());
            process_all_files(&latest_file, context)
        }
        ParseLog::SingleFile => process_all_files(&stripped_path, context),
        ParseLog::Directory => process_all_files(&stripped_path, context),
    }
}

#[get("/process_latest")]
async fn process_latest(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let form: web::Query<FileFormData> = web::Query::from_query(req.query_string()).unwrap();
    println!("Latest Request: {:?}", form.log_path);

    let stripped_file = form.log_path.replace("\"", "");
    match index_handler::create_parser_job(get_last_modified_file_in_dir(&stripped_file.into())) {
        Ok(job) => {
            let result = job.process_logs(&context);

            let indexes = index_handler::find_all_summaries(&context.output_dir);
            index_handler::generate_index(&context, &indexes);
            create_job_response(&context, result)
        }
        Err(job) => create_job_response(&context, job),
    }
}

fn process_all_files(log_path: &PathBuf, context: web::Data<AppContext>) -> impl Responder {
    match index_handler::create_parser_job(log_path.into()) {
        Ok(job) => {
            let result = job.process_logs(&context);

            let indexes = index_handler::find_all_summaries(&context.output_dir);
            index_handler::generate_index(&context, &indexes);
            create_job_response(&context, result)
        }
        Err(job) => create_job_response(&context, job),
    }
}

#[get("/")]
async fn index(_: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let indexes = index_handler::find_all_summaries(&context.output_dir);

    let mut log_dirs: HashSet<PathBuf> = HashSet::new();
    for i in indexes {
        let f = Path::new(&i.log_file);
        if f.is_dir() {
            log_dirs.insert(f.to_path_buf());
        } else {
            log_dirs.insert(f.parent().unwrap().to_path_buf());
        }
    }
    let mut dir_list: Vec<PathBuf> = log_dirs.into_iter().collect();
    dir_list.sort();

    let mut index_context = Context::new();
    index_context.insert("log_dirs", &dir_list);
    let result = context.tera.render("index.html", &index_context);
    match result {
        Ok(data) => HttpResponse::Ok().body(data),
        Err(e) => panic!("Could not render {}:{:?}", "index.html", e),
    }
}

#[get("/index_table")]
async fn index_table(_: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let data = index_handler::load_summaries(&context);

    HttpResponse::Ok().body(data)
}

#[get("/damage_by_power")]
async fn damage_by_power(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let qs_non_strict = serde_qs::Config::new(5, false);
    let query: DamageByPowerQuery = qs_non_strict.deserialize_str(&req.query_string()).unwrap();

    let mut table_context = Context::new();
    damage_by_power_table::process(&mut table_context, &query);
    let result = context.tera.render("damage_by_power.html", &table_context);
    match result {
        Ok(data) => HttpResponse::Ok().body(data),
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

#[get("/summary")]
async fn player_summary_query(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let query: web::Query<SummaryQuery> = web::Query::from_query(req.query_string()).unwrap();
    let mut report_context = Context::new();

    player_summary_table::process(&context, &mut report_context, &query);
    let result = context
        .tera
        .render("player_attack_report.html", &report_context);
    match result {
        Ok(data) => HttpResponse::Ok().body(data),
        Err(e) => panic!("Could not render {}:{:?}", "player_attack_report.html", e),
    }
}

#[actix_web::main]
pub async fn start(context: AppContext) -> std::io::Result<()> {
    let address = context.web_address.to_string();
    let port = context.web_port;
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(context.clone()))
            .service(index)
            .service(index_table)
            .service(parse_request)
            .service(process_latest)
            .service(player_summary_query)
            .service(damage_by_power)
            .service(damage_table)
            .service(powers_and_mobs_query)
            .service(fs::Files::new(
                "/resources",
                context.resources_dir.to_owned(),
            ))
            .service(
                fs::Files::new("/", context.output_dir.to_owned())
                    .index_file("index.html")
                    .show_files_listing(),
            )
    });
    server.bind((address, port))?.run().await
}
