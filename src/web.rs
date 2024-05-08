use actix_files as fs;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::path::*;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
pub async fn start(address: String, port: usize, output_dir: String) -> std::io::Result<()> {
    //HttpServer::new(|| App::new().service(hello))
    HttpServer::new(move || {
        App::new().service(
            fs::Files::new("/", output_dir.to_owned())
                .index_file("index.html")
                .show_files_listing(),
        )
    })
    .bind((address, port as u16))?
    .run()
    .await
}
