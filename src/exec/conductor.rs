use std::sync::{Arc, Condvar, Mutex};
use std::thread::spawn;
use std::{fs, io};

use crate::cache::build_cache::BuildCache;
use crate::exec::rule_builder::build_rule;
use crate::exec::work_dir::WorkDirManager;
use crate::exec::work_list::WorkList;
use crate::graph::planner::BuildPlan;
use crate::graph::task::Task;

/// Run a build plan to completion.
pub fn conduct_build(plan: &BuildPlan, build_cache: &Arc<BuildCache>) -> Result<(), io::Error> {
    fs::create_dir_all("out")?;

    let work_list = Arc::new(Mutex::new(WorkList::default()));
    let work_list_condvar = Arc::new(Condvar::new());

    // Schedule tasks that have no dependencies
    {
        let mut work_list = work_list.lock().unwrap();
        for task in plan.tasks.values() {
            if task.lock().unwrap().ready_to_run() {
                work_list.pending_tasks.push(task.clone());
            }
        }
    }

    // Start workers
    for i in 0..4 {
        let work_list = work_list.clone();
        let work_list_condvar = work_list_condvar.clone();
        let build_cache = build_cache.clone();
        spawn(move || run_worker(i, work_list, work_list_condvar, build_cache));
    }

    wait_for_workers(work_list, work_list_condvar)?;
    build_cache.maybe_gc()?;

    Ok(())
}

/// Run a worker that builds tasks. It will grab tasks from the WorkList,
/// build them, and schedule new tasks that then become possible.
fn run_worker(
    worker_id: u32,
    work_list: Arc<Mutex<WorkList>>,
    work_list_condvar: Arc<Condvar>,
    build_cache: Arc<BuildCache>,
) {
    let work_dir = WorkDirManager::new(worker_id);

    loop {
        // Grab a task from the pending list
        let task = match get_task_from_worklist(&work_list, &work_list_condvar) {
            Some(value) => value,
            None => return,
        };
        let mut task = task.lock().unwrap();

        let build_result = check_cache_or_build_now(worker_id, &mut task, &build_cache, &work_dir);

        // Remove from running tasks
        let mut work_list = work_list.lock().unwrap();
        work_list.running_tasks.remove(&task.rule_name());

        // Shut down if an error happened
        if let Err(error) = build_result {
            println!("[worker {worker_id}] {error}");

            work_list.error_occurred = true;
            work_list.pending_tasks.clear();

            work_list_condvar.notify_all();

            return;
        };

        // Add dependent tasks that are now ready to run
        for used_by in &task.used_by {
            let mut used_by_locked = used_by.lock().unwrap();
            if used_by_locked.dependency_finished() == 0 {
                // This task is now ready to run
                work_list.pending_tasks.push(used_by.clone());
            }
        }

        work_list_condvar.notify_all();
    }
}

fn check_cache_or_build_now(
    worker_id: u32,
    task: &mut Task,
    build_cache: &Arc<BuildCache>,
    work_dir: &WorkDirManager,
) -> Result<(), io::Error> {
    if build_cache.retrieve_outputs(&task.rule)? {
        println!(
            "[worker {worker_id}] Retrieved outputs of {} from cache",
            task.rule.name
        );
    } else {
        build_rule(worker_id, &task.rule, work_dir)?;
        build_cache.insert_outputs(&task.rule)?;
    }

    task.build_finished();

    Ok(())
}

/// Retrieve a task from the worklist. Return None if there are no more tasks
/// and the worker should exit. If this returns a task, it will also put it
/// in the list of running tasks in the worklist.
fn get_task_from_worklist(
    work_list: &Arc<Mutex<WorkList>>,
    work_list_condvar: &Arc<Condvar>,
) -> Option<Arc<Mutex<crate::graph::task::Task>>> {
    let mut work_list = work_list.lock().unwrap();

    loop {
        if work_list.pending_tasks.is_empty() && work_list.running_tasks.is_empty() {
            // All work is done
            return None;
        }

        if !work_list.pending_tasks.is_empty() {
            // There are tasks in the list, now. Take one and return it.
            let last_index = work_list.pending_tasks.len() - 1;
            let task = work_list.pending_tasks.remove(last_index);
            work_list
                .running_tasks
                .insert(task.lock().unwrap().rule_name());
            return Some(task);
        }

        // There are no available tasks. Go to sleep and wait for
        // the work list to change.
        work_list = work_list_condvar.wait(work_list).unwrap();
    }
}

/// Wait for all workers to be finished. This is done by
/// checking the work list for active and pending work.
fn wait_for_workers(
    work_list: Arc<Mutex<WorkList>>,
    work_list_condvar: Arc<Condvar>,
) -> Result<(), io::Error> {
    let mut work_list = work_list.lock().unwrap();
    while !work_list.pending_tasks.is_empty() || !work_list.running_tasks.is_empty() {
        work_list = work_list_condvar.wait(work_list).unwrap();
    }

    if work_list.error_occurred {
        Err(io::Error::other("BUILD FAILED"))
    } else {
        Ok(())
    }
}
