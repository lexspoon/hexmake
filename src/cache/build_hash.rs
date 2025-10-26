use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::ops::Deref;
use std::rc::Rc;

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
    pub fn hash<Vfs: VirtualFileSystem>(
        vfs: &Vfs,
        env: &BTreeMap<Rc<String>, Rc<String>>,
        rule: &HexRule,
    ) -> Result<BuildHash, io::Error> {
        let mut context = Context::new(&SHA256);

        hash_rule(&mut context, rule);
        hash_env(&mut context, env);
        hash_trees(&mut context, vfs, &rule.inputs)?;

        let digest = context.finish();

        Ok(BuildHash(hex_string_for_digest(digest)))
    }

    /// Hash a file tree by itself
    pub fn hash_tree<Vfs: VirtualFileSystem>(
        vfs: &Vfs,
        output_path: &&HexPath,
    ) -> Result<BuildHash, io::Error> {
        let mut context = Context::new(&SHA256);
        hash_tree(&mut context, vfs, output_path)?;
        let digest = context.finish();
        Ok(BuildHash(hex_string_for_digest(digest)))
    }
}

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
fn hash_env(context: &mut Context, env: &BTreeMap<Rc<String>, Rc<String>>) {
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
fn hash_trees<Vfs: VirtualFileSystem>(
    context: &mut Context,
    vfs: &Vfs,
    paths: &[HexPath],
) -> Result<(), io::Error> {
    hash_usize(context, paths.len());
    for path in paths {
        hash_tree(context, vfs, path)?;
    }
    Ok(())
}

/// Hash a filesystem tree.
/// This will handle both files and directory trees.
/// It will return an error, though, if the tree doesn't exist at all.
fn hash_tree<Vfs: VirtualFileSystem>(
    context: &mut Context,
    vfs: &Vfs,
    path: &HexPath,
) -> Result<(), io::Error> {
    if vfs.is_file(path)? {
        // Use 0 to indicate a file rather than a directory. This
        // integer would be the number of
        hash_usize(context, 0);

        let contents = vfs.read(path)?;
        hash_bytes(context, &contents);
    } else {
        let children = vfs.list_dir(path)?;

        // Hash the number of children
        hash_usize(context, children.len());

        // Hash each subtree
        for child in children {
            hash_tree(context, vfs, &child)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::file_system::fake::FakeFileSystem;

    use super::*;

    #[test]
    fn test_hash() {
        let mut test_hashes: Vec<BuildHash> = Vec::new();

        // Set up a test rule, test environment, and test filesystem
        let mut rule = HexRule::new("test".into());
        rule.outputs = vec!["out/test.txt".into()];
        rule.inputs = vec!["test.txt".into()];
        rule.commands = vec!["cp test.txt out/text.txt".into()];

        let mut env: BTreeMap<Rc<String>, Rc<String>> = BTreeMap::new();
        env.insert("ENV1".to_string().into(), "env1".to_string().into());
        env.insert("ENV2".to_string().into(), "env2".to_string().into());

        let mut vfs = FakeFileSystem::default();
        vfs.write(&"test.txt".into(), b"test").unwrap();
        vfs.write(&"out/test.txt".into(), b"test").unwrap();

        // Get a base hash to compare the others against
        let base_hash = BuildHash::hash(&vfs, &env, &rule).unwrap();
        test_hashes.push(base_hash.clone());

        // A hash should be a hex string
        assert_eq!(
            &base_hash.0,
            "E2A8C1518C5A25644A9A5EDAC5454BF82BA7E899F92196ACB74E5D008386BB42"
        );

        // Hashing twice gives back the same value
        let hash = BuildHash::hash(&vfs, &env, &rule).unwrap();
        assert_eq!(hash, base_hash);

        // Changing an input file will affect the hash
        {
            let mut vfs = vfs.clone();
            vfs.write(&"test.txt".into(), b"test2").unwrap();
            let hash = BuildHash::hash(&vfs, &env, &rule).unwrap();
            test_hashes.push(hash);
        }

        // Changing an output file will not affect the hash
        {
            let mut vfs = vfs.clone();
            vfs.write(&"out/test.txt".into(), b"test2").unwrap();
            let hash = BuildHash::hash(&vfs, &env, &rule).unwrap();
            assert_eq!(hash, base_hash);
        }

        // Changing the commands will affect the hash
        {
            let mut rule = rule.clone();
            rule.commands = vec!["/usr/bin/cp test.txt out/text.txt".into()];
            let hash = BuildHash::hash(&vfs, &env, &rule).unwrap();
            test_hashes.push(hash);
        }

        // Changing the environment will affect the hash
        {
            let mut env = env.clone();
            env.insert(
                "ENV1".to_string().into(),
                "different-env1".to_string().into(),
            );
            let hash = BuildHash::hash(&vfs, &env, &rule).unwrap();
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
