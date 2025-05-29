use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use leptos::logging::log;
// use wasm_bindgen::prelude::*;

#[server]
pub async fn print_value(value: String) -> Result<String, ServerFnError> {
    println!("Received value from client: {}", value);
    log!("Received value from client: {}", value);
    Ok(format!("Server received: {}", value))
}

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
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    // For the POST request
    let input_value = RwSignal::new(String::new());
    let response = RwSignal::new(String::new());

    // Function to handle the POST request using reqwest
    let send_post_request = move |_| {
        let value = input_value.get();
        spawn_local(async move {
            // Create a client
            let res = print_value(value).await.unwrap();
            response.set(res);
        });
    };

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <div>
            <button on:click=on_click>"Click Me: " {count}</button>
        </div>
        <div style="margin-top: 20px;">
            <h2>"Send POST Request to Server"</h2>
            <input 
                type="text" 
                placeholder="Enter value to send" 
                on:input=move |ev| {
                    let value = event_target_value(&ev);
                    input_value.set(value);
                }
                prop:value=input_value
            />
            <button on:click=send_post_request>"Send to Server"</button>
            <div style="margin-top: 10px;">
                <p>"Server Response: " {response}</p>
            </div>
        </div>
    }
}
