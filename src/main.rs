use std::path::PathBuf;

use anyhow::{Result, anyhow, ensure};
use app::app;
use clap::Parser;

mod app;
mod geometry;
mod obj;
mod texture;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: Option<PathBuf>,

    #[arg(short, long)]
    pattern: String,

    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}\n{}", err, err.backtrace());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    let input = args.input.unwrap_or_else(|| PathBuf::from("."));
    let files = glob_input_files(&input, &args.pattern)?;
    ensure!(
        !files.is_empty(),
        "No files found in {:?} with pattern {}",
        input,
        args.pattern
    );
    let output = args.output;
    for file in &files {
        app(file, &output)?;
    }
    Ok(())
}

fn glob_input_files(input: &PathBuf, pattern: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let pattern_path = input.join(pattern);
    let pattern_str = pattern_path.to_str().ok_or(anyhow!("Invalid pattern"))?;
    let walker = glob::glob(pattern_str)?;
    for entry in walker {
        let path = entry?;
        if path.is_file() {
            files.push(path);
        }
    }
    Ok(files)
}
