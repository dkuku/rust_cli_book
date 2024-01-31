use clap::Parser;
use regex::{Error as RegexError, Regex};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use walkdir::WalkDir;

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
    let entries = find_files(&config.files, config.recursive);
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => match open(&filename) {
                Err(e) => eprintln!("{}: {}", filename, e),
                Ok(file) => {
                    let matches = find_lines(file, &config.pattern, config.invert_match);
                    println!("Found {:?}", matches);
                }
            },
        }
    }
    Ok(())
}
pub fn get_args() -> GrepResult<Config> {
    Ok(Config::parse())
}
fn parse_regex(pattern: &str) -> Result<Regex, RegexError> {
    Regex::new(pattern)
}
fn find_lines<T: BufRead>(file: T, pattern: &Regex, invert_match: bool) -> GrepResult<Vec<String>> {
    let result = file
        .lines()
        .flat_map(|line| {
            if pattern.is_match(line.as_ref().unwrap()) ^ invert_match {
                Some(line.unwrap())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Ok(result)
}
fn find_files(paths: &[String], recursive: bool) -> Vec<GrepResult<String>> {
    let mut result = Vec::new();
    paths.iter().for_each(|path| {
        let path_struct = Path::new(path);
        if !path_struct.exists() {
            result.push(Err(format!("{path} Path does not exist").into()));
            return;
        };
        if recursive {
            if path_struct.is_dir() {
                WalkDir::new(path).into_iter().for_each(|p| match p {
                    Err(_e) => result.push(Err("walkdir error".into())),
                    Ok(p) => {
                        if Path::new(p.path()).is_file() {
                            result.push(Ok(p.path().display().to_string()))
                        }
                    }
                });
            } else {
                result.push(Err(format!("{path} is a directory").into()))
            }
        } else if path_struct.is_file() {
            result.push(Ok(path.clone()))
        } else {
            result.push(Err(format!("{path} is a directory").into()))
        }
    });
    result
}
fn open(filename: &str) -> GrepResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
#[cfg(test)]
mod test {
    use super::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;
    #[test]
    fn test_find_lines_standard() {
        // The pattern _or_ should match the one line, "Lorem"
        let text = b"Lorem\nIpsum\r\nDOLOR";
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
    #[test]
    fn test_find_lines_inverted() {
        // When inverted, the function should match the other two lines
        let text = b"Lorem\nIpsum\r\nDOLOR";
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
    }
    #[test]
    fn test_find_lines_standard_case_insensitive() {
        // This regex will be case-insensitive
        let text = b"Lorem\nIpsum\r\nDOLOR";
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();
        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
    }
    #[test]
    fn test_find_lines_inverted_case_insensitive() {
        // When inverted, the one remaining line should match
        let text = b"Lorem\nIpsum\r\nDOLOR";
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
    #[test]
    fn test_find_file_that_exists() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");
    }
    #[test]
    fn test_find_files_rejects_directory_without_recursive_option() {
        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }
    }
    #[test]
    fn test_find_files_with_recursive_option() {
        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace('\\', "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );
    }
    #[test]
    fn test_find_files_with_non_existent() {
        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}
