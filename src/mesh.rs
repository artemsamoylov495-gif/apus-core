use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: String,
    pub text: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshRoom {
    pub room_id: String,
    pub room_name: String,
    pub peers: Vec<String>,
    pub messages: Vec<ChatMessage>,
}

#[derive(Clone)]
pub struct MeshEngine {
    pub active_room: Arc<RwLock<Option<MeshRoom>>>,
}

impl MeshEngine {
    pub fn new() -> Self {
        Self {
            active_room: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn create_room(&self, name: &str) -> String {
        // Подмешиваем наносекунды, чтобы хэш ВСЕГДА был уникальным
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        
        let seed = format!("{}-{}", name, now);
        let hash = Sha256::digest(seed.as_bytes());
        let room_id = format!("apus-{:02x}{:02x}", hash[0], hash[1]);

        let room = MeshRoom {
            room_id: room_id.clone(),
            room_name: name.to_string(),
            peers: vec!["LocalHost".to_string()],
            messages: vec![],
        };
        let mut lock = self.active_room.write().await;
        *lock = Some(room);
        format!("apus://room/{}", room_id)
    }

    pub async fn join_room(&self, invite_link: &str) -> bool {
        let room_id = invite_link.trim_start_matches("apus://room/").to_string();
        let room = MeshRoom {
            room_id: room_id.clone(),
            room_name: format!("Joined Room ({})", room_id),
            peers: vec!["LocalHost".to_string(), "RemotePeer".to_string()],
            messages: vec![],
        };
        let mut lock = self.active_room.write().await;
        *lock = Some(room);
        true
    }

    pub async fn send_message(&self, sender: &str, text: &str) {
        let mut lock = self.active_room.write().await;
        if let Some(ref mut room) = *lock {
            let msg = ChatMessage {
                sender: sender.to_string(),
                text: text.to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            room.messages.push(msg);
        }
    }
}
