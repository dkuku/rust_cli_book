use clap::{arg, command, Parser};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

type HeadResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(version, author, about)]
pub struct Config {
    /// Input files
    #[arg(name = "FILES", default_value = "-")]
    files: Vec<String>,
    /// Number of lines to print
    #[arg(short = 'n', long, default_value_t = 10, value_parser=parse_num)]
    lines: usize,
    /// Number of bytes to print
    #[arg(short = 'c', long, conflicts_with = "lines", value_parser=parse_num)]
    bytes: Option<usize>,
}
fn parse_num(val: &str) -> Result<usize, String> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(format!("{}", val)),
    }
}

pub fn run(config: Config) -> HeadResult<()> {
    let files_count = config.files.len();
    let multiple_files = files_count > 1;
    for (index, filename) in config.files.into_iter().enumerate() {
        // display filename when multiple files
        if multiple_files {
            println!("==> {} <==", &filename);
        }
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", &filename, err),
            Ok(mut file) => show_file_content(&mut file, config.lines, config.bytes)?,
        }
        // add empty line between filenames
        if multiple_files && index < files_count - 1 {
            println!("");
        }
    }
    Ok(())
}
fn show_file_content(
    file: &mut Box<dyn BufRead>,
    lines: usize,
    bytes: Option<usize>,
) -> HeadResult<()> {
    if let Some(num_bytes) = bytes {
        let bytes: Result<Vec<_>, _> = file.bytes().take(num_bytes).collect();
        print!("{}", String::from_utf8_lossy(&bytes?));
    } else {
        let mut line = String::new();
        for _ in 0..lines {
            let bytes = file.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            print!("{}", line);
            line.clear();
        }
    }
    Ok(())
}
fn open(filename: &str) -> HeadResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
pub fn get_args() -> HeadResult<Config> {
    Ok(Config::parse())
}
