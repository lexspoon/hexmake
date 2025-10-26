#![allow(unused)] // TODO(lex)

use std::collections::BTreeMap;
use std::io;
use std::rc::Rc;

use crate::ast::hex_path::HexPath;
use crate::cache::build_hash::BuildHash;
use crate::{ast::hexmake_file::HexRule, file_system::vfs::VirtualFileSystem};

/// A cache of previously built outputs
pub struct BuildCache<Vfs: VirtualFileSystem> {
    root: HexPath,
    vfs: Vfs,
    env: Rc<BTreeMap<Rc<String>, Rc<String>>>,
}

impl<Vfs: VirtualFileSystem> BuildCache<Vfs> {
    pub fn new(mut vfs: Vfs, env: Rc<BTreeMap<Rc<String>, Rc<String>>>) -> Result<Self, io::Error> {
        let root = HexPath::from(".hex/cache");

        vfs.create_dir_all(&root.child("inputmaps"))?;
        vfs.create_dir_all(&root.child("outputs"))?;

        Ok(BuildCache { root, vfs, env })
    }

    /// Try to retrieve previously built outputs of the given rule.
    /// Return Ok(true) if there was a cache hit and the retrieval succeeded.
    pub fn retrieve_outputs(&mut self, rule: &HexRule) -> Result<bool, io::Error> {
        let rule_hash = BuildHash::hash(&self.vfs, &self.env, rule)?;
        let inputmap_path = self.root.child("inputmaps").child(&rule_hash);
        if !self.vfs.is_file(&inputmap_path)? {
            return Ok(false);
        }

        let inputmap = String::from_utf8(self.vfs.read(&inputmap_path)?).unwrap();
        let output_hashes: Vec<&str> = inputmap.split("\n").collect();

        for (output_path, output_hash) in rule.outputs.iter().zip(output_hashes.iter()) {
            let cached_path = self.root.child("outputs").child(output_hash);
            self.vfs.copy(&cached_path, output_path)?;
        }

        Ok(true)
    }

    /// Add build outputs to the cache
    pub fn insert_outputs(&mut self, rule: &HexRule) -> Result<(), io::Error> {
        let mut inputmap = String::new();
        for output_path in rule.outputs.iter() {
            // Copy the output to the cached dir
            let output_hash = BuildHash::hash_tree(&self.vfs, &output_path)?;
            let cached_path = self.root.child("outputs").child(&output_hash);
            self.vfs.copy(output_path, &cached_path)?;

            // Add it to the inputmap
            inputmap.push_str(&format!("{}\n", output_hash.0));
        }

        let rule_hash = BuildHash::hash(&self.vfs, &self.env, rule)?;
        let inputmap_path = self.root.child("inputmaps").child(&rule_hash);
        self.vfs.write(&inputmap_path, inputmap.as_bytes())?;

        Ok(())
    }

    /// Garbage collect the cache if it is has grown too large
    pub fn maybe_gc(&mut self) -> Result<(), io::Error> {
        // Not yet implemented

        // Scan the directory tree to see how large all the binaries are

        // If they are too large, then do a garbage collection right now

        Ok(())
    }
}
