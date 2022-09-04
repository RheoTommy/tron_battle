use actix_web::web::Json;
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use game::{Board, DEPTH, Request};
use std::time::Instant;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(handle))
        .bind("localhost:6583")?
        .run()
        .await
}

#[post("/")]
async fn handle(req: Json<Request>) -> impl Responder {
    eprintln!("Access");
    let state = Board::from_request(req.into_inner());
    let start = Instant::now();
    let op = state.min_max(DEPTH);
    eprintln!("{}\n{:?}", state, op);
    eprintln!("{}ms", start.elapsed().as_millis());
    HttpResponse::Ok().json(op)
}
