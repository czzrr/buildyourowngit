use crate::common::{hash_to_path, parse_tree_entries, zlib_decode, TreeEntry};
use anyhow::Ok;

/// Return entries in tree object identified by `hash`.
pub fn run(hash: &str) -> anyhow::Result<Vec<TreeEntry>> {
    let object_path = hash_to_path(hash)?;
    let object_contents = std::fs::read(object_path)?;
    let decoded_object_contents = zlib_decode(&object_contents);

    let tree_entries = parse_tree_entries(&decoded_object_contents).unwrap().1;

    Ok(tree_entries)
}
