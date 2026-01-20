use std::process::{Command, Stdio};
use std::{env, io};

use crate::ast::hexmake_file::HexRule;
use crate::exec::work_dir::WorkDirManager;

/// Build the given rule right now. Assume that all of its
/// dependencies have been built and are available in `out`.
pub fn build_rule(worker_id: u32, rule: &HexRule, work_dir: &WorkDirManager) -> io::Result<()> {
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
        println!("[worker {worker_id}] Running: {}", command);

        // Spawn the command and buffer its output
        let output = Command::new(&shell)
            .arg("-c")
            .arg(command)
            .current_dir(work_dir.root())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

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

        if !output.status.success() {
            // Leave the work directory intact for inspection on failure
            return Err(io::Error::other(format!("Command failed: {command}")));
        }
    }

    // Copy output files back to the main workspace
    work_dir.copy_outputs(&rule.outputs)?;

    // Clean up the work directory after successful build
    work_dir.clean()?;

    Ok(())
}
