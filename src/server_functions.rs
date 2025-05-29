use leptos::prelude::*;
use leptos::logging::log;

#[server]
pub async fn print_value(value: String) -> Result<String, ServerFnError> {
    println!("Received value from client: {}", value);
    log!("Received value from client: {}", value);
    Ok(format!("Server received: {}", value))
}