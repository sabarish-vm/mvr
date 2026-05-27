#![allow(dead_code)]
#![allow(unused)]

mod argparse_clap;
mod errors;
mod network;
mod structs;

use clap::{Arg, ArgAction, Command};
use regex::Regex;
use std::cell::Cell;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::io;
use std::panic;

use std::boxed::Box;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;

use crate::argparse_clap::argparse;
use crate::network::{Edge, Node};
use crate::structs::{Failed, IsIn, Success};

// Ensure you close the braces and parentheses properly

fn main() -> io::Result<()> {
    errors::panic_hook_fxn();

    let opts = argparse();

    //     // let p1 = DirectionalNode {};
    //     if opts.move_bool {
    //         // println!("{source_filename:?} --> {:?}", dest_filename);
    //         // match fs::rename(f, t1) {
    //         //     Ok(_) => success.files.push(f),
    //         //     Err(_) => failed.files.push(f),
    //         // }
    //     } else if opts.copy_bool {
    //         // match fs::copy(f, t1) {
    //         //     Ok(_) => success.files.push(f),
    //         //     Err(_) => failed.files.push(f),
    //         // }
    //     }
    // };
    Ok(())
}
