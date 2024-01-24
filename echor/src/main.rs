use clap::{arg, command, Parser};
// use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version = "0.1.0", author = "dkuku", about = "Rust echo")]
pub struct Config {
    /// Files to cat
    #[arg(name = "text", required = true)]
    text: Vec<String>,
    /// "Do not print newline"
    #[arg(short = 'n', long)]
    omit_newline: bool,
}
///
///    /// Input text
///    #[arg(short = 'n', long, default_value_t = 10, value_parser=parse_line_count)]
///    lines: usize,
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();
    let ending = if config.omit_newline { "" } else { "\n" };
    if config.text.is_empty() {
        Ok(())
    } else {
        println!(
            "{}{}",
            config
                .text
                .into_iter()
                .map(|v| v)
                .collect::<Vec<_>>()
                .join(" "),
            ending
        );
        Ok(())
    }
}
