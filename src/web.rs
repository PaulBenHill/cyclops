use actix_files as fs;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::path::*;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
pub async fn start(output_dir: String) -> std::io::Result<()> {
    //HttpServer::new(|| App::new().service(hello))
    HttpServer::new(move || {
        App::new().service(fs::Files::new("/", output_dir.to_owned()).show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
