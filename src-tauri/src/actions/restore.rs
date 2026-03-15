use std::{
    fs,
    path::{Path, PathBuf},
};

use rusqlite::Connection;
use serde_json::json;

use super::{
    ActionError, ActionResult, AuditWriteRequest, QuarantineAsset, QuarantineManifest,
    ensure_allowed_restore_path, ensure_managed_path, move_path, remove_empty_parents,
    write_audit_event,
};

pub fn restore_session(
    manifest_path: &Path,
    quarantine_root: &Path,
    allowed_restore_roots: &[PathBuf],
    actor: &str,
    connection: &Connection,
) -> ActionResult<QuarantineManifest> {
    let manifest_text = fs::read_to_string(manifest_path)?;
    let manifest: QuarantineManifest = serde_json::from_str(&manifest_text)?;
    let actual_manifest_path = ensure_managed_path(manifest_path, quarantine_root, "manifest path")?;
    let recorded_manifest_path =
        ensure_managed_path(&manifest.manifest_path, quarantine_root, "manifest path")?;

    if actual_manifest_path != recorded_manifest_path {
        return Err(ActionError::Precondition(
            "manifest path must match the managed quarantine root".to_string(),
        ));
    }

    let quarantined_path =
        ensure_managed_path(&manifest.quarantined_path, quarantine_root, "quarantined payload path")?;
    let original_path = ensure_allowed_restore_path(
        &manifest.original_path,
        allowed_restore_roots,
        quarantine_root,
        "restore target path",
    )?;
    let related_assets = manifest
        .related_assets
        .iter()
        .map(|asset| {
            Ok(QuarantineAsset {
                original_path: ensure_allowed_restore_path(
                    &asset.original_path,
                    allowed_restore_roots,
                    quarantine_root,
                    "restore target path",
                )?,
                quarantined_path: ensure_managed_path(
                    &asset.quarantined_path,
                    quarantine_root,
                    "quarantined payload path",
                )?,
            })
        })
        .collect::<ActionResult<Vec<_>>>()?;

    move_path(&quarantined_path, &original_path)?;
    for asset in &related_assets {
        move_path(&asset.quarantined_path, &asset.original_path)?;
    }
    fs::remove_file(&actual_manifest_path)?;
    for asset in &related_assets {
        remove_empty_parents(&asset.quarantined_path, quarantine_root)?;
    }

    remove_empty_parents(&actual_manifest_path, quarantine_root)?;

    write_audit_event(
        connection,
        AuditWriteRequest {
            event_type: "restore",
            target_type: "session",
            target_id: &manifest.session_id,
            actor,
            before_state: Some(
                json!({
                    "quarantined_path": quarantined_path,
                    "manifest_path": actual_manifest_path,
                    "related_assets": related_assets.len(),
                })
                .to_string(),
            ),
            after_state: Some(
                json!({
                    "restored_path": original_path,
                    "related_assets": related_assets.len(),
                })
                .to_string(),
            ),
            result: "success",
        },
    )?;

    Ok(manifest)
}
