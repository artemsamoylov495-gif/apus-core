use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkInfo {
    pub id: usize,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateManifest {
    pub update_id: String,
    pub file_name: String,
    pub total_size_bytes: u64,
    pub chunk_size_bytes: usize,
    pub chunks: Vec<ChunkInfo>,
}

impl UpdateManifest {
    /// Создает новый манифест обновлений
    pub fn new(update_id: &str, file_name: &str, total_size_bytes: u64, chunk_size_bytes: usize) -> Self {
        Self {
            update_id: update_id.to_string(),
            file_name: file_name.to_string(),
            total_size_bytes,
            chunk_size_bytes,
            chunks: Vec::new(),
        }
    }

    /// Добавляет чанк в манифест
    pub fn add_chunk(&mut self, id: usize, hash: &str) {
        self.chunks.push(ChunkInfo {
            id,
            hash: hash.to_string(),
        });
    }

    /// Парсит JSON-манифест, пришедший с сервера/от пира
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Экспортирует манифест в JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Возвращает общее количество чанков
    pub fn total_chunks(&self) -> usize {
        self.chunks.len()
    }
}
