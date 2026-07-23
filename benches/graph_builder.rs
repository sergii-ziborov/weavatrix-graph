use std::time::Instant;
use weavatrix_graph::{
    Confidence, Edge, EdgeKind, EvidenceKind, Graph, GraphBuilder, Node, NodeId, NodeKind,
    Provenance,
};

fn main() {
    let start = Instant::now();
    let graph = build_graph(10_000, 30_000);
    let build_elapsed = start.elapsed();

    let lookup_start = Instant::now();
    let hits = (0..10_000)
        .filter(|index| graph.node(&format!("node:{index:05}")).is_some())
        .count();
    let outgoing = graph.outgoing(&graph.nodes()[0].id).count();
    let incoming = graph.incoming(&graph.nodes()[999].id).count();
    let lookup_elapsed = lookup_start.elapsed();

    assert_eq!(hits, 10_000);
    assert_eq!(graph.node_count(), 10_000);
    assert_eq!(graph.edge_count(), 30_000);
    println!(
        "graph_builder nodes={} edges={} outgoing0={} incoming999={} build_ms={:.3} lookup_ms={:.3}",
        graph.node_count(),
        graph.edge_count(),
        outgoing,
        incoming,
        build_elapsed.as_secs_f64() * 1_000.0,
        lookup_elapsed.as_secs_f64() * 1_000.0
    );
}

fn build_graph(node_count: usize, edge_count: usize) -> Graph {
    let mut builder = GraphBuilder::new();
    for index in 0..node_count {
        builder
            .add_node(
                Node::new(
                    format!("node:{index:05}"),
                    format!("node_{index}"),
                    NodeKind::Function,
                )
                .unwrap()
                .with_language("rust"),
            )
            .unwrap();
    }
    for index in 0..edge_count {
        let source = NodeId::new(format!("node:{:05}", index % node_count)).unwrap();
        let target = NodeId::new(format!("node:{:05}", (index * 37 + 17) % node_count)).unwrap();
        builder
            .add_edge(Edge::new(
                source,
                target,
                EdgeKind::Calls,
                Provenance::new("bench.graph", EvidenceKind::Resolved, Confidence::High)
                    .unwrap()
                    .with_detail(format!("edge:{index}")),
            ))
            .unwrap();
    }
    builder.build().unwrap()
}
