#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(unused_results, clippy::must_use_candidate)]

use graph_dag::dag_to_text;

fn main() {
    let dag = "A -> C\nA -> D -> C\nB -> D\nE -> C";
    println!("{}", dag_to_text(dag).unwrap());
}
