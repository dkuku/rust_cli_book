use clap::{arg, command, Parser};
use regex::Regex;
use std::error::Error;
use strum::EnumString;

type FindResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(version = "0.1.0", author = "dkuku", about = "Rust find")]
pub struct Config {
    /// Search paths
    #[arg(name = "PATH", default_value = ".")]
    paths: Vec<String>,
    /// Name
    #[arg(short, long = "name", name = "NAME", value_parser = parse_regex)]
    names: Vec<Regex>,
    /// Entry type
    #[arg(short, long = "type", name = "TYPE")]
    entry_types: Vec<EntryType>,
}
#[derive(Clone, EnumString, Debug, Parser)]
enum EntryType {
    #[strum(serialize = "dir", serialize = "d")]
    Dir,
    #[strum(serialize = "file", serialize = "f")]
    File,
    #[strum(serialize = "link", serialize = "l")]
    Link,
}
pub fn run(config: Config) -> FindResult<()> {
    dbg!(config);
    Ok(())
}
pub fn get_args() -> FindResult<Config> {
    Ok(Config::parse())
}
fn parse_regex(name: &str) -> Result<Regex, String> {
    Regex::new(&name).map_err(|_| format!("invalid --name \"{}\"", &name))
}
