use crate::TakeValue::*;
use clap::{arg, command, Parser};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

type TailResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Clone)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}
impl ToString for TakeValue {
    fn to_string(&self) -> String {
        match self {
            TakeValue::PlusZero => "+0".to_string(),
            TakeValue::TakeNum(n) => n.to_string(),
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
    println!("{:#?}", config);
    Ok(())
}
fn open(filename: &str) -> TailResult<Box<dyn BufRead>> {
    Ok(Box::new(BufReader::new(File::open(filename)?)))
}
pub fn get_args() -> TailResult<Config> {
    Ok(Config::parse())
}
#[cfg(test)]
mod tests {
    use super::{parse_num, TakeValue::*};
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
}
