use crate::dag::dag_to_text;
use insta::assert_snapshot;

#[test]
fn test_dag_to_graph_1() {
    assert_snapshot!(dag_to_text("A -> B -> C\nA -> D -> C").unwrap());
}

#[test]
fn test_dag_to_graph_2() {
    assert_snapshot!(dag_to_text("A -> B -> C\nA -> D -> C\nB -> D").unwrap());
}

#[test]
fn test_dag_to_graph_3() {
    assert_snapshot!(dag_to_text("A -> B -> C\nA -> D -> C\nB -> D\nE").unwrap());
}

#[test]
fn test_dag_to_graph_4() {
    assert_snapshot!(dag_to_text("A -> C\nA -> D -> C\nB -> D\nE -> C").unwrap());
}

#[test]
fn test_dag_to_graph_cycle_1() {
    assert!(dag_to_text("A -> B\nA -> D\nB -> D\nD -> E\nE -> A").is_err());
}
