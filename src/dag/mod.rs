mod adapter;
mod context;

use crate::dag::adapter::Adapter;
use crate::dag::context::Context;
use petgraph::adj::IndexType;
use petgraph::prelude::NodeIndex;
use petgraph::visit::{IntoEdgeReferences, NodeCount};
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Index;

#[derive(Default, Clone, Debug)]
struct Node<N> {
    /* parsing */
    upward: HashSet<usize>,
    downward: HashSet<usize>,
    is_connector: bool,
    padding: i32,

    /* layering */
    layer: usize,
    row: usize,
    downward_closure: HashSet<usize>,
    upward_sorted: Vec<usize>,
    downward_sorted: Vec<usize>,

    /* rendering */
    width: i32,
    height: i32,
    x: i32,
    y: i32,

    index: NodeIndex<N>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Edge {
    up: usize,
    down: usize,
    x: i32,
    y: i32,
}

#[derive(Default)]
struct Layer {
    nodes: Vec<usize>,
    edges: Vec<Edge>,
    adapter: Adapter,
}

///  Convert Directed Acyclic Graph (DAG) from `petgraph` create to Unicode graphic
///
/// # Arguments
///
/// * `s`: Directed Acyclic Graph represented as lines of paths
///
/// returns: `Result<String, ProcessingError>`
///
/// # Errors
/// returns `ProcessingError::CycleFound` if cycle is detected in input graph
///
/// # Examples
///
/// ```
/// use graph_dag::dag_to_text;
/// let mut g = petgraph::graph::DiGraph::<&str, ()>::default();
/// let a = g.add_node("A");
/// let b = g.add_node("B");
/// let c = g.add_node("C");
/// let d = g.add_node("D");
/// let e = g.add_node("E");
/// g.add_edge(a, b, ());
/// g.add_edge(b, c, ());
/// g.add_edge(d, c, ());
/// g.add_edge(d, e, ());
/// let g = petgraph::acyclic::Acyclic::try_from_graph(g).unwrap();
/// let graph = dag_to_text(&g);
/// assert_eq!(
/// graph,
/// "┌───┐┌───┐  \n".to_owned() +
/// "│ A ││ D │  \n" +
/// "└┬──┘└┬─┬┘  \n" +
/// "┌▽──┐ │┌▽──┐\n" +
/// "│ B │ ││ E │\n" +
/// "└┬──┘ │└───┘\n" +
/// "┌▽────▽─┐   \n" +
/// "│   C   │   \n" +
/// "└───────┘   \n");
/// ```
///
pub fn dag_to_text<'a, G, N, O>(input: &'a petgraph::acyclic::Acyclic<G>) -> String
where
    G: petgraph::visit::Visitable
        + petgraph::visit::GraphBase<NodeId = NodeIndex<N>>
        + Index<petgraph::matrix_graph::NodeIndex<N>, Output = O>
        + NodeCount,
    &'a G: IntoEdgeReferences + petgraph::visit::GraphBase<NodeId = NodeIndex<N>>,
    NodeIndex<N>: Clone,
    O: Display,
    N: IndexType + Eq + Hash + Default + Clone,
{
    Context::process(input)
}
