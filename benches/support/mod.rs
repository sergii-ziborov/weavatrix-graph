#![allow(dead_code)]

use std::time::{Duration, Instant};
use weavatrix_graph::{
    Confidence, Edge, EdgeKind, EvidenceKind, Graph, Node, NodeId, NodeKind, Provenance,
};

pub struct Measurement {
    pub median: Duration,
    pub minimum: Duration,
}

pub fn measure<T>(mut operation: impl FnMut() -> T) -> Measurement {
    const WARMUPS: usize = 2;
    const RUNS: usize = 11;

    for _ in 0..WARMUPS {
        std::hint::black_box(operation());
    }
    let mut samples = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let start = Instant::now();
        std::hint::black_box(operation());
        samples.push(start.elapsed());
    }
    samples.sort_unstable();
    Measurement {
        median: samples[RUNS / 2],
        minimum: samples[0],
    }
}

pub fn print_measurement(mode: &str, details: &str, measurement: &Measurement) {
    println!(
        "mode={mode} {details} median_ms={:.3} min_ms={:.3}",
        measurement.median.as_secs_f64() * 1_000.0,
        measurement.minimum.as_secs_f64() * 1_000.0
    );
}

pub fn build_graph(node_count: usize, edge_count: usize) -> Graph {
    let (nodes, edges) = graph_parts(node_count, edge_count);
    Graph::try_from_parts(nodes, edges).unwrap()
}

pub fn graph_parts(node_count: usize, edge_count: usize) -> (Vec<Node>, Vec<Edge>) {
    let nodes = (0..node_count)
        .map(|index| {
            Node::new(
                format!("node:{index:05}"),
                format!("node_{index}"),
                NodeKind::Function,
            )
            .unwrap()
            .with_language("rust")
        })
        .collect();
    let edges = (0..edge_count)
        .map(|index| {
            let source_index = index % node_count;
            let layer = index / node_count;
            let mut target_index = (source_index * 37 + layer * 7_919 + 17) % node_count;
            if target_index == source_index {
                target_index = (target_index + 1) % node_count;
            }
            let source = NodeId::new(format!("node:{source_index:05}")).unwrap();
            let target = NodeId::new(format!("node:{target_index:05}")).unwrap();
            Edge::new(
                source,
                target,
                EdgeKind::Calls,
                Provenance::new("bench.graph", EvidenceKind::Resolved, Confidence::High)
                    .unwrap()
                    .with_detail(format!("edge:{index}")),
            )
        })
        .collect();
    (nodes, edges)
}
