use std::{fs, path::Path};

use crate::common::{file_to_blob_object, hash, zlib_encode};

/// Compute hash of `file`'s contents' blob object representation.
/// If `write` is `true`, write blob object.
pub fn hash_object(write: bool, file: impl AsRef<Path>) -> String {
    let blob = file_to_blob_object(file).contents;
    let blob_hash = hash(&blob);

    if write {
        // Zlib encode blob contents
        let encoded_blob = zlib_encode(&blob);

        // Save encoded blob contents to file
        let blob_dir = String::from_utf8(blob_hash.as_bytes()[..2].to_vec()).unwrap();
        let blob_file = String::from_utf8(blob_hash.as_bytes()[2..].to_vec()).unwrap();
        let blob_file_path = format!(".git/objects/{}/{}", blob_dir, blob_file);

        log::debug!("Saving blob to {}", blob_file_path);

        fs::create_dir_all(format!(".git/objects/{}", blob_dir)).unwrap();
        fs::write(blob_file_path, encoded_blob).unwrap();
    }

    blob_hash
}
