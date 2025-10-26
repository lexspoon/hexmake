#![allow(unused)]
use std::io;

use crate::ast::hex_path::HexPath;

/// An abstract file system that can be faked out for testing.
pub trait VirtualFileSystem {
    fn copy(&mut self, source: &HexPath, destination: &HexPath) -> Result<(), io::Error>;
    fn create_dir_all(&mut self, path: &HexPath) -> Result<(), io::Error>;
    fn is_file(&self, path: &HexPath) -> Result<bool, io::Error>;
    fn remove_file(&mut self, path: &HexPath) -> Result<(), io::Error>;
    fn list_dir(&self, path: &HexPath) -> Result<Vec<HexPath>, io::Error>;
    fn modtime(&self, path: &HexPath) -> Result<u64, io::Error>;
    fn read(&self, path: &HexPath) -> Result<Vec<u8>, io::Error>;
    fn rename(&mut self, old_path: &HexPath, new_path: &HexPath) -> Result<(), io::Error>;
    fn touch(&mut self, path: &HexPath) -> Result<(), io::Error>;
    fn write(&mut self, path: &HexPath, contents: &[u8]) -> Result<(), io::Error>;
}
