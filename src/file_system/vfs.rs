#![allow(unused)]
use std::io;

use crate::ast::hex_path::HexPath;

/// An abstract file system that can be faked out for testing.
pub trait VirtualFileSystem: Send + Sync {
    fn copy(&self, source: &HexPath, destination: &HexPath) -> Result<(), io::Error>;
    fn create_dir_all(&self, path: &HexPath) -> Result<(), io::Error>;
    fn exists(&self, path: &HexPath) -> Result<bool, io::Error>;
    fn file_size(&self, path: &HexPath) -> Result<u64, io::Error>;
    fn is_file(&self, path: &HexPath) -> Result<bool, io::Error>;
    fn list_dir(&self, path: &HexPath) -> Result<Vec<HexPath>, io::Error>;
    fn modtime(&self, path: &HexPath) -> Result<u64, io::Error>;
    fn read(&self, path: &HexPath) -> Result<Vec<u8>, io::Error>;
    fn remove_file(&self, path: &HexPath) -> Result<(), io::Error>;
    fn rename(&self, old_path: &HexPath, new_path: &HexPath) -> Result<(), io::Error>;
    fn touch(&self, path: &HexPath) -> Result<(), io::Error>;
    fn tree_walk(&self, path: &HexPath) -> Result<Vec<HexPath>, io::Error>;
    fn write(&self, path: &HexPath, contents: &[u8]) -> Result<(), io::Error>;
}
