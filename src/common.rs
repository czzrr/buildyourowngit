use std::{
    ffi::CStr,
    fmt::Display,
    io::{self, BufRead, BufReader, Cursor, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use nom::{bytes::complete::tag, IResult};
use sha1::{Digest, Sha1};

pub struct Object {
    pub ty: ObjectType,
    pub contents: Vec<u8>,
}

impl Object {
    pub fn read(object_hash: &str) -> anyhow::Result<Object> {
        let object_dir = object_hash
            .get(..2)
            .ok_or(anyhow::anyhow!("invalid object hash"))?;
        let object_file = &object_hash[2..];
        let object_path = format!(".git/objects/{}/{}", object_dir, object_file);
        let file_contents = std::fs::read(object_path).context("read object file")?;
        let decoded = zlib_decode(&file_contents);
        let mut buf_reader = BufReader::new(Cursor::new(decoded));
        let mut buf = Vec::new();
        buf_reader.read_until(0, &mut buf).context("read header")?;
        let header = CStr::from_bytes_with_nul(&buf).context("should end with a nul byte")?;
        let header = header.to_str().context("header should be valid utf-8")?;
        let (ty, size) = header
            .split_once(' ')
            .context("object type and size should be separated by a space")?;
        let ty = ObjectType::try_from(ty).map_err(|err| anyhow::anyhow!(err))?;
        let size: usize = size
            .parse()
            .context("expected object size to be decimal encoded")?;
        let mut contents = vec![0; size];
        buf_reader
            .read_exact(&mut contents)
            .context(format!("could not read {size} bytes"))?;
        let object = Object { ty, contents };

        Ok(object)
    }
}

pub fn zlib_encode(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(1));
    encoder.write_all(data).unwrap();
    let encoded = encoder.finish().unwrap();

    encoded
}

pub fn zlib_decode(data: &[u8]) -> Vec<u8> {
    let mut decoder = flate2::bufread::ZlibDecoder::new(&data[..]);
    let mut decoded: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut decoded).unwrap();

    decoded
}

pub struct TreeObject {
    pub contents: Vec<u8>,
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

impl TryFrom<&str> for ObjectType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            _ => Err("invalid variant"),
        }
    }
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

pub fn hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let hashed_blob = hasher.finalize();
    hex::encode(hashed_blob)
}

pub struct BlobObject {
    pub contents: Vec<u8>,
}

pub fn parse_tree_entries(input: &[u8]) -> IResult<&[u8], Vec<TreeEntry>> {
    let (input, _) =
        nom::sequence::tuple((tag("tree"), nom::number::complete::le_i32, tag("\0")))(input)?;
    let (input, entries) = nom::multi::many0(TreeEntry::parse)(input)?;

    Ok((input, entries))
}

pub fn file_to_blob_object(file: impl AsRef<Path>) -> BlobObject {
    let contents = std::fs::read(file).unwrap();
    let size = contents.len().to_string();

    let mut blob = Vec::new();
    blob.extend_from_slice(&b"blob "[..]);
    blob.extend_from_slice(size.as_bytes());
    blob.push('\0' as u8);
    blob.extend_from_slice(&contents);

    BlobObject { contents: blob }
}

pub fn sha_to_path(sha: &str) -> PathBuf {
    let prefix = String::from_utf8(sha.as_bytes()[..2].to_vec()).unwrap();
    let suffix = String::from_utf8(sha.as_bytes()[2..].to_vec()).unwrap();
    let file = PathBuf::from(format!(".git/objects/{}/{}", prefix, suffix));

    file
}
