
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use sha1::{Sha1, Digest};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum MyGitError {
    #[error("Invalid object name {0}")]
    InvalidObjectName(String),
}

pub fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    println!("Initialized git directory");
}

pub fn pretty_print(object: String) -> Result<String, MyGitError> {
    let blob_sha = object.as_bytes();
    if blob_sha.len() < 3 {
        return Err(MyGitError::InvalidObjectName(object));
    }
    let blob_dir = &blob_sha[..2];
    let blob_file = &blob_sha[2..];
    let blob_path = format!(
        ".git/objects/{}/{}",
        std::str::from_utf8(&blob_dir).unwrap(),
        std::str::from_utf8(&blob_file).unwrap()
    );
    let blob_contents = fs::read(blob_path).map_err(|_| MyGitError::InvalidObjectName(object))?;
    let mut decoder = flate2::bufread::ZlibDecoder::new(&blob_contents[..]);
    let mut decoded_blob = String::new();
    decoder.read_to_string(&mut decoded_blob).unwrap();
    let contents: String = decoded_blob
        .chars()
        .skip_while(|c| c != &'\0')
        .skip(1)
        .collect();

    Ok(contents)
}

pub fn hash_object(write: bool, file: PathBuf) -> String {
    let contents = std::fs::read_to_string(file).unwrap();
    let blob: String = format!("blob {}\0{}", contents.len(), contents);
    
    // Hash blob contents
    let mut hasher = Sha1::new();
    hasher.update(blob.as_bytes());
    let hashed_blob = hasher.finalize();
    let hashed_blob_hex = hex::encode(hashed_blob);

    if write {
        // Zlib encode blob contents
        let mut buf = Vec::new();
        let mut encoder = flate2::write::ZlibEncoder::new(&mut buf, flate2::Compression::new(1));
        encoder.write_all(blob.as_bytes()).unwrap();
        let encoded_blob_contents = encoder.finish().unwrap();

        // Save encoded blob contents to file
        let blob_dir = String::from_utf8(hashed_blob_hex.as_bytes()[..2].to_vec()).unwrap();
        let blob_file = String::from_utf8(hashed_blob_hex.as_bytes()[2..].to_vec()).unwrap();
        let blob_file_path = format!("mygit/objects/{}/{}", blob_dir, blob_file);
        
        log::debug!("Saving blob to {}", blob_file_path);
        fs::create_dir_all(format!("mygit/objects/{}", blob_dir)).unwrap();
        fs::write(blob_file_path, encoded_blob_contents).unwrap();
    }

    hashed_blob_hex
}

fn zlib_decode(data: &[u8]) -> Vec<u8> {
    let mut decoder = flate2::bufread::ZlibDecoder::new(&data[..]);
    let mut decoded: Vec<u8> = Vec::new();
    decoder.read(&mut decoded).unwrap();

    decoded
}

fn sha_to_path(sha: &str) -> PathBuf {
    let prefix = String::from_utf8(sha.as_bytes()[..2].to_vec()).unwrap();
    let suffix = String::from_utf8(sha.as_bytes()[2..].to_vec()).unwrap();
    let file = PathBuf::from(format!("mygit/objects/{}/{}", prefix, suffix));

    file
}

#[derive(Debug, Clone, Copy)]
pub enum FileMode {
    RegularFile,
    ExecutableFile,
    SymbolicLink
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectType {
    Blob,
    Tree,
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    mode: FileMode,
    ty: ObjectType,
    hash: String,
    file: String
}

fn parse_tree_entries(data: &[u8]) -> Vec<TreeEntry> {

    todo!()
}

pub fn ls_tree(object: &str) {
    let object_path = sha_to_path(object);
    let object_contents = std::fs::read(object_path).unwrap();

    let decoded_object_contents = zlib_decode(&object_contents);

    let tree_entries = parse_tree_entries(&decoded_object_contents);

    for entry in tree_entries {
        println!("{:?}", entry);
    }
}