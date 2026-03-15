use std::{fs, path::Path};

use rusqlite::Connection;
use serde_json::json;

use crate::domain::session::SessionRecord;

use super::{ActionResult, QuarantineManifest, move_path, write_audit_event};

pub struct SoftDeleteRequest<'a> {
    pub session: &'a SessionRecord,
    pub quarantine_root: &'a Path,
    pub actor: &'a str,
    pub connection: &'a Connection,
}

pub fn soft_delete_session(request: &SoftDeleteRequest<'_>) -> ActionResult<QuarantineManifest> {
    let source_path = Path::new(&request.session.source_path);
    let basename = source_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("session");
    let session_root = request.quarantine_root.join(&request.session.session_id);
    let quarantined_path = session_root.join("payload").join(basename);

    move_path(source_path, &quarantined_path)?;

    let manifest_path = session_root.join("manifest.json");
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let manifest = QuarantineManifest {
        session_id: request.session.session_id.clone(),
        original_path: source_path.to_path_buf(),
        quarantined_path: quarantined_path.clone(),
        manifest_path: manifest_path.clone(),
        deleted_at: chrono::Utc::now().to_rfc3339(),
    };

    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    write_audit_event(
        request.connection,
        "soft_delete",
        "session",
        &request.session.session_id,
        request.actor,
        Some(json!({ "source_path": request.session.source_path }).to_string()),
        Some(
            json!({
                "quarantined_path": quarantined_path,
                "manifest_path": manifest_path,
            })
            .to_string(),
        ),
        "success",
    )?;

    Ok(manifest)
}
