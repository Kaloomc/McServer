use directories::ProjectDirs;
use std::fs;
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use rcon::Connection;
use serde::Deserialize;

static SERVER_STATUSES: Lazy<DashMap<String, bool>> = Lazy::new(|| DashMap::new());
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn get_or_create_appdata_folder() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "mcserver", "mcserver")
        .expect("Impossible de récupérer le dossier AppData");

    let app_dir: PathBuf = proj_dirs.data_dir().to_path_buf();

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("Impossible de créer le dossier AppData");
    }

    app_dir
}

#[tauri::command]
fn get_server_version(folder_name: String) -> Vec<String> {
    let mut path: PathBuf = get_or_create_appdata_folder();
    path.push(folder_name);
    path.push("versions");
    let mut files = Vec::new();
    if !path.exists(){
        return files
    }

    let paths = fs::read_dir(&path).unwrap();

    for entry in paths {
        if let Ok(entry) = entry {
            if let Ok(filename) = entry.file_name().into_string() {
                files.push(filename);
            }
        }
    }
    files
}

#[tauri::command]
fn create_new_data_folder(folder_name: String) {
    let mut path: PathBuf = get_or_create_appdata_folder();
    path.push(folder_name);
    fs::create_dir_all(&path).ok();
}

#[tauri::command]
fn get_description_server(folder_name: String) -> String {
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::PathBuf;

    let mut path: PathBuf = get_or_create_appdata_folder();
    path.push(folder_name);
    path.push("server.properties");

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return String::new(),
    };

    let reader = io::BufReader::new(file);

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("motd=") {
            return value.to_string();
        }
    }

    String::new()
}

#[tauri::command]
fn get_data_folder_list() -> Vec<String> {
    let appdata = get_or_create_appdata_folder();
    let paths = fs::read_dir(&appdata).unwrap();

    let mut files = Vec::new();
    for entry in paths {
        if let Ok(entry) = entry {
            if let Ok(filename) = entry.file_name().into_string() {
                files.push(filename);
            }
        }
    }

    files
}

#[tauri::command]
async fn open_folder(folder_name: String) {
    let mut path: PathBuf = get_or_create_appdata_folder();
    path.push(folder_name);
    let _ = open::that(&path);
}

#[tauri::command]
async fn open_server(folder_name: String) {
    let mut path: PathBuf = get_or_create_appdata_folder();
    path.push(folder_name);

    Command::new("cmd")
    .current_dir(&path)
    .args(["/C", "Start.bat"])
    .creation_flags(CREATE_NO_WINDOW)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .expect("failed to run command");
}

#[tauri::command]
async fn stop_server(folder_name: String) {
    use tokio::time::{timeout, Duration};

    tokio::spawn(async move {
        let mut path: PathBuf = get_or_create_appdata_folder();
        path.push(&folder_name);
        path.push("server.properties");

        let content = fs::read_to_string(&path).unwrap();

        let mut rcon_port = "25575".to_string();
        let mut rcon_password = "".to_string();

        for line in content.lines() {
            if line.starts_with("rcon.port=") {
                rcon_port = line["rcon.port=".len()..].to_string();
            }
            if line.starts_with("rcon.password=") {
                rcon_password = line["rcon.password=".len()..].to_string();
            }
        }

        if rcon_password.is_empty() {
            SERVER_STATUSES.insert(folder_name.clone(), false);
            return;
        }

        let address = format!("192.168.1.192:{}", rcon_port);

        let result = timeout(Duration::from_millis(800), async {
            let mut conn = Connection::builder()
                .enable_minecraft_quirks(true)
                .connect(&address, &rcon_password)
                .await
                .ok()?;

            conn.cmd("stop").await.ok()?;

            Some(())
        })
        .await;
    });
}

#[tauri::command]
async fn is_server_running(folder_name: String) -> bool {
    use std::fs;
    use std::path::PathBuf;
    use tokio::time::{timeout, Duration};

    let folder = folder_name.clone();

    tokio::spawn(async move {
        let mut path: PathBuf = get_or_create_appdata_folder();
        path.push(&folder);
        path.push("server.properties");

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => {
                SERVER_STATUSES.insert(folder.clone(), false);
                return;
            }
        };

        let mut rcon_port = "25575".to_string();
        let mut rcon_password = "".to_string();

        for line in content.lines() {
            if line.starts_with("rcon.port=") {
                rcon_port = line["rcon.port=".len()..].to_string();
            }
            if line.starts_with("rcon.password=") {
                rcon_password = line["rcon.password=".len()..].to_string();
            }
        }

        if rcon_password.is_empty() {
            SERVER_STATUSES.insert(folder.clone(), false);
            return;
        }

        let address = format!("192.168.1.192:{}", rcon_port);

        let result = timeout(Duration::from_millis(800), async {
            let mut conn = Connection::builder()
                .enable_minecraft_quirks(true)
                .connect(&address, &rcon_password)
                .await
                .ok()?;

            conn.cmd("list").await.ok()?;

            Some(())
        })
        .await;

        SERVER_STATUSES.insert(folder.clone(), result.is_ok());
    });

    SERVER_STATUSES.get(&folder_name).map(|status| *status).unwrap_or(false)
}

#[derive(Debug, Deserialize)]
struct PaperVersionsResponse {
    project_id: String,
    project_name: String,
    versions: Vec<String>,
}

// ✅ Nouvelle API PaperMC
#[tauri::command]
async fn get_paper_versions() -> Result<Vec<String>, String> {
    println!("→ Fetching Paper versions...");
    // Nouvelle URL de l'API Downloads
    let url = "https://api.papermc.io/v2/projects/paper";

    let client = reqwest::Client::builder()
        .user_agent("MCServerManager/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    match client.get(url).send().await {
        Ok(resp) => {
            let status = resp.status();
            println!("→ HTTP status: {}", status);
            
            if !status.is_success() {
                return Err(format!("HTTP error: {}", status));
            }
            
            let text = resp.text().await.map_err(|e| e.to_string())?;
            println!("→ Response length: {} bytes", text.len());
            
            if text.is_empty() {
                return Err("Empty response from API".to_string());
            }
            
            let parsed: PaperVersionsResponse = serde_json::from_str(&text)
                .map_err(|e| format!("JSON parse error: {}. First 100 chars: {}", e, &text[..100.min(text.len())]))?;
            
            println!("→ Found {} versions", parsed.versions.len());
            Ok(parsed.versions)
        }
        Err(e) => Err(format!("HTTP request failed: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            create_new_data_folder,
            get_data_folder_list,
            open_folder,
            open_server,
            get_server_version,
            get_description_server,
            is_server_running,
            stop_server,
            get_paper_versions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}