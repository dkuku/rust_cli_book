use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type CatResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
    show_ends: bool,
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
    let matches = App::new("catr")
        .version("0.1.0")
        .author("dkuku")
        .about("Rust cat")
        .arg(
            Arg::with_name("filename")
                .value_name("FILENAME")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("number_lines")
                .short("n")
                .long("number")
                .conflicts_with("number_nonblank_lines")
                .help("Add numbers to lines")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("show_ends")
                .short("E")
                .long("show-ends")
                .help("Display $ at the end of the line")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("number_nonblank_lines")
                .short("b")
                .long("number-nonblank")
                .help("Add numbers to non blank lines")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("squeeze_blank")
                .short("s")
                .long("squeeze-blank")
                .help("Suppress repeated empty output lines")
                .takes_value(false),
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("filename").unwrap(),
        number_lines: matches.is_present("number_lines"),
        number_nonblank_lines: matches.is_present("number_nonblank_lines"),
        show_ends: matches.is_present("show_ends"),
        squeeze_blank: matches.is_present("squeeze_blank"),
    })
}
