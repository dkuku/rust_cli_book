use clap::{command, ArgGroup, Parser};
use core::num::NonZeroUsize;
use core::result::Result;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::recognize,
    multi::{many0, many1, separated_list0},
    sequence::{terminated, tuple},
    IResult,
};
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::ops::Range;

type PositionList = Vec<Range<usize>>;
type CutResult<T> = Result<T, Box<dyn Error>>;
#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}
#[derive(Parser, Debug)]
#[command(version, author, about = "Rust uniq")]
#[clap(group(ArgGroup::new("filters")
    .required(true)
    .args(&["chars", "bytes", "fields"])
))]
pub struct Config {
    /// Input file
    #[arg(name = "FILES", default_value = "")]
    files: Vec<String>,
    /// Field delimiter
    #[arg(short, long = "delim", default_value = "\t", value_parser = parse_delimiter)]
    delimiter: u8,
    /// Selected characters
    #[arg(short, long, default_value = None, allow_hyphen_values(true), value_parser = parse_position)]
    chars: Option<PositionList>,
    /// Selected bytes
    #[arg(short, long, default_value = None, allow_hyphen_values(true),  value_parser = parse_position)]
    bytes: Option<PositionList>,
    /// Selected fields
    #[arg(short, long, default_value = None, allow_hyphen_values(true), value_parser = parse_position)]
    fields: Option<PositionList>,
}
pub fn run(config: Config) -> CutResult<()> {
    let Config {
        files,
        delimiter,
        chars,
        bytes,
        fields,
    } = config;
    for filename in files {
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                if let Some(byte_pos) = &bytes {
                    for line in file.lines() {
                        println!("{}", extract_bytes(&line?, byte_pos));
                    }
                } else if let Some(char_pos) = &chars {
                    for line in file.lines() {
                        println!("{}", extract_chars(&line?, char_pos));
                    }
                } else if let Some(field_pos) = &fields {
                    let mut reader = ReaderBuilder::new()
                        .delimiter(delimiter)
                        .has_headers(false)
                        .from_reader(file);
                    let mut wtr = WriterBuilder::new()
                        .delimiter(delimiter)
                        .from_writer(io::stdout());
                    for record in reader.records() {
                        let record = record?;
                        wtr.write_record(extract_fields(&record, field_pos))?;
                    }
                } else {
                    unimplemented!()
                }
            }
        }
    }

    Ok(())
}
fn extract_bytes(line: &str, chars_pos: &[Range<usize>]) -> String {
    let mut buffer = Vec::new();
    for cp in chars_pos.iter() {
        line.bytes().enumerate().for_each(|(idx, c)| {
            if cp.contains(&idx) {
                buffer.push(c)
            }
        });
    }
    String::from_utf8_lossy(&buffer).to_string()
}
fn extract_chars(line: &str, chars_pos: &[Range<usize>]) -> String {
    let mut buffer = String::new();
    for cp in chars_pos.iter() {
        line.chars().enumerate().for_each(|(idx, c)| {
            if cp.contains(&idx) {
                buffer.push(c)
            }
        });
    }
    buffer
}
fn extract_fields<'a>(line: &'a StringRecord, chars_pos: &[Range<usize>]) -> Vec<&'a str> {
    let mut buffer = Vec::new();
    for cp in chars_pos.iter() {
        line.into_iter().enumerate().for_each(|(idx, c)| {
            if cp.contains(&idx) {
                buffer.push(c)
            }
        });
    }
    buffer
}
pub fn get_args() -> CutResult<Config> {
    Ok(Config::parse())
}
fn open(filename: &str) -> CutResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
fn parse_delimiter(delim: &str) -> Result<u8, String> {
    let mut delim_iter = delim.bytes();
    let result = delim_iter.next().ok_or(format_delim_err(delim));
    match delim_iter.next() {
        None => result,
        Some(_) => Err(format_delim_err(delim)),
    }
}
fn parse_position(input: &str) -> Result<PositionList, String> {
    let inputs = match separated_list0(tag(","), range_input)(input).map_err(format_val_err) {
        Ok(("", inputs)) => Ok(inputs),
        Ok((result, _inputs)) => Err(format_val_err(result)),
        Err(e) => Err(e),
    };
    let result: Result<Vec<Range<usize>>, String> = inputs?
        .iter()
        .map(|parsed| parsed_to_range(parsed))
        .collect();
    result
}
fn parsed_to_range(parsed: &str) -> Result<Range<usize>, String> {
    match parsed.split('-').collect::<Vec<_>>()[..] {
        ["", to] => Ok(0..parse(to)?),
        [from, ""] => Ok((parse(from)? - 1)..usize::MAX),
        [result] => {
            let pos = parse(result)?;
            Ok((pos - 1)..pos)
        }
        [from, to] => {
            let from = parse(from)?;
            let to = parse(to)?;
            if from > to {
                return Err(format_range_err(from, to));
            }
            let range = (from - 1)..to;
            Ok(range)
        }
        _ => Err(format_val_err(parsed)),
    }
}
fn parse(input: &str) -> Result<usize, String> {
    match input.parse::<NonZeroUsize>() {
        Err(_) => Err(format_val_err(input)),
        Ok(v) => Ok(v.into()),
    }
}
fn format_range_err(from: usize, to: usize) -> String {
    format!(
        "First number in range ({}) must be lower than second number ({})",
        from, to
    )
}
fn format_delim_err(val: impl Display) -> String {
    format!("Invalid delimiter: \"{}\"", val)
}
fn format_val_err(val: impl Display) -> String {
    format!("illegal list value: \"{}\"", val)
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
    use super::{extract_bytes, extract_chars, extract_fields, parse_position};
    use csv::StringRecord;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_position0() {
        // The empty string is an error
        assert!(parse_position("0").is_err());
    }
    #[test]
    fn test_parse_position01() {
        let res = parse_position("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);
    }
    #[test]
    fn test_parse_position_plus1() {
        // A leading "+" is an error
        let res = parse_position("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);
    }
    #[test]
    fn test_parse_position_plus12() {
        let res = parse_position("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"",);
    }
    #[test]
    fn test_parse_position_1plus2() {
        let res = parse_position("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+2\"",);
    }
    #[test]
    fn test_parse_position_non_number() {
        // Any non-number is an error
        let res = parse_position("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);
    }
    #[test]
    fn test_parse_position_number_non_number() {
        let res = parse_position("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \",a\"",);
    }
    #[test]
    fn test_parse_position_number_non_number_range() {
        let res = parse_position("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);
    }
    #[test]
    fn test_parse_position_non_number_number_range() {
        let res = parse_position("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);
    }
    #[test]
    fn test_parse_position_empty_range() {
        // Wonky ranges
        let res = parse_position("-");
        assert!(res.is_err());
    }
    #[test]
    fn test_parse_position_comma() {
        let res = parse_position(",");
        assert!(res.is_err());
    }
    #[test]
    fn test_parse_position_nothing_after_comma() {
        let res = parse_position("1,");
        assert!(res.is_err());
    }
    #[test]
    fn test_parse_position_invalid_range_int() {
        let res = parse_position("1-1-1");
        assert!(res.is_err());
    }
    #[test]
    fn test_parse_position_invalid_range_char() {
        let res = parse_position("1-1-a");
        assert!(res.is_err());
    }
    #[test]
    fn test_parse_position_invalid_range_reverse() {
        let res = parse_position("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );
    }
    #[test]
    fn test_parse_position() {
        // Zero is an error
        let res = parse_position("");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![]);
    }
    #[test]
    fn test_parse_position_ok_single() {
        // All the following are acceptable
        let res = parse_position("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
    }
    #[test]
    fn test_parse_position_ok_single_with_0() {
        let res = parse_position("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
    }
    #[test]
    fn test_parse_position_ok_comma() {
        let res = parse_position("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
    }
    #[test]
    fn test_parse_position_ok_comma_with_0() {
        let res = parse_position("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
    }
    #[test]
    fn test_parse_position_ok_range() {
        let res = parse_position("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
    }
    #[test]
    fn test_parse_position_ok_range_same() {
        let res = parse_position("1-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
    }
    #[test]
    fn test_parse_position_ok_range_with_0() {
        let res = parse_position("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
    }
    #[test]
    fn test_parse_position_ok_multiple_x() {
        let res = parse_position("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);
    }
    #[test]
    fn test_parse_position_ok_multiple_xx() {
        let res = parse_position("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }
    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }
}
