use std::fmt::Display;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use sha1::{Digest, Sha1};
use thiserror::Error;

use nom::bytes::complete::tag;
use nom::IResult;

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
    let decoded_blob = zlib_decode(&blob_contents);
    let contents: Vec<u8> = decoded_blob.into_iter()
        .skip_while(|c| *c != '\0' as u8)
        .skip(1)
        .collect();
    let contents = String::from_utf8(contents).unwrap();
    
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
        let encoded_blob_contents = zlib_encode(blob.as_bytes());

        // Save encoded blob contents to file
        let blob_dir = String::from_utf8(hashed_blob_hex.as_bytes()[..2].to_vec()).unwrap();
        let blob_file = String::from_utf8(hashed_blob_hex.as_bytes()[2..].to_vec()).unwrap();
        let blob_file_path = format!(".git/objects/{}/{}", blob_dir, blob_file);

        log::debug!("Saving blob to {}", blob_file_path);
        fs::create_dir_all(format!(".git/objects/{}", blob_dir)).unwrap();
        fs::write(blob_file_path, encoded_blob_contents).unwrap();
    }

    hashed_blob_hex
}

fn zlib_encode(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(1));
    encoder.write_all(data).unwrap();
    let encoded = encoder.finish().unwrap();

    encoded
}

fn zlib_decode(data: &[u8]) -> Vec<u8> {
    let mut decoder = flate2::bufread::ZlibDecoder::new(&data[..]);
    let mut decoded: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut decoded).unwrap();

    decoded
}

fn sha_to_path(sha: &str) -> PathBuf {
    let prefix = String::from_utf8(sha.as_bytes()[..2].to_vec()).unwrap();
    let suffix = String::from_utf8(sha.as_bytes()[2..].to_vec()).unwrap();
    let file = PathBuf::from(format!(".git/objects/{}/{}", prefix, suffix));

    file
}

#[derive(Debug, Clone, Copy)]
pub enum FileMode {
    RegularFile,
    ExecutableFile,
    SymbolicLink,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectType {
    Blob,
    Tree,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Blob => f.write_str("blob"),
            ObjectType::Tree => f.write_str("tree"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub ty: ObjectType,
    pub hash: String,
    pub file: String,
}

impl Display for TreeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:0>6} {} {}\t{}",
            self.mode, self.ty, self.hash, self.file
        ))
    }
}

fn parse_tree_entry(input: &[u8]) -> IResult<&[u8], TreeEntry> {
    let (input, mode) = nom::branch::alt((tag("100644"), tag("40000"), tag("100755")))(input)?;
    let (input, _) = nom::character::complete::space0(input)?;
    let (input, file) = nom::bytes::complete::take_while(|s| s != '\0' as u8)(input)?;
    let (input, _) = tag("\0")(input)?;
    let (input, hash) = nom::bytes::complete::take(20usize)(input)?;

    let hash = hex::encode(hash);
    Ok((
        input,
        TreeEntry {
            mode: String::from_utf8(mode.to_vec()).unwrap(),
            ty: match mode {
                b"100644" | b"100755" => ObjectType::Blob,
                b"40000" => ObjectType::Tree,
                _ => panic!("{:?}", mode),
            },
            hash,
            file: String::from_utf8(file.to_vec()).unwrap(),
        },
    ))
}

fn parse_tree_entries(input: &[u8]) -> IResult<&[u8], Vec<TreeEntry>> {
    let (input, _) =
        nom::sequence::tuple((tag("tree"), nom::number::complete::le_i32, tag("\0")))(input)?;
    let (input, entries) = nom::multi::many0(parse_tree_entry)(input)?;

    Ok((input, entries))
}

pub fn ls_tree(object: &str) -> Vec<TreeEntry> {
    let object_path = sha_to_path(object);
    let object_contents = std::fs::read(object_path).unwrap();
    let decoded_object_contents = zlib_decode(&object_contents);

    let tree_entries = parse_tree_entries(&decoded_object_contents).unwrap().1;

    tree_entries
}
