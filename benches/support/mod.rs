#![allow(dead_code)]

use std::time::{Duration, Instant};
use weavatrix_graph::{
    Confidence, Edge, EdgeKind, EvidenceKind, Graph, GraphBuilder, Node, NodeId, NodeKind,
    Provenance,
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
