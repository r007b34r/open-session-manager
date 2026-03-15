use sha2::{Digest, Sha256};

pub fn mask_secret(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= 4 {
        return "***".to_string();
    }

    let suffix = trimmed
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    format!("***{suffix}")
}

pub fn fingerprint_secret(value: &str) -> String {
    let digest = Sha256::digest(value.trim().as_bytes());
    let fingerprint = digest
        .iter()
        .take(6)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{fingerprint}")
}
