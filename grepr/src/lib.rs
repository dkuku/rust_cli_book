use clap::Parser;
use regex::{Error as RegexError, Regex};
use std::error::Error;

type GrepResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Config {
    ///Search pattern
    #[arg(name = "PATTERN", required = true, value_parser = parse_regex)]
    pattern: Regex,
    ///Input file(s)
    #[arg(name = "FILES", default_value = "-")]
    files: Vec<String>,
    ///Recursive search
    #[arg(short, long)]
    recursive: bool,
    ///Case insensiive
    #[arg(short, long)]
    insensitive: bool,
    ///Count occurences
    #[arg(short, long)]
    count: bool,
    ///Invert match
    #[arg(short = 'v', long)]
    invert_match: bool,
}
pub fn run(config: Config) -> GrepResult<()> {
    println!("{:#?}", config);
    Ok(())
}
pub fn get_args() -> GrepResult<Config> {
    Ok(Config::parse())
}
fn parse_regex(pattern: &str) -> Result<Regex, RegexError> {
    Regex::new(pattern)
}
