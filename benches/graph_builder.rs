mod support;

use support::{build_graph, measure, print_measurement};

const NODE_COUNT: usize = 10_000;
const EDGE_COUNT: usize = 30_000;

fn main() {
    println!("statistic=median runs=11 warmups=2");
    let build = measure(|| build_graph(NODE_COUNT, EDGE_COUNT));
    print_measurement("build", "nodes=10000 edges=30000", &build);

    let graph = build_graph(NODE_COUNT, EDGE_COUNT);
    let node_ids = graph
        .nodes()
        .iter()
        .map(|node| node.id.as_str().to_owned())
        .collect::<Vec<_>>();
    let expected = query_checksum(&graph, &node_ids);
    let queries = measure(|| {
        let checksum = query_checksum(&graph, &node_ids);
        assert_eq!(checksum, expected);
        checksum
    });
    print_measurement(
        "indexed-queries",
        "node_lookups=10000 adjacency_walks=20000",
        &queries,
    );
}

fn query_checksum(graph: &weavatrix_graph::Graph, node_ids: &[String]) -> usize {
    node_ids
        .iter()
        .map(|node_id| {
            let node = graph.node(node_id).is_some();
            let parsed = weavatrix_graph::NodeId::new(node_id).unwrap();
            usize::from(node) + graph.outgoing(&parsed).count() + graph.incoming(&parsed).count()
        })
        .sum()
}
