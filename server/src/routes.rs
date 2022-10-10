use crate::structs::*;
use crate::webrtc_impl::*;
use anyhow::Result;
use axum::response::IntoResponse;
use axum::Json;
use axum::{extract, extract::Extension};
use axum_macros::debug_handler;
use color_eyre::eyre::eyre;
use std::sync::{Arc, RwLock};

pub async fn get_sources_list(
    Extension(state): Extension<Arc<RwLock<Sources>>>,
) -> Result<impl IntoResponse, ReportError> {
    Ok(Json(SourcesView::new(&(*state).read().unwrap())))
}

pub async fn add_source(
    Extension(state): Extension<Arc<RwLock<Sources>>>,
) -> Result<impl IntoResponse, ReportError> {
    let track = new_track().await;
    let mut data = (*state).write().unwrap();
    data.add("127.0.0.1:5004".to_owned(), track);
    //data.add("rtsp://wowzaec2demo.streamlock.net/vod/mp4:BigBuckBunny_115k.mp4");
    Ok("Done")
}

#[debug_handler]
pub async fn view(
    extract::Json(payload): extract::Json<SessionOffer>,
    Extension(state): Extension<Arc<RwLock<Sources>>>,
) -> Result<impl IntoResponse, ReportError> {
    let source = (*state).read().unwrap().list[payload.id]
        .connect()
        .expect("this source isn't active");

    match webrtc_session(payload.session_description, source).await {
        Ok(session_description) => {
            log::debug!("AAAAAAAAAAAAAAAAAAA");
            return Ok(Json(SessionOffer {
                session_description,
                id: payload.id,
            }));
        }
        Err(err) => {
            log::error!("{}", err);
            return Err(ReportError(eyre!("error when try to connect to peer")));
        }
    };
}
