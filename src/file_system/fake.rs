#![cfg(test)]

use std::{collections::BTreeMap, io};

use crate::ast::hex_path::HexPath;
use crate::file_system::vfs::VirtualFileSystem;

#[derive(Clone, Default)]
pub struct FakeFileSystem {
    files: BTreeMap<HexPath, FakeFile>,
    clock: u64,
}

impl VirtualFileSystem for FakeFileSystem {
    fn copy(&mut self, source: &HexPath, destination: &HexPath) -> Result<(), io::Error> {
        let contents = self.read(source)?;
        self.write(destination, &contents)?;
        Ok(())
    }

    fn create_dir_all(&mut self, _path: &HexPath) -> Result<(), io::Error> {
        // Nothing to do, for the fake file system
        Ok(())
    }

    fn is_file(&self, path: &HexPath) -> Result<bool, io::Error> {
        Ok(self.files.contains_key(path))
    }

    fn remove_file(&mut self, path: &HexPath) -> Result<(), io::Error> {
        self.files.remove(path);
        Ok(())
    }

    fn list_dir(&self, path: &HexPath) -> Result<Vec<HexPath>, io::Error> {
        let prefix = format!("{}/", path);

        let mut result = Vec::new();
        for file in self.files.keys() {
            if file.starts_with(&prefix) {
                result.push(file.clone());
            }
        }

        Ok(result)
    }

    fn modtime(&self, path: &HexPath) -> Result<u64, io::Error> {
        let file = self.get_file(path)?;

        Ok(file.modtime)
    }

    fn read(&self, path: &HexPath) -> Result<Vec<u8>, io::Error> {
        let file = self.get_file(path)?;
        Ok(file.contents.clone())
    }

    fn rename(&mut self, old_path: &HexPath, new_path: &HexPath) -> Result<(), io::Error> {
        let file = self
            .files
            .remove(old_path)
            .ok_or_else(|| file_not_found(old_path))?;

        self.files.insert(new_path.clone(), file);

        Ok(())
    }

    fn touch(&mut self, path: &HexPath) -> Result<(), io::Error> {
        self.files
            .entry(path.clone())
            .and_modify(|file| file.modtime = self.clock)
            .or_insert_with(|| FakeFile {
                contents: Vec::new(),
                modtime: self.clock,
            });

        self.clock += 1;

        Ok(())
    }

    fn write(&mut self, path: &HexPath, contents: &[u8]) -> Result<(), io::Error> {
        self.files.insert(
            path.clone(),
            FakeFile {
                contents: contents.to_vec(),
                modtime: self.clock,
            },
        );

        self.clock += 1;

        Ok(())
    }
}

impl FakeFileSystem {
    /// Look up a file entry. Return an appropriate error
    fn get_file(&self, path: &HexPath) -> Result<&FakeFile, io::Error> {
        self.files.get(path).ok_or_else(|| file_not_found(path))
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
