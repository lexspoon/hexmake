mod ast;
mod cache;
mod error_exit;
mod exec;
mod file_system;
mod graph;

use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::rc::Rc;
use std::{env, io};

use crate::ast::hexmake_file::HexmakeFile;
use crate::cache::build_cache::BuildCache;
use crate::error_exit::error_exit;
use crate::exec::conductor::conduct_build;
use crate::file_system::posix::PosixFileSystem;
use crate::graph::planner::plan_build;

fn main() {
    if let Err(error) = main_internal() {
        error_exit!("Error: {}", error);
    }
}

fn main_internal() -> Result<(), io::Error> {
    let hexmake_file = load_hexmake_file();
    let targets = parse_arguments();

    let plan = plan_build(&hexmake_file, &targets);
    let env = get_environment(&hexmake_file);

    let mut build_cache = BuildCache::new(PosixFileSystem::default(), env)?;

    conduct_build(&plan, &mut build_cache)?;

    Ok(())
}

/// Parse the command line arguments
fn parse_arguments() -> Vec<Rc<String>> {
    let result: Vec<Rc<String>> = env::args().skip(1).map(Rc::new).collect();

    if result.is_empty() {
        usage_exit();
    }

    result
}

fn usage_exit() -> ! {
    error_exit!("Usage: hexmake target...\nAt least one target must be supplied.");
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

/// Make a map of the environment variables that should be passed through
fn get_environment(hexmake_file: &HexmakeFile) -> Rc<BTreeMap<Rc<String>, Rc<String>>> {
    let mut result = BTreeMap::new();

    for variable in &hexmake_file.environ {
        if let Ok(value) = env::var(variable.as_str()) {
            result.insert(variable.clone(), Rc::new(value));
        }
    }

    Rc::new(result)
}
