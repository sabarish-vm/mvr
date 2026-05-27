use clap::{Arg, ArgAction, Command};
use regex::Regex;
use std::cell::Cell;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
pub(crate) struct Opts {
    pub(crate) files: Vec<PathBuf>,
    pub(crate) copy_bool: bool,
    pub(crate) move_bool: bool,
    pub(crate) source_pattern: String,
    pub(crate) dest_pattern: String,
}

pub(crate) struct Success<'a> {
    pub(crate) files: Vec<&'a PathBuf>,
}

pub(crate) struct Failed<'a> {
    pub(crate) files: Vec<&'a PathBuf>,
}

impl<'a> Success<'a> {
    pub(crate) fn new() -> Self {
        let files: Vec<&'a PathBuf> = Vec::new();
        Self { files }
    }
}

impl<'a> Failed<'a> {
    pub(crate) fn new() -> Self {
        let files: Vec<&'a PathBuf> = Vec::new();
        Self { files }
    }
}

pub(crate) enum IsIn {
    Source,
    Destination,
}
