use clap::{command, value_parser, Arg, ArgAction};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = command!()
        .version("0.1.0")
        .author("dkuku")
        .about("Rust echo")
        .args([
            Arg::new("text")
                .value_name("TEXT")
                .value_parser(value_parser!(PathBuf))
                .help("Input text")
                .action(ArgAction::Append)
                .required(true),
            Arg::new("omit_newline")
                .short('n')
                .action(ArgAction::SetTrue)
                .help("Do not print newline"),
        ])
        .get_matches();

    let ending = if matches.get_flag("omit_newline") {
        ""
    } else {
        "\n"
    };
    match matches.get_raw("text") {
        Some(text) => {
            println!(
                "{}{}",
                text.into_iter()
                    .map(|v| v.to_str())
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" "),
                ending
            );
        }
        None => (),
    }
    Ok(())
}
