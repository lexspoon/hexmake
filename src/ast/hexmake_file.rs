use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use crate::ast::hex_path::HexPath;
use serde::Deserialize;

/// An entire Hexmake file
#[derive(Debug, Deserialize, PartialEq)]
pub struct HexmakeFile {
    #[serde(default)]
    pub environ: Vec<Arc<String>>,
    pub rules: Vec<Arc<HexRule>>,
}

impl Display for HexmakeFile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Hexmake file with {} rules", self.rules.len())
    }
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq)]
/// One rule in a Hexmake file
pub struct HexRule {
    pub name: RuleName,
    pub outputs: Vec<HexPath>,
    pub inputs: Vec<HexPath>,
    pub commands: Vec<String>,
}

impl HexRule {
    #[cfg(test)]
    pub fn new(name: RuleName) -> HexRule {
        HexRule {
            name,
            outputs: vec![],
            inputs: vec![],
            commands: vec![],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct RuleName {
    pub name: Arc<String>,
}

impl Display for RuleName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<String> for RuleName {
    fn from(name: String) -> Self {
        RuleName {
            name: Arc::new(name),
        }
    }
}

impl From<&Arc<String>> for RuleName {
    fn from(name: &Arc<String>) -> Self {
        RuleName { name: name.clone() }
    }
}

impl From<&str> for RuleName {
    fn from(name: &str) -> Self {
        RuleName {
            name: Arc::new(name.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_parse() {
        let input = indoc! {r###"
            {
                "rules": [
                  {
                    "name": "out/lib.o",
                    "outputs": [
                      "out/lib.o"
                    ],
                    "inputs": [
                      "lib.c",
                      "lib.h"
                    ],
                    "commands": [
                      "gcc -o out/lib.o -c lib.c"
                    ]
                  },
                  {
                    "name": "out/main.o",
                    "outputs": [
                      "out/main.o"
                    ],
                    "inputs": [
                      "lib.h",
                      "main.c"
                    ],
                    "commands": [
                      "gcc -o out/main.o -c main.c"
                    ]
                  },
                  {
                    "name": "out/main",
                    "outputs": [
                      "out/main"
                    ],
                    "inputs": [
                      "out/lib.o",
                      "out/main.o"
                    ],
                    "commands": [
                      "gcc -o out/main out/lib.o out/main.o"
                    ]
                  }
                ]
            }"###
        };

        let hexmake_file: HexmakeFile = serde_json::from_str(input).unwrap();

        assert_eq!(
            hexmake_file,
            HexmakeFile {
                environ: vec![],
                rules: vec![
                    HexRule {
                        name: "out/lib.o".to_string().into(),
                        outputs: vec![HexPath::try_from("out/lib.o").unwrap()],
                        inputs: vec![
                            HexPath::try_from("lib.c").unwrap(),
                            HexPath::try_from("lib.h").unwrap()
                        ],
                        commands: vec!["gcc -o out/lib.o -c lib.c".to_string()]
                    }
                    .into(),
                    HexRule {
                        name: "out/main.o".to_string().into(),
                        outputs: vec![HexPath::try_from("out/main.o").unwrap()],
                        inputs: vec![
                            HexPath::try_from("lib.h").unwrap(),
                            HexPath::try_from("main.c").unwrap()
                        ],
                        commands: vec!["gcc -o out/main.o -c main.c".to_string()]
                    }
                    .into(),
                    HexRule {
                        name: "out/main".to_string().into(),
                        outputs: vec![HexPath::try_from("out/main").unwrap()],
                        inputs: vec![
                            HexPath::try_from("out/lib.o").unwrap(),
                            HexPath::try_from("out/main.o").unwrap()
                        ],
                        commands: vec!["gcc -o out/main out/lib.o out/main.o".to_string()]
                    }
                    .into()
                ]
            }
        );
    }

    #[test]
    fn test_bad_path() {
        let input = indoc! {r###"
            {
                "rules": [
                  {
                    "name": "out/lib.o",
                    "outputs": [
                      "/out/lib.o"
                    ],
                    "inputs": [
                      "lib.c",
                      "lib.h"
                    ],
                    "commands": [
                      "gcc -o out/lib.o -c lib.c"
                    ]
                  }
                ]
            }"###
        };

        let result: serde_json::Result<HexmakeFile> = serde_json::from_str(input);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Path `/out/lib.o` starts with a slash at line 7 column 9"
        );
    }
}
