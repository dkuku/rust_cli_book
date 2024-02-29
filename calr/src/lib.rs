use ansi_term::Style;
use chrono::{Datelike, Days, Local, NaiveDate};

use clap::{arg, command, Parser};
use std::error::Error;

type CalResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(version, author, about)]
pub struct Config {
    /// Year (1-9999)
    #[arg(name = "YEAR", default_value_t=Local::now().year(), value_parser=parse_year)]
    year: i32,
    /// Month name or number (1-12)
    #[arg(short, value_parser=parse_month)]
    month: Option<u32>,
    #[arg(skip)]
    today: NaiveDate,
    /// Show whole current year
    #[arg(short='y', long="year", name="SHOW_YEAR", conflicts_with_all = &["YEAR", "month"])]
    show_current_year: bool,
}
const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
pub fn run(config: Config) -> CalResult<()> {
    let config = Config {
        today: Local::now().naive_local().into(),
        ..config
    };
    format_month(
        config.year,
        config.month.unwrap(),
        config.show_current_year,
        config.today,
    )
    .iter()
    .for_each(|row| println!("{}", row));
    Ok(())
}
fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let mut month_vec = vec![
        format_month_header(year, month, print_year),
        format_days_header(),
    ];
    let mut days_vec = format_days(year, month, today);
    month_vec.append(&mut days_vec);
    month_vec
}
fn format_month_header(year: i32, month: u32, print_year: bool) -> String {
    let month_name = MONTH_NAMES.get(month as usize - 1).unwrap().to_string();
    if print_year {
        format!("{:^20}  ", format!("{} {}", month_name, year))
    } else {
        format!("{:^20}  ", month_name)
    }
}
fn format_days_header() -> String {
    "Su Mo Tu We Th Fr Sa  ".to_string()
}
fn format_days(year: i32, month: u32, today: NaiveDate) -> Vec<String> {
    let mut all_cells: Vec<String> = vec!["  ".to_string(); 42];
    let first_day_in_month_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let first_row_padding_days = first_day_in_month_date.weekday().num_days_from_sunday();
    let last_day_in_month_date = last_day_in_month(year, month);
    let days_in_month = last_day_in_month_date.day();
    for day in 1..=days_in_month {
        let formatted_day = if day == today.day()
            && today >= first_day_in_month_date
            && today <= last_day_in_month_date
        {
            Style::new()
                .reverse()
                .paint(format!("{:>2}", day))
                .to_string()
        } else {
            format!("{:>2}", day)
        };
        all_cells[(day + first_row_padding_days - 1) as usize] = formatted_day
    }
    all_cells
        .chunks(7)
        .map(|chunk| chunk.join(" ") + "  ")
        .collect()
}
fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap()
    .checked_sub_days(Days::new(1))
    .unwrap()
}
fn parse_month(val: &str) -> Result<u32, String> {
    match val.parse::<u32>() {
        Ok(val) if (1..=12).contains(&val) => Ok(val),
        _month => {
            let lower = &val.to_lowercase();
            if let Some(month) = MONTH_NAMES.iter().enumerate().find_map(|(i, name)| {
                if name.to_lowercase().starts_with(lower) {
                    Some(i as u32 + 1)
                } else {
                    None
                }
            }) {
                Ok(month)
            } else {
                Err(format!("month '{}' not in the range 1 through 12", val))
            }
        }
    }
}
fn parse_year(val: &str) -> Result<i32, String> {
    match val.parse::<i32>() {
        Ok(val) if (1..9999).contains(&val) => Ok(val),
        _ => Err(format!("year '{}' not in the range 1 through 9999", val)),
    }
}
pub fn get_args() -> CalResult<Config> {
    Ok(Config::parse())
}
#[cfg(test)]
mod tests {
    use super::{
        format_days, format_month, format_month_header, last_day_in_month, parse_month, parse_year,
    };
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);
        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);
        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);
        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month '0' not in the range 1 through 12"
        );
        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month '13' not in the range 1 through 12"
        );
        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month 'foo' not in the range 1 through 12"
        );
    }
    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);
        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);
        let res = parse_year("0");
        assert_eq!(
            res.unwrap_err().to_string(),
            "year '0' not in the range 1 through 9999"
        );
        let res = parse_year("10000");
        assert_eq!(
            res.unwrap_err().to_string(),
            "year '10000' not in the range 1 through 9999"
        );
        let res = parse_year("foo");
        assert!(res.is_err());
    }
    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);
        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);
        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }
    #[test]
    fn test_format_month_header() {
        let february = "   February 2020      ";
        assert_eq!(format_month_header(2020, 2, true), february);
        let may = "        May           ";
        assert_eq!(format_month_header(2020, 5, false), may);
        let april = "     April 2021       ";
        assert_eq!(format_month_header(2021, 4, true), april);
    }
    #[test]
    fn test_format_days() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_days(2020, 2, today), leap_february);
        let may = vec![
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_days(2020, 5, today), may);
        let april_hl = vec![
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_days(2021, 4, today), april_hl);
    }
    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
    }
}
