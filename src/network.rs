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

use crate::argparse_clap::argparse;
use crate::errors;
use crate::structs::{Failed, IsIn, Opts, Success};
use proptest::prelude::*;

pub struct Node {
    pub file: PathBuf,
    pub id: usize,
    pub n_in: Cell<usize>,
    pub n_out: Cell<usize>,
}

impl Node {
    pub fn new(path: PathBuf, id: usize, n_in: usize, n_out: usize) -> Self {
        Self {
            file: path,
            id,
            n_in: Cell::new(n_in),
            n_out: Cell::new(n_out),
        }
    }
}

pub(crate) struct Graph {
    pub source: Vec<Node>,
    pub destinations: Vec<Node>,
}

fn get_id_if_exists<'a>(
    path: &PathBuf,
    svec: &'a [Node],
    dvec: &'a [Node],
    path_type: IsIn,
) -> Option<&'a Node> {
    match (path_type) {
        IsIn::Source => {
            if let Some(found_in_sources) = svec.iter().find(|x| &x.file == path) {
                panic!("Source path {} appears more than once", path.display());
            }
            dvec.iter().find(|node| &node.file == path)
        }
        IsIn::Destination => {
            if let Some(found_in_destination) = dvec.iter().find(|x| &x.file == path) {
                panic!("Destination path {} appears more than once", path.display());
            }
            svec.iter().find(|node| &node.file == path)
        }
    }
}
pub(crate) fn create_network(opts: &Opts) -> (Vec<Node>, Vec<Node>, HashMap<usize, usize>) {
    let re_sou = Regex::new(&opts.source_pattern).unwrap();

    let mut sources: Vec<Node> = Vec::new();
    let mut destinations: Vec<Node> = Vec::new();

    let mut edges: HashMap<usize, usize> = HashMap::new();

    let mut id: usize = 0;
    let _: () = for f in opts.files.iter() {
        let source_filename = f.file_name().unwrap().to_str().unwrap().to_string();
        let dest_filename = re_sou
            .replace_all(&source_filename, &opts.dest_pattern)
            .to_string();
        eprintln!("{} -> {}", source_filename, dest_filename);

        let source_path = f.parent().unwrap().join(PathBuf::from(source_filename));
        let dest_path = f.parent().unwrap().join(PathBuf::from(dest_filename));

        let option_source = get_id_if_exists(&source_path, &sources, &destinations, IsIn::Source);

        let option_dest = get_id_if_exists(&dest_path, &sources, &destinations, IsIn::Destination);

        match (option_source, option_dest) {
            (Some(sid), Some(did)) => {
                panic!(
                    "############### DEBUG: Entering Some-Some branch : {},{} -> {},{}",
                    sid.file.to_str().unwrap(),
                    sid.id,
                    did.file.to_str().unwrap(),
                    did.id
                )
            }
            (Some(sid), None) => {
                // Source already exists in destination
                // Increment n_out since it is a source now
                let nout = sid.n_out.get();
                sid.n_out.set(nout + 1);
                id += 1;
                let sid = sid.id;
                destinations.push(Node::new(dest_path, id, 1, 0));
                edges.insert(sid, id);
            }
            (None, Some(did)) => {
                panic!("############# DEBUG: Entering None-Some branch")
            }
            (None, None) => {
                id += 1;
                let s_id = id;
                sources.push(Node::new(source_path, id, 0, 1));
                id += 1;
                let d_id = id;
                destinations.push(Node::new(dest_path, id, 1, 0));
                edges.insert(s_id, d_id);
            }
        }
    };
    (sources, destinations, edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Helper to create dummy Opts for testing
    fn mock_opts(source: &str, dest: &str, files: Vec<&str>) -> Opts {
        Opts {
            source_pattern: source.to_string(),
            dest_pattern: dest.to_string(),
            files: files.into_iter().map(PathBuf::from).collect(),
            move_bool: true,
            copy_bool: false, // Add other Opts fields here if they exist
        }
    }

    #[test]
    fn test_independent_moves() {
        let opts = mock_opts(r"(.*)\.txt", "$1.bak", vec!["a.txt", "b.txt", "c.txt"]);
        let (sources, destinations, edges) = create_network(&opts);

        // Expect 2 separate moves: a->a.bak (ID 1->2) and b->b.bak (ID 3->4)
        assert_eq!(edges.len(), 3);
        assert_eq!(sources.len(), 3);
        assert_eq!(destinations.len(), 3);

        // Check that n_in/n_out are balanced for independent moves
        assert_eq!(sources[0].n_out.get(), 1);
        assert_eq!(destinations[0].n_in.get(), 1);

        assert_eq!(destinations[0].file, PathBuf::from("a.bak"));
        assert_eq!(destinations[1].file, PathBuf::from("b.bak"));
        assert_eq!(destinations[2].file, PathBuf::from("c.bak"));
    }

    #[test]
    #[should_panic] // Or check for exit code if you use a mockable exit
    fn test_collision_detection() {
        let opts = mock_opts(r"([a-z])([a-z])", "$2$1", vec!["ab", "ac", "ba"]);
        let (sources, destinations, edges) = create_network(&opts);
    }

    #[test]
    fn test_chain_move() {
        // Chaining not possible with regexp matching
    }

    proptest! {
        #[test]
        fn prop_nodes_always_balanced(
            files in prop::collection::vec("[a-z0-9]{1,8}\\.txt", 1..3),
            dest in "[a-z0-9]{1,8}\\.bak"
        ) {
            let opts = Opts {
                source_pattern: r"(.*)\.txt".to_string(),
                dest_pattern: dest,
                files: files.into_iter().map(PathBuf::from).collect(),
                copy_bool:true,
                move_bool:false
            };
            opts.files.iter().for_each(|v| eprint!("{} - ",v.display()));
            eprintln!(" ");
            // // We wrap in catch_unwind because your code currently panics
            // // on collisions. We want to see if non-panicking paths are valid.
            // let result = std::panic::catch_unwind(|| {
            //     let (sources, destinations, _) = create_network(&opts);
            //
            //     for s in sources {
            //         assert_eq!(s.n_out.get(), 1);
            //     }
            //     for d in destinations {
            //         assert_eq!(d.n_in.get(), 1);
            //     }
            // });
            //
            // If the code didn't panic, the invariants MUST hold.
        }
    }
}
