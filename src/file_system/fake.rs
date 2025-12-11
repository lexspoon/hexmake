#![cfg(test)]

use std::sync::{Arc, Mutex};
use std::{collections::BTreeMap, io};

use crate::ast::hex_path::HexPath;
use crate::file_system::vfs::VirtualFileSystem;

#[derive(Default)]
pub struct FakeFileSystem {
    state: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    files: BTreeMap<HexPath, Arc<Mutex<FakeFile>>>,
    clock: u64,
}

impl Clone for FakeFileSystem {
    /// Make a separate, independent clone of this file system.
    /// This is manually implement because cloning an Arc will
    /// just increase a reference count.
    fn clone(&self) -> Self {
        let old_state = self.state.lock().unwrap();
        let clock = old_state.clock;
        let mut files = BTreeMap::new();
        for (path, file) in &old_state.files {
            files.insert(
                path.clone(),
                Arc::new(Mutex::new(file.lock().unwrap().clone())),
            );
        }

        let new_state = State { clock, files };

        Self {
            state: Arc::new(Mutex::new(new_state)),
        }
    }
}

impl VirtualFileSystem for FakeFileSystem {
    fn copy(&self, source: &HexPath, destination: &HexPath) -> Result<(), io::Error> {
        let contents = self.read(source)?;
        self.write(destination, &contents)?;
        Ok(())
    }

    fn create_dir_all(&self, _path: &HexPath) -> Result<(), io::Error> {
        // Nothing to do, for the fake file system
        Ok(())
    }

    fn is_file(&self, path: &HexPath) -> Result<bool, io::Error> {
        let state = self.state.lock().unwrap();
        Ok(state.files.contains_key(path))
    }

    fn remove_file(&self, path: &HexPath) -> Result<(), io::Error> {
        let mut state = self.state.lock().unwrap();
        state.files.remove(path);
        Ok(())
    }

    fn list_dir(&self, path: &HexPath) -> Result<Vec<HexPath>, io::Error> {
        let state = self.state.lock().unwrap();

        let prefix = format!("{}/", path);

        let mut result = Vec::new();
        for file in state.files.keys() {
            if file.starts_with(&prefix) {
                result.push(file.clone());
            }
        }

        Ok(result)
    }

    fn modtime(&self, path: &HexPath) -> Result<u64, io::Error> {
        let file = self.get_file(path)?;

        Ok(file.lock().unwrap().modtime)
    }

    fn read(&self, path: &HexPath) -> Result<Vec<u8>, io::Error> {
        let file = self.get_file(path)?;
        Ok(file.lock().unwrap().contents.clone())
    }

    fn rename(&self, old_path: &HexPath, new_path: &HexPath) -> Result<(), io::Error> {
        let mut state = self.state.lock().unwrap();

        let file = state
            .files
            .remove(old_path)
            .ok_or_else(|| file_not_found(old_path))?;

        state.files.insert(new_path.clone(), file);

        Ok(())
    }

    fn touch(&self, path: &HexPath) -> Result<(), io::Error> {
        let mut state = self.state.lock().unwrap();
        let clock = state.clock;

        state
            .files
            .entry(path.clone())
            .and_modify(|file| file.lock().unwrap().modtime = clock)
            .or_insert_with(|| {
                Arc::new(Mutex::new(FakeFile {
                    contents: Vec::new(),
                    modtime: clock,
                }))
            });

        state.clock += 1;

        Ok(())
    }

    fn write(&self, path: &HexPath, contents: &[u8]) -> Result<(), io::Error> {
        let mut state = self.state.lock().unwrap();

        let modtime = state.clock;
        state.files.insert(
            path.clone(),
            Arc::new(Mutex::new(FakeFile {
                contents: contents.to_vec(),
                modtime,
            })),
        );

        state.clock += 1;

        Ok(())
    }

    fn tree_walk(&self, path: &HexPath) -> Result<Vec<HexPath>, io::Error> {
        let state = self.state.lock().unwrap();
        let mut result = Vec::new();

        // If the path itself is a file, return just that file
        if state.files.contains_key(path) {
            result.push(path.clone());
            return Ok(result);
        }

        // Otherwise, walk all files under this directory
        let prefix = format!("{}/", path);
        for file_path in state.files.keys() {
            if file_path.starts_with(&prefix) {
                result.push(file_path.clone());
            }
        }

        Ok(result)
    }

    fn exists(&self, path: &HexPath) -> Result<bool, io::Error> {
        self.is_file(path)
    }
}

impl FakeFileSystem {
    /// Look up a file entry. Return an appropriate error
    fn get_file(&self, path: &HexPath) -> Result<Arc<Mutex<FakeFile>>, io::Error> {
        let state = self.state.lock().unwrap();

        state
            .files
            .get(path)
            .cloned()
            .ok_or_else(|| file_not_found(path))
    }
}

/// Construct an IO error corresponding to a file not existing
fn file_not_found(path: &HexPath) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("File not found: {}", path))
}

/// A file that lives in memory and can be used for testing.
#[derive(Clone, Default)]
struct FakeFile {
    contents: Vec<u8>,
    #[allow(unused)]
    modtime: u64,
}
