use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

use crate::structs::Opts;

pub(crate) fn argparse() -> Opts {
    let matches = Command::new("File stat")
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
        .arg(Arg::new("paths").action(ArgAction::Append).required(true))
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
        .get_matches();
    let files = matches
        .get_many::<String>("paths")
        .unwrap()
        .map(|s| s.into())
        .collect::<Vec<PathBuf>>();
    let m_bool: bool = matches.get_flag("move");
    let c_bool: bool = matches.get_flag("copy");
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
        files,
        source_pattern: sp,
        dest_pattern: dp,
    }
}
