use std::{fs::DirEntry, os::unix::fs::PermissionsExt, path::Path};

use crate::{
    common::{hash, FileMode, Object, ObjectType, TreeEntry, TreeObject},
    hash_object::hash_object,
};

/// Write tree object for current directory and return its hash.
pub fn write_tree() -> anyhow::Result<String> {
    let contents = compute_tree_contents(".")?;

    let tree_object = Object {
        ty: ObjectType::Tree,
        contents,
    };

    let hash = tree_object.write_to_objects_store()?;

    Ok(hash)
}

/// Compute the tree entries for all files in `dir`.
fn compute_tree_contents(dir: impl AsRef<Path>) -> anyhow::Result<Vec<u8>> {
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
            let hash = hash_object(false, &file_name_abs).unwrap();
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
            let tree_contents = compute_tree_contents(file_name_abs)?;
            let hash: String = hash(&tree_contents);
            tree_entries.push(TreeEntry {
                mode: FileMode::Directory,
                ty: ObjectType::Tree,
                hash,
                file: file_name.to_str().unwrap().to_owned(),
            })
        }
    }

    Ok(TreeObject::from(tree_entries).contents)
}
