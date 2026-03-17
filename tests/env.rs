use assert_cmd::Command;
use assert_cmd::cargo_bin;
use std::fs::read_to_string;
use std::fs::remove_dir_all;

#[test]
fn test_env_is_limited_to_declared_vars() {
    // Clear the output directory and cache
    let _ = remove_dir_all("integration-tests/env/out");
    let _ = remove_dir_all("integration-tests/env/.hex");

    // Build the env-output rule with a known variable set
    hexmake_command()
        .in_test_dir()
        .env("HEXMAKE_TEST_VAR", "hello-from-test")
        .env("HEXMAKE_TEST_OTHER", "should-be-ignored")
        .arg("env-output")
        .assert()
        .success();

    // Read the env output written by the build command
    let env_output =
        read_to_string("integration-tests/env/out/env.txt").expect("env.txt not found");

    // The declared variable must be present
    assert!(
        env_output.contains("HEXMAKE_TEST_VAR=hello-from-test"),
        "expected HEXMAKE_TEST_VAR in env output, got:\n{env_output}"
    );

    // Variables not declared in the Hexmake file must not leak through
    assert!(
        !env_output.contains("HEXMAKE_TEST_OTHER="),
        "HEXMAKE_TEST_OTHER should not be passed to build commands, but was found in:\n{env_output}"
    );
}

/// A command for running `hexmake`
fn hexmake_command() -> Command {
    Command::new(cargo_bin!())
}

/// Extensions to Command for this test
trait CommandExt {
    fn in_test_dir(&mut self) -> &mut Self;
}

impl CommandExt for Command {
    fn in_test_dir(&mut self) -> &mut Self {
        self.current_dir("integration-tests/env")
    }
}
