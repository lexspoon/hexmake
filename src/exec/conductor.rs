use std::cell::RefCell;
use std::{fs, io};

use crate::cache::build_cache::BuildCache;
use crate::exec::rule_builder::build_rule;
use crate::file_system::vfs::VirtualFileSystem;
use crate::graph::planner::BuildPlan;
use crate::graph::task::Task;

/// Run a build plan to completion.
pub fn conduct_build<Vfs: VirtualFileSystem>(
    plan: &BuildPlan,
    build_cache: &mut BuildCache<Vfs>,
) -> Result<(), io::Error> {
    fs::create_dir_all("out")?;
    for rule_name in &plan.target_rules {
        let task = &plan.tasks[rule_name];
        run_task_with_deps(task, plan, build_cache)?;
    }

    Ok(())
}

/// Run the dependencies of a task and then the
/// task itself. Do nothing if the task has already
/// been completed.
fn run_task_with_deps<Vfs: VirtualFileSystem>(
    task: &RefCell<Task>,
    build_plan: &BuildPlan,
    build_cache: &mut BuildCache<Vfs>,
) -> Result<(), io::Error> {
    // Check if it is already built
    if task.borrow().is_built {
        return Ok(());
    }

    // Build the dependencies
    for dep in &task.borrow().depends_on {
        run_task_with_deps(&build_plan.tasks[dep], build_plan, build_cache)?;
    }

    // Check the build cache
    if build_cache.retrieve_outputs(&task.borrow().rule)? {
        println!(
            "Retrieved outputs of {} from cache",
            task.borrow().rule.name
        );
        task.borrow_mut().build_finished();
        return Ok(());
    } else {
        // The rule needs its commands run
        build_rule(&task.borrow().rule);
    }

    // All done
    task.borrow_mut().build_finished();
    build_cache.insert_outputs(&task.borrow().rule)?;

    Ok(())
}
