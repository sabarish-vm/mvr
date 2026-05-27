mod argparse_clap;
mod file_ops;
mod network;
mod structs;

use crate::argparse_clap::argparse;
use crate::network::HasCollisionCheck;
use crate::structs::Color;

fn main() {
    let opts = argparse();
    let graph =
        network::UniqueLinks::new(&opts.files, &opts.source_pattern, &opts.dest_pattern, true)
            .unwrap();
    println!("\n{} This is a DRY run {}", Color::Blue, Color::Default);
    graph.print_graph(true);
    graph.collision_check();

    if opts.move_bool && opts.force_run {
        graph.rename();
    } else if opts.copy_bool && opts.force_run {
        graph.copy();
    }
}
