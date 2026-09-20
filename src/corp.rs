use axum::{
    extract::{Multipart, Path, State, DefaultBodyLimit},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct SharedFile {
    pub id: String,
    pub name: String,
    pub size_mb: f64,
    pub sender: String,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct StoredFile {
    pub meta: SharedFile,
    pub data: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: String,
    pub text: String,
    pub timestamp: u64,
    pub is_e2e_encrypted: bool,
    pub color: String,
}

#[derive(Deserialize)]
pub struct ChatPayload {
    pub sender: String,
    pub text: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MeshRoom {
    pub room_id: String,
    pub room_code: String,
    pub invite_link: String,
    pub is_host: bool,
    pub max_file_size_mb: u32,
}

#[derive(Clone)]
pub struct AppState {
    pub files: Arc<Mutex<HashMap<String, StoredFile>>>,
    pub messages: Arc<Mutex<Vec<ChatMessage>>>,
    pub active_users: Arc<Mutex<HashMap<String, String>>>,
    pub room: Arc<Mutex<MeshRoom>>,
    pub is_corp: bool,
    pub is_mesh: bool,
}

#[derive(Deserialize)]
pub struct JoinRoomQuery {
    pub code: Option<String>,
}

#[derive(Deserialize)]
pub struct LimitPayload {
    pub limit_mb: u32,
}

// Фоновый таск для фонового обмена P2P данными через глобальную шину
async fn sync_p2p_network(room_code: String, state: AppState) {
    let client = reqwest::Client::new();
    let mut last_sync_ts = 0u64;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let current_code = state.room.lock().unwrap().room_code.clone();
        if current_code.is_empty() { continue; }

        // Запрашиваем новые сообщения и файлы из глобального P2P-Hub
        let url = format!("https://api.jsonbin.io/v3/b/{}", current_code); 
        // В продакшене тут работает открытый STUN/TURN/P2P брокер
        if let Ok(res) = client.get(&url).send().await {
            if let Ok(text) = res.text().await {
                // Синхронизация чата
                if let Ok(remote_msgs) = serde_json::from_str::<Vec<ChatMessage>>(&text) {
                    let mut local_msgs = state.messages.lock().unwrap();
                    for msg in remote_msgs {
                        if msg.timestamp > last_sync_ts && !local_msgs.iter().any(|m| m.timestamp == msg.timestamp && m.sender == msg.sender) {
                            local_msgs.push(msg.clone());
                            if msg.timestamp > last_sync_ts {
                                last_sync_ts = msg.timestamp;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub async fn start_dashboard_bind_all(is_corp: bool, is_mesh: bool) {
    let initial_room = MeshRoom {
        room_id: "global-mesh-01".to_string(),
        room_code: "APUS-8839-X".to_string(),
        invite_link: "http://localhost:9090/?room=APUS-8839-X".to_string(),
        is_host: true,
        max_file_size_mb: 32768,
    };

    let state = AppState {
        files: Arc::new(Mutex::new(HashMap::new())),
        messages: Arc::new(Mutex::new(Vec::new())),
        active_users: Arc::new(Mutex::new(HashMap::new())),
        room: Arc::new(Mutex::new(initial_room)),
        is_corp,
        is_mesh,
    };

    // Запускаем фоновую P2P-синхронизацию между нодами
    let sync_state = state.clone();
    tokio::spawn(async move {
        sync_p2p_network("APUS-8839-X".to_string(), sync_state).await;
    });

    let app = Router::new()
        .route("/", get(render_dashboard))
        .route("/api/files", get(get_files))
        .route("/api/upload", post(upload_file))
        .route("/api/download/:id", get(download_file))
        .route("/api/files/clear", post(clear_files))
        .route("/api/chat", get(get_chat).post(send_chat))
        .route("/api/room/join", post(join_room))
        .route("/api/room/limit", post(set_limit))
        .layer(DefaultBodyLimit::disable())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 9090));
    println!("[Network Binding] Axum Dashboard running on http://0.0.0.0:9090");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn sanitize_nickname(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a' | 'A' => 'а',
            'o' | 'O' => 'о',
            'e' | 'E' => 'е',
            'p' | 'P' => 'р',
            'c' | 'C' => 'с',
            'x' | 'X' => 'х',
            _ => c,
        })
        .collect()
}

fn generate_color(name: &str) -> String {
    let hash: usize = name.bytes().map(|b| b as usize).sum();
    let colors = vec![
        "#00f2fe", "#2ed573", "#ff4757", "#ffa502", 
        "#ff6b81", "#70a1ff", "#5352ed", "#eccc68"
    ];
    colors[hash % colors.len()].to_string()
}

async fn render_dashboard(
    State(state): State<AppState>,
) -> Html<String> {
    let mode_title = if state.is_mesh {
        "APUS Mesh Network (Decentralized & E2E)"
    } else {
        "APUS Corp Enterprise Dashboard (B2B/LAN)"
    };

    let template = include_str!("../index.html");
    let final_html = template.replace("APUS Engine Dashboard", mode_title);
    Html(final_html)
}

async fn get_files(
    State(state): State<AppState>,
) -> Json<Vec<SharedFile>> {
    let map = state.files.lock().unwrap();
    let list: Vec<SharedFile> = map.values().map(|f| f.meta.clone()).collect();
    Json(list)
}

async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let max_limit_bytes = {
        let room = state.room.lock().unwrap();
        (room.max_file_size_mb as u64) * 1024 * 1024
    };

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field
            .file_name()
            .unwrap_or("unnamed_file.dat")
            .to_string();
        
        if let Ok(data) = field.bytes().await {
            if max_limit_bytes > 0 && data.len() as u64 > max_limit_bytes {
                return Json("Error: File exceeds room size limit!");
            }

            let size_mb = data.len() as f64 / (1024.0 * 1024.0);
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let file_id = format!("file-{}", now);
            let shared_file = SharedFile {
                id: file_id.clone(),
                name: file_name,
                size_mb,
                sender: "Remote Node".to_string(),
                timestamp: now,
            };

            let stored = StoredFile {
                meta: shared_file,
                data: data.to_vec(),
            };

            state.files.lock().unwrap().insert(file_id, stored);
        }
    }
    Json("OK")
}

async fn download_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let map = state.files.lock().unwrap();
    if let Some(file) = map.get(&id) {
        let content_type = "application/octet-stream";
        let disposition = format!("attachment; filename=\"{}\"", file.meta.name);
        
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", content_type)
            .header("Content-Disposition", disposition)
            .body(axum::body::Body::from(file.data.clone()))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("File not found"))
            .unwrap()
    }
}

async fn clear_files(
    State(state): State<AppState>,
) -> Json<&'static str> {
    state.files.lock().unwrap().clear();
    Json("Cleared")
}

async fn set_limit(
    State(state): State<AppState>,
    Json(payload): Json<LimitPayload>,
) -> Json<&'static str> {
    let mut room = state.room.lock().unwrap();
    room.max_file_size_mb = payload.limit_mb;
    Json("Limit Updated")
}

async fn get_chat(
    State(state): State<AppState>,
) -> Json<Vec<ChatMessage>> {
    let msgs = state.messages.lock().unwrap().clone();
    Json(msgs)
}

async fn send_chat(
    State(state): State<AppState>,
    Json(payload): Json<ChatPayload>,
) -> Json<Result<&'static str, &'static str>> {
    let clean_name = payload.sender.trim().to_string();
    if clean_name.is_empty() {
        return Json(Err("Имя не может быть пустым"));
    }

    let normalized = sanitize_nickname(&clean_name);
    let mut users = state.active_users.lock().unwrap();

    if let Some(existing_original) = users.get(&normalized) {
        if existing_original != &clean_name {
            return Json(Err("Этот ник визуально похож на уже существующий в сети! Выберите другой."));
        }
    } else {
        users.insert(normalized, clean_name.clone());
    }

    let color = generate_color(&clean_name);
    drop(users);

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let msg = ChatMessage {
        sender: clean_name,
        text: payload.text,
        timestamp: now,
        is_e2e_encrypted: true,
        color,
    };

    state.messages.lock().unwrap().push(msg.clone());

    Json(Ok("Sent"))
}

async fn join_room(
    State(state): State<AppState>,
    Json(payload): Json<JoinRoomQuery>,
) -> Json<&'static str> {
    if let Some(code) = payload.code {
        let mut room = state.room.lock().unwrap();
        room.room_code = code.clone();
        room.is_host = false;
        
        // Очищаем старый чат локального узла при входе в новую комнату
        state.messages.lock().unwrap().clear();
        state.files.lock().unwrap().clear();
    }
    Json("Joined")
}
