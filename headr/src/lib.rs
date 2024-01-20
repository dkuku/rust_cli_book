use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

type HeadResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn run(config: Config) -> HeadResult<()> {
    let files_count = config.files.len();
    let multiple_files = files_count > 1;
    for (index, filename) in config.files.into_iter().enumerate() {
        // display filename when multiple files
        if multiple_files {
            println!("==> {} <==", &filename);
        }
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", &filename, err),
            Ok(mut file) => show_file_content(&mut file, config.lines, config.bytes)?,
        }
        // add empty line between filenames
        if multiple_files && index < files_count - 1 {
            println!("");
        }
    }
    Ok(())
}
fn show_file_content(
    file: &mut Box<dyn BufRead>,
    lines: usize,
    bytes: Option<usize>,
) -> HeadResult<()> {
    if let Some(num_bytes) = bytes {
        let bytes: Result<Vec<_>, _> = file.bytes().take(num_bytes).collect();
        print!("{}", String::from_utf8_lossy(&bytes?));
    } else {
        let mut line = String::new();
        for _ in 0..lines {
            let bytes = file.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            print!("{}", line);
            line.clear();
        }
    }
    Ok(())
}
fn open(filename: &str) -> HeadResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
pub fn get_args() -> HeadResult<Config> {
    let matches = App::new("headr")
        .version("0.1.0")
        .author("dkuku")
        .about("Rust head")
        .arg(
            Arg::with_name("filename")
                .value_name("FILENAME")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .value_name("LINES")
                .short("n")
                .long("lines")
                .help("Numbers of lines")
                .default_value("10")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .short("c")
                .long("bytes")
                .conflicts_with("lines")
                .help("Numbers of bytes")
                .takes_value(true),
        )
        .get_matches();

    let lines = matches
        .value_of("lines")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?;
    let bytes = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files: matches.values_of_lossy("filename").unwrap(),
        lines: lines.unwrap(),
        bytes,
    })
}

fn parse_positive_int(val: &str) -> HeadResult<usize> {
    match val.parse() {
        Ok(val) if val > 0 => Ok(val),
        _ => Err(val.into()),
    }
}
#[test]
fn test_parse_valid_positive_int() {
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);
}
#[test]
fn test_parse_invalid_positive_int() {
    let res = parse_positive_int("-1");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "-1".to_string());

    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());
}
