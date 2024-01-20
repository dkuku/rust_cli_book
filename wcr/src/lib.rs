use clap::{arg, command, Parser};
use core::ops::AddAssign;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type WcResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(version = "0.1.0", author = "dkuku", about = "Rust wc")]
pub struct Config {
    /// Files to cat
    #[arg(name = "FILE", default_value = "-")]
    files: Vec<String>,
    /// Show byte count
    #[arg(short = 'c', long, conflicts_with = "chars")]
    bytes: bool,
    /// Show character count
    #[arg(short = 'm', long)]
    chars: bool,
    /// Show line count
    #[arg(short, long)]
    lines: bool,
    /// Show word count
    #[arg(short, long)]
    words: bool,
}
#[derive(Default, Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}
impl AddAssign for FileInfo {
    fn add_assign(&mut self, other: FileInfo) {
        *self = Self {
            num_lines: self.num_lines + other.num_lines,
            num_words: self.num_words + other.num_words,
            num_bytes: self.num_bytes + other.num_bytes,
            num_chars: self.num_chars + other.num_chars,
        }
    }
}

pub fn run(config: Config) -> WcResult<()> {
    let multiple_files = config.files.len() > 1;
    let mut total = FileInfo::default();
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(filehandle) => {
                if let Ok(info) = count(filehandle) {
                    let _ = display(&info, &config, filename);
                    if multiple_files {
                        total += info;
                    }
                }
            }
        }
    }
    if multiple_files {
        let _ = display(&total, &config, "total");
    }
    Ok(())
}
fn display(info: &FileInfo, config: &Config, filename: &str) -> WcResult<()> {
    if config.lines {
        print!("{:>8}", info.num_lines);
    }
    if config.words {
        print!("{:>8}", info.num_words);
    }
    if config.bytes {
        print!("{:>8}", info.num_bytes);
    }
    if config.chars {
        print!("{:>8}", info.num_chars);
    }
    if filename != "-" {
        println!(" {}", filename);
    } else {
        println!();
    };
    Ok(())
}
fn count(mut file: impl BufRead) -> WcResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut line = String::new();
    loop {
        let line_bytes = file.read_line(&mut line)?;
        if line_bytes == 0 {
            break;
        }

        num_lines += 1;
        num_chars += line.chars().count();
        num_words += line.split_whitespace().count();
        num_bytes += line_bytes;
        line.clear();
    }
    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}
fn open(filename: &str) -> WcResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
pub fn get_args() -> WcResult<Config> {
    let config = Config::parse();
    if config.bytes || config.chars || config.lines || config.words {
        Ok(config)
    } else {
        Ok(Config {
            files: config.files,
            chars: false,
            bytes: true,
            lines: true,
            words: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count_ascii() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
    #[test]
    fn test_count_non_ascii() {
        let text = "Frétt hefir öld óvu, þá er endr of gerðu\r";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 9,
            num_chars: 41,
            num_bytes: 47,
        };
        assert_eq!(info.unwrap(), expected);
    }
    #[test]
    fn test_count_ascii_multiline() {
        let text = "I don't want the world. I just want your half.\r\nI don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 2,
            num_words: 20,
            num_chars: 96,
            num_bytes: 96,
        };
        assert_eq!(info.unwrap(), expected);
    }
}
