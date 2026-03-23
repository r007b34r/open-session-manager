use std::{fs, path::Path};

use rusqlite::Connection;
use serde_json::json;

use crate::domain::session::SessionRecord;

use super::{
    ActionError, ActionResult, AuditWriteRequest, QuarantineAsset, QuarantineManifest,
    has_successful_cleanup_checklist, has_successful_markdown_export,
    latest_export_resume_artifact_path, latest_session_end_hook_failed, move_path,
    safe_managed_name, write_audit_event,
};

pub struct SoftDeleteRequest<'a> {
    pub session: &'a SessionRecord,
    pub quarantine_root: &'a Path,
    pub actor: &'a str,
    pub connection: &'a Connection,
}

pub fn soft_delete_session(request: &SoftDeleteRequest<'_>) -> ActionResult<QuarantineManifest> {
    if !has_successful_markdown_export(request.connection, &request.session.session_id)? {
        return Err(ActionError::Precondition(
            "soft delete requires a successful Markdown export first".to_string(),
        ));
    }

    if !has_successful_cleanup_checklist(request.connection, &request.session.session_id)? {
        return Err(ActionError::Precondition(
            "soft delete requires a successful cleanup checklist first".to_string(),
        ));
    }

    if latest_session_end_hook_failed(request.connection, &request.session.session_id)? {
        return Err(ActionError::Precondition(
            "soft delete is blocked because the latest session-end hook failed".to_string(),
        ));
    }

    let source_path = Path::new(&request.session.source_path);
    let basename = source_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("session");
    let session_root = request
        .quarantine_root
        .join(safe_managed_name(&request.session.session_id));
    let quarantined_path = session_root.join("payload").join(basename);
    let related_assets = collect_related_assets(request.session, &session_root);
    let resume_artifact_path =
        latest_export_resume_artifact_path(request.connection, &request.session.session_id)?;

    move_path(source_path, &quarantined_path)?;
    for asset in &related_assets {
        move_path(&asset.original_path, &asset.quarantined_path)?;
    }

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
        resume_artifact_path: resume_artifact_path.clone(),
        related_assets,
    };

    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    write_audit_event(
        request.connection,
        AuditWriteRequest {
            event_type: "soft_delete",
            target_type: "session",
            target_id: &request.session.session_id,
            actor: request.actor,
            before_state: Some(json!({ "source_path": request.session.source_path }).to_string()),
            after_state: Some(
                json!({
                    "quarantined_path": quarantined_path,
                    "manifest_path": manifest_path,
                    "resume_artifact_path": resume_artifact_path,
                })
                .to_string(),
            ),
            result: "success",
        },
    )?;

    Ok(manifest)
}

fn collect_related_assets(session: &SessionRecord, session_root: &Path) -> Vec<QuarantineAsset> {
    if session.assistant != "opencode" {
        return Vec::new();
    }

    let source_path = Path::new(&session.source_path);
    let Some(storage_root) = source_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
    else {
        return Vec::new();
    };

    let safe_session_id = safe_managed_name(&session.session_id);
    let candidates = [
        (
            storage_root
                .join("session")
                .join("message")
                .join(&session.session_id),
            session_root
                .join("payload")
                .join("related")
                .join("message")
                .join(&safe_session_id),
        ),
        (
            storage_root
                .join("session")
                .join("part")
                .join(&session.session_id),
            session_root
                .join("payload")
                .join("related")
                .join("part")
                .join(&safe_session_id),
        ),
    ];

    candidates
        .into_iter()
        .filter(|(original_path, _)| original_path.exists())
        .map(|(original_path, quarantined_path)| QuarantineAsset {
            original_path,
            quarantined_path,
        })
        .collect()
}
