use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use wasm_bindgen::JsCast;
use web_sys::{FormData, HtmlFormElement, SubmitEvent};
use leptos::logging::log;
use crate::server_functions::*;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/pan_server.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    const GRID_SIZE: usize = 5;
    let (in_use_boxes, set_in_use_boxes) = signal::<Vec<BoxStatus>>(vec![]);
    let on_click = move |_| {
        spawn_local(async move {
            let response = check_box_status().await.unwrap();
            log!("Response: {:?}", response);
            set_in_use_boxes.set(response.list);
        });
    };

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
        let form_data = FormData::new_with_form(&target).unwrap();
        let file = form_data
            .get("file_to_upload")
            .unchecked_into::<web_sys::File>();
        let filename = file.name();
        log!("{}", filename);
        spawn_local(async move {
            upload_file(form_data.into()).await.unwrap();
        });
    };

    // Create a 5x5 grid with some cells filled and some empty
    // The grid generation is now directly inside the view macro for reactivity.
    view! {
        <div class="grid-container">
            {(0..GRID_SIZE).map(move |row| {
                (0..GRID_SIZE).map(move |col| {
                    let cell_class = move || {
                        let box_list = in_use_boxes.get(); // Reactive read
                        if box_list.iter().any(|b| b.id == (row * GRID_SIZE + col) as u8) {
                            "grid-cell filled"
                        } else {
                            "grid-cell empty"
                        }
                    };
                    view! {
                        <div class=cell_class></div>
                    }
                }).collect_view()
            }).collect_view()}
        </div>
        <div>
            <button on:click=on_click>
                "Check Box Status"
            </button>
        </div>
        <div style="margin-top: 20px;">
            <form on:submit=on_submit>
                <input type="file" name="file_to_upload" />
                <input type="submit" />
            </form>
        </div>
    }
}
