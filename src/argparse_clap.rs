use anyhow::Context;
use clap::{Arg, ArgAction, ArgGroup, Command, builder::NonEmptyStringValueParser};
use clap_complete::Shell;
use std::io::IsTerminal;
use std::path::PathBuf;

use crate::structs::Opts;

fn build_parser() -> Command {
    Command::new("mvr")
        .author("Sabarish, github.com/sabarish-vm")
        .about("An alternative to zmv command from zsh written in rust")
        .arg_required_else_help(true)
        .arg(Arg::new("source_pattern").action(ArgAction::Set))
        .arg(Arg::new("dest_pattern").action(ArgAction::Set))
        .arg(
            Arg::new("paths")
                .action(ArgAction::Append)
                .value_parser(NonEmptyStringValueParser::new()),
        )
        .arg(
            Arg::new("copy")
                .short('c')
                .long("copy")
                .help("Flag to enable copying")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("move")
                .short('m')
                .long("move")
                .help("Flag to enable moving")
                .action(ArgAction::SetTrue),
        )
        .group(
            ArgGroup::new("patterns_group")
                .args(["source_pattern", "dest_pattern"])
                .multiple(true),
        )
        .group(
            ArgGroup::new("operation_group")
                .args(["copy", "move"])
                .multiple(false),
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .help("Flag to force the changes")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("log")
                .short('l')
                .long("log")
                .help("Flag to enable logging of changes to json file")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("completions")
                .long("completions")
                .help("Output the SHELL completions")
                .conflicts_with("operation_group")
                .conflicts_with("patterns_group")
                .value_parser(clap::builder::EnumValueParser::<Shell>::new()),
        )
}

pub fn argparse() -> Result<(Opts, Option<Shell>), anyhow::Error> {
    let matches = build_parser().get_matches();

    // If --completions was passed, return early without requiring main args
    if let Some(shell) = matches.get_one::<Shell>("completions").copied() {
        return Ok((Opts::default(), Some(shell)));
    }

    let files = matches
        .get_many::<String>("paths")
        .with_context(
            || "Error: No input files provided. Usage: mvr <pattern> <replacement> <files...>",
        )?
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
    Ok((
        Opts {
            move_bool: m_bool,
            copy_bool: c_bool,
            force_run: f_bool,
            log_bool,
            files,
            source_pattern: sp,
            dest_pattern: dp,
        },
        None,
    ))
}

pub(crate) fn generate_completions(shell: Shell) {
    let mut cmd = build_parser();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
    // Ensure output ends with a newline when stdout is not a terminal
    if !std::io::stdout().is_terminal() {
        println!();
    }
}

#[cfg(test)]
#[path = "./tests/argparse_test.rs"]
mod argparse_test;
