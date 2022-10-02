use axum::response::{IntoResponse, Response};
use axum::{extract::Extension, Router};
use axum::{http::StatusCode, routing::get, Json}; //extract::Path
use axum_extra::routing::SpaRouter;
use clap::Parser;
use color_eyre::Report; //eyre::eyre,
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
//use uuid::Uuid;

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "127.0.0.1")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "../dist")]
    static_dir: String,
}
struct ReportError(Report);

impl From<Report> for ReportError {
    fn from(err: Report) -> Self {
        ReportError(err)
    }
}

impl IntoResponse for ReportError {
    fn into_response(self) -> Response {
        // {:?} shows the backtrace / spantrace, see
        // https://docs.rs/eyre/0.6.7/eyre/struct.Report.html#display-representations
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal server error: {:?}", self.0),
        )
            .into_response()
    }
}

#[derive(Serialize, Clone)]
struct Source<'a> {
    state: bool,
    url: &'a str,
}

#[derive(Serialize, Clone)]
struct Sources<'b> {
    list: Vec<Source<'b>>,
}
impl<'a> Sources<'a> {
    fn new() -> Self {
        Self {
            list: Vec::with_capacity(5),
        }
    }
    fn add(&mut self, url: &'a str) {
        self.list.push(Source { state: true, url });
    }
}
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
        .route("/api/hello", get(hello))
        .route("/api/get_sources_list", get(get_sources_list))
        .route("/api/add_source", get(add_source))
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

async fn hello() -> impl IntoResponse {
    //async fn hello(Extension(state): Extension<Arc<RwLock<Sources<'_>>>>) -> impl IntoResponse {
    //let data = (*state).read().unwrap();
    /*let res_str = data
        .list
        .iter()
        .map(|x| &*((*x).name))
        .collect::<Vec<&str>>()
        .join(", ");
    format!("Hello, World {res_str}")*/
    "AAAA"
}

async fn get_sources_list(
    Extension(state): Extension<Arc<RwLock<Sources<'_>>>>,
) -> Result<impl IntoResponse + '_, ReportError> {
    let data = (*state).read().unwrap();
    Ok(Json(data.clone()))
}
async fn add_source(
    Extension(state): Extension<Arc<RwLock<Sources<'_>>>>,
) -> Result<impl IntoResponse + '_, ReportError> {
    let mut data = (*state).write().unwrap();
    data.add("127.0.0.1:5004");
    Ok("Done")
}
