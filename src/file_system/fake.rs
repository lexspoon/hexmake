#![cfg(test)]

use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::BTreeMap, io};

use crate::ast::hex_path::HexPath;
use crate::file_system::vfs::VirtualFileSystem;

#[derive(Default)]
pub struct FakeFileSystem {
    state: RefCell<State>,
}

#[derive(Default)]
struct State {
    files: BTreeMap<HexPath, Rc<RefCell<FakeFile>>>,
    clock: u64,
}

impl Clone for FakeFileSystem {
    /// Make a separate, independent clone of this file system.
    /// This is manually implement because cloning an Arc will
    /// just increase a reference count.
    fn clone(&self) -> Self {
        let old_state = self.state.borrow();
        let clock = old_state.clock;
        let mut files = BTreeMap::new();
        for (path, file) in &old_state.files {
            files.insert(path.clone(), Rc::new(RefCell::new(file.borrow().clone())));
        }

        let new_state = State { clock, files };

        Self {
            state: RefCell::new(new_state),
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
        let state = self.state.borrow();
        Ok(state.files.contains_key(path))
    }

    fn remove_file(&self, path: &HexPath) -> Result<(), io::Error> {
        let mut state = self.state.borrow_mut();
        state.files.remove(path);
        Ok(())
    }

    fn list_dir(&self, path: &HexPath) -> Result<Vec<HexPath>, io::Error> {
        let state = self.state.borrow();

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

        Ok(file.borrow().modtime)
    }

    fn read(&self, path: &HexPath) -> Result<Vec<u8>, io::Error> {
        let file = self.get_file(path)?;
        Ok(file.borrow().contents.clone())
    }

    fn rename(&self, old_path: &HexPath, new_path: &HexPath) -> Result<(), io::Error> {
        let mut state = self.state.borrow_mut();

        let file = state
            .files
            .remove(old_path)
            .ok_or_else(|| file_not_found(old_path))?;

        state.files.insert(new_path.clone(), file);

        Ok(())
    }

    fn touch(&self, path: &HexPath) -> Result<(), io::Error> {
        let mut state = self.state.borrow_mut();
        let clock = state.clock;

        state
            .files
            .entry(path.clone())
            .and_modify(|file| file.borrow_mut().modtime = clock)
            .or_insert_with(|| {
                Rc::new(RefCell::new(FakeFile {
                    contents: Vec::new(),
                    modtime: clock,
                }))
            });

        state.clock += 1;

        Ok(())
    }

    fn write(&self, path: &HexPath, contents: &[u8]) -> Result<(), io::Error> {
        let mut state = self.state.borrow_mut();

        let modtime = state.clock;
        state.files.insert(
            path.clone(),
            Rc::new(RefCell::new(FakeFile {
                contents: contents.to_vec(),
                modtime,
            })),
        );

        state.clock += 1;

        Ok(())
    }
}

impl FakeFileSystem {
    /// Look up a file entry. Return an appropriate error
    fn get_file(&self, path: &HexPath) -> Result<Rc<RefCell<FakeFile>>, io::Error> {
        let state = self.state.borrow();

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
