mod support;

use graaf::{AddArc, AdjacencyList, Empty, InNeighbors, OutNeighbors};
use petgraph::{Direction, Graph as PetGraph};
use std::{collections::HashMap, hint::black_box};
use support::{build_graph, graph_parts, measure, print_measurement};
use weavatrix_graph::{Edge, Graph, Node};

const NODE_COUNT: usize = 10_000;
const EDGE_COUNT: usize = 30_000;

fn main() {
    println!("statistic=median runs=11 warmups=2");
    compare_build();
    compare_adjacency();
}

fn compare_build() {
    let (nodes, mut edges) = graph_parts(NODE_COUNT, EDGE_COUNT);
    let canonicalizing_edges = edges.clone();
    edges.sort_unstable();
    let canonicalizing =
        measure(|| Graph::try_from_parts(nodes.clone(), canonicalizing_edges.clone()).unwrap());
    let sorted = measure(|| Graph::try_from_sorted_parts(nodes.clone(), edges.clone()).unwrap());
    let petgraph_payload = measure(|| build_petgraph_payload(&nodes, &edges));
    let petgraph = measure(build_petgraph);
    let graaf = measure(build_graaf);
    print_measurement(
        "evidence-build-canonicalizing",
        "weavatrix-graph",
        &canonicalizing,
    );
    print_measurement("evidence-build-sorted-input", "weavatrix-graph", &sorted);
    print_measurement("evidence-build", "petgraph-adapter", &petgraph_payload);
    print_measurement("bare-topology-build", "petgraph", &petgraph);
    print_measurement("bare-topology-build", "graaf", &graaf);
}

fn compare_adjacency() {
    let ours = build_graph(NODE_COUNT, EDGE_COUNT);
    let petgraph = build_petgraph();
    let graaf = build_graaf();
    let expected = EDGE_COUNT * 2;
    let ours_indices = (0..NODE_COUNT)
        .map(|index| ours.node_index(&format!("node:{index:05}")).unwrap())
        .collect::<Vec<_>>();

    let ours_measurement = measure(|| {
        let count = ours_indices
            .iter()
            .map(|&node| ours.out_degree(node).unwrap() + ours.in_degree(node).unwrap())
            .sum::<usize>();
        assert_eq!(count, expected);
        black_box(count)
    });
    let petgraph_measurement = measure(|| {
        let count = petgraph
            .node_indices()
            .map(|node| {
                petgraph
                    .neighbors_directed(node, Direction::Outgoing)
                    .count()
                    + petgraph
                        .neighbors_directed(node, Direction::Incoming)
                        .count()
            })
            .sum::<usize>();
        assert_eq!(count, expected);
        black_box(count)
    });
    let graaf_measurement = measure(|| {
        let count = (0..NODE_COUNT)
            .map(|node| graaf.out_neighbors(node).count() + graaf.in_neighbors(node).count())
            .sum::<usize>();
        assert_eq!(count, expected);
        black_box(count)
    });
    print_measurement(
        "bidirectional-degree-sum",
        "weavatrix-graph",
        &ours_measurement,
    );
    print_measurement(
        "bidirectional-degree-sum",
        "petgraph",
        &petgraph_measurement,
    );
    print_measurement("bidirectional-degree-sum", "graaf", &graaf_measurement);
}

fn build_petgraph() -> PetGraph<(), ()> {
    let mut graph = PetGraph::with_capacity(NODE_COUNT, EDGE_COUNT);
    let nodes = (0..NODE_COUNT)
        .map(|_| graph.add_node(()))
        .collect::<Vec<_>>();
    for (source, target) in topology_edges() {
        graph.add_edge(nodes[source], nodes[target], ());
    }
    assert_eq!(graph.node_count(), NODE_COUNT);
    assert_eq!(graph.edge_count(), EDGE_COUNT);
    graph
}

fn build_petgraph_payload(nodes: &[Node], edges: &[Edge]) -> PetGraph<Node, Edge> {
    let mut graph = PetGraph::with_capacity(nodes.len(), edges.len());
    let mut positions = HashMap::with_capacity(nodes.len());
    for node in nodes {
        let index = graph.add_node(node.clone());
        positions.insert(node.id.clone(), index);
    }
    for edge in edges {
        graph.add_edge(
            positions[&edge.source],
            positions[&edge.target],
            edge.clone(),
        );
    }
    assert_eq!(graph.node_count(), NODE_COUNT);
    assert_eq!(graph.edge_count(), EDGE_COUNT);
    graph
}

fn build_graaf() -> AdjacencyList {
    let mut graph = AdjacencyList::empty(NODE_COUNT);
    for (source, target) in topology_edges() {
        graph.add_arc(source, target);
    }
    assert_eq!(graph.out_neighbors(0).count(), 3);
    graph
}

fn topology_edges() -> impl Iterator<Item = (usize, usize)> {
    (0..EDGE_COUNT).map(|index| {
        let source = index % NODE_COUNT;
        let layer = index / NODE_COUNT;
        let mut target = (source * 37 + layer * 7_919 + 17) % NODE_COUNT;
        if target == source {
            target = (target + 1) % NODE_COUNT;
        }
        (source, target)
    })
}
