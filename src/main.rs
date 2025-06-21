use crate::dag_to_graph::dag_to_text;

mod screen;
mod dag_to_graph;

fn main() {
    let dag = r#"
        AAAAAAAAAAAAA -> B -> C
        AAAAAAAAAAAAA -> D -> C
    "#;
    println!("{}", dag_to_text(dag));
}
