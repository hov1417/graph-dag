use std::collections::HashMap;
use crate::dag_to_text;
use petgraph::Graph;
use petgraph::acyclic::Acyclic;
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
        panic::set_hook(Box::new(move |_| println!("{err}")));
        assert!(
            dag_to_text(&dag).is_ok(),
            "failed convert dag to text for following graph\n'{dag:?}'"
        );
    }
}

fn create_random_dag(max_vertex: u32, max_edge: u32) -> Acyclic<Graph<String, ()>> {
    let mut g = petgraph::graph::DiGraph::<String, ()>::default();
    let mut nodes = HashMap::new();
    let vert_num = (rand::random::<u32>() % max_vertex) + 1;
    let edge_num = (rand::random::<u32>() % max_edge) + 1;
    for _ in 0..edge_num {
        let mut a = rand::random::<u32>() % vert_num;
        let mut b = rand::random::<u32>() % vert_num;
        if a > b {
            std::mem::swap(&mut a, &mut b);
        } else if a == b {
            continue;
        }
        let a_node = *nodes.entry(a).or_insert_with(|| g.add_node(a.to_string()));
        let b_node = *nodes.entry(b).or_insert_with(|| g.add_node(b.to_string()));
        g.add_edge(a_node, b_node, ());
    }

    Acyclic::try_from_graph(g).unwrap()
}
