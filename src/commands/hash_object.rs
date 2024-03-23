use std::path::Path;

use crate::common::{Object, ObjectType};

/// Compute hash of `file`'s contents' blob object representation.
/// If `write` is `true`, write blob object.
pub fn run(write: bool, file: impl AsRef<Path>) -> anyhow::Result<String> {
    let contents = std::fs::read(file)?;
    let blob = Object {
        ty: ObjectType::Blob,
        contents,
    };
    let hash = if write {
        blob.write_to_objects_store()?
    } else {
        blob.write(std::io::sink())?
    };

    Ok(hash)
}
