use actix_files as fs;
use actix_web::{
    get,
    web::{self},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use index_handler::{IndexSearch, IndexSearchQuery};
use player_summary_table::SummaryQuery;
use powers_and_mobs_table::calc_overkill;
use tera::Context;
use web_structs_enums::{DamageByPowerQuery, ParseLog, ParseLogRequest, PowersMobsData, SortDirection, TableNames, TableQuery};

mod damage_by_power_table;
mod damage_dealt_by_type_table;
mod damage_taken_by_mob_power_table;
mod damage_taken_by_mob_table;
mod damage_taken_by_type_table;
mod dps_interval_table;
mod index_handler;
mod player_summary_table;
mod powers_and_mobs_table;
pub mod web_structs_enums;

use crate::{
    db, game_data::{self, get_mob_hp}, get_last_modified_file_in_dir, log_processing::{self, ParserJob}, AppContext
};

fn create_job_result(context: &AppContext, job: &ParserJob) -> HttpResponse {
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

fn create_job_start(context: &AppContext, job: &ParserJob) -> HttpResponse {
    let mut result_context = Context::new();
    result_context.insert("job", &job);
    result_context.insert("file_count", &job.files.len());
    let result = context.tera.render("job_start.html", &result_context);
    match result {
        Ok(data) => HttpResponse::Ok().body(data),
        Err(e) => HttpResponse::Ok().body(format!("ERROR CHECK LOGS: {:?}", e)),
    }
}

#[get("parse_request")]
async fn parse_request(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let form: web::Query<ParseLogRequest> = web::Query::from_query(req.query_string()).unwrap();
    println!("Latest Request: {:?}", form.log_path);
    let stripped_path = form.log_path.replace("\"", "");

    match form.action {
        ParseLog::ParsePath => match index_handler::create_parser_job(&stripped_path) {
            Ok(job) => {
                log_processing::add_job(job.clone());
                create_job_start(&context, &job)
            }
            Err(e) => create_job_result(&context, &e),
        },
        ParseLog::LatestFile => {
            let latest_file = get_last_modified_file_in_dir(&stripped_path);
            match index_handler::create_parser_job(&latest_file) {
                Ok(job) => {
                    log_processing::add_job(job.clone());
                    create_job_start(&context, &job)
                }
                Err(e) => create_job_result(&context, &e),
            }
        }
    }
}

#[get("/execute_job")]
async fn execute_job(_: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    match log_processing::get_job() {
        Some(job) => create_job_result(&context, &job.process_logs(&context)),
        None => HttpResponse::NoContent().into(),
    }
}

#[get("/")]
async fn index(_: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    index_handler::find_all_summaries(&context.output_dir);

    let result = context.tera.render("index.html", &Context::new());
    match result {
        Ok(data) => HttpResponse::Ok().body(data),
        Err(e) => panic!("Could not render {}:{:?}", "index.html", e),
    }
}

#[get("/refresh_actions")]
async fn refresh_actions(_: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let cache = index_handler::find_all_summaries(&context.output_dir);

    let mut index_context = Context::new();
    if cache.log_dirs.len() > 0 {
        index_context.insert("log_dirs", &cache.log_dirs);
    }
    let result = context.tera.render("index_actions.html", &index_context);
    match result {
        Ok(data) => HttpResponse::Ok()
            .insert_header(("HX-Trigger", "{\"refreshTable\": \"load\"}"))
            .body(data),
        Err(e) => panic!("Could not render {}:{:?}", "index.html", e),
    }
}

#[get("/index_table")]
async fn index_table(_: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let data = index_handler::load_summaries(&context);
    HttpResponse::Ok().body(data)
}

#[get("/index_search")]
async fn index_search(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let query: web::Query<IndexSearchQuery> = web::Query::from_query(req.query_string()).unwrap();

    match query.action {
        IndexSearch::PlayerName => {
            let player_name = query.player_name.clone().unwrap().replace("_", " ");
            let data = index_handler::search_by_player_name(&player_name, &context);
            HttpResponse::Ok().body(data)
        }
        IndexSearch::LogDirectory => {
            let data =
                index_handler::search_by_directory(&query.log_path.clone().unwrap(), &context);
            HttpResponse::Ok().body(data)
        }
        IndexSearch::LogFile => {
            let data =
                index_handler::search_by_log_file(&query.log_file.clone().unwrap(), &context);
            HttpResponse::Ok().body(data)
        }
    }
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
        &db::queries::get_damaging_powers(&selected),
    );
    table_context.insert("mobs_damaged", &db::queries::get_mobs_damaged(&selected));
    table_context.insert("mob_levels", &game_data::MINION_HP_TABLE.as_slice());
    table_context.insert("headers", &powers_and_mobs_table::headers());
    if selected.mob_level.is_some() {
        table_context.insert("mob_level", &i32::from_str_radix(&selected.mob_level.as_ref().unwrap(), 10).unwrap());
    } else {
        table_context.insert("mob_level", &54);
    } 

    match db::queries::get_damage_dealt_by_power_or_mob(&selected) {
        Some(mut data) => {
            let mob_level = selected.mob_level.as_ref().unwrap();
            data.iter_mut()
                .for_each(|r| r.overkill = 
                           calc_overkill(r.damage_per_hit,
                           get_mob_hp(&mob_level).into()));

            if selected.sort_field.is_some() {
                powers_and_mobs_table::sort(
                    selected.sort_field.clone().unwrap(),
                    selected.sort_dir.clone().unwrap(),
                    &mut data,
                );
            }

            if selected.power_name.is_some() && !selected.power_name.as_ref().unwrap().is_empty() {
                table_context.insert("power_name", &selected.power_name);
            } else if selected.mob_name.is_some() && !selected.mob_name.as_ref().unwrap().is_empty() {
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
            .service(refresh_actions)
            .service(index_table)
            .service(index_search)
            .service(execute_job)
            .service(parse_request)
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
