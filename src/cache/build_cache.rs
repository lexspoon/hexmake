use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::sync::Arc;

use crate::ast::hex_path::HexPath;
use crate::ast::hexmake_file::HexRule;
use crate::cache::build_hash::BuildHash;
use crate::file_system::vfs::VirtualFileSystem;

/// A cache of previously built outputs
pub struct BuildCache {
    root: HexPath,
    env: Arc<BTreeMap<Arc<String>, Arc<String>>>,
    vfs: Box<dyn VirtualFileSystem>,
}

/*
 * A cache of previously built outputs. It has two kinds of files:
 * 1. Inputmaps. The file `.hex/cache/inputmaps/ABCD` has an input map for
 *    the build rule with the given hash. The file will contain a list of hashes,
 *    one per line, of the outputs of the build rule, in the same order that the
 *    outputs appear in the "outputs" field of the rule.
 * 2. The output files themselves. The file `.hex/cache/outputs/ABCD` holds
 *    a file whose hash is ABCD. It is possible fo the same output to be used
 *    by multiple inputmaps; that means that Hexmake ran a build but determined
 *    that it already had the output for that rule, after all.
 */
impl BuildCache {
    pub fn new(
        env: Arc<BTreeMap<Arc<String>, Arc<String>>>,
        vfs: Box<dyn VirtualFileSystem>,
    ) -> Result<Self, io::Error> {
        let root = HexPath::from(".hex/cache");

        vfs.create_dir_all(&root.child("inputmaps"))?;
        vfs.create_dir_all(&root.child("outputs"))?;

        Ok(BuildCache { root, env, vfs })
    }

    /// Try to retrieve previously built outputs of the given rule.
    /// Return Ok(true) if there was a cache hit and the retrieval succeeded.
    pub fn retrieve_outputs(&self, rule: &HexRule) -> Result<bool, io::Error> {
        let rule_hash = BuildHash::hash(&self.env, rule, &*self.vfs)?;
        let inputmap_path = self.root.child("inputmaps").child(&rule_hash);

        if !self.vfs.exists(&inputmap_path)? {
            return Ok(false);
        }

        let inputmap = String::from_utf8(self.vfs.read(&inputmap_path)?).unwrap();
        let output_hashes: Vec<&str> = inputmap.split("\n").collect();

        for (output_path, output_hash) in rule.outputs.iter().zip(output_hashes.iter()) {
            let cached_path = self.root.child("outputs").child(output_hash);
            self.vfs.remove_file(output_path)?;
            self.vfs.copy(&cached_path, output_path)?;
        }

        Ok(true)
    }

    /// Add build outputs to the cache
    pub fn insert_outputs(&self, rule: &HexRule) -> Result<(), io::Error> {
        let mut inputmap = String::new();
        for output_path in rule.outputs.iter() {
            // Copy the output to the cached dir
            let output_hash = BuildHash::hash_tree(&output_path, self.vfs.as_ref())?;
            let cached_path = self.root.child("outputs").child(&output_hash);
            self.vfs.copy(output_path, &cached_path)?;

            // Add it to the inputmap
            inputmap.push_str(&format!("{}\n", output_hash.0));
        }

        let rule_hash = BuildHash::hash(&self.env, rule, self.vfs.as_ref())?;
        let inputmap_path = self.root.child("inputmaps").child(&rule_hash);
        self.vfs.write(&inputmap_path, inputmap.as_bytes())?;

        Ok(())
    }

    /// Garbage collect the cache if it has grown too large
    pub fn maybe_gc(&self) -> Result<(), io::Error> {
        const MAX_SIZE: u64 = 200 * 1024 * 1024; // 200 MB
        const TARGET_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

        let outputs_dir = self.root.child("outputs");

        // Scan all output files and compute their total size
        let mut output_files: Vec<(HexPath, u64, u64)> = Vec::new(); // (path, size, modtime)
        let mut total_size: u64 = 0;

        for file_path in self.vfs.list_dir(&outputs_dir)? {
            if self.vfs.is_file(&file_path)? {
                let size = self.vfs.file_size(&file_path)?;
                let modtime = self.vfs.modtime(&file_path)?;
                output_files.push((file_path, size, modtime));
                total_size += size;
            }
        }

        // If we're over the limit, delete oldest files
        if total_size > MAX_SIZE {
            // Sort by modification time (oldest first)
            output_files.sort_by_key(|(_, _, modtime)| *modtime);

            // Delete oldest files until we're under the target size
            let mut remaining_outputs = BTreeSet::new();
            for (file_path, size, _) in output_files {
                if total_size <= TARGET_SIZE {
                    remaining_outputs.insert(file_path);
                } else {
                    self.vfs.remove_file(&file_path)?;
                    total_size -= size;
                }
            }

            // Delete inputmaps that reference missing outputs, and collect the set of
            // outputs that are still referenced by valid inputmaps
            let referenced_outputs = self.cleanup_orphaned_inputmaps(&remaining_outputs)?;

            // Delete orphaned outputs (outputs not referenced by any inputmap)
            self.cleanup_orphaned_outputs(&remaining_outputs, &referenced_outputs)?;
        }

        Ok(())
    }

    /// Remove inputmap files that reference non-existent output files.
    /// Returns the set of output files that are referenced by valid inputmaps.
    fn cleanup_orphaned_inputmaps(
        &self,
        existing_outputs: &BTreeSet<HexPath>,
    ) -> Result<BTreeSet<HexPath>, io::Error> {
        let inputmaps_dir = self.root.child("inputmaps");
        let mut referenced_outputs = BTreeSet::new();

        for inputmap_path in self.vfs.list_dir(&inputmaps_dir)? {
            if !self.vfs.is_file(&inputmap_path)? {
                continue;
            }

            // Read the inputmap and check if all referenced outputs exist
            let inputmap = String::from_utf8(self.vfs.read(&inputmap_path)?).unwrap();
            let output_hashes: Vec<&str> = inputmap.split('\n').collect();

            let mut has_missing_output = false;
            let mut this_inputmap_outputs = Vec::new();

            for output_hash in output_hashes {
                if output_hash.is_empty() {
                    continue;
                }
                let output_path = self.root.child("outputs").child(output_hash);
                this_inputmap_outputs.push(output_path.clone());

                if !existing_outputs.contains(&output_path) {
                    has_missing_output = true;
                    break;
                }
            }

            // If any output is missing, delete this inputmap
            if has_missing_output {
                self.vfs.remove_file(&inputmap_path)?;
            } else {
                // This is a valid inputmap, track its outputs as referenced
                for output_path in this_inputmap_outputs {
                    referenced_outputs.insert(output_path);
                }
            }
        }

        Ok(referenced_outputs)
    }

    /// Remove orphaned output files (outputs not referenced by any inputmap)
    fn cleanup_orphaned_outputs(
        &self,
        existing_outputs: &BTreeSet<HexPath>,
        referenced_outputs: &BTreeSet<HexPath>,
    ) -> Result<(), io::Error> {
        for output_path in existing_outputs {
            if !referenced_outputs.contains(output_path) {
                self.vfs.remove_file(output_path)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_system::fake::FakeFileSystem;
    use crate::file_system::vfs::VirtualFileSystem;

    #[test]
    fn test_gc_does_nothing_when_under_limit() {
        let vfs = Box::new(FakeFileSystem::default()) as Box<dyn VirtualFileSystem>;
        let fake_vfs =
            unsafe { &*(vfs.as_ref() as *const dyn VirtualFileSystem as *const FakeFileSystem) };

        let env = Arc::new(BTreeMap::new());
        let cache = BuildCache::new(env, vfs).unwrap();

        // Create some small files (total well under 200 MB)
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/file1"), 1024 * 1024)
            .unwrap(); // 1 MB
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/file2"), 1024 * 1024)
            .unwrap(); // 1 MB
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/file3"), 1024 * 1024)
            .unwrap(); // 1 MB

        // GC should do nothing
        cache.maybe_gc().unwrap();

        // All files should still exist
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/outputs/file1"))
                .unwrap()
        );
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/outputs/file2"))
                .unwrap()
        );
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/outputs/file3"))
                .unwrap()
        );
    }

    #[test]
    fn test_gc_deletes_oldest_files_when_over_limit() {
        let vfs = Box::new(FakeFileSystem::default()) as Box<dyn VirtualFileSystem>;
        let fake_vfs =
            unsafe { &*(vfs.as_ref() as *const dyn VirtualFileSystem as *const FakeFileSystem) };

        let env = Arc::new(BTreeMap::new());
        let cache = BuildCache::new(env, vfs).unwrap();

        // Create files totaling over 200 MB (will trigger GC)
        // These will have different modification times due to the fake clock
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/old1"), 80 * 1024 * 1024)
            .unwrap(); // 80 MB, oldest
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/old2"), 80 * 1024 * 1024)
            .unwrap(); // 80 MB
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/new1"), 80 * 1024 * 1024)
            .unwrap(); // 80 MB, newest

        // Create an inputmap that references new1 so it's not orphaned
        fake_vfs
            .write(&HexPath::from(".hex/cache/inputmaps/map1"), b"new1\n")
            .unwrap();

        // Total is 240 MB, over the 200 MB limit
        // GC should delete oldest files until we're under 100 MB
        cache.maybe_gc().unwrap();

        // The two oldest files should be deleted, newest should remain
        assert!(
            !cache
                .vfs
                .exists(&HexPath::from(".hex/cache/outputs/old1"))
                .unwrap()
        );
        assert!(
            !cache
                .vfs
                .exists(&HexPath::from(".hex/cache/outputs/old2"))
                .unwrap()
        );
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/outputs/new1"))
                .unwrap()
        );
    }

    #[test]
    fn test_gc_does_not_prune_when_under_limit() {
        let vfs = Box::new(FakeFileSystem::default()) as Box<dyn VirtualFileSystem>;
        let fake_vfs =
            unsafe { &*(vfs.as_ref() as *const dyn VirtualFileSystem as *const FakeFileSystem) };

        let env = Arc::new(BTreeMap::new());
        let cache = BuildCache::new(env, vfs).unwrap();

        // Create output files
        fake_vfs
            .write(&HexPath::from(".hex/cache/outputs/output1"), b"data")
            .unwrap();
        fake_vfs
            .write(&HexPath::from(".hex/cache/outputs/output2"), b"data")
            .unwrap();

        // Create inputmaps - one referencing existing outputs, one referencing missing output
        fake_vfs
            .write(&HexPath::from(".hex/cache/inputmaps/map1"), b"output1\n")
            .unwrap();
        fake_vfs
            .write(&HexPath::from(".hex/cache/inputmaps/map2"), b"output2\n")
            .unwrap();
        fake_vfs
            .write(&HexPath::from(".hex/cache/inputmaps/orphan"), b"missing\n")
            .unwrap();

        // Run GC - won't do anything because we're under the limit (no pruning)
        cache.maybe_gc().unwrap();

        // All inputmaps should still exist (no pruning happened)
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/inputmaps/map1"))
                .unwrap()
        );
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/inputmaps/map2"))
                .unwrap()
        );
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/inputmaps/orphan"))
                .unwrap()
        );
    }

    #[test]
    fn test_gc_cleans_up_orphaned_inputmaps() {
        let vfs = Box::new(FakeFileSystem::default()) as Box<dyn VirtualFileSystem>;
        let fake_vfs =
            unsafe { &*(vfs.as_ref() as *const dyn VirtualFileSystem as *const FakeFileSystem) };

        let env = Arc::new(BTreeMap::new());
        let cache = BuildCache::new(env, vfs).unwrap();

        // Create large output files to trigger GC (over 200 MB total)
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/out1"), 150 * 1024 * 1024)
            .unwrap();
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/out2"), 60 * 1024 * 1024)
            .unwrap();
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/out3"), 10 * 1024 * 1024)
            .unwrap();

        // Create inputmap with multiple outputs, one of which is missing
        fake_vfs
            .write(
                &HexPath::from(".hex/cache/inputmaps/multi"),
                b"out1\nout2\nmissing\n",
            )
            .unwrap();

        // Create inputmap with all valid outputs that will survive GC
        // (out2 and out3 will survive because only out1 needs to be deleted to get under 100MB)
        fake_vfs
            .write(
                &HexPath::from(".hex/cache/inputmaps/valid"),
                b"out2\nout3\n",
            )
            .unwrap();

        // Total is 220 MB, will trigger GC which deletes old files and cleans orphans
        // GC will delete out1 (150 MB) to get under 100 MB, leaving out2 and out3
        cache.maybe_gc().unwrap();

        // Inputmap with missing output should be deleted (references "missing" which doesn't exist)
        assert!(
            !cache
                .vfs
                .exists(&HexPath::from(".hex/cache/inputmaps/multi"))
                .unwrap()
        );

        // Inputmap with all valid outputs should remain (out2 and out3 both survived GC)
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/inputmaps/valid"))
                .unwrap()
        );
    }

    #[test]
    fn test_gc_deletes_unreferenced_outputs() {
        let vfs = Box::new(FakeFileSystem::default()) as Box<dyn VirtualFileSystem>;
        let fake_vfs =
            unsafe { &*(vfs.as_ref() as *const dyn VirtualFileSystem as *const FakeFileSystem) };

        let env = Arc::new(BTreeMap::new());
        let cache = BuildCache::new(env, vfs).unwrap();

        // Create output files that total over 200 MB to trigger GC
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/old"), 150 * 1024 * 1024)
            .unwrap();
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/ref1"), 30 * 1024 * 1024)
            .unwrap();
        fake_vfs
            .write_all_zeros(&HexPath::from(".hex/cache/outputs/ref2"), 30 * 1024 * 1024)
            .unwrap();
        fake_vfs
            .write_all_zeros(
                &HexPath::from(".hex/cache/outputs/orphan"),
                20 * 1024 * 1024,
            )
            .unwrap();

        // Create an inputmap that only references ref1 and ref2
        // Note: "orphan" is not referenced by any inputmap, so it's an unreferenced output
        fake_vfs
            .write(&HexPath::from(".hex/cache/inputmaps/map1"), b"ref1\nref2\n")
            .unwrap();

        // Total is 230 MB, will trigger GC
        // GC deletes "old" (150 MB) to get under 100 MB
        // Then it should also delete "orphan" because no inputmap references it
        cache.maybe_gc().unwrap();

        // The old file should be deleted by GC
        assert!(
            !cache
                .vfs
                .exists(&HexPath::from(".hex/cache/outputs/old"))
                .unwrap()
        );

        // Referenced outputs should remain
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/outputs/ref1"))
                .unwrap()
        );
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/outputs/ref2"))
                .unwrap()
        );

        // Orphaned output (not referenced by any inputmap) should be deleted
        assert!(
            !cache
                .vfs
                .exists(&HexPath::from(".hex/cache/outputs/orphan"))
                .unwrap()
        );

        // The inputmap should still exist (it references valid outputs)
        assert!(
            cache
                .vfs
                .exists(&HexPath::from(".hex/cache/inputmaps/map1"))
                .unwrap()
        );
    }
}
