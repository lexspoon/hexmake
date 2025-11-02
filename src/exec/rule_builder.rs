use std::env;
use std::process::{Command, exit};

use crate::ast::hexmake_file::HexRule;

/// Build the given rule right now. Assume that all of its
/// dependencies have been built and are available in `out`.
/// Exit if the build command fails.
pub fn build_rule(worker_id: u32, rule: &HexRule) {
    if let Err(error) = build_rule_internal(worker_id, rule) {
        println!("{}", error);
        exit(2);
    }
}

fn build_rule_internal(worker_id: u32, rule: &HexRule) -> std::io::Result<()> {
    let shell = env::var("SHELL").unwrap_or("sh".to_string());

    for command in &rule.commands {
        println!("[worker {worker_id}] {}", command);

        let status = Command::new(&shell).arg("-c").arg(command).status()?;

        if !status.success() {
            exit(1);
        }
    }

    Ok(())
}
