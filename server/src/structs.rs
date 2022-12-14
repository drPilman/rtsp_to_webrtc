use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use clap::Parser;
use color_eyre::Report;
use gst::prelude::ElementExtManual;
use serde::{Deserialize, Serialize};
use std::mem;
use std::sync::Arc;
use tokio;
use tokio::runtime::Runtime;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocal;

#[derive(Deserialize, Serialize)]
pub struct SessionOffer {
    pub session_description: String,
    pub id: usize,
}

#[derive(Debug)]
pub struct Source {
    pub state: bool,
    pub url: String,
    pub track: Arc<TrackLocalStaticRTP>,
    pub pipe: Arc<gst::Pipeline>,
}

impl Source {
    pub fn connect<'a>(&'a self) -> Result<Arc<dyn TrackLocal + Send + Sync>, &'static str> {
        if self.state {
            Ok(Arc::clone(&self.track) as Arc<dyn TrackLocal + Send + Sync>)
        } else {
            Err("this source isn't active")
        }
    }
    pub fn stop(&mut self) {
        self.state = false;
        self.pipe.send_event(gst::event::Eos::new());
        /*let rt = mem::replace(&mut self.thread, None).unwrap();

        let _ = tokio::block_on(async move {
            rt.unwrap()
                .shutdown_timeout(std::time::Duration::from_secs(1));
        })
        .await;*/
        log::debug!("destroy");
    }
}

pub struct Sources {
    pub list: Vec<Source>,
}

impl Sources {
    pub fn new() -> Self {
        Self {
            list: Vec::with_capacity(5),
        }
    }
    pub fn add(&mut self, url: String, track: Arc<TrackLocalStaticRTP>, pipe: Arc<gst::Pipeline>) {
        self.list.push(Source {
            state: true,
            url,
            track,
            pipe,
        });
    }
}

#[derive(Serialize, Debug)]
pub struct SourceView {
    pub state: bool,
    pub url: String,
}
impl SourceView {
    pub fn new(source: &Source) -> Self {
        SourceView {
            state: source.state.clone(),
            url: source.url.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct SourcesView {
    pub list: Vec<SourceView>,
}
impl SourcesView {
    pub fn new(sources: &Sources) -> Self {
        let mut list_view = Vec::<SourceView>::with_capacity(sources.list.len());
        for source in sources.list.iter() {
            list_view.push(SourceView::new(source));
        }
        SourcesView { list: list_view }
    }
}

#[derive(Deserialize)]
pub struct NewSource {
    pub url: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct StopSource {
    pub id: usize,
    pub token: String,
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
    #[clap(short = 'a', long = "addr", default_value = "0.0.0.0")]
    pub addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    pub port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "../dist")]
    pub static_dir: String,
}
