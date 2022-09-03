use crate::game::Board;
use actix_web::web::Json;
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

pub mod game;

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
    let op = state.min_max(10);
    HttpResponse::Ok().json(op)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Request {
    pub size: Point,
    pub player_pos: Point,
    pub ai_pos: Point,
    pub board: Vec<isize>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}
