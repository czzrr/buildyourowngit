use std::fmt::Display;
use std::fs;
use std::fs::DirEntry;
use std::io;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use sha1::{Digest, Sha1};
use thiserror::Error;

use nom::bytes::complete::tag;
use nom::IResult;

pub mod clone;

pub use clone::*;

#[derive(Debug, Clone, Error)]
pub enum MyGitError {
    #[error("Invalid object name {0}")]
    InvalidObjectName(String),
}

struct TreeObject {
    contents: Vec<u8>,
}

impl From<Vec<TreeEntry>> for TreeObject {
    fn from(tree_entries: Vec<TreeEntry>) -> Self {
        let mut buf = Vec::new();

        for tree_entry in tree_entries.into_iter() {
            buf.extend(tree_entry.mode.to_string().as_bytes());
            buf.extend(b" ");
            buf.extend(tree_entry.file.as_bytes());
            buf.extend(b"\0");
            buf.extend(hex::decode(&tree_entry.hash).unwrap());
        }
        let mut newbuf = Vec::from(b"tree ");
        newbuf.extend(buf.len().to_string().as_bytes());
        newbuf.extend(b"\0");
        newbuf.extend(buf);

        TreeObject { contents: newbuf }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FileMode {
    RegularFile,
    ExecutableFile,
    Directory,
}

impl FileMode {
    pub fn parse(input: &[u8]) -> IResult<&[u8], FileMode> {
        let (input, mode) = nom::branch::alt((tag("100644"), tag("40000"), tag("100755")))(input)?;
        Ok((
            input,
            String::from_utf8(mode.to_vec())
                .unwrap()
                .as_str()
                .try_into()
                .unwrap(),
        ))
    }
}

impl Display for FileMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileMode::RegularFile => f.write_str("100644"),
            FileMode::ExecutableFile => f.write_str("100755"),
            FileMode::Directory => f.write_str("40000"),
        }
    }
}

impl TryFrom<&str> for FileMode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "100644" => Ok(FileMode::RegularFile),
            "100755" => Ok(FileMode::ExecutableFile),
            "40000" => Ok(FileMode::Directory),
            _ => Err(value.to_owned()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl From<FileMode> for ObjectType {
    fn from(value: FileMode) -> Self {
        match value {
            FileMode::RegularFile | FileMode::ExecutableFile => ObjectType::Blob,
            FileMode::Directory => ObjectType::Tree,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub mode: FileMode,
    pub ty: ObjectType,
    pub hash: String,
    pub file: String,
}

impl TreeEntry {
    pub fn parse(input: &[u8]) -> IResult<&[u8], TreeEntry> {
        let (input, mode) = FileMode::parse(input)?;
        let (input, _) = nom::character::complete::space0(input)?;
        let (input, file) = nom::bytes::complete::take_while(|s| s != '\0' as u8)(input)?;
        let (input, _) = tag("\0")(input)?;
        let (input, hash) = nom::bytes::complete::take(20usize)(input)?;

        let ty = ObjectType::from(mode);
        let hash = hex::encode(hash);
        let file = String::from_utf8(file.to_vec()).unwrap();

        Ok((
            input,
            TreeEntry {
                mode,
                ty,
                hash,
                file,
            },
        ))
    }
}

impl Display for TreeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:0>6} {} {}\t{}",
            self.mode.to_string(),
            self.ty,
            self.hash,
            self.file
        ))
    }
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
    let contents: Vec<u8> = decoded_blob
        .into_iter()
        .skip_while(|c| *c != '\0' as u8)
        .skip(1)
        .collect();
    let contents = String::from_utf8(contents).unwrap();

    Ok(contents)
}

fn hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let hashed_blob = hasher.finalize();
    hex::encode(hashed_blob)
}

struct BlobObject {
    contents: Vec<u8>,
}

fn file_to_blob_object(file: impl AsRef<Path>) -> BlobObject {
    let contents = std::fs::read(file).unwrap();
    let size = contents.len().to_string();

    let mut blob = Vec::new();
    blob.extend_from_slice(&b"blob "[..]);
    blob.extend_from_slice(size.as_bytes());
    blob.push('\0' as u8);
    blob.extend_from_slice(&contents);

    BlobObject { contents: blob }
}

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

fn parse_tree_entries(input: &[u8]) -> IResult<&[u8], Vec<TreeEntry>> {
    let (input, _) =
        nom::sequence::tuple((tag("tree"), nom::number::complete::le_i32, tag("\0")))(input)?;
    let (input, entries) = nom::multi::many0(TreeEntry::parse)(input)?;

    Ok((input, entries))
}

/// Return entries in tree object identified by `hash`.
pub fn ls_tree(hash: &str) -> Vec<TreeEntry> {
    let object_path = sha_to_path(hash);
    let object_contents = std::fs::read(object_path).unwrap();
    let decoded_object_contents = zlib_decode(&object_contents);

    let tree_entries = parse_tree_entries(&decoded_object_contents).unwrap().1;

    tree_entries
}

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

pub fn commit_tree(
    tree_sha: String,
    parent_sha: String,
    message: String,
) -> Result<(), MyGitError> {
    if !sha_to_path(&tree_sha).exists() {
        return Err(MyGitError::InvalidObjectName(tree_sha));
    }
    if !sha_to_path(&parent_sha).exists() {
        return Err(MyGitError::InvalidObjectName(parent_sha));
    }

    let mut buf: Vec<u8> = Vec::new();
    buf.extend(b"tree ");
    buf.extend(tree_sha.as_bytes());
    buf.extend(b"\n");
    buf.extend(b"parent ");
    buf.extend(parent_sha.as_bytes());
    buf.extend(b"\n");
    buf.extend(b"author John Doe <john@doe.com> 1710605448 +0100\n");
    buf.extend(b"committer John Doe <john@doe.com> 1710605448 +0100\n");
    buf.extend(b"\n");
    buf.extend(message.as_bytes());
    buf.extend(b"\n");

    let mut newbuf: Vec<u8> = Vec::new();
    newbuf.extend(b"commit ");
    newbuf.extend(buf.len().to_string().as_bytes());
    newbuf.extend(&buf);

    io::stdout().write_all(&buf).unwrap();

    Ok(())
}
