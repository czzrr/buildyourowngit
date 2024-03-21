use crate::common::zlib_decode;
use anyhow::Context;
use std::fs;

pub fn pretty_print(object: String) -> anyhow::Result<String> {
    let blob_sha = object.as_bytes();
    let blob_dir = &blob_sha.get(..2).context("invalid object hash")?;
    let blob_file = &blob_sha[2..];
    let blob_path = format!(
        ".git/objects/{}/{}",
        std::str::from_utf8(&blob_dir).unwrap(),
        std::str::from_utf8(&blob_file).unwrap()
    );
    let blob_contents = fs::read(blob_path).context("invalid object hash")?;
    let decoded_blob = zlib_decode(&blob_contents);
    let contents: Vec<u8> = decoded_blob
        .into_iter()
        .skip_while(|c| *c != '\0' as u8)
        .skip(1)
        .collect();
    let contents = String::from_utf8(contents).unwrap();

    Ok(contents)
}
