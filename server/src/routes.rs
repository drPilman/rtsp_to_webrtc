use crate::structs::*;
use crate::webrtc_impl::*;
use anyhow::Result;
use axum::response::IntoResponse;
use axum::Json;
use axum::{extract, extract::Extension};
use color_eyre::eyre::eyre;
use std::sync::{Arc, RwLock};

pub async fn get_sources_list(
    Extension(state): Extension<Arc<RwLock<Sources<'_>>>>,
) -> Result<impl IntoResponse + '_, ReportError> {
    let data = (*state).read().unwrap().list_view();
    Ok(Json(data))
}
pub async fn add_source(
    Extension(state): Extension<Arc<RwLock<Sources<'_>>>>,
) -> Result<impl IntoResponse + '_, ReportError> {
    let mut data = (*state).write().unwrap();
    data.add("127.0.0.1:5004");
    //data.add("rtsp://wowzaec2demo.streamlock.net/vod/mp4:BigBuckBunny_115k.mp4");
    Ok("Done")
}

pub async fn view(
    extract::Json(payload): extract::Json<SessionOffer>,
    Extension(state): Extension<Arc<RwLock<Sources<'_>>>>,
) -> Result<impl IntoResponse + '_, ReportError> {
    let source = (*state).read().unwrap().list[payload.id].clone();

    if source.state {
        match webrtc_session(payload.session_description, source.url).await {
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
    Err(ReportError(eyre!("this source isn't active")))
}
