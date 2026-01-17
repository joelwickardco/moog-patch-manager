use sha2::{Sha256, Digest};

/// Calculate SHA-256 hash of data, returning a 64-character hex string
pub fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

// We need to add hex to dependencies, or we can do it manually
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let data = b"test patch data";
        let hash = calculate_sha256(data);
        assert_eq!(hash.len(), 64); // SHA-256 is 64 hex chars
    }

    #[test]
    fn test_same_data_same_hash() {
        let data = b"identical content";
        let hash1 = calculate_sha256(data);
        let hash2 = calculate_sha256(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_data_different_hash() {
        let hash1 = calculate_sha256(b"content 1");
        let hash2 = calculate_sha256(b"content 2");
        assert_ne!(hash1, hash2);
    }
}
