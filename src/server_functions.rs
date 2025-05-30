use leptos::prelude::*;
use leptos::logging::log;
use leptos::server_fn::codec::{MultipartFormData, MultipartData};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;

#[server]
pub async fn print_value(value: String) -> Result<String, ServerFnError> {
    println!("Received value from client: {}", value);
    log!("Received value from client: {}", value);
    Ok(format!("Server received: {}", value))
}

#[server(input = MultipartFormData)]
pub async fn upload_file(data: MultipartData) -> Result<(), ServerFnError> {
    let upload_dir = PathBuf::from("upload_files");

    if let Err(e) = create_dir_all(&upload_dir) {
        let error_msg = format!("Failed to create upload directory '{}': {}", upload_dir.display(), e);
        leptos::logging::error!("Server Error: {}", error_msg);
        return Err(ServerFnError::ServerError(error_msg));
    }

    let mut data_processor = match data.into_inner() {
        Some(d) => d,
        None => {
            let error_msg = format!("Failed to process multipart data");
            leptos::logging::error!("Server Error: {}", error_msg);
            return Err(ServerFnError::ServerError(error_msg));
        }
    };

    while let Ok(Some(mut field)) = data_processor.next_field().await {
        let name = match field.file_name() {
            Some(fname) if !fname.is_empty() => fname.to_string(),
            _ => {
                leptos::logging::warn!("Multipart field skipped: missing filename or filename is empty.");
                continue;
            }
        };

        let file_path = upload_dir.join(&name);

        log!("[{}] Attempting to save to: {:?}", name, file_path);

        let mut file = match File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                let error_msg = format!("Failed to create file '{}': {}", file_path.display(), e);
                leptos::logging::error!("Server Error: {}", error_msg);
                continue;
            }
        };

        while let Ok(Some(chunk)) = field.chunk().await {
            if chunk.is_empty() { continue; }

            if let Err(e) = file.write_all(&chunk) {
                let error_msg = format!("Failed to write chunk to file '{}': {}", file_path.display(), e);
                leptos::logging::error!("Server Error: {}", error_msg);
                break;
            }
        }
        log!("[{}] Finished processing file.", name);
    }
    Ok(())
}