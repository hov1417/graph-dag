use crate::dag::dag_to_text;
use itertools::Itertools;
use std::panic::catch_unwind;

#[test]
fn dag_50_50() {
    #[cfg(debug_assertions)]
    let len = 10;
    #[cfg(not(debug_assertions))]
    let len = 400;
    for _ in 0..len {
        let dag = create_random_dag(50, 50);
        assert!(
            catch_unwind(|| dag_to_text(&dag)).is_ok(),
            "failed convert dag to text for following graph\n'{dag}'"
        );
    }
}

fn create_random_dag(max_vertex: u32, max_edge: u32) -> String {
    let vert_num = (rand::random::<u32>() % max_vertex) + 1;
    let edge_num = (rand::random::<u32>() % max_edge) + 1;
    let mut edges = Vec::new();
    for _ in 0..edge_num {
        let mut a = rand::random::<u32>() % vert_num;
        let mut b = rand::random::<u32>() % vert_num;
        if a > b {
            std::mem::swap(&mut a, &mut b);
        } else if a == b {
            continue;
        }
        edges.push(format!("{a} -> {b}"));
    }

    edges.into_iter().dedup().join("\n")
}
