use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

use serde::Deserialize;

/// A path that can be built and/or used as source code.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct HexPath {
    pub path: Arc<String>,
}

impl HexPath {
    pub fn new(path: Arc<String>) -> HexPath {
        HexPath { path }
    }

    #[allow(dead_code)]
    pub fn is_output(&self) -> bool {
        self.path.starts_with("out/")
    }

    /// Generate a path by appending a child path
    #[allow(dead_code)]
    pub fn child(&self, child_path: &str) -> HexPath {
        HexPath::from(format!("{}/{}", self.path, child_path))
    }

    pub fn relative_to(&self, root: &HexPath) -> Result<HexPath, String> {
        if self.path.starts_with(root.path.as_str()) {
            return Ok(HexPath::from(&self.path[root.path.len()..]));
        }
        Err(format!(
            "Could not find a relative path from {root} to {self}"
        ))
    }
}

impl From<&str> for HexPath {
    fn from(path: &str) -> Self {
        HexPath::new(Arc::new(path.to_string()))
    }
}

impl From<String> for HexPath {
    fn from(path: String) -> Self {
        HexPath::new(Arc::new(path))
    }
}

impl From<&String> for HexPath {
    fn from(path: &String) -> Self {
        HexPath::new(Arc::new(path.to_string()))
    }
}

impl From<&Arc<String>> for HexPath {
    fn from(path: &Arc<String>) -> Self {
        HexPath::new(path.clone())
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
        assert!(HexPath::from("out/foo.o").is_output());

        assert!(!HexPath::from("foo.c").is_output());
        assert!(!HexPath::from("src/foo.c").is_output());
        assert!(!HexPath::from("output.c").is_output());
    }
}
