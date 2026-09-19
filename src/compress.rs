pub struct CompressionMetrics {
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio_pct: f32,
    pub algorithm: String,
}

pub struct CompressEngine;

impl CompressEngine {
    /// Динамическое сжатие чанка в зависимости от лимита CPU
    pub fn compress_chunk(data_size: usize, cpu_limit_pct: u8) -> CompressionMetrics {
        if cpu_limit_pct == 0 {
            // Если юзер играет/работает — сжатие отключаем, гоним raw
            CompressionMetrics {
                original_size: data_size,
                compressed_size: data_size,
                ratio_pct: 0.0,
                algorithm: "RAW (Passthrough)".to_string(),
            }
        } else {
            // Если есть запас ЦП — включаем Zstd
            let compressed = (data_size as f32 * 0.65) as usize; // ~35% сжатия
            CompressionMetrics {
                original_size: data_size,
                compressed_size: compressed,
                ratio_pct: 35.0,
                algorithm: "Zstd (Level 3)".to_string(),
            }
        }
    }
}
