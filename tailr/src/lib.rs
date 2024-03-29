use crate::TakeValue::*;
use clap::{arg, command, Parser};
use std::error::Error;
use std::fmt::{Formatter, Result as FmtResult};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

type TailResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Clone)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}
impl std::fmt::Display for TakeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TakeValue::PlusZero => write!(f, "+0"),
            TakeValue::TakeNum(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, author, about)]
pub struct Config {
    /// Input files
    #[arg(name = "FILES", required = true)]
    files: Vec<String>,
    /// Number of lines to print
    #[arg(short = 'n', long, default_value_t = TakeNum(10), value_parser=parse_num)]
    lines: TakeValue,
    /// Number of bytes to print
    #[arg(short = 'c', long, conflicts_with = "lines", value_parser=parse_num)]
    bytes: Option<TakeValue>,
    /// Supress headers
    #[arg(short, long)]
    quiet: bool,
}
fn parse_num(val: &str) -> Result<TakeValue, String> {
    match (val.parse::<i64>(), val.starts_with('+')) {
        (Ok(0), true) => Ok(PlusZero),
        (Ok(n), true) => Ok(TakeNum(n)),
        (Ok(n), false) if n > 0 => Ok(TakeNum(-n)),
        (Ok(n), false) => Ok(TakeNum(n)),
        _ => Err(val.to_string()),
    }
}

pub fn run(config: Config) -> TailResult<()> {
    let num_files = config.files.len();
    for (file_num, filename) in config.files.iter().enumerate() {
        match File::open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                if !config.quiet && num_files > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }
                let (total_lines, total_bytes) = count_lines_bytes(&filename)?;
                let file = BufReader::new(file);
                let _ = if let Some(bytes) = &config.bytes {
                    print_bytes(file, bytes, total_bytes)
                } else {
                    print_lines(file, &config.lines, total_lines)
                };
            }
        }
    }
    Ok(())
}
fn count_lines_bytes(filename: &str) -> TailResult<(i64, i64)> {
    let file = File::open(filename)?;
    let bytes = &file.metadata().unwrap().len();
    let lines = BufReader::new(file).lines().count();
    Ok((lines as i64, *bytes as i64))
}
fn print_bytes<T>(mut file: T, num_bytes: &TakeValue, total_bytes: i64) -> TailResult<()>
where
    T: Read + Seek,
{
    if let Some(start_index) = get_start_index(num_bytes, total_bytes) {
        file.seek(SeekFrom::Start(start_index))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        if !buffer.is_empty() {
            print!("{}", String::from_utf8_lossy(&buffer));
        }
    }
    Ok(())
}
fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> TailResult<()> {
    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut line_num = 0;
        let mut buf = Vec::new();
        loop {
            let bytes_read = file.read_until(b'\n', &mut buf)?;
            if bytes_read == 0 {
                break;
            }
            if line_num >= start {
                print!("{}", String::from_utf8_lossy(&buf));
            }
            line_num += 1;
            buf.clear();
        }
    }
    Ok(())
}
fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match (take_val.clone(), total) {
        (_, 0) => None,
        (PlusZero, _) => Some(0),
        (TakeNum(0), _) => None,
        (TakeNum(take), total) if take > total => None,
        (TakeNum(take), _total) if take > 0 => Some((take - 1) as u64),
        (TakeNum(take), total) if take + total < 0 => Some(0),
        (TakeNum(take), total) => Some((take + total) as u64),
    }
}
pub fn get_args() -> TailResult<Config> {
    Ok(Config::parse())
}
#[cfg(test)]
mod tests {
    use super::{count_lines_bytes, get_start_index, parse_num, TakeValue::*};
    use pretty_assertions::assert_eq;
    #[test]
    fn test_parse_num_positive() {
        // A leading "+" should result in a positive number
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));
    }

    #[test]
    fn test_parse_num_negative() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));
    }

    #[test]
    fn test_parse_num_explicit_negative() {
        // An explicit "-" value should result in a negative number
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));
    }

    #[test]
    fn test_parse_num_zero() {
        // Zero is Zero
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));
    }

    #[test]
    fn test_parse_num_plus_zero() {
        // Plus zero is special
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);
    }

    #[test]
    fn test_parse_num_boundaries() {
        // Test boundaries
        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));
        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));
        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));
        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));
    }

    #[test]
    fn test_parse_num_float() {
        // A floating-point value is invalid
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");
    }

    #[test]
    fn test_parse_num_noninteger() {
        // Any noninteger string is invalid
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));
        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }
    #[test]
    fn test_get_start_index_empty_file() {
        assert_eq!(get_start_index(&PlusZero, 0), None);
    }

    #[test]
    fn test_get_start_index_plus_zero() {
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));
    }

    #[test]
    fn test_get_start_index_take_zero() {
        assert_eq!(get_start_index(&TakeNum(0), 1), None);
    }

    #[test]
    fn test_get_start_index_take_more_than_available() {
        assert_eq!(get_start_index(&TakeNum(2), 1), None);
    }

    #[test]
    fn test_get_start_index_take_one() {
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
    }

    #[test]
    fn test_get_start_index_take_two() {
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
    }

    #[test]
    fn test_get_start_index_take_three() {
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));
    }

    #[test]
    fn test_get_start_index_take_negative_one() {
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
    }

    #[test]
    fn test_get_start_index_take_negative_two() {
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
    }

    #[test]
    fn test_get_start_index_take_negative_three() {
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));
    }

    #[test]
    fn test_get_start_index_take_negative_more_than_available() {
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }
}
