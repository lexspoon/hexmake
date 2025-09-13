use std::env;
use std::fs::create_dir_all;
use std::process::{Command, exit};

use crate::ast::hexmake_file::HexRule;

/// Build the given rule right now. Assume that all of its
/// dependencies have been built and are available in `out`.
pub fn build_rule(rule: &HexRule) {
    match build_rule_internal(rule) {
        Err(error) => {
            println!("{}", error);
            exit(2);
        }
        Ok(_) => (),
    }
}

fn build_rule_internal(rule: &HexRule) -> std::io::Result<()> {
    let shell = env::var("SHELL").unwrap_or("sh".to_string());
    create_dir_all("out")?;

    for command in &rule.commands {
        println!("{}", command);

        let status = Command::new(&shell).arg("-c").arg(command).status()?;

        if !status.success() {
            exit(1);
        }
    }

    Ok(())
}
