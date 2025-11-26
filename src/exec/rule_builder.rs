use std::env;
use std::process::{Command, exit};

use crate::ast::hexmake_file::HexRule;
use crate::exec::work_dir::WorkDirManager;

/// Build the given rule right now. Assume that all of its
/// dependencies have been built and are available in `out`.
/// Exit if the build command fails.
pub fn build_rule(worker_id: u32, rule: &HexRule, work_dir: &WorkDirManager) {
    if let Err(error) = build_rule_internal(worker_id, rule, work_dir) {
        println!("{}", error);
        exit(2);
    }
}

fn build_rule_internal(
    worker_id: u32,
    rule: &HexRule,
    work_dir: &WorkDirManager,
) -> std::io::Result<()> {
    // Clean the work directory for this build
    work_dir.clean()?;

    // Create the work directory
    work_dir.create_root()?;

    // Copy input files into the work directory
    work_dir.copy_inputs(&rule.inputs)?;

    // Prepare output directories in the work directory
    work_dir.prepare_output_directories(&rule.outputs)?;

    // Run the build commands in the work directory
    let shell = env::var("SHELL").unwrap_or("sh".to_string());

    for command in &rule.commands {
        println!("[worker {worker_id}] {}", command);

        let status = Command::new(&shell)
            .arg("-c")
            .arg(command)
            .current_dir(work_dir.root())
            .status()?;

        if !status.success() {
            // Leave the work directory intact for inspection on failure
            exit(1);
        }
    }

    // Copy output files back to the main workspace
    work_dir.copy_outputs(&rule.outputs)?;

    // Clean up the work directory after successful build
    work_dir.clean()?;

    Ok(())
}
