use clap::{arg, command, Parser};
use regex::Regex;
use std::error::Error;
use strum::EnumString;
use walkdir::WalkDir;

type FindResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug, Clone)]
#[command(version = "0.1.0", author = "dkuku", about = "Rust find")]
pub struct Config {
    /// Search paths
    #[arg(name = "PATH", default_value = ".")]
    paths: Vec<String>,
    /// Name
    #[arg(short, long = "name", name = "NAME", num_args(1..), value_parser = parse_regex)]
    names: Vec<Regex>,
    /// Entry type
    #[arg(short = 't', long = "type", name = "TYPE", num_args(1..))]
    entry_types: Vec<EntryType>,
}
#[derive(Clone, EnumString, Debug, Parser, PartialEq)]
enum EntryType {
    #[strum(serialize = "dir", serialize = "d")]
    Dir,
    #[strum(serialize = "file", serialize = "f")]
    File,
    #[strum(serialize = "link", serialize = "l")]
    Link,
}
pub fn run(config: Config) -> FindResult<()> {
    run_borrow(&config)
}
pub fn run_borrow(config: &Config) -> FindResult<()> {
    let Config {
        entry_types,
        names,
        paths,
    } = config;
    for path in paths {
        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| match e {
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
                Ok(val) => Some(val),
            })
            .filter(|e| {
                entry_types.is_empty()
                    || entry_types.iter().any(|entry_type| match entry_type {
                        EntryType::Link => e.file_type().is_symlink(),
                        EntryType::Dir => e.file_type().is_dir(),
                        EntryType::File => e.file_type().is_file(),
                    })
            })
            .filter(|e| {
                names.is_empty()
                    || names.iter().any(|re| match e.path().file_name() {
                        Some(file_name) => {
                            re.is_match(file_name.to_str().expect("filename unavailable"))
                        }
                        None => false,
                    })
            })
            .for_each(|e| println!("{}", e.path().display()));
    }
    Ok(())
}
pub fn get_args() -> FindResult<Config> {
    Ok(Config::parse())
}
fn parse_regex(name: &str) -> Result<Regex, String> {
    Regex::new(name).map_err(|_| format!("invalid --name \"{}\"", &name))
}
