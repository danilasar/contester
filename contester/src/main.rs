mod actors;
mod podman;
use actix::prelude::*;
use actix_web::{App, HttpServer, http::header::q, web};
use futures_util::StreamExt;
//use futures_util::io::AsyncWriteExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::client::conn::http1::{Connection, SendRequest};
use hyper::{Method, Request, upgrade};
use hyper_util::rt::TokioIo;
use podman_api::{Podman, api::Exec, opts::*};
use serde::Deserialize;
use serde_json::json;
use std::path::Path;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    podman::podman_test().await;
    let addr = podman::MyActor.start();
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(addr.clone()))
            .route("/ping", web::get().to(podman::ping_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
