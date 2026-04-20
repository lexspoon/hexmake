use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::str::is_match;
use std::thread;

/// Test that Hexmake will refuse to run twice at the same time
/// for a given .hex directory.
#[test]
fn test_global_lock_file() {
    // Clear the output directory and cache
    let _ = fs_err::remove_dir_all("integration-tests/lock/out");
    let _ = fs_err::remove_dir_all("integration-tests/lock/.hex");

    // Start a first hexmake instance in a background thread; it will sleep for 2 seconds.
    let first = thread::spawn(|| {
        hexmake_command()
            .in_test_dir()
            .arg("main")
            .assert()
            .success()
    });

    // Give the first instance time to acquire the lock before we try the second.
    thread::sleep(std::time::Duration::from_millis(500));

    // The second instance should fail with a lock error.
    hexmake_command()
        .in_test_dir()
        .arg("main")
        .assert()
        .failure()
        .stdout(
            is_match("Another instance of Hexmake is already running for this directory").unwrap(),
        );

    first.join().unwrap();
}

/// A command for running `hexmake`
fn hexmake_command() -> Command {
    Command::new(cargo_bin!())
}

/// Extensions to Command for this test
trait CommandExt {
    /// Set the current directory to the one for this test
    fn in_test_dir(&mut self) -> &mut Self;
}

impl CommandExt for Command {
    fn in_test_dir(&mut self) -> &mut Self {
        self.current_dir("integration-tests/lock")
    }
}
