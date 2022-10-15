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
    extract::Json(payload): extract::Json<NewSource>,
    Extension(state): Extension<Arc<RwLock<Sources>>>,
    Extension(admin_token): Extension<String>,
) -> Result<impl IntoResponse, ReportError> {
    if admin_token.ne(&payload.token) {
        return Err(ReportError(eyre!("token is wrong")));
    }
    let d = (*state).read().unwrap().list.len();
    let arc_state = Arc::clone(&state);
    let s = payload.url.clone();
    let res = tokio::task::spawn_blocking(move || new_track(&s, arc_state, d))
        .await
        .unwrap();
    match res {
        Some((track, pipe)) => {
            let mut data = (*state).write().unwrap();
            data.add(payload.url.clone(), track, pipe);
            Ok("Done")
        }
        None => Ok("Not Done"),
    }
    //data.add("rtsp://wowzaec2demo.streamlock.net/vod/mp4:BigBuckBunny_115k.mp4");
}

pub async fn stop_source(
    extract::Json(payload): extract::Json<StopSource>,
    Extension(state): Extension<Arc<RwLock<Sources>>>,
    Extension(admin_token): Extension<String>,
) -> Result<impl IntoResponse, ReportError> {
    if admin_token.ne(&payload.token) {
        return Err(ReportError(eyre!("token is wrong")));
    }
    let mut source = &mut ((*state).write().unwrap().list[payload.id]);
    if source.state {
        source.stop();
        return Ok("Done");
    };
    Ok("It's already was stoped")
    //data.add("rtsp://wowzaec2demo.streamlock.net/vod/mp4:BigBuckBunny_115k.mp4");
}

pub async fn view(
    extract::Json(payload): extract::Json<SessionOffer>,
    Extension(state): Extension<Arc<RwLock<Sources>>>,
) -> Result<impl IntoResponse, ReportError> {
    let source = match (*state).read().unwrap().list[payload.id].connect() {
        Ok(a) => a,
        Err(err) => return Err(ReportError(eyre!(err))),
    };

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
