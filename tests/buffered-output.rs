use assert_cmd::Command;
use assert_cmd::cargo_bin;
use indoc::indoc;
use pretty_assertions::assert_eq;
use regex::Regex;
use std::fs;

#[test]
fn test_buffered_output_no_interleaving() {
    // Clear the output directory and cache
    let _ = fs::remove_dir_all("integration-tests/buffered-output/out");
    let _ = fs::remove_dir_all("integration-tests/buffered-output/.hex");

    // Run hexmake to build all three rules in parallel.
    let result = hexmake_command()
        .in_test_dir()
        .arg("rule1")
        .arg("rule2")
        .arg("rule3")
        .assert()
        .success()
        .stderr("");

    let output = result.get_output();

    // The output for each command should be grouped up. For example,
    // rule1-line1 through rule1-line3 should be together in the output.
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let stdout = Regex::new(".rule.. Running: sleep.*")
        .unwrap()
        .replace_all(&stdout, "[ruleN] Running: sleep ...")
        .to_string();
    assert_eq!(
        stdout,
        indoc! {r"
                [ruleN] Running: sleep ...
                [ruleN] Running: sleep ...
                [ruleN] Running: sleep ...
                [rule1] rule1-line1
                [rule1] rule1-line2
                [rule1] rule1-line3
                [rule1] Running: touch out/rule1.txt
                [rule2] rule2-line1
                [rule2] rule2-line2
                [rule2] rule2-line3
                [rule2] Running: touch out/rule2.txt
                [rule3] rule3-line1
                [rule3] rule3-line2
                [rule3] rule3-line3
                [rule3] Running: touch out/rule3.txt
        "},
    );

    // Verify output files were created
    assert!(fs::metadata("integration-tests/buffered-output/out/rule1.txt").is_ok());
    assert!(fs::metadata("integration-tests/buffered-output/out/rule2.txt").is_ok());
    assert!(fs::metadata("integration-tests/buffered-output/out/rule3.txt").is_ok());
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
        self.current_dir("integration-tests/buffered-output")
    }
}
