use std::cell::RefCell;
use std::io;
use std::process::Output;
use std::sync::{Arc, Mutex};

/// Logging facility for command output
#[derive(Clone, Default)]
pub struct CommandLogger {
    state: Arc<Mutex<RefCell<CommandLoggerState>>>,
}

#[derive(Default)]
struct CommandLoggerState {
    // Whether an error has occurred so far
    error_occurred: bool,
}

impl CommandLogger {
    /// Log the output that results from the given command. Suppress
    /// output from successful commands if there have been any non-successful commands.
    pub fn log_output(&self, output: &Output, worker_id: u32) -> Result<(), io::Error> {
        let state = self.state.lock().unwrap();
        state.borrow_mut().log_output(output, worker_id)
    }
}

impl CommandLoggerState {
    pub fn log_output(&mut self, output: &Output, worker_id: u32) -> Result<(), io::Error> {
        // Update the cumulative error status
        self.error_occurred |= !output.status.success();

        // Print this command if either there are no errors at all,
        // or if this command was itself an error.
        if !self.error_occurred || !output.status.success() {
            // Print all buffered output
            for line in str::from_utf8(&output.stderr)
                .map_err(|_| io::Error::other("Bad UTF-8"))?
                .lines()
            {
                println!("[worker {worker_id}] {}", line);
            }

            for line in str::from_utf8(&output.stdout)
                .map_err(|_| io::Error::other("Bad UTF-8"))?
                .lines()
            {
                println!("[worker {worker_id}] {}", line);
            }
        }

        Ok(())
    }
}
