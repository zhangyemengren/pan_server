use leptos::prelude::*;
use leptos::logging::log;
use server_fn::codec::{MultipartFormData, MultipartData, Json};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::fs::{self, File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const MAX_BOXES: usize = 25;

#[cfg(not(target_arch = "wasm32"))]
static BOX_LIST: LazyLock<Arc<Mutex<Vec<BoxStatus>>>> = LazyLock::new(|| {
    use rand::seq::SliceRandom;
    println!("Initializing BOX_LIST...");
    let mut rng = rand::rng();
    let mut boxes_vec = Vec::new();
    for i in 0..MAX_BOXES {
        boxes_vec.push(BoxStatus {
            id: i as u8,
            name: format!("Box {}", i),
            in_use: false,
        });
    }

    let upload_dir = PathBuf::from("upload_files");
    match fs::read_dir(&upload_dir) {
        Ok(entries) => {
            let files_in_upload_dir: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_file())
                .collect();

            if !files_in_upload_dir.is_empty() {
                let num_items_to_mark_as_in_use = std::cmp::min(files_in_upload_dir.len(), MAX_BOXES);
                
                let mut box_indices: Vec<usize> = (0..MAX_BOXES).collect();
                box_indices.shuffle(&mut rng);

                for i in 0..num_items_to_mark_as_in_use {
                    let target_box_index = box_indices[i];
                    if let Some(box_to_update) = boxes_vec.get_mut(target_box_index) {
                        box_to_update.in_use = true;
                    }
                }
                println!(
                    "Marked {} boxes as 'in_use' based on files in '{}'.",
                    num_items_to_mark_as_in_use,
                    upload_dir.display()
                );
            } else {
                println!(
                    "No files found in '{}' or directory is empty. All boxes remain 'not in use'.",
                    upload_dir.display()
                );
            }
        }
        Err(e) => {
            println!(
                "Warning: Could not read '{}' directory during BOX_LIST initialization: {}. All boxes will be marked 'not in use'.",
                upload_dir.display(),
                e
            );
        }
    }
    Arc::new(Mutex::new(boxes_vec))
});

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckResponse{
    pub list: Vec<BoxStatus>
}
impl CheckResponse{
    pub fn new() -> CheckResponse{
        CheckResponse{list: Vec::new()}
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BoxStatus{
    pub id: u8,
    pub name: String,
    pub in_use: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadResponse{
    pub success: bool,
    pub list: Vec<BoxStatus>,
}

impl UploadResponse{
    pub fn new(success: bool) -> UploadResponse{
        UploadResponse{success, list: Vec::new()}
    }
    pub fn update_list(&mut self, list: Vec<BoxStatus>){
        self.list = list;
    }
}

#[server(output = Json)]
pub async fn check_box_status() -> Result<CheckResponse, ServerFnError>{
    let box_list_guard = BOX_LIST.lock().map_err(|e| -> ServerFnError {
        leptos::logging::error!("Failed to lock BOX_LIST for reading: {:?}", e);
        ServerFnError::ServerError("Failed to access box status data due to lock error.".to_string())
    })?;
    
    let current_boxes_status = box_list_guard.clone();
    
    Ok(CheckResponse { list: current_boxes_status })
}

#[server(
    input = MultipartFormData,
    output = Json
)]
pub async fn upload_file(data: MultipartData) -> Result<UploadResponse, ServerFnError> {
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

    let mut box_id: String = String::new();

    while let Ok(Some(mut field)) = data_processor.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        // 处理文本字段（如 box_id）
        if field_name == "box_id" {
            let mut text_value = String::new();
            while let Ok(Some(chunk)) = field.chunk().await {
                if let Ok(chunk_str) = std::str::from_utf8(&chunk) {
                    text_value.push_str(chunk_str);
                }
            }
            box_id = text_value.trim().to_string();
            log!("Received box_id: {:?}", box_id);
            if box_id.is_empty() {
                let error_msg = "box_id is empty".to_string();
                leptos::logging::error!("Server Error: {}", error_msg);
                return Err(ServerFnError::ServerError(error_msg));
            }
            let box_list_guard = BOX_LIST.lock().map_err(|e| -> ServerFnError {
                leptos::logging::error!("Failed to lock BOX_LIST for writing: {:?}", e);
                ServerFnError::ServerError("Failed to access box status data due to lock error.".to_string())
            })?;
        
            let box_list = box_list_guard.clone();
            if let Some(index) = box_list.iter().position(|b| b.id == box_id.parse::<u8>().unwrap()) {
                if box_list[index].in_use {
                    let error_msg = "box is already in use".to_string();
                    leptos::logging::error!("Server Error: {}", error_msg);
                    return Err(ServerFnError::ServerError(error_msg));
                }
            }

            continue;
        }

        // 处理文件字段
        let name = match field.file_name() {
            Some(fname) if !fname.is_empty() => format!("box_{}__{}", box_id, fname.to_string()),
            _ => {
                let error_msg = "Multipart field error: missing filename or filename is empty.".to_string();
                leptos::logging::warn!("{}", error_msg);
                return Err(ServerFnError::ServerError(error_msg));
            }
        };

        let file_path = upload_dir.join(&name);

        log!("[{}] Attempting to save to: {:?}", name, file_path);

        let mut file = match File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                let error_msg = format!("Failed to create file '{}': {}", file_path.display(), e);
                leptos::logging::error!("Server Error: {}", error_msg);
                return Err(ServerFnError::ServerError(error_msg));
            }
        };

        while let Ok(Some(chunk)) = field.chunk().await {
            if chunk.is_empty() { continue; }

            if let Err(e) = file.write_all(&chunk) {
                let error_msg = format!("Failed to write chunk to file '{}': {}", file_path.display(), e);
                leptos::logging::error!("Server Error: {}", error_msg);
                return Err(ServerFnError::ServerError(error_msg));
            }
        }
        log!("[{}] Finished processing file.", name);
    }

    log!("Upload completed for box_id: {}", box_id);
    let mut box_list_guard = BOX_LIST.lock().map_err(|e| -> ServerFnError {
        leptos::logging::error!("Failed to lock BOX_LIST for writing: {:?}", e);
        ServerFnError::ServerError("Failed to access box status data due to lock error.".to_string())
    })?;

    let box_index = box_list_guard.iter().position(|b| b.id == box_id.parse::<u8>().unwrap());
    if let Some(index) = box_index {
        box_list_guard[index].in_use = true;
    }
    
    // 获取更新后的状态用于响应
    let updated_box_list = box_list_guard.clone();
    let mut upload_response = UploadResponse::new(true);
    log!("Box list: {:?}", updated_box_list);
    upload_response.update_list(updated_box_list);
    Ok(upload_response)
}