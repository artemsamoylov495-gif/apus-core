use sha2::{Sha256, Digest};

pub struct BlockValidator;

impl BlockValidator {
    /// Проверяет целостность скачанного P2P-блока по SHA-256 хешу
    pub fn verify_block(data: &[u8], expected_hash: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let calculated_hash = format!("{:x}", result);

        calculated_hash == expected_hash
    }
}
