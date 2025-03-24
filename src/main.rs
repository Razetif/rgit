use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use std::{
    env,
    error::Error,
    fs,
    io::{self, Read, Write},
    path::{self, PathBuf},
};

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
        "cat-file" => cat_file(args)?,
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
    let write_to_db = get_bool_flag(&mut args, "-w");
    let use_stdin = get_bool_flag(&mut args, "--stdin");

    let contents = if use_stdin {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        vec![buf]
    } else {
        args.iter()
            .map(|file| fs::read_to_string(file))
            .collect::<Result<_, _>>()?
    };

    for content in contents {
        let header = format!("blob {}\0", content.bytes().len());
        let store = header + content.as_str();
        let obj_id = format!("{:x}", Sha1::digest(&store));

        if write_to_db {
            let subdir = &obj_id[..SUBDIR_LEN];
            let obj_dir = path::absolute(GIT_DIR)?.join(OBJECTS_DIR).join(subdir);
            if !obj_dir.try_exists()? {
                fs::create_dir(&obj_dir)?;
            }

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(store.as_bytes())?;
            let compressed = encoder.finish()?;

            let filename = &obj_id[SUBDIR_LEN..];
            let file_path = obj_dir.join(filename);
            fs::write(file_path, compressed)?;
        }

        println!("{obj_id}");
    }

    Ok(())
}

fn cat_file(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = args.collect();
    let show_object_type = get_bool_flag(&mut args, "-t");
    let show_object_size = get_bool_flag(&mut args, "-s");
    let print_object_content = get_bool_flag(&mut args, "-p");

    check_mutually_exclusive_flags(vec![
        ("-t", show_object_type),
        ("-s", show_object_size),
        ("-p", print_object_content),
    ])?;

    let object_id = args.last().unwrap_or_else(|| {
        todo!("Show help");
    });
    let object_path = PathBuf::from(GIT_DIR)
        .join(OBJECTS_DIR)
        .join(&object_id[..SUBDIR_LEN])
        .join(&object_id[SUBDIR_LEN..]);
    let compressed_content = fs::read(object_path)?;

    let mut decoder = ZlibDecoder::new(compressed_content.as_slice());
    let mut content = String::new();
    decoder.read_to_string(&mut content)?;

    let object_type = content
        .split_whitespace()
        .next()
        .ok_or_else(|| "Malformed content")?;
    if show_object_type {
        println!("{object_type}");
    } else if show_object_size {
        println!("{}", content.as_bytes().len());
    } else if print_object_content {
        match object_type {
            "blob" => println!("{content}"),
            _ => todo!(),
        }
    }

    Ok(())
}

fn get_bool_flag(args: &mut Vec<String>, flag: &str) -> bool {
    args.iter()
        .position(|arg| arg.as_str() == flag)
        .map(|index| {
            args.remove(index);
            true
        })
        .unwrap_or(false)
}

fn check_mutually_exclusive_flags(flags: Vec<(&str, bool)>) -> Result<(), String> {
    let trues: Vec<_> = flags
        .iter()
        .filter(|(_, value)| *value)
        .map(|(name, _)| *name)
        .collect();

    if trues.len() > 2 {
        return Err(format!("switch {} is incompatible with {}", trues[0], trues[1]).into());
    }

    Ok(())
}
