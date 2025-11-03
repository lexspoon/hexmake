use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::str::is_match;

#[test]
fn test_basics() {
    // Clear the output directory and cache
    let _ = std::fs::remove_dir_all("examples/c-basic/out");
    let _ = std::fs::remove_dir_all("examples/c-basic/.hex");

    // Build the main routine
    hexmake_command()
        .in_test_dir()
        .arg("main")
        .assert()
        .success()
        .stdout(is_match(".worker .. cc -o out/main out/lib.o out/main.o").unwrap());

    // Run main
    main_command()
        .in_test_dir()
        .arg("3")
        .arg("4")
        .assert()
        .success()
        .stdout("Sum: 7\n");

    // Rebuild it; it should use the cache
    hexmake_command()
        .in_test_dir()
        .arg("main")
        .assert()
        .success()
        .stdout(is_match(".worker .. Retrieved outputs of main from cache").unwrap());

    // Run it again
    main_command()
        .in_test_dir()
        .arg("3")
        .arg("4")
        .assert()
        .success()
        .stdout("Sum: 7\n");
}

/// A command for running `hexmake`
fn hexmake_command() -> Command {
    Command::new(cargo_bin!())
}

/// A command for running the main program that is built in this example
fn main_command() -> Command {
    Command::new("out/main")
}

/// Extensions to Command for this test
trait CommandExt {
    /// Set the current directory to the one for this example
    fn in_test_dir(&mut self) -> &mut Self;
}

impl CommandExt for Command {
    fn in_test_dir(&mut self) -> &mut Self {
        self.current_dir("examples/c-basic")
    }
}
