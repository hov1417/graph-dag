#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(clippy::must_use_candidate)]

fn main() {
    let g = petgraph::graph::DiGraph::<(), i32>::from_edges(&[(1, 2), (2, 3), (1, 10)])
        .map_owned(|n, _| format!("{}", n.index()), |e, _| e.index());
    let g = petgraph::acyclic::Acyclic::try_from_graph(g).unwrap();
    println!("{}", graph_dag::dag_to_text(&g).unwrap());
}
