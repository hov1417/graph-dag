#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(clippy::must_use_candidate)]


#[cfg(not(feature = "petgraph"))]
fn main() {
    let dag = "A -> C\nA -> D -> C\nB -> D\nE -> C";
    println!("{}", graph_dag::dag_to_text(dag).unwrap());
}

#[cfg(feature = "petgraph")]
fn main() {
    let g = petgraph::graph::DiGraph::<(), i32>::from_edges(&[(1, 2), (2, 3), (1, 10)]);
    let g = petgraph::acyclic::Acyclic::try_from_graph(g).unwrap();
    println!(
        "{}",
        graph_dag::petgraph_dag_to_text(&g, |n| n.index().to_string()).unwrap()
    );
}
