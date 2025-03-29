use crate::object::Type;
use crate::utils::{self, OBJECT_ID_SPLIT_MID, OBJECTS_DIR};
use anyhow::Result;
use clap::Args;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::Digest;
use sha1::Sha1;
use std::io::Write;
use std::{
    fs,
    io::{self, Read},
};

#[derive(Args, Debug)]
pub struct HashObjectArgs {
    #[arg(short = 'w')]
    write_to_db: bool,

    #[arg(long = "stdin")]
    use_stdin: bool,

    files: Vec<String>,
}

pub fn run(args: &HashObjectArgs) -> Result<()> {
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
            let (subdir, filename) = object_id.split_at(OBJECT_ID_SPLIT_MID);
            let object_dir_path = utils::resolve_path(&[OBJECTS_DIR, subdir])?;
            if !object_dir_path.try_exists()? {
                fs::create_dir(&object_dir_path)?;
            }

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(contents.as_slice())?;
            let compressed = encoder.finish()?;

            let file_path = object_dir_path.join(filename);
            fs::write(file_path, compressed)?;
        }

        println!("{object_id}");
    }

    Ok(())
}
