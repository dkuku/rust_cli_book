use crate::Column::*;
use clap::{command, ArgAction, Parser};
use std::cmp::Ordering::{Equal, Greater, Less};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type CommResult<T> = Result<T, Box<dyn Error>>;
enum Column {
    File1,
    File2,
    Both,
}
#[derive(Parser, Debug)]
#[command(version, author, about = "Rust comm")]
pub struct Config {
    ///Input file 1
    #[arg(name = "FILE 1")]
    file1: String,
    ///Input file 2
    #[arg(name = "FILE 2")]
    file2: String,
    ///Supress printing of column 1
    #[arg(short = '1', action = ArgAction::SetFalse)]
    show_col1: bool,
    ///Supress printing of column 2
    #[arg(short = '2', action = ArgAction::SetFalse)]
    show_col2: bool,
    ///Supress printing of column 3
    #[arg(short = '3', action = ArgAction::SetFalse)]
    show_col3: bool,
    ///Case insensiive
    #[arg(short)]
    insensitive: bool,
    ///Output delimiter
    #[arg(short, long = "output-delimiter", default_value = "\t")]
    delimiter: String,
}

pub fn run(config: Config) -> CommResult<()> {
    let file1 = &config.file1;
    let file2 = &config.file2;
    if file1 == "-" && file2 == "-" {
        return Err(From::from("Both input files cannot be STDIN (\"-\")"));
    }
    let mut lines1 = open(file1)?
        .lines()
        .map_while(Result::ok)
        .map(|s| case(s, &config.insensitive));
    let mut lines2 = open(file2)?
        .lines()
        .map_while(Result::ok)
        .map(|s| case(s, &config.insensitive));
    let mut line1 = lines1.next();
    let mut line2 = lines2.next();
    while line1.is_some() || line2.is_some() {
        match (&line1, &line2) {
            (None, None) => (),
            (Some(val1), None) => {
                print(val1, File1, &config);
                line1 = lines1.next();
            }
            (None, Some(val2)) => {
                print(val2, File2, &config);
                line2 = lines2.next();
            }
            (Some(val1), Some(val2)) => match val1.cmp(val2) {
                Equal => {
                    print(val1, Both, &config);
                    line1 = lines1.next();
                    line2 = lines2.next();
                }
                Less => {
                    print(val1, File1, &config);
                    line1 = lines1.next();
                }
                Greater => {
                    print(val2, File2, &config);
                    line2 = lines2.next();
                }
            },
        }
    }
    Ok(())
}

fn print(val: &str, col: Column, config: &Config) {
    match (config.show_col1, config.show_col2, config.show_col3, col) {
        (true, _, _, File1) => format_line(val, 0, &config.delimiter),
        (false, true, _, File2) => format_line(val, 0, &config.delimiter),
        (true, true, _, File2) => format_line(val, 1, &config.delimiter),
        (false, false, true, Both) => format_line(val, 0, &config.delimiter),
        (true, false, true, Both) => format_line(val, 1, &config.delimiter),
        (false, true, true, Both) => format_line(val, 1, &config.delimiter),
        (true, true, true, Both) => format_line(val, 2, &config.delimiter),
        _ => print!(""),
    };
}
fn case(line: String, insensitive: &bool) -> String {
    if *insensitive {
        line.to_lowercase()
    } else {
        line
    }
}
fn format_line(val: &str, pos: u8, delimiter: &str) {
    match pos {
        0 => println!("{}", val),
        1 => println!("{}{}", delimiter, val),
        _ => println!("{}{}{}", delimiter, delimiter, val),
    };
}
pub fn get_args() -> CommResult<Config> {
    Ok(Config::parse())
}
fn open(filename: &str) -> CommResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
