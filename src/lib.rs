use clap::{Args, Parser, Subcommand};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use index::{Entry, Index};
use object::Type;
use sha1::{Digest, Sha1};
use std::{
    error::Error,
    fs::{self, File, OpenOptions, metadata},
    io::{self, Read, Seek, Write},
    os::unix::fs::MetadataExt,
    path::{self, PathBuf},
};

mod index;
mod object;

const GIT_DIR: &str = ".rgit";
const INDEX_FILE: &str = "index";
const OBJECTS_DIR: &str = "objects";
const OBJECTS_INFO_DIR: &str = "info";
const OBJECTS_PACK_DIR: &str = "pack";
const REFS_DIR: &str = "refs";
const REFS_HEADS_DIR: &str = "heads";
const REFS_TAGS_DIR: &str = "refs";

const SUBDIR_LEN: usize = 2;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init,
    HashObject(HashObjectArgs),
    CatFile(CatFileArgs),
    UpdateIndex(UpdateIndexArgs),
}

#[derive(Args, Debug)]
pub struct HashObjectArgs {
    #[arg(short = 'w')]
    write_to_db: bool,

    #[arg(long = "stdin")]
    use_stdin: bool,

    files: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CatFileArgs {
    #[arg(short = 't', group = "input")]
    show_object_type: bool,

    #[arg(short = 's', group = "input")]
    show_object_size: bool,

    #[arg(short = 'p', group = "input")]
    print_object_content: bool,

    object: String,
}

#[derive(Args, Debug)]
pub struct UpdateIndexArgs {
    #[arg(long = "add")]
    add: bool,

    #[arg(long = "remove")]
    remove: bool,

    #[arg(long = "verbose")]
    verbose: bool,

    files: Vec<PathBuf>,
}

impl Commands {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Init => self.init(),
            Self::HashObject(args) => self.hash_object(&args),
            Self::CatFile(args) => self.cat_file(&args),
            Self::UpdateIndex(args) => self.update_index(&args),
        }
    }

    fn init(&self) -> Result<(), Box<dyn Error>> {
        let git_dir_path = path::absolute(GIT_DIR)?;
        let message = if git_dir_path.try_exists()? {
            "Reinitialized existing Git repository in"
        } else {
            "Initialized empty Git repository in"
        };
        println!("{} {}", message, git_dir_path.display());

        fs::create_dir_all(git_dir_path.join(OBJECTS_DIR).join(OBJECTS_INFO_DIR))?;
        fs::create_dir_all(git_dir_path.join(OBJECTS_DIR).join(OBJECTS_PACK_DIR))?;
        fs::create_dir_all(git_dir_path.join(REFS_DIR).join(REFS_HEADS_DIR))?;
        fs::create_dir_all(git_dir_path.join(REFS_DIR).join(REFS_TAGS_DIR))?;

        Ok(())
    }

    fn hash_object(&self, args: &HashObjectArgs) -> Result<(), Box<dyn Error>> {
        let file_contents_list = if args.use_stdin {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf)?;
            vec![buf]
        } else {
            args.files
                .iter()
                .map(|file| fs::read(file))
                .collect::<Result<_, _>>()?
        };

        for body in file_contents_list {
            let header = format!("{} {}\0", Type::Blob, body.len()).into_bytes();
            let mut contents = header;
            contents.extend_from_slice(&body);
            let object_id = format!("{:x}", Sha1::digest(&contents));

            if args.write_to_db {
                let subdir = &object_id[..SUBDIR_LEN];
                let object_dir_path = path::absolute(GIT_DIR)?.join(OBJECTS_DIR).join(subdir);
                if !object_dir_path.try_exists()? {
                    fs::create_dir(&object_dir_path)?;
                }

                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(contents.as_slice())?;
                let compressed = encoder.finish()?;

                let filename = &object_id[SUBDIR_LEN..];
                let file_path = object_dir_path.join(filename);
                fs::write(file_path, compressed)?;
            }

            println!("{object_id}");
        }

        Ok(())
    }

    fn cat_file(&self, args: &CatFileArgs) -> Result<(), Box<dyn Error>> {
        let object_id = &args.object;
        let object_file_path = PathBuf::from(GIT_DIR)
            .join(OBJECTS_DIR)
            .join(&object_id[..SUBDIR_LEN])
            .join(&object_id[SUBDIR_LEN..]);
        let compressed_contents = fs::read(object_file_path)?;

        let mut decoder = ZlibDecoder::new(compressed_contents.as_slice());
        let mut contents = Vec::new();
        decoder.read_to_end(&mut contents)?;

        let (header, body) = {
            let mut parts = contents.splitn(2, |byte| *byte == b'\0');
            let header = parts.next().ok_or_else(|| "Malformed input")?;
            let body = parts.next().ok_or_else(|| "Malformed input")?;
            (header, body)
        };
        let object_type = {
            let header_str = String::from_utf8(header.into())?;
            let (typ, _) = header_str
                .split_once(' ')
                .ok_or_else(|| "Malformed input")?;
            Type::build(typ)?
        };

        if args.show_object_type {
            println!("{object_type}");
        } else if args.show_object_size {
            println!("{}", contents.len());
        } else if args.print_object_content {
            match object_type {
                Type::Blob => {
                    let body = String::from_utf8_lossy(body);
                    println!("{body}")
                }
                _ => todo!(),
            }
        }

        Ok(())
    }

    fn update_index(&self, args: &UpdateIndexArgs) -> Result<(), Box<dyn Error>> {
        let index_file_path = PathBuf::from(GIT_DIR).join(INDEX_FILE);
        let mut index_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&index_file_path)?;
        let mut index = if metadata(index_file_path)?.size() == 0 {
            Index::empty()
        } else {
            let mut buf = Vec::new();
            index_file.read_to_end(&mut buf)?;
            Index::parse(buf)?
        };

        let entries: Vec<_> = args
            .files
            .iter()
            .map(|filename| {
                let mut file = File::open(filename)?;
                let entry = Entry::from(
                    filename.to_str().ok_or_else(|| "Malformed input")?,
                    &mut file,
                );
                entry
            })
            .collect::<Result<_, _>>()?;
        for entry in entries {
            if args.remove {
                if index.entries.contains(&entry) {
                    index.entries.retain(|e| *e != entry);
                    if args.verbose {
                        println!("remove '{}'", entry.filename);
                    }
                }
            }

            if args.add {
                if !index.entries.contains(&entry) {
                    let filename = entry.filename.clone();
                    index.entries.push(entry);
                    if args.verbose {
                        println!("add '{}'", filename);
                    }
                }
            }
        }

        let content = index.serialize()?;
        index_file.rewind()?;
        index_file.write_all(&content)?;
        index_file.set_len(content.len() as u64)?;

        Ok(())
    }
}
