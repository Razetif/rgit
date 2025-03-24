use flate2::{Compression, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use std::{env, error::Error, fs, io::Write, path};

const GIT_DIR: &str = ".rgit";
const OBJECTS_DIR: &str = "objects";

const SUBDIR_LEN: usize = 2;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();
    let subcommand = args.next().unwrap_or_else(|| {
        todo!("Show help");
    });

    match subcommand.as_str() {
        "init" => init()?,
        "hash-object" => hash_object(args)?,
        _ => return Err(format!("Unknown subcommand: {}", subcommand).into()),
    }

    Ok(())
}

fn init() -> Result<(), Box<dyn Error>> {
    let git_dir_path = path::absolute(GIT_DIR)?;
    if git_dir_path.try_exists()? {
        println!(
            "Reinitialized existing Git repository in {}",
            git_dir_path.display()
        );
    } else {
        println!(
            "Initialized empty Git repository in {}",
            git_dir_path.display()
        );
    }

    fs::create_dir_all(git_dir_path.join("objects").join("info"))?;
    fs::create_dir_all(git_dir_path.join("objects").join("pack"))?;
    fs::create_dir_all(git_dir_path.join("refs").join("heads"))?;
    fs::create_dir_all(git_dir_path.join("refs").join("tags"))?;

    Ok(())
}

fn hash_object(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = args.collect();
    let write_to_db = args
        .iter()
        .position(|arg| arg.as_str() == "-w")
        .map(|index| {
            args.remove(index);
            true
        })
        .unwrap_or(false);

    let files = args;
    for file in files {
        let content = fs::read_to_string(file)?;
        let header = format!("blob {}\0", content.bytes().len());
        let store = header + content.as_str();
        let hash = format!("{:x}", Sha1::digest(&store));

        if write_to_db {
            let subdir = &hash[..SUBDIR_LEN];
            let obj_dir = path::absolute(GIT_DIR)?.join(OBJECTS_DIR).join(subdir);
            if !obj_dir.try_exists()? {
                fs::create_dir(&obj_dir)?;
            }

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(store.as_bytes())?;
            let compressed = encoder.finish()?;

            let filename = &hash[SUBDIR_LEN..];
            let file_path = obj_dir.join(filename);
            fs::write(file_path, compressed)?;
        }

        println!("{hash}");
    }

    Ok(())
}
