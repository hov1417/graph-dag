use petgraph::visit::IntoNeighborsDirected;
use crate::dag::context::Context;
use crate::ProcessingError;

impl Context {
    pub fn process_petgraph<'a, G, N, F>(
        input: &'a petgraph::acyclic::Acyclic<G>,
        serializer: F,
    ) -> Result<String, ProcessingError>
    where
        G: petgraph::visit::Visitable + petgraph::visit::GraphBase<NodeId = N>,
        &'a G: petgraph::visit::IntoEdgesDirected + petgraph::visit::GraphRef<NodeId = N>,
        F: Fn(&N) -> String,
    {
        let mut ctx = Self::default();
        for node in input.nodes_iter() {
            let source = serializer(&node);
            ctx.add_node(&source);
            let edges = input.neighbors_directed(node, petgraph::Direction::Outgoing);
            for edge in edges {
                let target = serializer(&edge);
                ctx.add_node(&target);
                ctx.add_vertex(&source, &target);
            }
        }

        if ctx.is_empty() {
            return Ok(String::new());
        }
        ctx.toposort()?;
        ctx.complete();
        ctx.build_layers();
        ctx.resolve_crossings();
        ctx.layout();
        Ok(ctx.render())
    }
}
