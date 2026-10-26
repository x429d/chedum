use sha2::{Digest, Sha256};

/// SHA-256 в hex — стабильный идентификатор ручных импортов.
pub fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_hex() {
        let first = sha256_hex("1. e4 e5 *");
        let second = sha256_hex("1. e4 e5 *");
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
        assert!(first.chars().all(|ch| ch.is_ascii_hexdigit()));
    }
}
