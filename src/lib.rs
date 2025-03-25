use clap::{Args, Parser, Subcommand};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use std::{
    error::Error,
    fs,
    io::{self, Read, Write},
    path::{self, PathBuf},
};

const GIT_DIR: &str = ".rgit";
const OBJECTS_DIR: &str = "objects";

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

impl Commands {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Init => self.init(),
            Self::HashObject(args) => self.hash_object(&args),
            Self::CatFile(args) => self.cat_file(&args),
        }
    }

    fn init(&self) -> Result<(), Box<dyn Error>> {
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

    fn hash_object(&self, args: &HashObjectArgs) -> Result<(), Box<dyn Error>> {
        let contents = if args.use_stdin {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            vec![buf]
        } else {
            args.files
                .iter()
                .map(|file| fs::read_to_string(file))
                .collect::<Result<_, _>>()?
        };

        for content in contents {
            let header = format!("blob {}\0", content.bytes().len());
            let store = header + content.as_str();
            let obj_id = format!("{:x}", Sha1::digest(&store));

            if args.write_to_db {
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

    fn cat_file(&self, args: &CatFileArgs) -> Result<(), Box<dyn Error>> {
        let object_id = &args.object;
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
        if args.show_object_type {
            println!("{object_type}");
        } else if args.show_object_size {
            println!("{}", content.as_bytes().len());
        } else if args.print_object_content {
            match object_type {
                "blob" => println!("{content}"),
                _ => todo!(),
            }
        }

        Ok(())
    }
}
