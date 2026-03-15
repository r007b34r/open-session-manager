use sha2::{Digest, Sha256};

use crate::domain::config::CredentialArtifact;

use super::{
    SecretMaterial,
    redaction::{fingerprint_secret, mask_secret},
};

pub fn build_credential_artifacts(secrets: &[SecretMaterial]) -> Vec<CredentialArtifact> {
    secrets
        .iter()
        .map(|secret| CredentialArtifact {
            artifact_id: credential_artifact_id(secret),
            provider: secret.provider.clone(),
            kind: secret.kind.clone(),
            location: secret.location.clone(),
            source_type: secret.source_type.clone(),
            masked_value: mask_secret(&secret.value),
            fingerprint: fingerprint_secret(&secret.value),
            official_or_proxy: secret.official_or_proxy.clone(),
            last_modified_at: secret.last_modified_at.clone(),
        })
        .collect()
}

fn credential_artifact_id(secret: &SecretMaterial) -> String {
    let payload = format!(
        "{}:{}:{}:{}",
        secret.provider, secret.kind, secret.location, secret.source_type
    );
    let digest = Sha256::digest(payload.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
