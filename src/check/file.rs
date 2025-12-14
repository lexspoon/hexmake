use crate::ast::hexmake_file::HexmakeFile;

/// Check that a Hexmake file is valid
pub fn check_file(hexmake_file: &HexmakeFile) -> Result<(), String> {
    for rule in &hexmake_file.rules {
        for output in &rule.outputs {
            if !output.starts_with("out/") {
                return Err(format!("Output `{}` is not in `out/`", output));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_file() {
        // Valid file
        let hexmake_file = serde_json::from_str(
            r#"{
                "environ": [],
                "rules": [
                    {
                        "name": "foo",
                        "outputs": ["out/foo"],
                        "inputs": [],
                        "commands": ["touch out/foo"]
                    }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(check_file(&hexmake_file), Ok(()));

        // Output that is not in out/
        let hexmake_file = serde_json::from_str(
            r#"{
                "environ": [],
                "rules": [
                    {
                        "name": "foo",
                        "outputs": ["target/foo"],
                        "inputs": [],
                        "commands": ["touch target/foo"]
                    }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(
            check_file(&hexmake_file),
            Err("Output `target/foo` is not in `out/`".to_string())
        );
    }
}
