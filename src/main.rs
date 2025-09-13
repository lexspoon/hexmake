mod ast;
mod error_exit;
mod exec;
mod graph;

use std::env;
use std::fs::read_to_string;
use std::rc::Rc;

use crate::ast::hexmake_file::HexmakeFile;
use crate::error_exit::error_exit;
use crate::exec::conductor::conduct_build;
use crate::graph::planner::plan_build;

fn main() {
    let hexmake_file = load_hexmake_file();
    let targets = parse_arguments();

    let plan = plan_build(&hexmake_file, &targets);

    conduct_build(&plan);
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
