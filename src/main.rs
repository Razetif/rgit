use std::{env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();
    let subcommand = args.next().unwrap_or_else(|| {
        todo!("Show help");
    });

    match subcommand.as_str() {
        "init" => init()?,
        _ => return Err(format!("Unknown subcommand: {}", subcommand).into()),
    }

    Ok(())
}

fn init() -> Result<(), Box<dyn Error>> {
    let git_dir_path = std::path::absolute(".rgit")?;
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
