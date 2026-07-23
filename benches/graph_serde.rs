mod support;

use support::{build_graph, measure, print_measurement};
use weavatrix_graph::Graph;

const NODE_COUNT: usize = 5_000;
const EDGE_COUNT: usize = 15_000;

fn main() {
    println!("statistic=median runs=11 warmups=2");
    let graph = build_graph(NODE_COUNT, EDGE_COUNT);
    let json = serde_json::to_vec(&graph).unwrap();

    let serialize = measure(|| serde_json::to_vec(&graph).unwrap());
    print_measurement(
        "serialize-json",
        &format!("nodes={NODE_COUNT} edges={EDGE_COUNT} bytes={}", json.len()),
        &serialize,
    );

    let deserialize = measure(|| {
        let decoded = serde_json::from_slice::<Graph>(&json).unwrap();
        assert_eq!(decoded.node_count(), NODE_COUNT);
        assert_eq!(decoded.edge_count(), EDGE_COUNT);
        decoded
    });
    print_measurement(
        "validated-deserialize-json",
        &format!("nodes={NODE_COUNT} edges={EDGE_COUNT} bytes={}", json.len()),
        &deserialize,
    );
}
