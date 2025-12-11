use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::ops::Deref;
use std::sync::Arc;

use ring::digest::{Context, Digest, SHA256};

use crate::ast::hex_path::HexPath;
use crate::ast::hexmake_file::HexRule;
use crate::file_system::vfs::VirtualFileSystem;

/// A hash of a build rule and its inputs. This is the key
/// for the build cache.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BuildHash(pub String);

impl Display for BuildHash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "BuildHash({})", self.0)
    }
}

impl Debug for BuildHash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Deref for BuildHash {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BuildHash {
    /// Construct a build hash from the given rule and filesystem state
    pub fn hash(
        env: &BTreeMap<Arc<String>, Arc<String>>,
        rule: &HexRule,
        vfs: &dyn VirtualFileSystem,
    ) -> Result<BuildHash, io::Error> {
        let mut context = Context::new(&SHA256);

        hash_rule(&mut context, rule);
        hash_env(&mut context, env);
        hash_trees(&mut context, &rule.inputs, vfs)?;

        let digest = context.finish();

        Ok(BuildHash(hex_string_for_digest(digest)))
    }

    /// Hash a file tree by itself
    pub fn hash_tree(path: &&HexPath, vfs: &dyn VirtualFileSystem) -> Result<BuildHash, io::Error> {
        let mut context = Context::new(&SHA256);
        hash_tree(&mut context, path, vfs)?;
        let digest = context.finish();
        Ok(BuildHash(hex_string_for_digest(digest)))
    }
}

/// Convert the result of hashing into a hex string
fn hex_string_for_digest(digest: Digest) -> String {
    let mut hex_digest = String::new();
    for b in digest.as_ref() {
        hex_digest.push_str(&format!("{:02X}", b));
    }
    hex_digest
}

/// Hash a rule definition. This does not look at the filesystem, only at
/// the rule itself.
fn hash_rule(context: &mut Context, rule: &HexRule) {
    hash_usize(context, rule.outputs.len());
    for output in &rule.outputs {
        hash_string(context, output);
    }

    hash_usize(context, rule.inputs.len());
    for input in &rule.inputs {
        hash_string(context, input);
    }

    hash_usize(context, rule.commands.len());
    for command in &rule.commands {
        hash_string(context, command);
    }
}

/// Hash the environment variables. This will encode the number of variables
/// followed by the name and value of each variable.
fn hash_env(context: &mut Context, env: &BTreeMap<Arc<String>, Arc<String>>) {
    hash_usize(context, env.len());
    for (name, value) in env {
        hash_string(context, name);
        hash_string(context, value);
    }
}

// Add a 64-bit integer to a hash
fn hash_u64(context: &mut Context, value: u64) {
    context.update(&value.to_le_bytes());
}

// Add a usize to a hash
fn hash_usize(context: &mut Context, value: usize) {
    hash_u64(context, value as u64);
}

// Add a string to a hash. This will encode the length of the string followed
// by its bytes.
fn hash_string(context: &mut Context, value: &str) {
    hash_bytes(context, value.as_bytes());
}

// Add bytes to a hash. This will prefix the bytes by the number of bytes.
fn hash_bytes(context: &mut Context, value: &[u8]) {
    hash_usize(context, value.len());
    context.update(value);
}

/// Hash a list of filesystem trees
fn hash_trees(
    context: &mut Context,
    paths: &[HexPath],
    vfs: &dyn VirtualFileSystem,
) -> Result<(), io::Error> {
    hash_usize(context, paths.len());
    for path in paths {
        hash_tree(context, path, vfs)?;
    }
    Ok(())
}

/// Hash a filesystem tree.
/// This will handle both files and directory trees.
/// It will return an error, though, if the tree doesn't exist at all.
fn hash_tree(
    context: &mut Context,
    path: &HexPath,
    vfs: &dyn VirtualFileSystem,
) -> Result<(), io::Error> {
    if !vfs.exists(path)? {
        return Err(io::Error::other(format!("{path} does not exist")));
    }

    for entry_path in vfs.tree_walk(path)? {
        hash_string(context, &entry_path);
        if vfs.is_file(&entry_path)? {
            // Use 0 to mean the path is a file
            hash_usize(context, 0);
            let contents = vfs.read(&entry_path)?;
            hash_bytes(context, &contents);
        } else {
            // Use 1 for a directory
            hash_usize(context, 1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::file_system::fake::FakeFileSystem;

    #[test]
    fn test_hash() {
        let vfs = Box::new(FakeFileSystem::default()) as Box<dyn VirtualFileSystem>;

        let mut test_hashes: Vec<BuildHash> = Vec::new();

        // Set up a test rule, test environment, and test filesystem
        let mut rule = HexRule::new("test".into());
        rule.outputs = vec!["out/test.txt".into()];
        rule.inputs = vec!["test.txt".into()];
        rule.commands = vec!["cp test.txt out/text.txt".into()];

        let mut env: BTreeMap<Arc<String>, Arc<String>> = BTreeMap::new();
        env.insert("ENV1".to_string().into(), "env1".to_string().into());
        env.insert("ENV2".to_string().into(), "env2".to_string().into());

        vfs.write(&HexPath::from("test.txt"), b"test").unwrap();
        vfs.write(&HexPath::from("out/test.txt"), b"test").unwrap();

        // Get a base hash to compare the others against
        let base_hash = BuildHash::hash(&env, &rule, &*vfs).unwrap();
        test_hashes.push(base_hash.clone());

        // A hash should be a hex string (this specific value depends on the VFS implementation)
        assert_eq!(
            &base_hash.0,
            "6EB9CD32A5CB18E0D77E012C8958F924B1DA9A441A19736EF623B6582C73FCA8"
        );

        // Hashing twice gives back the same value
        let hash = BuildHash::hash(&env, &rule, &*vfs).unwrap();
        assert_eq!(hash, base_hash);

        // Changing an output file will not affect the hash
        {
            vfs.write(&HexPath::from("out/test.txt"), b"test2").unwrap();
            let hash = BuildHash::hash(&env, &rule, &*vfs).unwrap();
            assert_eq!(hash, base_hash);
        }

        // Changing an input file will affect the hash
        {
            vfs.write(&HexPath::from("test.txt"), b"test2").unwrap();
            let hash = BuildHash::hash(&env, &rule, &*vfs).unwrap();
            test_hashes.push(hash);
        }

        // Changing the commands will affect the hash
        {
            let mut rule = rule.clone();
            rule.commands = vec!["/usr/bin/cp test.txt out/text.txt".into()];
            let hash = BuildHash::hash(&env, &rule, &*vfs).unwrap();
            test_hashes.push(hash);
        }

        // Changing the environment will affect the hash
        {
            let mut env = env.clone();
            env.insert(
                "ENV1".to_string().into(),
                "different-env1".to_string().into(),
            );
            let hash = BuildHash::hash(&env, &rule, &*vfs).unwrap();
            test_hashes.push(hash);
        }

        assert_eq!(
            test_hashes.len(),
            BTreeSet::from_iter(test_hashes.iter().cloned()).len(),
            "{:#?}",
            test_hashes
        );
    }
}
