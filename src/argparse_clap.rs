use anyhow::Context;
use clap::{Arg, ArgAction, Command, builder::NonEmptyStringValueParser};
use std::path::PathBuf;

use crate::structs::Opts;

fn build_parser() -> Command {
    Command::new("File stat")
        .author("Sabarish, github.com/sabarish-vm")
        .about("An alternative to zmv command from zsh written in rust")
        .arg_required_else_help(true)
        .arg(
            Arg::new("source_pattern")
                .action(ArgAction::Set)
                .required(true),
        )
        .arg(
            Arg::new("dest_pattern")
                .action(ArgAction::Set)
                .required(true),
        )
        .arg(
            Arg::new("paths")
                .action(ArgAction::Append)
                .required(true)
                .value_parser(NonEmptyStringValueParser::new()),
        )
        .arg(
            Arg::new("copy")
                .short('c')
                .long("copy")
                .help("Flag to enable copying")
                .action(ArgAction::SetTrue)
                .default_value("false"),
        )
        .arg(
            Arg::new("move")
                .short('m')
                .long("move")
                .help("Flag to enable moving")
                .action(ArgAction::SetTrue)
                .default_value("false"),
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .help("Flag to force the changes")
                .action(ArgAction::SetTrue)
                .default_value("false"),
        )
        .arg(
            Arg::new("log")
                .short('l')
                .long("log")
                .help("Flag to enable logging of changes to json file")
                .action(ArgAction::SetTrue)
                .default_value("false"),
        )
}

pub fn argparse() -> Result<Opts, anyhow::Error> {
    let matches = build_parser().get_matches();
    let files = matches
        .get_many::<String>("paths")
        .with_context(|| "Error while parsing the passed input paths")?
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>();
    let m_bool: bool = matches.get_flag("move");
    let c_bool: bool = matches.get_flag("copy");
    let f_bool: bool = matches.get_flag("force");
    let log_bool: bool = matches.get_flag("log");
    let sp = matches
        .get_one::<String>("source_pattern")
        .with_context(|| "Cli arg parsing error while parsing the pattern for the source file")?
        .to_owned();
    let dp = matches
        .get_one::<String>("dest_pattern")
        .with_context(
            || "Cli arg parsing error while parsing the pattern for the destination file",
        )?
        .to_owned();
    Ok(Opts {
        move_bool: m_bool,
        copy_bool: c_bool,
        force_run: f_bool,
        log_bool,
        files,
        source_pattern: sp,
        dest_pattern: dp,
    })
}

#[cfg(test)]
#[path = "./tests/argparse_test.rs"]
mod argparse_test;
