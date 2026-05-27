use clap::{Arg, ArgAction, Command, builder::NonEmptyStringValueParser};
use std::path::PathBuf;

use crate::structs::Opts;

fn build_parser() -> Command {
    Command::new("File stat")
        .author("Sabarish, github.com/sabarish-vm")
        .about("An alternative to stat command written in rust an os-independent solution")
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
                .help("Flag to enable copying")
                .action(ArgAction::SetTrue)
                .default_value("false"),
        )
        .arg(
            Arg::new("move")
                .short('m')
                .help("Flag to enable moving")
                .action(ArgAction::SetTrue)
                .default_value("false"),
        )
        .arg(
            Arg::new("force")
                .short('f')
                .help("Flag to force the changes")
                .action(ArgAction::SetTrue)
                .default_value("false"),
        )
}

pub(crate) fn argparse() -> Opts {
    let matches = build_parser().get_matches();
    let files = matches
        .get_many::<String>("paths")
        .unwrap()
        .map(|s| s.into())
        .collect::<Vec<PathBuf>>();
    let m_bool: bool = matches.get_flag("move");
    let c_bool: bool = matches.get_flag("copy");
    let f_bool: bool = matches.get_flag("force");
    let sp = matches
        .get_one::<String>("source_pattern")
        .unwrap()
        .to_owned();
    let dp = matches
        .get_one::<String>("dest_pattern")
        .unwrap()
        .to_owned();
    Opts {
        move_bool: m_bool,
        copy_bool: c_bool,
        force_run: f_bool,
        files,
        source_pattern: sp,
        dest_pattern: dp,
    }
}

#[cfg(test)]
mod clap_argparse {
    use super::*;
    #[test]
    #[should_panic]
    fn test_empty_set() {
        {
            let args = vec!["-m", "(.*)", "$1.bak", ""];
            let parser = build_parser();
            parser.try_get_matches_from(args).unwrap();
        }
        {
            let args = vec!["-m", "(.*)", "$1.bak", " "];
            let parser = build_parser();
            parser.try_get_matches_from(args).unwrap();
        }
    }
}
