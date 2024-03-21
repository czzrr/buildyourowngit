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
        object_hash: String,
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
        object: String,
    },
    /// Write tree object for staging area
    WriteTree,
    /// Commit tree object
    CommitTree {
        /// Tree SHA
        tree_sha: String,
        /// Parent commit
        #[arg(short)]
        parent_sha: String,
        /// Commit message
        #[arg(short)]
        message: String,
    },
    Clone {
        repo_url: String,
    },
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct CatFileFlag {
    /// Pretty-print object's contents
    #[arg(short)]
    pretty: bool,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Cli::parse();
    match args.command {
        Command::Init => init::init(),
        Command::CatFile { flag, object_hash } => {
            anyhow::ensure!(flag.pretty, "-p must be used");
            cat_file::cat_file(&object_hash)?;
        }
        Command::HashObject { write, file } => {
            let hash = hash_object::hash_object(write, file);
            println!("{}", hash);
        }
        Command::LsTree { name_only, object } => {
            let tree_entries = ls_tree::ls_tree(&object);
            for entry in tree_entries {
                if name_only {
                    println!("{}", entry.file);
                } else {
                    println!("{}", entry)
                }
            }
        }
        Command::WriteTree => {
            let hash = write_tree::write_tree();
            println!("{}", hash);
        }
        Command::CommitTree {
            tree_sha,
            parent_sha: parent_commit,
            message,
        } => {
            if let Err(err) = commit_tree::commit_tree(tree_sha, parent_commit, message) {
                println!("{}", err);
            }
        }
        Command::Clone {
            repo_url: repository_url,
        } => {
            clone::clone(repository_url);
        }
    };
    Ok(())
}
