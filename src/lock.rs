use fs_err::{File, create_dir_all};

use crate::error::Error;
use std::thread::sleep;
use std::time::Duration;

/// Obtain a lock on the output directory, pausing if necessary.
/// Returns the lock file, which holds the lock until dropped.
pub fn obtain_lock() -> Result<File, Error> {
    if let Some(file) = try_lock()? {
        return Ok(file);
    }

    println!("Waiting on another Hexmake instance that is already running.");

    // Loop with exponential backoff
    let mut delay = Duration::from_millis(50);
    loop {
        sleep(delay);

        if let Some(file) = try_lock()? {
            return Ok(file);
        }

        delay = next_delay(delay);
    }
}

/// Try to acquire a lock, right now. Returns error for I/O errors, but None if
/// the lock attempt simply failed.
fn try_lock() -> Result<Option<File>, Error> {
    create_dir_all(".hex")?;
    let file = File::create(".hex/lock")?;

    if file.try_lock().is_err() {
        return Ok(None);
    }

    Ok(Some(file))
}

/// Compute the next delay to use. This will slowly back off up to a
/// maximum of 5 seconds
fn next_delay(duration: Duration) -> Duration {
    // Add 50%
    let millis = duration.as_millis() as u64;
    let duration = Duration::from_millis(millis + (millis >> 1));

    // Cap it at 5 seconds
    duration.min(Duration::from_secs(5))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_delay() {
        // Typical small values
        assert_eq!(
            next_delay(Duration::from_millis(100)),
            Duration::from_millis(150)
        );
        assert_eq!(
            next_delay(Duration::from_millis(1000)),
            Duration::from_millis(1500)
        );

        // Cap the delay at 5 seconds
        assert_eq!(
            next_delay(Duration::from_millis(4000)),
            Duration::from_millis(5000)
        );
        assert_eq!(
            next_delay(Duration::from_millis(5000)),
            Duration::from_millis(5000)
        );
    }
}
