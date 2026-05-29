use crate::file_ops;
use dissimilar::{Chunk, diff};

use crate::structs::OperationStatus;
use anyhow::Context;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) trait HasCollisionCheck {
    fn collision_check(&self);
}

#[allow(dead_code)]
pub(crate) struct UniqueLinks {
    pub input_file_count: usize,
    pub processing_file_count: usize,
    pub sources: Vec<usize>,
    pub destinations: Vec<usize>,
    pub link_map: HashMap<usize, usize>,
    pub id_to_path: HashMap<usize, &'static str>,
    pub unchanged_sources: Vec<usize>,
    pub missing_sources: Vec<usize>,
    pub sources_metadata: Vec<fs::Metadata>,
}

impl HasCollisionCheck for UniqueLinks {
    fn collision_check(&self) {
        let x: HashSet<_> = self.sources.iter().collect();
        let y: HashSet<_> = self.destinations.iter().collect();
        if y.len() != x.len() {
            panic!("Collision detected: Mapping between input to output files is not one-to-one");
        }
        if x.intersection(&y).next().is_some() {
            panic!("Chain detected: An output file is used as an input elsewhere!");
        }
    }
}

impl UniqueLinks {
    pub(crate) fn new(
        input_files: &[PathBuf],
        source_pattern: &str,
        dest_pattern: &str,
        test_mode: bool,
    ) -> Result<Self, anyhow::Error> {
        let re_sou = Regex::new(source_pattern)
            .with_context(|| anyhow::anyhow!("Irregular pattern provided {}", source_pattern))?;
        let input_file_count: usize = input_files.len();
        let mut string_to_id: HashMap<&'static str, usize> =
            HashMap::with_capacity(2 * input_file_count + 1);
        let mut link_map: HashMap<usize, usize> = HashMap::with_capacity(input_file_count + 1);
        let mut unchanged_sources: Vec<usize> = Vec::with_capacity(input_file_count + 1);
        let mut missing_sources: Vec<usize> = Vec::with_capacity(input_file_count + 1);
        let mut sources_metadata: Vec<fs::Metadata> = Vec::with_capacity(input_file_count + 1);
        let mut sources: Vec<usize> = Vec::with_capacity(input_file_count + 1);
        let mut destinations: Vec<usize> = Vec::with_capacity(input_file_count + 1);
        let mut next_id = 0..;

        if input_file_count > 0 {
            for f in input_files {
                let cow_path = f.to_string_lossy();
                let source_path_str: &'static str = match cow_path {
                    std::borrow::Cow::Borrowed(x) => String::leak(x.to_string()),
                    std::borrow::Cow::Owned(_) => {
                        println!("Non UTF8 characters found in the file name {:?}", f);
                        continue;
                    }
                };
                let source_path: &'static Path = Path::new(source_path_str);
                let source_filename: &'static str = match Path::new(source_path_str).file_name() {
                    Some(x) => match x.to_str() {
                        Some(string) => string,
                        None => continue,
                    },
                    None => continue,
                };

                let dest_filename = re_sou
                    .replace_all(source_filename, dest_pattern)
                    .into_owned();
                let dest_path_str = match source_path.parent() {
                    Some(x) => x
                        .join(Path::new(&dest_filename))
                        .to_string_lossy()
                        .into_owned(),
                    None => continue,
                };
                let dest_path_str: &'static str = String::leak(dest_path_str);
                let s_id = *string_to_id
                    .entry(source_path_str)
                    .or_insert_with(|| next_id.next().unwrap());
                let d_id = *string_to_id
                    .entry(dest_path_str)
                    .or_insert_with(|| next_id.next().unwrap());
                match test_mode {
                    true => {
                        if dest_filename == source_filename {
                            unchanged_sources.push(s_id);
                            continue;
                        } else {
                            link_map.insert(s_id, d_id);
                            sources.push(s_id);
                            destinations.push(d_id);
                        }
                    }
                    false => match fs::metadata(f) {
                        Ok(x) => {
                            if dest_filename == source_filename {
                                unchanged_sources.push(s_id);
                                continue;
                            } else {
                                link_map.insert(s_id, d_id);
                                sources.push(s_id);
                                destinations.push(d_id);
                                sources_metadata.push(x);
                            }
                        }
                        Err(x) => match x.kind() {
                            std::io::ErrorKind::NotFound => {
                                missing_sources.push(s_id);
                                continue;
                            }
                            _ => continue,
                        },
                    },
                }
            }

            let mut id_to_path: HashMap<usize, &'static str> =
                HashMap::with_capacity(2 * input_file_count + 1);
            for (path, id) in string_to_id.into_iter() {
                id_to_path.insert(id, path);
            }
            Ok(Self {
                input_file_count,
                processing_file_count: sources.len(),
                sources_metadata,
                sources,
                destinations,
                missing_sources,
                link_map,
                id_to_path,
                unchanged_sources,
            })
        } else {
            std::process::exit(0);
        }
    }

    /// Injects red ANSI escape sequences into matching sub-slices of a text string
    fn highlight_path_diff(old: &str, new: &str, color: crate::Color) -> String {
        // 1. Calculate the semantic character-level diff
        let chunks = diff(old, new);
        let mut result = String::new();

        // 2. Loop through differences and inject color wrappers
        for chunk in chunks {
            match chunk {
                Chunk::Equal(text) => result.push_str(text),

                Chunk::Delete(text) => {
                    result.push_str(color.as_str());
                    result.push_str(text);
                    result.push_str(crate::Color::Default.as_str());
                }

                Chunk::Insert(_) => {}
            }
        }
        result
    }
    pub fn print_graph(&self, display_paths: bool) {
        for (sid, did) in &self.link_map {
            let source_path = self.id_to_path.get(sid).unwrap();
            let dest_path = self.id_to_path.get(did).unwrap();
            let colored_source =
                UniqueLinks::highlight_path_diff(source_path, dest_path, crate::Color::Red);
            let colored_dest =
                UniqueLinks::highlight_path_diff(dest_path, source_path, crate::Color::Green);

            if display_paths {
                println!("{}  -->  {}", colored_source, colored_dest);
            } else {
                println!("{}  -->  {}", sid, did);
            }
        }
    }

    pub fn get_err_code(err: &anyhow::Error) -> (String, String) {
        let err_string = format!("{:#}", err);
        match err_string.find(':') {
            Some(index) => (
                err_string[..index].trim().to_string(),
                err_string[..index].trim().to_string(),
            ),
            None => ("UNKNOWN".to_string(), err_string.trim().to_string()),
        }
    }

    pub fn copy(&self) -> OperationStatus {
        let mut status = OperationStatus::new(self.processing_file_count);
        for (sid, did) in &self.link_map {
            let source_path = Path::new(self.id_to_path.get(sid).unwrap());
            let dest_path = Path::new(self.id_to_path.get(did).unwrap());
            match file_ops::atomic_copy(source_path, dest_path) {
                Ok(_) => {
                    status
                        .files
                        .push((self.id_to_path[sid], self.id_to_path[did]));
                    status.status.push((
                        "Success".to_string(),
                        "Operation successfully completed".to_string(),
                    ));
                }
                Err(err) => {
                    println!("{:#}", err);
                    let err_code = UniqueLinks::get_err_code(&err);
                    status
                        .files
                        .push((self.id_to_path[sid], self.id_to_path[did]));
                    status.status.push(err_code);
                }
            }
        }
        status
    }
    pub(crate) fn rename(&self) -> OperationStatus {
        let mut status = OperationStatus::new(self.processing_file_count);
        for (sid, did) in &self.link_map {
            let source_path = Path::new(self.id_to_path.get(sid).unwrap());
            let dest_path = Path::new(self.id_to_path.get(did).unwrap());
            match file_ops::atomic_rename(source_path, dest_path) {
                Ok(_) => {
                    status
                        .files
                        .push((self.id_to_path[sid], self.id_to_path[did]));
                    status.status.push((
                        "Success".to_string(),
                        "Operation successfully completed".to_string(),
                    ));
                }
                Err(err) => {
                    println!("{:#}", err);
                    let err_code = UniqueLinks::get_err_code(&err);
                    status
                        .files
                        .push((self.id_to_path[sid], self.id_to_path[did]));
                    status.status.push(err_code);
                }
            }
        }
        status
    }
}

#[cfg(test)]
#[path = "./tests/network_file_test.rs"]
mod network_file_test;
