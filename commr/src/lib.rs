use clap::{command, ArgAction, Parser};
use std::error::Error;
type CommResult<T> = Result<T, Box<dyn Error>>;
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

pub fn run(config: Config) -> CommResult<Config> {
    println!("{:#?}", config);
    todo!()
}

pub fn get_args() -> CommResult<Config> {
    Ok(Config::parse())
}
