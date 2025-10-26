use std::{
    fs::{self, OpenOptions},
    io,
    time::UNIX_EPOCH,
};

use crate::{ast::hex_path::HexPath, file_system::vfs::VirtualFileSystem};

/// The underlying Posix filesystem
#[derive(Default)]
pub struct PosixFileSystem {}

impl VirtualFileSystem for PosixFileSystem {
    fn copy(&mut self, source: &HexPath, destination: &HexPath) -> Result<(), io::Error> {
        fs::copy(source, destination)?;
        Ok(())
    }

    fn create_dir_all(&mut self, path: &HexPath) -> Result<(), io::Error> {
        fs::create_dir_all(path)
    }

    fn is_file(&self, path: &HexPath) -> Result<bool, io::Error> {
        if !fs::exists(path)? {
            return Ok(false);
        }

        fs::metadata(path).map(|metadata| metadata.is_file())
    }

    fn list_dir(&self, path: &HexPath) -> Result<Vec<HexPath>, io::Error> {
        let read_dir = fs::read_dir(path)?;
        let mut result = Vec::new();

        for entry in read_dir {
            let file_name = entry?.path().to_string_lossy().to_string();
            result.push(path.child(&file_name));
        }

        result.sort();

        Ok(result)
    }

    fn modtime(&self, path: &HexPath) -> Result<u64, io::Error> {
        Ok(fs::metadata(path)?
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs())
    }

    fn read(&self, path: &HexPath) -> Result<Vec<u8>, io::Error> {
        fs::read(path)
    }

    fn remove_file(&mut self, path: &HexPath) -> Result<(), io::Error> {
        fs::remove_file(path)
    }

    fn rename(&mut self, old_path: &HexPath, new_path: &HexPath) -> Result<(), io::Error> {
        fs::rename(old_path, new_path)
    }

    fn touch(&mut self, path: &HexPath) -> Result<(), io::Error> {
        // Open the file in append mode. This should update the modification
        // time.
        let _ = OpenOptions::new().append(true).create(true).open(path)?;
        Ok(())
    }

    fn write(&mut self, path: &HexPath, contents: &[u8]) -> Result<(), io::Error> {
        // So that the write is atomic, write to a side file and then rename it
        let side_file = format!("{}.tmp", path);

        fs::write(&side_file, contents)?;
        fs::rename(side_file, path)?;

        Ok(())
    }
}
