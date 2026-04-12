use clap::{Parser, Subcommand};
use rgit::{domain::{blob::Blob, commit::Commit, hash::Hash, tree::Tree, status::FileState}, store::{object_store, repo::Repo}};
use rgit::domain;
use std::{collections::HashMap, path::Path};

#[derive(Parser)]
#[command(name = "rgit")]
#[command(about = "A tiny Git implementation in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    HashObject {
        file: String,
        #[arg(short, long)]
        write: bool,
    },
    CatFile {
        hash: String,
        #[arg(short, long)]
        pretty: bool,
    },
    LsTree {
        hash: String,
        #[arg(short, long)]
        name_only: bool,
    },
    WriteTree,
    Commit {
        #[arg(short, long)]
        message: String,
        #[arg(short, long, default_value = "User <user@example.com>")]
        author: String,
    },
    Log,
    Status,
}

fn main() {
    let cli = Cli::parse();
    let repo = Repo::new(Path::new("."));

    match &cli.command {
        Commands::Init => {
            if let Err(e) = repo.init() {
                eprintln!("Error: initializing repository {}", e);
            } else {
                println!("Initialized empty rgit repository in .rgit/");
            }
        }

       Commands::HashObject { file, write } => {
            let content = std::fs::read(file).expect("Could not read file");
            if *write{
                let hash = Blob::store(&repo.root, &content).unwrap();
                println!("{}", hash.as_str());
            } else {
                let hash = Hash::from_bytes(&Blob::prepare(&content));
                println!("{}", hash.as_str());
            }
       }

        Commands::CatFile { hash, pretty } => {
            let hash_obj = Hash::new(hash.clone());
            match object_store::read(&repo.root, &hash_obj) {
                Ok(full_data) => {
                    if let Some(null_pos) = full_data.iter().position(|&b| b == 0) {
                        let content = &full_data[null_pos + 1..];
                        if *pretty {
                            print!("{}", String::from_utf8_lossy(content));
                        } else {
                            use std::io::Write;
                            std::io::stdout().write_all(content).unwrap();
                        }
                    }
                }
                Err(e) => eprintln!("fatal: Not a valid object name {}: {}", hash, e),
            }
        }

        Commands::LsTree { hash, name_only: _ } => {
            let hash_obj = Hash::new(hash.clone());
            match object_store::read(&repo.root, &hash_obj) {
                Ok(data) => {
                    let null_pos = data.iter().position(|&b| b == 0).unwrap();
                    let entries = Tree::parse(&data[null_pos + 1..]);
                    for entry in entries {
                        println!("{:06} {} {}\t{}", entry.mode, entry.otype.as_str(), entry.hash.as_str(), entry.name);
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Commands::WriteTree => {
            match Tree::write_from_path(&repo.root) {
                Ok(hash) => println!("{}", hash.as_str()),
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Commands::Commit { message, author } => {
            let tree_hash = Tree::write_from_path(&repo.root).expect("Tree write failed");
            let parent_hash = repo.get_head_hash().map(Hash::new);

            let commit = Commit {
                tree_hash,
                parent_hash,
                author: author.clone(),
                message: message.clone(),
            };

            let commit_data = commit.prepare();
            let commit_hash = object_store::write(&repo.root, &commit_data).expect("Commit storage failed");

            repo.update_head(&commit_hash).expect("Failed to update HEAD");
            println!("[main {}] {}", &commit_hash.as_str()[..7], message);
        }

        Commands::Log => {
            let mut current_hash = repo.get_head_hash().map(Hash::new);
            while let Some(hash) = current_hash {
                let data = object_store::read(&repo.root, &hash).unwrap();
                let commit = Commit::parse(&data);

                println!("\x1b[33commit {}\x1b[0m]", hash.as_str());
                println!("Author: {}", commit.author);
                println!("\n   {}\n", commit.message);

                current_hash = commit.parent_hash;
            }
        }

        Commands::Status => {
            let head_hash = repo.get_head_hash().map(Hash::new);
            let head_entries = if let Some(hash) = head_hash {
                let commit_data = object_store::read(&repo.root, &hash).unwrap();
                let commit = Commit::parse(&commit_data);
                Tree::get_entries_map(&repo.root, &commit.tree_hash).unwrap_or_default()
            } else {
                HashMap::new()
            };

            let result = domain::status::calculate_status(&repo.root, &head_entries).unwrap();

            println!("On branch main\n");
            if result.changes.is_empty() {
                println!("nothing to commit, working tree clean");
            } else {
                for (name, state) in result.changes {
                    let color = match state {
                        FileState::Modified => "\x1b[31mmodified:  ",
                        FileState::Untracked=> "\x1b[31muntracked:  ",
                        FileState::Deleted=> "\x1b[31mdeleted:  ",
                    };
                    println!("{} {}\x1b[0m", color, name);
                }
            }
        }
    }
}
