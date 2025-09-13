#![allow(dead_code)]

use std::{
    collections::BTreeSet,
    fmt::{Display, Formatter},
    rc::Rc,
};

use crate::ast::hexmake_file::{HexRule, RuleName};

/// A task to be executed, along with dependency and status information.
pub struct Task {
    pub rule: Rc<HexRule>,
    pub depends_on: BTreeSet<RuleName>,
    pub used_by: BTreeSet<RuleName>,
    unbuilt_dependencies: usize,
    pub is_built: bool,
}

impl Task {
    pub fn new(rule: Rc<HexRule>) -> Task {
        Task {
            rule: rule.clone(),
            depends_on: BTreeSet::new(),
            used_by: BTreeSet::new(),
            unbuilt_dependencies: 0,
            is_built: false,
        }
    }

    pub fn rule_name(&self) -> RuleName {
        self.rule.name.clone()
    }

    /// Add a new dependency
    pub fn add_depends_on(&mut self, other_task: &mut Task) {
        if !self.depends_on.insert(other_task.rule_name()) {
            // This dependency was already present
            return;
        }
        other_task.used_by.insert(self.rule_name());
        self.unbuilt_dependencies += 1;
    }

    /// Inform this task that one of its dependencies finished building.
    /// Return the remaining number of dependencies that still need to be built.
    pub fn dependency_finished(&mut self) -> usize {
        assert!(self.unbuilt_dependencies > 0);
        let new_count = self.unbuilt_dependencies - 1;
        self.unbuilt_dependencies = new_count;
        new_count
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
