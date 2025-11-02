use std::process::exit;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::spawn;
use std::{fs, io};

use crate::cache::build_cache::BuildCache;
use crate::exec::rule_builder::build_rule;
use crate::graph::planner::BuildPlan;
use crate::graph::task::Task;
use crossbeam_channel::{Receiver, Sender};

/// Run a build plan to completion.
pub fn conduct_build(plan: &BuildPlan, build_cache: &Arc<BuildCache>) -> Result<(), io::Error> {
    fs::create_dir_all("out")?;

    // Create a channel to submit tasks on when they are ready to run
    let (task_sender, task_receiver) = crossbeam_channel::unbounded();

    // Create a condvar for communicating back to the main thread as tasks
    // are finished.
    let task_finished = Arc::new(Condvar::new());

    // Start workers
    for i in 0..4 {
        let task_sender = task_sender.clone();
        let task_receiver = task_receiver.clone();
        let task_finished = task_finished.clone();
        let build_cache = build_cache.clone();
        spawn(move || run_worker(i, task_sender, task_receiver, &task_finished, build_cache));
    }

    // Start running tasks that have no dependencies
    for task in plan.tasks.values() {
        if task.lock().unwrap().ready_to_run() {
            task_sender.send(task.clone()).unwrap();
        }
    }

    // Wait for the targets
    for rule_name in &plan.target_rules {
        wait_for_task(&plan.tasks[rule_name], &task_finished);
    }

    Ok(())
}

/// Block until the given task has finished building
fn wait_for_task(task: &Arc<Mutex<Task>>, task_finished: &Condvar) {
    let mut task = task.lock().unwrap();
    while !task.is_built {
        task = task_finished.wait(task).unwrap();
    }
}

/// Run a worker that builds tasks. It will read tasks from the given channel,
/// build them, and schedule new tasks that then become possible.
fn run_worker(
    worker_id: u32,
    task_sender: Sender<Arc<Mutex<Task>>>,
    task_receiver: Receiver<Arc<Mutex<Task>>>,
    task_finished: &Condvar,
    build_cache: Arc<BuildCache>,
) {
    if let Err(error) = run_worker_internal(
        worker_id,
        task_sender,
        task_receiver,
        task_finished,
        build_cache,
    ) {
        println!("[worker {worker_id}] {error}");
        exit(2);
    }
}

/// The full implementation of [run_worker]. The function is divided into two
/// parts so that this internal part can use the ? syntax for returning an io:Error.
fn run_worker_internal(
    worker_id: u32,
    task_sender: Sender<Arc<Mutex<Task>>>,
    task_receiver: Receiver<Arc<Mutex<Task>>>,
    task_finished: &Condvar,
    build_cache: Arc<BuildCache>,
) -> Result<(), io::Error> {
    while let Ok(task) = task_receiver.recv() {
        let mut task = task.lock().unwrap();

        // Check the build cache
        if build_cache.retrieve_outputs(&task.rule)? {
            println!(
                "[worker {worker_id}] Retrieved outputs of {} from cache",
                task.rule.name
            );
        } else {
            // The rule needs its commands run
            build_rule(worker_id, &task.rule);
            build_cache.insert_outputs(&task.rule)?;
        }

        task.build_finished();
        task_finished.notify_all();
        update_waiting_tasks(&task, &task_sender);
    }

    Ok(())
}

/// Update tasks that are waiting on the given task to tell them
/// that the build of this task has finished
fn update_waiting_tasks(task: &Task, task_sender: &Sender<Arc<Mutex<Task>>>) {
    for used_by in &task.used_by {
        let mut used_by_locked = used_by.lock().unwrap();
        if used_by_locked.dependency_finished() == 0 {
            task_sender.send(used_by.clone()).unwrap();
        }
    }
}
