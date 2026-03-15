use std::path::Path;

use rusqlite::Connection;

use crate::{
    actions::{
        delete::{SoftDeleteRequest, soft_delete_session},
        export::{ExportRequest, ExportResult, export_session_markdown},
        restore::restore_session,
    },
    domain::session::{SessionInsight, SessionRecord},
};

pub fn export_session(
    session: &SessionRecord,
    insight: &SessionInsight,
    output_root: &Path,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<ExportResult> {
    export_session_markdown(&ExportRequest {
        session,
        insight,
        output_root,
        actor,
        connection,
    })
}

pub fn delete_session(
    session: &SessionRecord,
    quarantine_root: &Path,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<crate::actions::QuarantineManifest> {
    soft_delete_session(&SoftDeleteRequest {
        session,
        quarantine_root,
        actor,
        connection,
    })
}

pub fn restore_deleted_session(
    manifest_path: &Path,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<crate::actions::QuarantineManifest> {
    restore_session(manifest_path, actor, connection)
}
