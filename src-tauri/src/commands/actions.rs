use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::{
    actions::{
        config_writeback::{
            ConfigWritebackRequest, ConfigWritebackResult, ConfigWritebackUpdate, write_config,
        },
        delete::{SoftDeleteRequest, soft_delete_session},
        export::{ExportRequest, ExportResult, export_session_markdown},
        restore::restore_session,
    },
    audit::config_audit::ConfigAuditTarget,
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

pub fn write_config_artifact(
    target: &ConfigAuditTarget,
    update: &ConfigWritebackUpdate,
    backup_root: &Path,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<ConfigWritebackResult> {
    write_config(&ConfigWritebackRequest {
        target,
        update,
        backup_root,
        actor,
        connection,
    })
}

pub fn restore_deleted_session(
    manifest_path: &Path,
    quarantine_root: &Path,
    allowed_restore_roots: &[PathBuf],
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<crate::actions::QuarantineManifest> {
    restore_session(
        manifest_path,
        quarantine_root,
        allowed_restore_roots,
        actor,
        connection,
    )
}
