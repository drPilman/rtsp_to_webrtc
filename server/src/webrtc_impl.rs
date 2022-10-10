use crate::utils::*;

use anyhow::{anyhow, Result};
use serde_json;
use std::sync::Arc;
use tokio::net::UdpSocket;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_VP8};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};
use webrtc::Error;

pub async fn new_track<'a>() -> Arc<TrackLocalStaticRTP> {
    let (local_track_chan_tx, mut local_track_chan_rx) =
        tokio::sync::mpsc::channel::<Arc<TrackLocalStaticRTP>>(1);

    let listener = UdpSocket::bind("127.0.0.1:5004")
        .await
        .expect("couldn't bind to address");
    /*listener
    .connect(source_url)
    .await
    .expect("couldn't connect to address");*/
    //let done_tx3 = done_tx.clone();

    tokio::spawn(async move {
        let local_track = Arc::new(TrackLocalStaticRTP::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_VP8.to_owned(),
                ..Default::default()
            },
            "video".to_owned(),
            "webrtc-rs".to_owned(),
        ));
        let _ = local_track_chan_tx.send(Arc::clone(&local_track)).await;
        let mut inbound_rtp_packet = vec![0u8; 1600]; // UDP MTU

        while let Ok((n, _)) = listener.recv_from(&mut inbound_rtp_packet).await {
            if let Err(err) = local_track.write(&inbound_rtp_packet[..n]).await {
                if Error::ErrClosedPipe == err {
                    log::debug!("The peerConnection has been closed.");
                    // The peerConnection has been closed.
                } else {
                    log::debug!("video_track write err: {}", err);
                }
                return;
            }
        }
    });
    local_track_chan_rx.recv().await.unwrap()
}

pub async fn webrtc_session(
    desc_data: String,
    track: Arc<dyn TrackLocal + Send + Sync>,
) -> Result<String> {
    let mut m = MediaEngine::default();

    m.register_default_codecs()?;

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    // Create Track that we send video back to browser on
    // Add this newly created track to the PeerConnection
    let rtp_sender = peer_connection.add_track(track).await?;

    // Read incoming RTCP packets
    // Before these packets are returned they are processed by interceptors. For things
    // like NACK this needs to be called.
    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
        Result::<()>::Ok(())
    });
    log::debug!("OK 239");
    let (done_tx, _) = tokio::sync::mpsc::channel::<()>(1);

    let done_tx1 = done_tx.clone();
    // Set the handler for ICE connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection
        .on_ice_connection_state_change(Box::new(move |connection_state: RTCIceConnectionState| {
            log::debug!("Connection State has changed {}", connection_state);
            if connection_state == RTCIceConnectionState::Failed {
                let _ = done_tx1.try_send(());
            }
            Box::pin(async {})
        }))
        .await;

    log::debug!("OK 255");
    let done_tx2 = done_tx.clone();
    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection
        .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            log::debug!("Peer Connection State has changed: {}", s);

            if s == RTCPeerConnectionState::Failed {
                // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                log::debug!("Peer Connection has gone to failed exiting: Done forwarding");
                let _ = done_tx2.try_send(());
            }

            Box::pin(async {})
        }))
        .await;

    // Wait for the offer to be pasted
    //let line = signal::must_read_stdin()?;
    let desc_data = decode(&desc_data)?;

    //let offer = serde_json::from_str::<RTCSessionDescription>(&desc_data)?;

    log::debug!("OK 280");
    let offer = serde_json::from_str::<RTCSessionDescription>(&desc_data)?;
    // Set the remote SessionDescription
    peer_connection.set_remote_description(offer).await?;
    log::debug!("OK 285");
    // Create an answer
    let answer = peer_connection.create_answer(None).await?;
    log::debug!("OK 288");

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    // Output the answer in base64 so we can paste it in browser
    let json_str = if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;

        let b64 = encode(&json_str);
        log::debug!("OK 303");
        Ok(b64)
        //log::debug!("{}", b64);
    } else {
        log::debug!("generate local_description failed!");
        Err(anyhow!("generate local_description failed!"))
    };

    //log::debug!("OK 314");
    // Open a UDP Listener for RTP Packets on port 5004
    /*let listener = UdpSocket::bind("127.0.0.1:5004")
    .await
    .expect("couldn't bind to address");*/
    /*listener
    .connect(source_url)
    .await
    .expect("couldn't connect to address");*/
    //let done_tx3 = done_tx.clone();

    // Read RTP packets forever and send them to the WebRTC Client
    /*tokio::spawn(async move {
        let mut inbound_rtp_packet = vec![0u8; 1600]; // UDP MTU
        while let Ok((n, _)) = listener.recv_from(&mut inbound_rtp_packet).await {
            if let Err(err) = video_track.write(&inbound_rtp_packet[..n]).await {
                if Error::ErrClosedPipe == err {
                    log::debug!("The peerConnection has been closed.");
                    // The peerConnection has been closed.
                } else {
                    log::debug!("video_track write err: {}", err);
                }
                let _ = done_tx3.try_send(());
                return;
            }
        }
    });*/
    //log::debug!("OK 336");
    /*log::debug!("Press ctrl-c to stop");
    tokio::select! {
        _ = done_rx.recv() => {
            log::debug!("received done signal!");
        }
        _ = tokio::signal::ctrl_c() => {
            log::debug!("");
        }
    };*/

    //peer_connection.close().await?;

    json_str
}
