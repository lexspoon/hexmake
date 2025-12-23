use assert_cmd::Command;
use assert_cmd::cargo_bin;
use indoc::indoc;
use predicates::ord::eq;
use predicates::str::is_match;

#[test]
fn test_version() {
    hexmake_command()
        .in_test_dir()
        .arg("--version")
        .assert()
        .success()
        .stdout(is_match("^hexmake [0-9.]+\n$").unwrap());
}

#[test]
fn test_help() {
    hexmake_command()
        .in_test_dir()
        .arg("--help")
        .assert()
        .success()
        .stdout(eq(LONG_HELP_STRING));
}

#[test]
fn test_no_args() {
    hexmake_command()
        .in_test_dir()
        .assert()
        .code(2)
        .stderr(eq(SHORT_HELP_STRING));
}

#[test]
fn test_list_targets() {
    hexmake_command()
        .in_test_dir()
        .arg("--list-targets")
        .assert()
        .success()
        .stdout(
            is_match(indoc! {r#"
                lib.o
                main
                main.o
                out/lib.o
                out/main
                out/main.o
            "#})
            .unwrap(),
        );
}

/// A command for running `hexmake`
fn hexmake_command() -> Command {
    Command::new(cargo_bin!())
}

/// Extensions to Command for this test
trait CommandExt {
    /// Set the current directory to the one for this example
    fn in_test_dir(&mut self) -> &mut Self;
}

impl CommandExt for Command {
    fn in_test_dir(&mut self) -> &mut Self {
        self.current_dir("integration-tests/args")
    }
}

/// The current help string the tool outputs
const LONG_HELP_STRING: &str = r#"Hexmake runs a multi-step build using caching. You give it a file describing all
the possible build steps along with their inputs and outputs. The tool will then
chain them together to produce an output, use cached results from prior builds
when possible.


Usage: hexmake [OPTIONS] [TARGETS]...

Arguments:
  [TARGETS]...
          The rules or output files to build

Options:
      --list-targets
          List available targets and exit

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

The tool expects a Hexmake file to exist in the current directory.
A Hexmake file looks like this:

```json
{
  "rules": [
    {
      "name": "main",
      "inputs": [
        "main.c",
      ],
      "outputs": [
        "out/main"
      ],
      "commands": [
        "cc -o out/main main.c"
      ]
    }
  ]
}
```
"#;

/// The current short help string the tool outputs
const SHORT_HELP_STRING: &str = r#"Run a multi-step build with caching

Usage: hexmake [OPTIONS] [TARGETS]...

Arguments:
  [TARGETS]...  The rules or output files to build

Options:
      --list-targets  List available targets and exit
  -h, --help          Print help (see more with '--help')
  -V, --version       Print version
"#;
