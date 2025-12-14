use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::str::is_match;

#[test]
fn test_basics() {
    // Clear the output directory and cache
    let _ = std::fs::remove_dir_all("integration-tests/output-not-in-out/out");
    let _ = std::fs::remove_dir_all("integration-tests/output-not-in-out/.hex");

    // Try to build; it should error out
    hexmake_command()
        .in_test_dir()
        .arg("main")
        .assert()
        .failure()
        .stdout(is_match("Error: Output `lib.o` is not in `out/`").unwrap());
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
        self.current_dir("integration-tests/output-not-in-out")
    }
}
