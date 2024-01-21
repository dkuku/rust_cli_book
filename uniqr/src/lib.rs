#![feature(buf_read_has_data_left)]
use clap::{arg, command, Parser};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

type UniqResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(version = "0.1.0", author = "dkuku", about = "Rust uniq")]
pub struct Config {
    /// Input file
    #[arg(name = "IN_FILE", default_value = "-")]
    in_file: String,
    /// Output file
    #[arg(name = "OUT_FILE")]
    out_file: Option<String>,
    /// Show counts
    #[arg(short, long)]
    count: bool,
}
pub fn run(config: Config) -> UniqResult<()> {
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line_count: u64 = 0;
    let mut previous_line = String::new();
    let mut line = String::new();

    let mut out_file: Box<dyn Write> = match &config.out_file {
        Some(out_name) => Box::new(File::create(out_name)?),
        _ => Box::new(io::stdout()),
    };
    let mut print = |count: u64, text: &str| -> UniqResult<()> {
        if count > 0 {
            if config.count {
                write!(out_file, "{:>4} {}", count, text)?;
            } else {
                write!(out_file, "{}", text)?;
            }
        };
        Ok(())
    };
    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        if previous_line.trim_end() != line.trim_end() {
            let _ = print(line_count, &previous_line);
            line_count = 0;
            previous_line = line.clone();
        }
        line_count += 1;
        line.clear();
    }
    let _ = print(line_count, &previous_line);

    Ok(())
}
pub fn get_args() -> UniqResult<Config> {
    Ok(Config::parse())
}
fn open(filename: &str) -> UniqResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
