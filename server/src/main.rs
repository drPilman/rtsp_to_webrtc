use axum::{extract::Extension, Router};
use axum::{routing::get, routing::post}; //extract::Path
use axum_extra::routing::SpaRouter;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
//use uuid::Uuid;

//use clap::{AppSettings, Arg, Command};
use clap::Parser;

pub mod routes;
pub mod structs;
pub mod utils;
pub mod webrtc_impl;

use routes::*;
use structs::*;

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    // enable console logging
    tracing_subscriber::fmt::init();

    let shared_state = Arc::new(RwLock::new(Sources::new()));

    let app = Router::new()
        .route("/api/get_sources_list", get(get_sources_list))
        .route("/api/add_source", get(add_source))
        .route("/api/view", post(view))
        .merge(SpaRouter::new("/assets", opt.static_dir))
        .layer(Extension(shared_state))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        opt.port,
    ));

    log::info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");
}
