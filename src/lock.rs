use std::fs::{File, create_dir_all};

use crate::error::Error;

/// Try to acquire an exclusive lock on `.hex/lock`.
///
/// Returns the lock file, which holds the lock until dropped.
/// Returns an error if another instance of hexmake is already running.
pub fn try_lock() -> Result<File, Error> {
    create_dir_all(".hex")?;
    let file = File::create(".hex/lock")?;
    file.try_lock().map_err(|_| {
        Error::Hexmake(
            "Another instance of Hexmake is already running for this directory".to_string(),
        )
    })?;
    Ok(file)
}
