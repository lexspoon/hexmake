use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;

use serde::Deserialize;

/// A path that can be built and/or used as source code.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct HexPath {
    pub path: Rc<String>,
}

impl HexPath {
    pub fn new(path: Rc<String>) -> HexPath {
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
}

impl From<&str> for HexPath {
    fn from(path: &str) -> Self {
        HexPath::new(Rc::new(path.to_string()))
    }
}

impl From<String> for HexPath {
    fn from(path: String) -> Self {
        HexPath::new(Rc::new(path))
    }
}

impl From<&Rc<String>> for HexPath {
    fn from(path: &Rc<String>) -> Self {
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
