use std::{fs, path::Path};

use rusqlite::Connection;
use serde_json::json;

use super::{ActionResult, QuarantineManifest, move_path, remove_empty_parents, write_audit_event};

pub fn restore_session(
    manifest_path: &Path,
    actor: &str,
    connection: &Connection,
) -> ActionResult<QuarantineManifest> {
    let manifest_text = fs::read_to_string(manifest_path)?;
    let manifest: QuarantineManifest = serde_json::from_str(&manifest_text)?;

    move_path(&manifest.quarantined_path, &manifest.original_path)?;
    fs::remove_file(manifest_path)?;

    if let Some(session_root) = manifest_path.parent() {
        remove_empty_parents(manifest_path, session_root)?;
    }

    write_audit_event(
        connection,
        "restore",
        "session",
        &manifest.session_id,
        actor,
        Some(
            json!({
                "quarantined_path": manifest.quarantined_path,
                "manifest_path": manifest.manifest_path,
            })
            .to_string(),
        ),
        Some(json!({ "restored_path": manifest.original_path }).to_string()),
        "success",
    )?;

    Ok(manifest)
}
