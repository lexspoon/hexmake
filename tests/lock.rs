use assert_cmd::Command;
use assert_cmd::cargo_bin;
use std::thread::spawn;

/// Test that Hexmake will refuse to run twice at the same time
/// for a given .hex directory.
#[test]
fn test_global_lock_file() {
    // Clear the output directory and cache
    let _ = fs_err::remove_dir_all("integration-tests/lock/out");
    let _ = fs_err::remove_dir_all("integration-tests/lock/.hex");

    // Run the build twice, in two background threads

    let thread1 = spawn(|| {
        hexmake_command()
            .in_test_dir()
            .arg("main")
            .assert()
            .success()
    });
    let thread2 = spawn(|| {
        hexmake_command()
            .in_test_dir()
            .arg("main")
            .assert()
            .success()
    });

    // Wait on both of them
    thread1.join().unwrap();
    thread2.join().unwrap();
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
