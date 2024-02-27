use clap::{arg, command, Parser};
use rand::prelude::SliceRandom;
use rand::SeedableRng;
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

type FortuneResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(version, author, about)]
pub struct Config {
    /// Input files or directories
    #[arg(name = "FILES", required = true)]
    sources: Vec<String>,
    /// Pattern
    #[arg(short = 'm')]
    pattern: Option<String>,
    /// Random seed
    #[arg(short, long, value_parser=parse_u64)]
    seed: Option<u64>,
    /// Case-insensitive pattern matching
    #[arg(short, long, default_value_t = false)]
    insensitive: bool,
}

#[derive(Debug)]
struct Fortune {
    source: String,
    text: String,
}
pub fn run(config: Config) -> FortuneResult<()> {
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;
    if let Some(pattern) = &config.pattern {
        let re = parse_pattern(pattern, config.insensitive)?;
        let mut filtered_fortunes = fortunes
            .iter()
            .filter(|fortune| re.is_match(&fortune.source) || re.is_match(&fortune.text))
            .peekable();
        if filtered_fortunes.peek().is_some() {
            filtered_fortunes.for_each(|fortune| println!("{}", fortune.text));
        } else {
            println!("No fortunes found");
        }
    } else if let Some(fortune) = pick_fortune(&fortunes, config.seed) {
        println!("{}", fortune);
    }
    Ok(())
}

fn parse_pattern(pattern: &str, insensitive: bool) -> FortuneResult<Regex> {
    RegexBuilder::new(pattern)
        .case_insensitive(insensitive)
        .build()
        .map_err(|_| format!("invalid pattern: {}", pattern).into())
}
pub fn get_args() -> FortuneResult<Config> {
    Ok(Config::parse())
}
fn parse_u64(val: &str) -> Result<u64, String> {
    val.parse()
        .map_err(|_| format!("'{}' not a valid integer", val))
}
fn find_files(paths: &[String]) -> FortuneResult<Vec<PathBuf>> {
    let dat = OsStr::new("dat");
    let mut files = Vec::new();
    for path in paths {
        let path_struct = Path::new(path);
        if !path_struct.exists() {
            return Err(format!("{path}: Path does not exist").into());
        };
        if path_struct.is_dir() {
            let directory_files = WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|p| p.file_type().is_file() && p.path().extension() != Some(dat))
                .map(|p| p.path().into());
            files.extend(directory_files);
        } else if path_struct.is_file() {
            files.push(path_struct.into())
        } else {
            return Err(format!("{path} is a directory").into());
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}
fn read_fortunes(paths: &[PathBuf]) -> FortuneResult<Vec<Fortune>> {
    let mut fortunes = Vec::new();
    for path in paths.iter() {
        let content = BufReader::new(File::open(path)?);
        content.split(b'%').flatten().for_each(|fortune| {
            let fortune = String::from_utf8(fortune).unwrap();
            let fortune = fortune.trim();
            if !fortune.is_empty() {
                fortunes.push(Fortune {
                    source: path.file_name().unwrap().to_str().unwrap().to_owned(),
                    text: fortune.to_owned(),
                })
            };
        });
    }
    Ok(fortunes)
}
fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    let fortune = match seed {
        None => fortunes.choose(&mut rand::thread_rng()),
        Some(number) => fortunes.choose(&mut rand::rngs::StdRng::seed_from_u64(number)),
    };
    fortune.map(|f| f.text.clone())
}

#[cfg(test)]
mod tests {
    use super::{find_files, parse_u64, pick_fortune, read_fortunes, Fortune};
    use std::path::PathBuf;
    #[test]
    fn test_parse_u64() {
        let res = parse_u64("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "'a' not a valid integer");
        let res = parse_u64("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);
        let res = parse_u64("4");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 4);
    }
    #[test]
    fn test_find_files_single() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );
    }

    #[test]
    fn test_find_files_bad_file() {
        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());
    }

    #[test]
    fn test_find_files_all_inputs() {
        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());
        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));
    }

    #[test]
    fn test_find_files_multiple_sources() {
        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        println!("{:#?}", res);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }
    #[test]
    fn test_read_fortunes() {
        // One input file
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());
        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }
        // Multiple input files
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }
    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                      attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];
        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
