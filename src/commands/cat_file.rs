use anyhow::Result;
use clap::Args;
use flate2::read::ZlibDecoder;
use std::{fs, io::Read};

use crate::{
    error::MalformedError,
    object::Type,
    utils::{self, OBJECT_ID_SPLIT_MID, OBJECTS_DIR},
};

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

pub fn run(args: &CatFileArgs) -> Result<()> {
    let object_id = &args.object;
    let (subdir, filename) = object_id.split_at(OBJECT_ID_SPLIT_MID);
    let object_file_path = utils::resolve_path(&[OBJECTS_DIR, subdir, filename])?;
    let compressed = fs::read(object_file_path)?;

    let mut decoder = ZlibDecoder::new(compressed.as_slice());
    let mut contents = Vec::new();
    decoder.read_to_end(&mut contents)?;

    let (header, body) = {
        let mut parts = contents.split_inclusive(|byte| *byte == b'\0');
        let header = parts.next().ok_or(MalformedError)?;
        let body = parts.next().ok_or(MalformedError)?;
        (header, body)
    };
    let object_type = {
        let header_str = String::from_utf8(header.into())?;
        let (typ, _) = header_str.split_once(' ').ok_or(MalformedError)?;
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
            _ => todo!("Handle other object types"),
        }
    }

    Ok(())
}
