use gloo_net::http::Request;
use serde::Deserialize;
//use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
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
    #[at("/hello-server")]
    HelloServer,
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! { <h1>{ "Hello Frontend" }</h1> },
        Route::HelloServer => html! { <HelloServer /> },
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

#[function_component(HelloServer)]
fn hello_server() -> Html {
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
                <select>
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

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();

    yew::start_app::<App>();
}
