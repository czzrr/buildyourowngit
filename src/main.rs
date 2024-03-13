#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use sha1::{Sha1, Digest};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize empty Git repository
    Init,
    /// Inspect Git objects
    CatFile {
        #[command(flatten)]
        flag: CatFileFlag,
        object: String,
    },
    /// Create a Git object
    HashObject {
        /// Write the object into the object database
        #[arg(short)]
        write: bool,
        // File with object contents
        file: PathBuf,
    }
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct CatFileFlag {
    /// Pretty-print object's contents
    #[arg(short)]
    pretty: bool,
    // /// Show object type
    // #[arg(name = "type", short)]
    // ty: bool,
    // /// Show object size
    // #[arg(short)]
    // size: bool
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Cli::parse();
    match args.command {
        Command::Init => init(),
        Command::CatFile { flag, object } => {
            if flag.pretty {
                let contents = pretty_print(object)?;
                print!("{}", contents);
            }
        },
        Command::HashObject { write, file } => {
            let hash = hash_object(write, file);
            println!("{}", hash);
        }
    };
    Ok(())
}

#[derive(Debug, Clone, Error)]
enum MyGitError {
    #[error("Invalid object name {0}")]
    InvalidObjectName(String),
}

fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    println!("Initialized git directory");
}

fn pretty_print(object: String) -> Result<String, MyGitError> {
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

fn hash_object(write: bool, file: PathBuf) -> String {
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