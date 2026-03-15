use std::{
    fmt,
    fs,
    io,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::domain::session::SessionRecord;

pub trait SessionAdapter {
    fn assistant_name(&self) -> &'static str;
    fn discover_session_files(&self, root: &Path) -> AdapterResult<Vec<PathBuf>>;
    fn parse_session(&self, source: &Path) -> AdapterResult<SessionRecord>;
}

pub type AdapterResult<T> = Result<T, AdapterError>;

#[derive(Debug)]
pub enum AdapterError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidSession(String),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
            Self::InvalidSession(message) => write!(f, "invalid session: {message}"),
        }
    }
}

impl std::error::Error for AdapterError {}

impl From<io::Error> for AdapterError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for AdapterError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub fn collect_files(root: &Path, predicate: &dyn Fn(&Path) -> bool) -> AdapterResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    visit_dirs(root, predicate, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit_dirs(
    current: &Path,
    predicate: &dyn Fn(&Path) -> bool,
    files: &mut Vec<PathBuf>,
) -> AdapterResult<()> {
    if !current.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            visit_dirs(&path, predicate, files)?;
        } else if predicate(&path) {
            files.push(path);
        }
    }

    Ok(())
}

pub fn hash_file(path: &Path) -> AdapterResult<String> {
    let bytes = fs::read(path)?;
    Ok(hash_bytes(&bytes))
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
