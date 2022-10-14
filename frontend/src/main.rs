use gloo_net::http::Request;
use serde::Deserialize;
//use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::{html, Properties};
use yew_router::prelude::*;

#[derive(Deserialize, Clone, Properties, PartialEq)]
struct Source {
    state: bool,
    url: String,
}

#[derive(Deserialize, Clone, Properties, PartialEq)]
struct Sources {
    list: Vec<Source>,
}

#[function_component(VideosList)]
fn videos_list(Sources { list }: &Sources) -> Html {
    list.iter()
        .enumerate()
        .map(|(i, video)| {
            html! {
                <option value={i.to_string()} style={if video.state {""} else {"display: none;"}}>{video.url.clone()}</option>
                //<p>{format!("{}: {}", video.state, video.url)}</p>
            }
        })
        .collect()
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/view")]
    View,
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! { <ChooseSource /> },
        Route::View => html! { <ViewSource />},
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

#[function_component(ChooseSource)]
fn choose_source() -> Html {
    //let data = use_state(|| None);

    let sources = use_state(|| Sources { list: vec![] });
    {
        let sources = sources.clone();
        use_effect_with_deps(
            move |_| {
                let sources = sources.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_videos: Sources = Request::get("/api/get_sources_list")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    sources.set(fetched_videos);
                });
                || ()
            },
            (),
        );
    }
    // Request `/api/hello` once
    /*{
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let fetched_videos: Vec<Video> =
                    //let resp = Request::get("/api/hello").send().await.unwrap();
                    /*let result = {
                        if !resp.ok() {
                            Err(format!(
                                "Error fetching data {} ({})",
                                resp.status(),
                                resp.status_text()
                            ))
                        } else {
                            resp.text().await.map_err(|err| err.to_string())
                        }
                    };*/
                    data.set(Some(result));
                });
            }

            || {}
        });
    }*/
    html! {
        <>
            <h1>{ "RustConf Explorer" }</h1>
            <form action="view" method="get">
                <h3>{"Videos to watch"}</h3>
                <select name={"id"}>
                    <option value="" style="display:none">{"Choose one source"}</option>
                    <VideosList list={sources.list.clone()}/>
                </select>
                <input type="submit" value="Open" />
            </form>

        </>
    }
    /*match data.as_ref() {
        None => {
            html! {
                <div>{"No server response"}</div>
            }
        }
        Some(Ok(data)) => {
            html! {
                <div>{"Got server response: "}{data}
                <form action="view" method="get">
                    <select>
                        <option value="" style="display:none">{"Choose one provider"}</option>
                        <option value="1">{"One"}</option>
                        //<option value="2">Two</option>
                    </select>
                </form>
                </div>
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{"Error requesting data from server: "}{err}</div>
            }
        }
    }*/
}

#[function_component(ViewSource)]
fn view_source() -> Html {
    //let data = use_state(|| None);

    /*let sources = use_state(|| Sources { list: vec![] });
    {
        let sources = sources.clone();
        use_effect_with_deps(
            move |_| {
                let sources = sources.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_videos: Sources = Request::get("/api/get_sources_list")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    sources.set(fetched_videos);
                });
                || ()
            },
            (),
        );
    }*/
    //let t = use_location().unwrap().search();
    html! {
        <>
            <h1>{ "View" }</h1>
            //<h2>{ t }</h2>
            <div id="remoteVideo"></div>
            <script type={"text/javascript"} src={"/assets/webrtc.js"}/>
        </>
    }
}
/*
async fn fff() -> Result<()>{
     // Create a MediaEngine object to configure the supported codec
     let mut m = MediaEngine::default();

     // Register default codecs
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

     // Create a datachannel with label 'data'
     let data_channel = peer_connection.create_data_channel("data", None).await?;

     let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

     // Set the handler for Peer connection state
     // This will notify you when the peer has connected/disconnected
     peer_connection
         .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
             println!("Peer Connection State has changed: {}", s);

             if s == RTCPeerConnectionState::Failed {
                 // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                 // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                 // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                 println!("Peer Connection has gone to failed exiting");
                 let _ = done_tx.try_send(());
             }

             Box::pin(async {})
         }))
         .await;

     // Register channel opening handling
     let d1 = Arc::clone(&data_channel);
     data_channel.on_open(Box::new(move || {
         println!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 5 seconds", d1.label(), d1.id());

         let d2 = Arc::clone(&d1);
         Box::pin(async move {
             let mut result = Result::<usize>::Ok(0);
             while result.is_ok() {
                 let timeout = tokio::time::sleep(Duration::from_secs(5));
                 tokio::pin!(timeout);

                 tokio::select! {
                     _ = timeout.as_mut() =>{
                         let message = math_rand_alpha(15);
                         println!("Sending '{}'", message);
                         result = d2.send_text(message).await.map_err(Into::into);
                     }
                 };
             }
         })
     })).await;

     // Register text message handling
     let d_label = data_channel.label().to_owned();
     data_channel
         .on_message(Box::new(move |msg: DataChannelMessage| {
             let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
             println!("Message from DataChannel '{}': '{}'", d_label, msg_str);
             Box::pin(async {})
         }))
         .await;

     // Create an offer to send to the browser
     let offer = peer_connection.create_offer(None).await?;

     // Create channel that is blocked until ICE Gathering is complete
     let mut gather_complete = peer_connection.gathering_complete_promise().await;

     // Sets the LocalDescription, and starts our UDP listeners
     peer_connection.set_local_description(offer).await?;

     // Block until ICE Gathering is complete, disabling trickle ICE
     // we do this because we only can exchange one signaling message
     // in a production application you should exchange ICE Candidates via OnICECandidate
     let _ = gather_complete.recv().await;

     // Output the answer in base64 so we can paste it in browser
     if let Some(local_desc) = peer_connection.local_description().await {
         let json_str = serde_json::to_string(&local_desc)?;
         let b64 = signal::encode(&json_str);
         println!("{}", b64);
     } else {
         println!("generate local_description failed!");
     }

     // Wait for the answer to be pasted
     let line = signal::must_read_stdin()?;
     let desc_data = signal::decode(line.as_str())?;
     let answer = serde_json::from_str::<RTCSessionDescription>(&desc_data)?;

     // Apply the answer as the remote description
     peer_connection.set_remote_description(answer).await?;


     //peer_connection.close().await?;
     Ok(())
}
*/
fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();

    yew::start_app::<App>();
}
