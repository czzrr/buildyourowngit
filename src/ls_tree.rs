use crate::common::{parse_tree_entries, sha_to_path, zlib_decode, TreeEntry};

/// Return entries in tree object identified by `hash`.
pub fn ls_tree(hash: &str) -> Vec<TreeEntry> {
    let object_path = sha_to_path(hash);
    let object_contents = std::fs::read(object_path).unwrap();
    let decoded_object_contents = zlib_decode(&object_contents);

    let tree_entries = parse_tree_entries(&decoded_object_contents).unwrap().1;

    tree_entries
}
