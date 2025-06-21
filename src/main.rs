#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(unused_results, clippy::must_use_candidate)]

use crate::dag::dag_to_text;

mod dag;
mod screen;
#[cfg(test)]
mod test;

fn main() {
    let dag = "A -> B -> C\nA -> D -> C";
    println!("{}", dag_to_text(dag));
}
