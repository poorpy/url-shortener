mod redis_actor;

use std::process;

use actix::{Actor, Addr};
use actix_web::{self, post, web, App, HttpResponse, HttpServer, Responder};
use base64::encode_config;
use crc64::crc64;
use redis_actor::RedisActor;
use serde::Deserialize;

use crate::redis_actor::PutCommand;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let actor = RedisActor::new("redis://127.0.0.1:6379")
        .await
        .unwrap_or_else(|err| {
            println!("Problem connecting to redis: {}", err);
            process::exit(1)
        });
    let addr = actor.start();
    HttpServer::new(move || App::new().service(handle).data(addr.clone()))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

#[derive(Deserialize)]
struct UrlRequest {
    pub url: String,
    pub user: String,
}

#[post("/")]
async fn handle(
    request: web::Json<UrlRequest>,
    actor: web::Data<Addr<RedisActor>>,
) -> impl Responder {
    let bytes: [u8; 8] = crc64(0, request.url.as_bytes()).to_be_bytes();
    let encoded = encode_config(bytes, base64::URL_SAFE);

    if let Err(err) = actor
        .send(PutCommand {
            key: encoded.clone(),
            value: request.url.clone(),
        })
        .await
    {
        return HttpResponse::InternalServerError().body(format!("Err: {}", err));
    };
    HttpResponse::Ok().body(format!(
        "Url: {} user: {} encoded: {}",
        request.url, request.user, encoded
    ))
}
