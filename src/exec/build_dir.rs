#![allow(dead_code)]

use std::fs::{create_dir_all, remove_dir_all};
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, atomic};

/// A utility for allocating build directories
pub struct BuildDirManager {
    root: Arc<String>,
    build_dirs_made: Arc<AtomicU32>,
}

impl BuildDirManager {
    pub fn new(root: Arc<String>) -> BuildDirManager {
        BuildDirManager {
            root,
            build_dirs_made: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Delete all build directories, including ones
    /// made by prior runs of the tool.
    pub fn clean(&self) -> io::Result<()> {
        remove_dir_all(Path::new(self.root.as_str()))
    }

    /// Make a new build directory. Return the path to it.
    pub fn make_build_dir(&self) -> io::Result<Arc<String>> {
        let build_dir_num = self.build_dirs_made.fetch_add(1, atomic::Ordering::SeqCst);
        let path = Path::new(&format!("{}/.hex/build{}", self.root, build_dir_num)).to_owned();
        create_dir_all(&path)?;
        Ok(Arc::new(path.to_string_lossy().to_string()))
    }
}
