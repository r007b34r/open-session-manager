#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

#[cfg(test)]
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[cfg(test)]
pub(crate) fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("lock env guard")
}
