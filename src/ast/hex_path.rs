use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

use serde::Deserialize;

/// A path that can be built and/or used as source code.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(try_from = "String")]
pub struct HexPath {
    pub path: Arc<String>,
}

impl HexPath {
    fn new(path: Arc<String>) -> HexPath {
        HexPath { path }
    }

    pub fn is_output(&self) -> bool {
        self.path.starts_with("out/")
    }

    /// Generate a path by appending a child path
    pub fn child(&self, child_path: &str) -> Result<HexPath, String> {
        HexPath::try_from(format!("{}/{}", self.path, child_path))
    }
}

impl TryFrom<&str> for HexPath {
    type Error = String;

    fn try_from(path: &str) -> Result<HexPath, String> {
        // TODO(lex) test cases for these
        if path.is_empty() {
            return Err("Empty path".to_string());
        }
        if path.starts_with("/") {
            return Err(format!("Path `{path}` starts with a slash"));
        }
        if path.ends_with("/") {
            return Err(format!("Path `{path}` ends with a slash"));
        }
        if path.contains("//") {
            return Err(format!("Path `{path}` contains a double slash"));
        }
        for part in path.split("/") {
            if part == "." {
                return Err(format!("Path `{path}` contains `.` as a component"));
            }
            if part == ".." {
                return Err(format!("Path `{path}` contains `..` as a component"));
            }
        }

        Ok(HexPath::new(Arc::new(path.to_string())))
    }
}

impl TryFrom<String> for HexPath {
    type Error = String;

    fn try_from(path: String) -> Result<HexPath, String> {
        HexPath::try_from(path.as_str())
    }
}

impl TryFrom<&String> for HexPath {
    type Error = String;

    fn try_from(path: &String) -> Result<HexPath, String> {
        HexPath::try_from(path.as_str())
    }
}

impl TryFrom<&Arc<String>> for HexPath {
    type Error = String;

    fn try_from(path: &Arc<String>) -> Result<HexPath, String> {
        HexPath::try_from(path.as_str())
    }
}

impl Display for HexPath {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl Deref for HexPath {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl AsRef<Path> for &HexPath {
    fn as_ref(&self) -> &Path {
        Path::new(&*self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_output_paths() {
        assert!(HexPath::try_from("out/foo.o").unwrap().is_output());

        assert!(!HexPath::try_from("foo.c").unwrap().is_output());
        assert!(!HexPath::try_from("src/foo.c").unwrap().is_output());
        assert!(!HexPath::try_from("output.c").unwrap().is_output());
    }

    #[test]
    fn test_try_from() {
        // Valid paths
        assert_eq!(
            HexPath::try_from("foo"),
            Ok(HexPath::new(Arc::new("foo".to_string())))
        );
        assert_eq!(
            HexPath::try_from("foo/bar"),
            Ok(HexPath::new(Arc::new("foo/bar".to_string())))
        );
        assert_eq!(
            HexPath::try_from("foo/.bar"),
            Ok(HexPath::new(Arc::new("foo/.bar".to_string())))
        );

        // Invalid paths
        assert_eq!(HexPath::try_from("").unwrap_err(), "Empty path");
        assert_eq!(
            HexPath::try_from("/foo").unwrap_err(),
            "Path `/foo` starts with a slash"
        );
        assert_eq!(
            HexPath::try_from("foo/").unwrap_err(),
            "Path `foo/` ends with a slash"
        );
        assert_eq!(
            HexPath::try_from("foo//bar").unwrap_err(),
            "Path `foo//bar` contains a double slash"
        );
        assert_eq!(
            HexPath::try_from("foo/./bar").unwrap_err(),
            "Path `foo/./bar` contains `.` as a component"
        );
        assert_eq!(
            HexPath::try_from("foo/../bar").unwrap_err(),
            "Path `foo/../bar` contains `..` as a component"
        );
    }
}
