use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};

use crate::ast::hexmake_file::{HexRule, RuleName};

/// A task to be executed, along with dependency and status information.
pub struct Task {
    pub rule: Arc<HexRule>,
    pub depends_on: Vec<Arc<Mutex<Task>>>,
    pub used_by: Vec<Arc<Mutex<Task>>>,
    unbuilt_dependencies: usize,

    /// Whether the task has finished building
    pub is_built: bool,
}

impl Task {
    pub fn new(rule: Arc<HexRule>) -> Task {
        Task {
            rule: rule.clone(),
            depends_on: Vec::new(),
            used_by: Vec::new(),
            unbuilt_dependencies: 0,
            is_built: false,
        }
    }

    pub fn rule_name(&self) -> RuleName {
        self.rule.name.clone()
    }

    /// Add a new dependency between two tasks.
    /// This accepts Arc<Mutex<Task>> because the arc needs to be cloned to be
    /// added in each direction.
    pub fn add_dependency(from_task: &Arc<Mutex<Task>>, to_task: &Arc<Mutex<Task>>) {
        // Check if the current task already depends on the other one
        let to_rule_name = to_task.lock().unwrap().rule_name();
        let mut from_task_locked = from_task.lock().unwrap();
        if from_task_locked.depends_on_rule(&to_rule_name) {
            // This dependency is already in the list
            return;
        }
        from_task_locked.depends_on.push(to_task.clone());
        from_task_locked.unbuilt_dependencies += 1;
        to_task.lock().unwrap().used_by.push(from_task.clone());
    }

    /// Whether this task depends on the given rule name
    pub fn depends_on_rule(&self, rule_name: &RuleName) -> bool {
        for dep in &self.depends_on {
            if dep.lock().unwrap().rule_name() == *rule_name {
                return true;
            }
        }
        false
    }

    /// Inform this task that one of its dependencies finished building.
    /// Return the remaining number of dependencies that still need to be built.
    #[allow(dead_code)]
    pub fn dependency_finished(&mut self) -> usize {
        assert!(self.unbuilt_dependencies > 0);
        let new_count = self.unbuilt_dependencies - 1;
        self.unbuilt_dependencies = new_count;
        new_count
    }

    /// Whether this task is ready to run, i.e. all of its dependencies are built
    pub fn ready_to_run(&self) -> bool {
        self.unbuilt_dependencies == 0
    }

    /// Inform this task that it has now been built
    pub fn build_finished(&mut self) {
        assert!(!self.is_built);
        self.is_built = true;
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let status: &str = if self.is_built {
            "built"
        } else {
            &format!("{} unbuilt deps", self.unbuilt_dependencies)
        };

        write!(f, "{} ({})", self.rule.name, status)
    }
}
