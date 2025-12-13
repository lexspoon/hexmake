use crate::ast::hexmake_file::RuleName;
use crate::graph::task::Task;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

/// A work list of things the conductor has in progress.
/// This is shared inside a mutex among the conductor and
/// all the workers.
#[derive(Default)]
pub struct WorkList {
    /// All tasks that are ready to run and that a worker should feel free to grab
    pub pending_tasks: Vec<Arc<Mutex<Task>>>,

    /// Tasks that are currently running.
    pub running_tasks: BTreeSet<RuleName>,

    /// Whether an error has occurred or not
    pub error_occurred: bool,
}
