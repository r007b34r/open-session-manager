use std::{
    path::{Path, PathBuf},
    process::Command,
};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{ActionError, ActionResult, AuditWriteRequest, write_audit_event};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitActionResult {
    pub repo_root: PathBuf,
    pub branch: String,
    pub summary: String,
    pub output: String,
}

pub struct GitCommitRequest<'a> {
    pub repo_root: &'a Path,
    pub message: &'a str,
    pub actor: &'a str,
    pub connection: &'a Connection,
}

pub struct GitBranchSwitchRequest<'a> {
    pub repo_root: &'a Path,
    pub branch: &'a str,
    pub actor: &'a str,
    pub connection: &'a Connection,
}

pub struct GitPushRequest<'a> {
    pub repo_root: &'a Path,
    pub remote: Option<&'a str>,
    pub actor: &'a str,
    pub connection: &'a Connection,
}

pub fn commit_project(request: &GitCommitRequest<'_>) -> ActionResult<GitActionResult> {
    let repo_root = ensure_repo_root(request.repo_root)?;
    let message = request.message.trim();
    if message.is_empty() {
        return Err(ActionError::Precondition(
            "git commit requires a non-empty message".to_string(),
        ));
    }
    if !has_dirty_worktree(&repo_root)? {
        return Err(ActionError::Precondition(
            "git commit requires uncommitted changes".to_string(),
        ));
    }

    run_git(&repo_root, &["add", "-A"])?;
    let output = run_git(&repo_root, &["commit", "-m", message])?;
    let branch = current_branch(&repo_root)?;
    let summary = latest_commit_summary(&repo_root)?;

    write_git_audit_event(
        request.connection,
        "git_commit",
        &repo_root,
        request.actor,
        &branch,
        &summary,
        &output,
    )?;

    Ok(GitActionResult {
        repo_root,
        branch,
        summary,
        output,
    })
}

pub fn switch_branch(request: &GitBranchSwitchRequest<'_>) -> ActionResult<GitActionResult> {
    let repo_root = ensure_repo_root(request.repo_root)?;
    let branch = request.branch.trim();
    if branch.is_empty() {
        return Err(ActionError::Precondition(
            "git branch switch requires a target branch".to_string(),
        ));
    }
    if has_dirty_worktree(&repo_root)? {
        return Err(ActionError::Precondition(
            "git branch switch requires a clean worktree".to_string(),
        ));
    }

    let output = if local_branch_exists(&repo_root, branch)? {
        run_git(&repo_root, &["switch", branch])?
    } else {
        run_git(&repo_root, &["switch", "-c", branch])?
    };
    let current_branch = current_branch(&repo_root)?;
    let summary = format!("Switched to {current_branch}");

    write_git_audit_event(
        request.connection,
        "git_branch_switch",
        &repo_root,
        request.actor,
        &current_branch,
        &summary,
        &output,
    )?;

    Ok(GitActionResult {
        repo_root,
        branch: current_branch,
        summary,
        output,
    })
}

pub fn push_project(request: &GitPushRequest<'_>) -> ActionResult<GitActionResult> {
    let repo_root = ensure_repo_root(request.repo_root)?;
    if has_dirty_worktree(&repo_root)? {
        return Err(ActionError::Precondition(
            "git push requires a clean worktree".to_string(),
        ));
    }

    let branch = current_branch(&repo_root)?;
    let remote = request
        .remote
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("origin");
    let output = if request
        .remote
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
    {
        run_git(&repo_root, &["push", remote, &branch])?
    } else {
        ensure_upstream_branch(&repo_root)?;
        run_git(&repo_root, &["push"])?
    };
    let output = if output.contains(remote) {
        output
    } else if output.is_empty() {
        format!("Pushed {branch} to {remote}")
    } else {
        format!("Pushed {branch} to {remote}\n{output}")
    };
    let summary = format!("Pushed {branch} to {remote}");

    write_git_audit_event(
        request.connection,
        "git_push",
        &repo_root,
        request.actor,
        &branch,
        &summary,
        &output,
    )?;

    Ok(GitActionResult {
        repo_root,
        branch,
        summary,
        output,
    })
}

fn ensure_repo_root(repo_root: &Path) -> ActionResult<PathBuf> {
    if !repo_root.exists() {
        return Err(ActionError::Precondition(format!(
            "git repo does not exist: {}",
            repo_root.display()
        )));
    }

    Ok(repo_root.to_path_buf())
}

fn has_dirty_worktree(repo_root: &Path) -> ActionResult<bool> {
    Ok(!run_git(repo_root, &["status", "--porcelain"])?.is_empty())
}

fn current_branch(repo_root: &Path) -> ActionResult<String> {
    run_git(repo_root, &["branch", "--show-current"])
}

fn latest_commit_summary(repo_root: &Path) -> ActionResult<String> {
    run_git(repo_root, &["log", "-1", "--pretty=%s"])
}

fn local_branch_exists(repo_root: &Path, branch: &str) -> ActionResult<bool> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ])
        .status()
        .map_err(ActionError::Io)?;

    Ok(status.success())
}

fn ensure_upstream_branch(repo_root: &Path) -> ActionResult<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args([
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ])
        .output()
        .map_err(ActionError::Io)?;

    if output.status.success() {
        Ok(())
    } else {
        Err(ActionError::Precondition(
            "git push requires an upstream tracking branch".to_string(),
        ))
    }
}

fn run_git(repo_root: &Path, args: &[&str]) -> ActionResult<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(ActionError::Io)?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let combined = [stdout.as_str(), stderr.as_str()]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if !output.status.success() {
        return Err(ActionError::Execution(if combined.is_empty() {
            format!("git {:?} failed", args)
        } else {
            combined
        }));
    }

    Ok(combined)
}

fn write_git_audit_event(
    connection: &Connection,
    event_type: &str,
    repo_root: &Path,
    actor: &str,
    branch: &str,
    summary: &str,
    output: &str,
) -> ActionResult<()> {
    write_audit_event(
        connection,
        AuditWriteRequest {
            event_type,
            target_type: "git_repo",
            target_id: &repo_root.display().to_string(),
            actor,
            before_state: None,
            after_state: Some(
                json!({
                    "repo_root": repo_root.display().to_string(),
                    "branch": branch,
                    "summary": summary,
                    "output": output
                })
                .to_string(),
            ),
            result: "success",
        },
    )?;

    Ok(())
}
