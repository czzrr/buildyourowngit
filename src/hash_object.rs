use std::{fs, path::Path};

use anyhow::Context;

use crate::common::{hash, zlib_encode, Object, ObjectType};

/// Compute hash of `file`'s contents' blob object representation.
/// If `write` is `true`, write blob object.
pub fn hash_object(write: bool, file: impl AsRef<Path>) -> anyhow::Result<String> {
    let contents = std::fs::read(file)?;
    let blob = Object {
        ty: ObjectType::Blob,
        contents,
    };
    let mut buf = Vec::new();
    blob.write(&mut buf)?;
    let blob_hash = hash(&buf);
    let blob = buf;

    if write {
        let encoded_blob = zlib_encode(&blob);

        let blob_dir = &blob_hash[..2];
        let blob_file = &blob_hash[2..];
        let blob_file_path = format!(".git/objects/{}/{}", blob_dir, blob_file);
        log::debug!("Writing blob to {}", blob_file_path);
        fs::create_dir_all(format!(".git/objects/{}", blob_dir)).context("create object dir")?;
        fs::write(blob_file_path, encoded_blob).context("write object to file")?;
    }

    Ok(blob_hash)
}
