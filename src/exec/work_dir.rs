use std::fs::{copy, create_dir_all, remove_dir_all};
use std::io;
use std::path::{Path, PathBuf};

use crate::ast::hex_path::HexPath;
use ignore::Walk;

/// A utility for managing a worker's isolated work directory. Commands are run
/// in a side directory so that if an input file is not listed in the Hexmake
/// file, the command will tend to fail.
pub struct WorkDirManager {
    root_dir: String,
}

impl WorkDirManager {
    /// Create a new work directory manager for the given worker ID.
    /// The work directory will be at `.hex/work/{worker_id}`.
    pub fn new(worker_id: u32) -> WorkDirManager {
        WorkDirManager {
            root_dir: format!(".hex/work/{}", worker_id),
        }
    }

    /// Get the root path of the work directory.
    pub fn root(&self) -> &str {
        &self.root_dir
    }

    /// Create the root work directory.
    pub fn create_root(&self) -> io::Result<()> {
        create_dir_all(&self.root_dir)?;
        Ok(())
    }

    /// Clean the work directory by removing it entirely.
    pub fn clean(&self) -> io::Result<()> {
        // Remove the entire directory if it exists
        if Path::new(&self.root_dir).exists() {
            remove_dir_all(&self.root_dir)?;
        }
        Ok(())
    }

    /// Copy input files from the main workspace into the work directory.
    /// Directory structure is preserved, e.g., `src/foo.c` -> `{workdir}/src/foo.c`.
    /// If an input is a directory, the entire tree is copied recursively, respecting
    /// .gitignore files.
    pub fn copy_inputs(&self, inputs: &[HexPath]) -> io::Result<()> {
        for input in inputs {
            let src = Path::new(input.as_ref());
            let dst = Path::new(&self.root_dir).join(input.as_ref());

            if src.is_file() {
                copy_one_file(src, dst)?;
            } else if src.is_dir() {
                for entry in Walk::new(src) {
                    let entry = entry.map_err(io::Error::other)?;
                    let entry_path = entry.path();

                    // Skip the root directory itself
                    if entry_path == src {
                        continue;
                    }

                    // Calculate relative path from the source directory
                    let relative_path = entry_path.strip_prefix(src).map_err(io::Error::other)?;

                    // Construct destination path preserving the input's base path
                    let dst = dst.join(relative_path);

                    if entry_path.is_dir() {
                        // Create the directory
                        create_dir_all(&dst)?;
                    } else if entry_path.is_file() {
                        copy_one_file(entry_path, dst)?;
                    }
                }
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Input path is neither a file nor a directory: {}",
                        src.display()
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Ensure that the parent directory is made for each output file
    pub fn prepare_output_directories(&self, outputs: &[HexPath]) -> io::Result<()> {
        for output in outputs {
            let dst = Path::new(&self.root_dir).join(output.as_ref());

            // Create parent directories if needed
            if let Some(parent) = dst.parent() {
                create_dir_all(parent)?;
            }
        }
        Ok(())
    }

    /// Copy output files from the work directory back to the main output
    /// directory.
    pub fn copy_outputs(&self, outputs: &[HexPath]) -> io::Result<()> {
        for output in outputs {
            let src = Path::new(&self.root_dir).join(output.as_ref());
            let dst = Path::new(output.as_ref());

            // Create parent directories if needed
            if let Some(parent) = dst.parent() {
                create_dir_all(parent)?;
            }

            // Copy the file
            copy(&src, dst)?;
        }
        Ok(())
    }
}

/// Copy one file
fn copy_one_file(src: &Path, dst: PathBuf) -> Result<(), io::Error> {
    if let Some(parent) = dst.parent() {
        create_dir_all(parent)?;
    }
    copy(src, &dst)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    /// Create a test directory structure and clean it up after the test
    fn with_test_dir<F>(test_name: &str, f: F)
    where
        F: FnOnce(&str),
    {
        let test_dir = format!(".hex/test/{}", test_name);
        let _ = fs::remove_dir_all(&test_dir);
        f(&test_dir);
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_create_root() {
        with_test_dir("create_root", |test_dir| {
            let work_dir = WorkDirManager {
                root_dir: test_dir.to_string(),
            };

            // Directory should not exist initially
            assert!(!Path::new(test_dir).exists());

            // Create it
            work_dir.create_root().unwrap();
            assert!(Path::new(test_dir).exists());
            assert!(Path::new(test_dir).is_dir());
        });
    }

    #[test]
    fn test_clean() {
        with_test_dir("clean", |test_dir| {
            let work_dir = WorkDirManager {
                root_dir: test_dir.to_string(),
            };

            // Create the directory and add a file
            fs::create_dir_all(test_dir).unwrap();
            let test_file = format!("{}/test.txt", test_dir);
            File::create(&test_file)
                .unwrap()
                .write_all(b"test")
                .unwrap();
            assert!(Path::new(&test_file).exists());

            // Clean should remove the directory
            work_dir.clean().unwrap();
            assert!(!Path::new(test_dir).exists());

            // Clean on non-existent directory should succeed
            work_dir.clean().unwrap();
        });
    }

    #[test]
    fn test_root() {
        with_test_dir("root", |test_dir| {
            let work_dir = WorkDirManager {
                root_dir: test_dir.to_string(),
            };

            assert_eq!(work_dir.root(), test_dir);
        });
    }

    #[test]
    fn test_copy_inputs() {
        with_test_dir("copy_inputs", |test_dir| {
            let work_dir = WorkDirManager {
                root_dir: test_dir.to_string(),
            };

            // Create test input files
            let input_dir = format!("{}_input", test_dir);
            fs::create_dir_all(&input_dir).unwrap();
            let simple_file = format!("{}/simple.txt", input_dir);
            let nested_file = format!("{}/subdir/nested.txt", input_dir);
            fs::create_dir_all(format!("{}/subdir", input_dir)).unwrap();
            let nested_file2 = format!("{}/subdir2/nested.txt", input_dir);
            let nested_subdir2 = format!("{}/subdir2", input_dir);
            fs::create_dir_all(format!("{}/subdir2", input_dir)).unwrap();
            File::create(&simple_file)
                .unwrap()
                .write_all(b"simple")
                .unwrap();
            File::create(&nested_file)
                .unwrap()
                .write_all(b"nested")
                .unwrap();
            File::create(&nested_file2)
                .unwrap()
                .write_all(b"nested2")
                .unwrap();

            // Create work directory
            work_dir.create_root().unwrap();

            // Copy inputs
            let inputs = vec![
                HexPath::from(&simple_file),
                HexPath::from(&nested_file),
                HexPath::from(&nested_subdir2),
            ];
            work_dir.copy_inputs(&inputs).unwrap();

            // Verify files were copied with correct structure
            let copied_simple = format!("{}/{}", test_dir, simple_file);
            let copied_nested = format!("{}/{}", test_dir, nested_file);
            let copied_nested2 = format!("{}/{}", test_dir, nested_file2);
            assert!(Path::new(&copied_simple).exists());
            assert!(Path::new(&copied_nested).exists());
            assert!(Path::new(&copied_nested2).exists());

            let content = fs::read_to_string(&copied_simple).unwrap();
            assert_eq!(content, "simple");
            let content = fs::read_to_string(&copied_nested).unwrap();
            assert_eq!(content, "nested");
            let content = fs::read_to_string(&copied_nested2).unwrap();
            assert_eq!(content, "nested2");

            // Clean up input directory
            fs::remove_dir_all(&input_dir).unwrap();
        });
    }

    #[test]
    fn test_prepare_outputs() {
        with_test_dir("prepare_outputs", |test_dir| {
            let work_dir = WorkDirManager {
                root_dir: test_dir.to_string(),
            };

            work_dir.create_root().unwrap();

            // Prepare output directories
            let outputs = vec![
                HexPath::from("out/lib.o"),
                HexPath::from("out/subdir/main.o"),
            ];
            work_dir.prepare_output_directories(&outputs).unwrap();

            // Verify directories were created
            assert!(Path::new(&format!("{}/out", test_dir)).exists());
            assert!(Path::new(&format!("{}/out/subdir", test_dir)).exists());
        });
    }

    #[test]
    fn test_copy_outputs() {
        with_test_dir("copy_outputs", |test_dir| {
            let work_dir = WorkDirManager {
                root_dir: test_dir.to_string(),
            };

            // Create work directory and output files
            work_dir.create_root().unwrap();
            let output_dir = format!("{}/out", test_dir);
            fs::create_dir_all(&output_dir).unwrap();
            let output_file = format!("{}/result.txt", output_dir);
            File::create(&output_file)
                .unwrap()
                .write_all(b"result")
                .unwrap();

            // Copy outputs back to workspace
            let outputs = vec![HexPath::from("out/result.txt")];
            work_dir.copy_outputs(&outputs).unwrap();

            // Verify file was copied to correct location
            assert!(Path::new("out/result.txt").exists());
            let content = fs::read_to_string("out/result.txt").unwrap();
            assert_eq!(content, "result");

            // Clean up
            fs::remove_dir_all("out").unwrap();
        });
    }

    #[test]
    fn test_new() {
        let work_dir = WorkDirManager::new(42);
        assert_eq!(work_dir.root(), ".hex/work/42");
    }
}
