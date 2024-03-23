use clap::Args;
use clap::Parser;
use clap::Subcommand;
use mygit::commands::*;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::path::PathBuf;

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
        commit_hash: String,
        /// Parent commit
        #[arg(short)]
        parent_commit_hash: String,
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
        Command::Init => init::run(),
        Command::CatFile { flag, object_hash } => {
            anyhow::ensure!(flag.pretty, "-p must be used");
            cat_file::run(&object_hash)?;
        }
        Command::HashObject { write, file } => {
            let hash = hash_object::run(write, file)?;
            println!("{}", hash);
        }
        Command::LsTree { name_only, object } => {
            let tree_entries = ls_tree::run(&object)?;
            for entry in tree_entries {
                if name_only {
                    println!("{}", entry.file);
                } else {
                    println!("{}", entry)
                }
            }
        }
        Command::WriteTree => {
            let hash = write_tree::run()?;
            println!("{}", hash);
        }
        Command::CommitTree {
            commit_hash,
            parent_commit_hash,
            message,
        } => {
            commit_tree::run(&commit_hash, &parent_commit_hash, &message)?;
        }
        Command::Clone {
            repo_url: repository_url,
        } => {
            clone::clone(repository_url);
        }
    };
    Ok(())
}
