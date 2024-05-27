use actix_files as fs;
use actix_web::{
    get, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;

use crate::{
    find_all_summaries, generate_index, get_last_modified_file_in_dir, log_processing,
    read_log_file_dir, AppContext,
};

#[derive(Deserialize)]
struct FileFormData {
    log_path: String,
}


#[get("/process_file")]
async fn process_file(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let form: web::Query<FileFormData> = web::Query::from_query(req.query_string()).unwrap();
    println!("Latest Request: {:?}", form.log_path);

    let stripped_file = form.log_path.replace("\"", "");
    let path = log_processing::verify_file(&stripped_file);
    let mut files: Vec<String> = Vec::new();
    files.push(path.to_path_buf().into_os_string().into_string().unwrap());

    println!("Process file {:?}", files);
    log_processing::process_logs(&context, files);

    let indexes = find_all_summaries(&context.output_dir);
    generate_index(&context, &indexes);


    HttpResponse::Ok()
        .insert_header(("refresh", "1;url=http://localhost:11227"))
        .insert_header(("no-cache", "no-cache"))
        .finish()
}

#[get("/process_latest")]
async fn process_latest(req: HttpRequest, context: web::Data<AppContext>) -> impl Responder {
    let form: web::Query<FileFormData> = web::Query::from_query(req.query_string()).unwrap();
    println!("Latest Request: {:?}", form.log_path);

    let path = log_processing::verify_file(&form.log_path);
    let mut files: Vec<String> = Vec::new();
    files.push(get_last_modified_file_in_dir(path.to_path_buf()));

    println!("Lastest file {:?}", files);
    log_processing::process_logs(&context, files);

    let indexes = find_all_summaries(&context.output_dir);
    generate_index(&context, &indexes);

    HttpResponse::Ok()
        .insert_header(("refresh", "1;url=http://localhost:11227"))
        .insert_header(("no-cache", "no-cache"))
        .finish()
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
    let path = log_processing::verify_file(&stripped_file);
    let files = read_log_file_dir(path.to_path_buf());

    println!("File count found in directory {:?}: {}", path, files.len());
    log_processing::process_logs(&context, files);

    let indexes = find_all_summaries(&context.output_dir);
    generate_index(&context, &indexes);

    HttpResponse::Ok()
        .insert_header(("refresh", "1;url=http://localhost:11227"))
        .insert_header(("no-cache", "no-cache"))
        .finish()
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
