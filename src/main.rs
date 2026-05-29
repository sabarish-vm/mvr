mod argparse_clap;
mod file_ops;
mod graph;
mod logger;
mod structs;

use crate::argparse_clap::argparse;
use crate::graph::HasCollisionCheck;
use crate::structs::Color;

fn main() {
    let opts = argparse();
    let graph =
        graph::UniqueLinks::new(&opts.files, &opts.source_pattern, &opts.dest_pattern, true)
            .unwrap();
    graph.collision_check();

    if opts.copy_bool && !opts.force_run {
        println!(
            "\n{}This is a DRY run for copying {}",
            Color::Blue,
            Color::Default
        );
    } else if opts.move_bool && !opts.force_run {
        println!(
            "\n{}This is a DRY run for renaming/moving {}",
            Color::Blue,
            Color::Default
        );
    } else {
        println!();
    }
    graph.print_graph(true);

    if opts.move_bool && opts.force_run {
        let status = graph.rename();
        if opts.log_bool {
            let _ = crate::logger::generate_json_string(&status);
        }
    } else if opts.copy_bool && opts.force_run {
        let status = graph.copy();
        if opts.log_bool {
            let _ = crate::logger::generate_json_string(&status);
        }
    }
}
