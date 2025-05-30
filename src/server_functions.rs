use leptos::prelude::*;
use leptos::logging::log;
use server_fn::codec::{MultipartFormData, MultipartData, Json};
use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;


#[derive(Serialize, Deserialize, Debug)]
pub struct CheckResponse{
    pub list: Vec<BoxStatus>
}
impl CheckResponse{
    pub fn new() -> CheckResponse{
        CheckResponse{list: Vec::new()}
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoxStatus{
    pub id: u8,
    pub name: String,
}

#[server(output = Json)]
pub async fn check_box_status() -> Result<CheckResponse, ServerFnError>{
    let mut list = CheckResponse::new();
    list.list.push(BoxStatus{id: 0, name: "Box 1".to_string()});
    list.list.push(BoxStatus{id: 5, name: "Box 1".to_string()});
    Ok(list)
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