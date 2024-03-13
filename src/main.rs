#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::path::PathBuf;

use clap::Args;
use clap::Parser;
use clap::Subcommand;

use mygit::*;

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
        // Hash of blob object
        object: String,
    },
    /// Create a Git object
    HashObject {
        /// Write the object into the object database
        #[arg(short)]
        write: bool,
        // File with object contents
        file: PathBuf,
    },
    /// Inspect a tree object
    LsTree {
        /// List only filenames
        #[arg(long)]
        name_only: bool,
        /// Hash of tree object
        object: String
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
        },
        Command::LsTree { name_only, object } => {
            ls_tree(&object);
        }
    };
    Ok(())
}

