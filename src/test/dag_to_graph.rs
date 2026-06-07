use crate::dag::dag_to_text;
use insta::assert_snapshot;

#[test]
fn test_dag_to_graph_1() {
    let mut g = petgraph::graph::DiGraph::<&str, ()>::default();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(a, d, ());
    g.add_edge(d, c, ());
    let g = petgraph::acyclic::Acyclic::try_from_graph(g).unwrap();
    assert_snapshot!(dag_to_text(&g).unwrap());
}

#[test]
fn test_dag_to_graph_2() {
    let mut g = petgraph::graph::DiGraph::<&str, ()>::default();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(a, d, ());
    g.add_edge(d, c, ());
    g.add_edge(b, d, ());
    let g = petgraph::acyclic::Acyclic::try_from_graph(g).unwrap();
    assert_snapshot!(dag_to_text(&g).unwrap());
}

#[test]
fn test_dag_to_graph_3() {
    let mut g = petgraph::graph::DiGraph::<&str, ()>::default();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    g.add_node("E");
    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(a, d, ());
    g.add_edge(d, c, ());
    g.add_edge(b, d, ());
    let g = petgraph::acyclic::Acyclic::try_from_graph(g).unwrap();
    assert_snapshot!(dag_to_text(&g).unwrap());
}

#[test]
fn test_dag_to_graph_4() {
    let mut g = petgraph::graph::DiGraph::<&str, ()>::default();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");
    g.add_edge(a, c, ());
    g.add_edge(a, d, ());
    g.add_edge(d, c, ());
    g.add_edge(b, d, ());
    g.add_edge(e, c, ());
    let g = petgraph::acyclic::Acyclic::try_from_graph(g).unwrap();
    assert_snapshot!(dag_to_text(&g).unwrap());
}
