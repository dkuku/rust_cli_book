use clap::{arg, command, Parser};
use nom::{
    branch::alt,
    character::complete::{char, one_of},
    combinator::{opt, recognize},
    multi::{many0, many1},
    sequence::{terminated, tuple},
    IResult,
};
use std::error::Error;
use std::ops::Range;

type CutResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;
#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}
#[derive(Parser, Debug)]
#[command(version = "0.1.0", author = "dkuku", about = "Rust uniq")]
pub struct Config {
    /// Input file
    #[arg(name = "FILE", default_value = "")]
    file: String,
    /// Field delimiter
    #[arg(short, long = "delim", default_value = "\t", value_parser = parse_delimiter)]
    delimiter: u8,
    /// Selected characters
    #[arg(short, long, default_value = "-", allow_hyphen_values(true), value_parser = parse_position)]
    chars: PositionList,
    /// Selected bytes
    #[arg(short, long, default_value = "-", allow_hyphen_values(true),  value_parser = parse_position)]
    bytes: PositionList,
    /// Selected fields
    #[arg(short, long, default_value = "-", allow_hyphen_values(true), value_parser = parse_position)]
    field: PositionList,
}
pub fn run(config: Config) -> CutResult<()> {
    dbg!(config);
    Ok(())
}
pub fn get_args() -> CutResult<Config> {
    Ok(Config::parse())
}
fn parse_delimiter(delim: &str) -> Result<u8, String> {
    delim
        .bytes()
        .next()
        .ok_or(String::from("Invalid delimiter"))
}
fn parse_position(input: &str) -> Result<PositionList, String> {
    let _ = match range_input(input) {
        Ok(("", result)) if result.starts_with("-") => Ok(vec![0..2]),
        Ok(("", result)) if result.ends_with("-") => Ok(vec![1..2]),
        Ok(("", result)) if result.contains("-") => Ok(vec![1..2]),
        Ok(("", _int)) => Ok(vec![1..2]),
        Ok((rest, _result)) => Err(format!("can't parse: {}", rest)),
        Err(_e) => Err("Invalid range".to_string()),
    };
    todo!("use lexer to get the values")
}
fn range_input(input: &str) -> IResult<&str, &str> {
    alt((
        // Case two: 42-42
        recognize(tuple((decimal, char('-'), decimal))),
        // Case one: 42-
        recognize(tuple((decimal, char('-')))),
        // Case one: -42
        recognize(tuple((char('-'), decimal))),
        // Case one: 42
        recognize(decimal),
    ))(input)
}

fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
}
#[cfg(test)]
mod unit_tests {
    //use super::{extract_bytes, extract_chars, extract_fields, parse_pos};
    use super::parse_position;
    //    use csv::StringRecord;

    #[test]
    fn test_parse_position() {
        // The empty string is an error
        assert!(parse_position("").is_err());

        // Zero is an error
        let res = parse_position("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        let res = parse_position("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        // A leading "+" is an error
        let res = parse_position("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);

        let res = parse_position("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"",);

        let res = parse_position("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"",);

        // Any non-number is an error
        let res = parse_position("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_position("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_position("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"",);

        let res = parse_position("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);

        // Wonky ranges
        let res = parse_position("-");
        assert!(res.is_err());

        let res = parse_position(",");
        assert!(res.is_err());

        let res = parse_position("1,");
        assert!(res.is_err());

        let res = parse_position("1-");
        assert!(res.is_err());

        let res = parse_position("1-1-1");
        assert!(res.is_err());

        let res = parse_position("1-1-a");
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_position("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_position("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_position("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_position("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_position("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_position("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_position("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_position("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_position("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_position("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    //    #[test]
    //    fn test_extract_fields() {
    //        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
    //        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
    //        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
    //        assert_eq!(
    //            extract_fields(&rec, &[0..1, 2..3]),
    //            &["Captain", "12345"]
    //        );
    //        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
    //        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    //    }
    //
    //    #[test]
    //    fn test_extract_chars() {
    //        assert_eq!(extract_chars("", &[0..1]), "".to_string());
    //        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
    //        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
    //        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
    //        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
    //        assert_eq!(
    //            extract_chars("ábc", &[0..1, 1..2, 4..5]),
    //            "áb".to_string()
    //        );
    //    }
    //
    //    #[test]
    //    fn test_extract_bytes() {
    //        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
    //        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
    //        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
    //        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
    //        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
    //        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    //    }
}
