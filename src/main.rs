use clap::{Parser, Subcommand};
use rgit::objects;
use std::fs;
use std::path::Path;

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
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            match init_repo() {
                Ok(_) => println!("Initialized empty rgit repository in .rgit/"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }

       Commands::HashObject { file, write } => {
            let path = Path::new(file);
            if !path.exists() {
                eprintln!("File not found: {}", file);
                return;
            }

            let content = fs::read(path).expect("Could not read file");

            if *write {
                let repo_root = Path::new(".");
                match rgit::objects::store_blob(repo_root, &content) {
                    Ok(hash) => println!("{}", hash.as_str()),
                    Err(e) => eprintln!("Error storing blob: {}", e),
                }
            } else {
                let hash = rgit::objects::hash_blob(&content);
                println!("{}", hash);
            }
        }

        Commands::CatFile { hash, pretty } => {
            let repo_root = Path::new(".");
            let hash_obj = rgit::objects::Hash::new(hash.clone());

            match rgit::objects::read_blob(repo_root, &hash_obj) {
                Ok(content) => {
                    if *pretty {
                        println!("{}", String::from_utf8_lossy(&content));
                    } else {
                        use std::io::Write;
                        std::io::stdout().write_all(&content).unwrap();
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Commands::LsTree { hash, name_only } => {
            let repo_root = std::path::Path::new(".");
            let hash_object = objects::Hash::new(hash.clone());

            match objects::read_blob(repo_root, &hash_object) {
                Ok(content) => {
                    let entries = objects::parse_tree(&content);
                    for entry in entries {
                        if *name_only {
                            println!("{}", entry.name);
                        } else {
                            println!(
                                "{:06} {} {}\t{}",
                                entry.mode,
                                entry.otype.as_str(),
                                entry.hash.as_str(),
                                entry.name
                            );
                        }
                    }
                }
                Err(_e) => eprintln!("fatal: Not a valid tree name {}", hash),
            }
        }

        Commands::WriteTree => {
            let repo_root = Path::new(".");
            match objects::write_tree(repo_root) {
                Ok(hash) => println!("{}", hash.as_str()),
                Err(e) => eprintln!("Error writing tree: {}", e),
            }
        }
    }
}

fn init_repo() -> std::io::Result<()> {
    let base_dir = Path::new(".rgit");

    if !base_dir.exists() {
        fs::create_dir(base_dir)?;
    }

    fs::create_dir_all(base_dir.join("objects"))?;
    fs::create_dir_all(base_dir.join("refs/heads"))?;

    let head_path = base_dir.join("HEAD");
    fs::write(head_path, "ref: refs/heads/main\n")?;

    Ok(())
}
