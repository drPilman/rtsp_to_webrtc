use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use clap::Parser;
use color_eyre::Report;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;

pub struct LocalTrack {
    pub tx: Sender<Arc<TrackLocalStaticRTP>>,
    pub rx: Receiver<Arc<TrackLocalStaticRTP>>,
}

#[derive(Deserialize, Serialize)]
pub struct SessionOffer {
    pub session_description: String,
    pub id: usize,
}

#[derive(Serialize)]
pub struct Source<'a> {
    pub state: bool,
    pub url: &'a str,
    #[serde(skip)]
    pub track: LocalTrack,
}

#[derive(Serialize)]
pub struct Sources<'b> {
    pub list: Vec<Source<'b>>,
}

impl<'a> Sources<'a> {
    pub fn new() -> Self {
        Self {
            list: Vec::with_capacity(5),
        }
    }
    pub fn add(&mut self, url: &'a str) {
        self.list.push(Source {
            state: true,
            url,
            track: LocalTrack { rx: 0, tx: 0 },
        });
    }
    pub fn list_view(&self) {
        let list_copy = Vec::<Source>::with_capacity(self.list.len());
        for (source, source_copy) in (&self.list).iter().zip(list_copy) {
            source_copy = Source {};
        }
    }
}

pub struct ReportError(pub Report);

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

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
pub struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    pub log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "127.0.0.1")]
    pub addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    pub port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "../dist")]
    pub static_dir: String,
}
