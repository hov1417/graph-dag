mod adapter;
mod context;

use crate::dag::adapter::Adapter;
use crate::dag::context::Context;
pub use crate::dag::context::ProcessingError;
use std::collections::HashSet;

#[derive(Default)]
struct Node {
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

fn split<'a>(s: &'a str, pat: &str) -> Vec<&'a str> {
    s.split(pat).filter(|x| !x.is_empty()).collect()
}

/// Convert Directed Acyclic Graph (DAG) into Unicode graphic
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
/// let graph = dag_to_text(r#"
///     A -> B -> C
///     D -> C
///     D -> E
/// "#);
/// assert_eq!(
/// &graph.unwrap(),
/// r#"┌───┐┌───┐  
/// │ A ││ D │  
/// └┬──┘└┬─┬┘  
/// ┌▽──┐ │┌▽──┐
/// │ B │ ││ E │
/// └┬──┘ │└───┘
/// ┌▽────▽─┐   
/// │   C   │   
/// └───────┘   
/// "#);
/// ```
pub fn dag_to_text(s: &str) -> Result<String, ProcessingError> {
    Context::process(s)
}
