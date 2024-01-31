use clap::Parser;
use regex::{Error as RegexError, Regex};
use std::error::Error;
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
    println!("{:#?}", config);
    Ok(())
}
pub fn get_args() -> GrepResult<Config> {
    Ok(Config::parse())
}
fn parse_regex(pattern: &str) -> Result<Regex, RegexError> {
    Regex::new(pattern)
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
#[cfg(test)]
mod tests {
    use super::find_files;
    use pretty_assertions::assert_eq;
    use rand::{distributions::Alphanumeric, Rng};

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
