use crate::dag_to_text;
use petgraph::Graph;
use petgraph::acyclic::Acyclic;
use petgraph::data::Build;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use std::panic;

#[test]
fn dag_50_50() {
    #[cfg(debug_assertions)]
    let len = 10;
    #[cfg(not(debug_assertions))]
    let len = 400;
    for _ in 0..len {
        let dag = create_random_dag(50, 50);

        let _ = panic::take_hook();
        let err = format!("failed convert dag to text for following graph\n'{dag:?}'");
        panic::set_hook(Box::new(move |e| println!("{err}\n{e:?}")));
        dag_to_text(&dag);
    }
}

fn create_random_dag(max_vertex: u32, max_edge: u32) -> Acyclic<Graph<String, ()>> {
    let mut g = petgraph::graph::DiGraph::<String, ()>::default();
    let mut nodes = HashMap::new();
    let vert_num = (rand::random::<u32>() % max_vertex) + 1;
    let edge_num = (rand::random::<u32>() % max_edge) + 1;
    for _ in 0..(edge_num / 2) {
        if add_random_edge(&mut g, &mut nodes, (vert_num / 2) + 1) {
            continue;
        }
    }

    let mut g = Acyclic::try_from_graph(g).unwrap();

    for _ in (edge_num / 2)..edge_num {
        if add_random_edge(&mut g, &mut nodes, vert_num) {
            continue;
        }
    }

    g
}

fn add_random_edge<G>(g: &mut G, nodes: &mut HashMap<u32, NodeIndex>, vert_num: u32) -> bool
where
    G: Build<NodeWeight = String, EdgeWeight = (), NodeId = NodeIndex>,
{
    let mut a = rand::random::<u32>() % vert_num;
    let mut b = rand::random::<u32>() % vert_num;
    if a > b {
        std::mem::swap(&mut a, &mut b);
    } else if a == b {
        return true;
    }
    let a_node = *nodes.entry(a).or_insert_with(|| g.add_node(a.to_string()));
    let b_node = *nodes.entry(b).or_insert_with(|| g.add_node(b.to_string()));
    g.add_edge(a_node, b_node, ());
    false
}
