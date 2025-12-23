mod args;
mod ast;
mod cache;
mod check;
mod error;
mod error_exit;
mod exec;
mod file_system;
mod graph;

use clap::Parser;
use std::collections::BTreeMap;
use std::env;
use std::fs::read_to_string;
use std::process::exit;
use std::sync::Arc;

use crate::args::Args;
use crate::ast::hexmake_file::HexmakeFile;
use crate::cache::build_cache::BuildCache;
use crate::check::file::check_file;
use crate::error::Error;
use crate::error_exit::error_exit;
use crate::exec::conductor::conduct_build;
use crate::file_system::posix::PosixFileSystem;
use crate::graph::planner::plan_build;

fn main() {
    if let Err(error) = main_internal() {
        error_exit!("Error: {}", error);
    }
}

fn main_internal() -> Result<(), Error> {
    let args: Args = Args::parse();
    let hexmake_file: HexmakeFile = load_hexmake_file();
    check_file(&hexmake_file)?;

    if args.list_targets {
        list_targets(&hexmake_file);
    }

    let plan = plan_build(&hexmake_file, &args.targets)?;
    let env = get_environment(&hexmake_file);

    let vfs = Box::new(PosixFileSystem::default());
    let build_cache = Arc::new(BuildCache::new(env, vfs)?);

    Ok(conduct_build(&plan, &build_cache)?)
}

/// Load and parse the Hexmake file
fn load_hexmake_file() -> HexmakeFile {
    let hexmake_source = match read_to_string("Hexmake") {
        Ok(source) => source,
        Err(error) => {
            error_exit!("Could not open Hexmake file: {}", error)
        }
    };

    let hexmake_file: HexmakeFile = match serde_json::from_str(&hexmake_source) {
        Ok(hexmake_file) => hexmake_file,
        Err(error) => error_exit!("Could not parse Hexmake file: {}", error),
    };
    hexmake_file
}

/// List available targets and then exit
fn list_targets(hexmake_file: &HexmakeFile) -> ! {
    let mut targets: Vec<String> = Vec::new();
    for rule in &hexmake_file.rules {
        targets.push(rule.name.to_string());
        for output in &rule.outputs {
            targets.push(output.to_string());
        }
    }
    targets.sort();
    for target in targets {
        println!("{}", target);
    }

    exit(0)
}

/// Make a map of the environment variables that should be passed through
fn get_environment(hexmake_file: &HexmakeFile) -> Arc<BTreeMap<Arc<String>, Arc<String>>> {
    let mut result = BTreeMap::new();

    for variable in &hexmake_file.environ {
        if let Ok(value) = env::var(variable.as_str()) {
            result.insert(variable.clone(), Arc::new(value));
        }
    }

    Arc::new(result)
}
