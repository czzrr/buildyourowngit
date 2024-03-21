use std::{fs::DirEntry, os::unix::fs::PermissionsExt, path::Path};

use crate::{
    common::{hash, sha_to_path, zlib_encode, FileMode, ObjectType, TreeEntry, TreeObject},
    hash_object::hash_object,
};

/// Write tree object for current directory and return its hash.
pub fn write_tree() -> String {
    let tree_entries = get_tree_entries(".");

    for tree_entry in &tree_entries {
        log::debug!("{}", tree_entry);
    }

    let tree_object = TreeObject::from(tree_entries);
    let hash = hash(&tree_object.contents);
    let encoded_tree_object = zlib_encode(&tree_object.contents);
    let file = sha_to_path(&hash);

    log::debug!("Writing object to {}", file.to_str().unwrap());

    // Create dir if it doesn't exist
    let dir = file.ancestors().skip(1).next().unwrap();
    std::fs::create_dir_all(dir).unwrap();

    std::fs::write(file, encoded_tree_object).unwrap();

    hash
}

/// Compute the tree entries for all files in `dir`.
fn get_tree_entries(dir: impl AsRef<Path>) -> Vec<TreeEntry> {
    let mut tree_entries = Vec::new();

    // Get Vec of sorted files in directory
    let mut files = std::fs::read_dir(&dir)
        .unwrap()
        .into_iter()
        .collect::<Result<Vec<DirEntry>, _>>()
        .unwrap();
    files.sort_by(|f1, f2| f1.file_name().cmp(&f2.file_name()));

    // Compute tree entry for each file
    for file in files {
        let file_name = file.file_name();
        let file_name_abs = dir
            .as_ref()
            .to_path_buf()
            .join(&file_name)
            .canonicalize()
            .unwrap();
        let file_type = file.file_type().unwrap();
        let is_exec = file.metadata().unwrap().permissions().mode() & 0o111 != 0;
        let file_mode = if file_type.is_file() {
            if is_exec {
                FileMode::ExecutableFile
            } else {
                FileMode::RegularFile
            }
        } else {
            assert!(file_type.is_dir());
            FileMode::Directory
        };

        if file_type.is_file() {
            // Blob.
            // Hash file contents.
            let hash = hash_object(false, &file_name_abs);
            let entry = TreeEntry {
                mode: file_mode,
                ty: ObjectType::from(file_mode),
                hash,
                file: file_name.to_str().unwrap().to_owned(),
            };
            tree_entries.push(entry);
        } else if file_type.is_dir()
            && !file_name_abs.as_path().to_str().unwrap().ends_with(".git")
            && !file_name_abs
                .as_path()
                .to_str()
                .unwrap()
                .ends_with("target")
        {
            // Tree.
            // Ignore `.git` and files in `.gitignore`.
            // Recursively compute tree entries.
            let entries = get_tree_entries(file_name_abs);
            let tree_object = TreeObject::from(entries);
            let hash: String = hash(&tree_object.contents);
            tree_entries.push(TreeEntry {
                mode: FileMode::Directory,
                ty: ObjectType::Tree,
                hash,
                file: file_name.to_str().unwrap().to_owned(),
            })
        }
    }

    tree_entries
}
