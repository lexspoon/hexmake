use std::collections::BTreeMap;
use std::sync::Arc;
use std::{fs, io};

use crate::ast::hex_path::HexPath;
use crate::ast::hexmake_file::HexRule;
use crate::cache::build_hash::BuildHash;
use std::path::Path;

/// A cache of previously built outputs
pub struct BuildCache {
    root: HexPath,
    env: Arc<BTreeMap<Arc<String>, Arc<String>>>,
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
    pub fn new(env: Arc<BTreeMap<Arc<String>, Arc<String>>>) -> Result<Self, io::Error> {
        let root = HexPath::from(".hex/cache");

        fs::create_dir_all(&root.child("inputmaps"))?;
        fs::create_dir_all(&root.child("outputs"))?;

        Ok(BuildCache { root, env })
    }

    /// Try to retrieve previously built outputs of the given rule.
    /// Return Ok(true) if there was a cache hit and the retrieval succeeded.
    pub fn retrieve_outputs(&self, rule: &HexRule) -> Result<bool, io::Error> {
        let rule_hash = BuildHash::hash(&self.env, rule, Path::new("."))?;
        let inputmap_path = self.root.child("inputmaps").child(&rule_hash);

        if !fs::exists(&inputmap_path)? {
            return Ok(false);
        }

        let inputmap = String::from_utf8(fs::read(&inputmap_path)?).unwrap();
        let output_hashes: Vec<&str> = inputmap.split("\n").collect();

        for (output_path, output_hash) in rule.outputs.iter().zip(output_hashes.iter()) {
            let cached_path = self.root.child("outputs").child(output_hash);
            fs::remove_file(output_path)?;
            fs::copy(&cached_path, output_path)?;
        }

        Ok(true)
    }

    /// Add build outputs to the cache
    pub fn insert_outputs(&self, rule: &HexRule) -> Result<(), io::Error> {
        let mut inputmap = String::new();
        for output_path in rule.outputs.iter() {
            // Copy the output to the cached dir
            let output_hash = BuildHash::hash_tree(&output_path)?;
            let cached_path = self.root.child("outputs").child(&output_hash);
            fs::copy(output_path, &cached_path)?;

            // Add it to the inputmap
            inputmap.push_str(&format!("{}\n", output_hash.0));
        }

        let rule_hash = BuildHash::hash(&self.env, rule, Path::new("."))?;
        let inputmap_path = self.root.child("inputmaps").child(&rule_hash);
        fs::write(&inputmap_path, inputmap.as_bytes())?;

        Ok(())
    }

    /// Garbage collect the cache if it is has grown too large
    #[allow(unused)]
    pub fn maybe_gc(&self) -> Result<(), io::Error> {
        // Not yet implemented

        // Scan the directory tree to see how large all the binaries are

        // If they are too large, then do a garbage collection right now

        Ok(())
    }
}
