use std::cell::RefCell;

use crate::exec::rule_builder::build_rule;
use crate::graph::planner::BuildPlan;
use crate::graph::task::Task;

/// Run a build plan to completion.
pub fn conduct_build(plan: &BuildPlan) {
    for task in plan.values() {
        run_task_with_deps(task, plan);
    }
}

/// Run the dependencies of a task and then the
/// task itself. Do nothing if the task has already
/// been completed.
fn run_task_with_deps(task: &RefCell<Task>, build_plan: &BuildPlan) {
    if task.borrow().is_built {
        return;
    }

    for dep in &task.borrow().depends_on {
        run_task_with_deps(&build_plan[dep], build_plan);
    }

    build_rule(&task.borrow().rule);
    task.borrow_mut().build_finished();
}
