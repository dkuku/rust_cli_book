use clap::{arg, command, Parser};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type CatResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(version = "0.1.0", author = "dkuku", about = "Rust cat")]
pub struct Config {
    /// Files to cat
    #[arg(name = "FILES", default_value = "-")]
    files: Vec<String>,
    /// Print line numbers
    #[arg(short, long = "number")]
    number_lines: bool,
    /// Print line numbers for nonblank lines
    #[arg(short = 'b', long = "number-nonblank")]
    number_nonblank_lines: bool,
    /// Show $ at the end of each line
    #[arg(short = 'E', long = "show-ends")]
    show_ends: bool,
    /// Squeeze multiple empty lines into a single line
    #[arg(short = 's', long = "squeeze")]
    squeeze_blank: bool,
}

pub fn run(config: Config) -> CatResult<()> {
    let end_char = if config.show_ends { "$" } else { "" };

    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(content) => {
                let mut line_number = 0;
                let mut previous_line_empty = false;

                for line_result in content.lines() {
                    let line = line_result?;
                    let is_empty = line.is_empty();
                    if config.squeeze_blank && previous_line_empty && is_empty {
                        continue;
                    } else {
                        previous_line_empty = is_empty;
                    }
                    if config.number_lines || config.number_nonblank_lines && !is_empty {
                        line_number += 1;
                        println!("{: >6}\t{}{}", line_number, line, end_char)
                    } else {
                        println!("{}{}", line, end_char)
                    }
                }
            }
        }
    }
    Ok(())
}
fn open(filename: &str) -> CatResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
pub fn get_args() -> CatResult<Config> {
    let config = Config::parse();
    Ok(config)
}
