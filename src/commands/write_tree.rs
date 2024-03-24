use crate::commands::hash_object;
use crate::common::{FileMode, Object, ObjectType, TreeEntry};
use anyhow::Context;
use std::{fs::DirEntry, os::unix::fs::PermissionsExt, path::Path};

/// Write tree object for current directory and return its hash.
pub fn run() -> anyhow::Result<String> {
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
    let mut files = std::fs::read_dir(&dir)?
        .into_iter()
        .collect::<Result<Vec<DirEntry>, _>>()?;
    files.sort_by(|f1, f2| {
        // https://github.com/git/git/blob/11c821f2f2a31e70fb5cc449f9a29401c333aad2/tree.c#L99
        let name1 = f1.file_name();
        let name1 = name1.as_encoded_bytes();
        let ty1 = f1.file_type().expect("file type");
        let name2 = f2.file_name();
        let name2 = name2.as_encoded_bytes();
        let ty2 = f2.file_type().expect("file type");

        let min_len = name1.len().min(name2.len());

        match name1[..min_len].cmp(&name2[..min_len]) {
            std::cmp::Ordering::Equal => (),
            ord => return ord,
        };

        let c1 = match name1.get(min_len).copied() {
            Some(c) => Some(c),
            _ if ty1.is_dir() => Some(b'/'),
            _ => None,
        };
        let c2 = match name2.get(min_len).copied() {
            Some(c) => Some(c),
            _ if ty2.is_dir() => Some(b'/'),
            _ => None,
        };

        c1.cmp(&c2)
    });

    // Compute tree entry for each file
    for file in files {
        let file_name = file.file_name();
        let file_name_abs = dir
            .as_ref()
            .to_path_buf()
            .join(&file_name)
            .canonicalize()
            .context(format!("get full path for {:?}", file_name))?;
        let file_type = file.file_type().unwrap();
        let is_exec = file
            .metadata()
            .context(format!("get metadata for {:?}", file_name))?
            .permissions()
            .mode()
            & 0o111
            != 0;
        let file_mode = if file_type.is_file() {
            if is_exec {
                FileMode::ExecutableFile
            } else {
                FileMode::RegularFile
            }
        } else {
            FileMode::Directory
        };

        if file_mode != FileMode::Directory {
            // Blob.
            // Hash file contents.
            let hash = hash_object::run(false, &file_name_abs).unwrap();
            let entry = TreeEntry {
                mode: file_mode,
                ty: ObjectType::from(file_mode),
                hash,
                file: file_name.to_str().unwrap().to_owned(),
            };
            tree_entries.push(entry);
        } else if !file_name_abs.as_path().to_str().unwrap().ends_with(".git")
            && !file_name_abs
                .as_path()
                .to_str()
                .unwrap()
                .ends_with("target")
        {
            // Tree.
            // Ignore `.git` and files in `.gitignore`.
            // Recursively compute tree entries.
            let contents = compute_tree_contents(file_name_abs)?;
            let tree_object = Object {
                ty: ObjectType::Tree,
                contents,
            };
            let hash = tree_object.write(std::io::sink())?;
            tree_entries.push(TreeEntry {
                mode: FileMode::Directory,
                ty: ObjectType::Tree,
                hash,
                file: file_name.to_str().unwrap().to_owned(),
            })
        }
    }

    let mut buf = Vec::new();
    for tree_entry in &tree_entries {
        tree_entry
            .write(&mut buf)
            .context("write tree entry into buffer")?;
    }

    Ok(buf)
}
