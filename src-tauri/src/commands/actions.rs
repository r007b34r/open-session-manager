use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::{
    actions::{
        config_writeback::{
            ConfigWritebackRequest, ConfigWritebackResult, ConfigWritebackUpdate, write_config,
        },
        delete::{SoftDeleteRequest, soft_delete_session},
        export::{ExportRequest, ExportResult, export_session_markdown},
        git_control::{
            GitActionResult, GitBranchSwitchRequest, GitCommitRequest, GitPushRequest,
            commit_project, push_project, switch_branch,
        },
        restore::restore_session,
        session_control::{
            SessionControlRequest, SessionControlResult, attach_session, continue_session,
            detach_session, pause_session, resume_session,
        },
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

pub fn resume_existing_session(
    session: &SessionRecord,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<SessionControlResult> {
    resume_session(&SessionControlRequest {
        session,
        actor,
        connection,
        prompt: None,
    })
}

pub fn continue_existing_session(
    session: &SessionRecord,
    prompt: &str,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<SessionControlResult> {
    continue_session(&SessionControlRequest {
        session,
        actor,
        connection,
        prompt: Some(prompt),
    })
}

pub fn pause_existing_session(
    session: &SessionRecord,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<SessionControlResult> {
    pause_session(&SessionControlRequest {
        session,
        actor,
        connection,
        prompt: None,
    })
}

pub fn attach_existing_session(
    session: &SessionRecord,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<SessionControlResult> {
    attach_session(&SessionControlRequest {
        session,
        actor,
        connection,
        prompt: None,
    })
}

pub fn detach_existing_session(
    session: &SessionRecord,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<SessionControlResult> {
    detach_session(&SessionControlRequest {
        session,
        actor,
        connection,
        prompt: None,
    })
}

pub fn commit_git_project(
    repo_root: &Path,
    message: &str,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<GitActionResult> {
    commit_project(&GitCommitRequest {
        repo_root,
        message,
        actor,
        connection,
    })
}

pub fn switch_git_project_branch(
    repo_root: &Path,
    branch: &str,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<GitActionResult> {
    switch_branch(&GitBranchSwitchRequest {
        repo_root,
        branch,
        actor,
        connection,
    })
}

pub fn push_git_project(
    repo_root: &Path,
    remote: Option<&str>,
    actor: &str,
    connection: &Connection,
) -> crate::actions::ActionResult<GitActionResult> {
    push_project(&GitPushRequest {
        repo_root,
        remote,
        actor,
        connection,
    })
}
