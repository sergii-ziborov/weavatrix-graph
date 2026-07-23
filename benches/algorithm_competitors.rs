mod support;

use petgraph::{
    algo::{dijkstra as pet_dijkstra, dinics, kosaraju_scc, min_spanning_tree},
    data::Element,
    graph::{DiGraph, UnGraph},
    visit::Bfs,
};
use std::hint::black_box;
use support::{measure, print_measurement, topology_pairs};
use weavatrix_graph::{
    EdgeEndpoints, NodeIndex, Topology, UndirectedTopology, bfs, dijkstra, maximum_flow,
    minimum_spanning_forest, strongly_connected_components,
};

const NODE_COUNT: usize = 10_000;
const EDGE_COUNT: usize = 30_000;

fn main() {
    println!("statistic=median runs=11 warmups=2");
    let pairs = topology_pairs(NODE_COUNT, EDGE_COUNT);
    let compact = compact_edges(&pairs);
    let ours = Topology::try_from_edges(NODE_COUNT, compact.iter().copied()).unwrap();
    let pet = pet_directed(&pairs);
    compare_traversal(&ours, &pet);
    compare_components(&ours, &pet);
    compare_shortest(&ours, &pet, &pairs);
    compare_mst(&compact, &pairs);
    compare_flow();
}

fn compare_traversal(ours: &Topology, pet: &DiGraph<(), u64>) {
    let ours = measure(|| black_box(bfs(ours, NodeIndex::new(0))));
    let pet = measure(|| {
        let mut bfs = Bfs::new(pet, petgraph::graph::NodeIndex::new(0));
        let mut visited = Vec::new();
        while let Some(node) = bfs.next(pet) {
            visited.push(node);
        }
        black_box(visited)
    });
    print_measurement("bfs", "library=weavatrix-graph", &ours);
    print_measurement("bfs", "library=petgraph", &pet);
}

fn compare_components(ours: &Topology, pet: &DiGraph<(), u64>) {
    let ours = measure(|| black_box(strongly_connected_components(ours)));
    let pet = measure(|| black_box(kosaraju_scc(pet)));
    print_measurement("scc", "library=weavatrix-graph", &ours);
    print_measurement("scc", "library=petgraph", &pet);
}

fn compare_shortest(ours: &Topology, pet: &DiGraph<(), u64>, pairs: &[(usize, usize)]) {
    let target = NodeIndex::new(u32::try_from(NODE_COUNT - 1).unwrap());
    let ours = measure(|| {
        black_box(dijkstra(ours, NodeIndex::new(0), target, |edge| {
            weight(pairs[edge.index()])
        }))
    });
    let pet = measure(|| {
        black_box(pet_dijkstra(
            pet,
            petgraph::graph::NodeIndex::new(0),
            Some(petgraph::graph::NodeIndex::new(NODE_COUNT - 1)),
            |edge| *edge.weight(),
        ))
    });
    print_measurement("dijkstra-target", "library=weavatrix-graph", &ours);
    print_measurement("dijkstra-target", "library=petgraph", &pet);
}

fn compare_mst(compact: &[EdgeEndpoints], pairs: &[(usize, usize)]) {
    let ours = UndirectedTopology::try_from_edges(NODE_COUNT, compact.iter().copied()).unwrap();
    let mut pet = UnGraph::<(), u64>::with_capacity(NODE_COUNT, EDGE_COUNT);
    let nodes = (0..NODE_COUNT)
        .map(|_| pet.add_node(()))
        .collect::<Vec<_>>();
    for &(source, target) in pairs {
        pet.add_edge(nodes[source], nodes[target], weight((source, target)));
    }
    let ours = measure(|| {
        black_box(minimum_spanning_forest(&ours, |edge| {
            weight(pairs[edge.index()])
        }))
    });
    let pet = measure(|| {
        black_box(
            min_spanning_tree(&pet)
                .filter(|element| matches!(element, Element::Edge { .. }))
                .count(),
        )
    });
    print_measurement("minimum-spanning-forest", "library=weavatrix-graph", &ours);
    print_measurement("minimum-spanning-forest", "library=petgraph", &pet);
}

fn compare_flow() {
    const FLOW_NODES: usize = 1_000;
    const FLOW_EDGES: usize = 5_000;
    let pairs = topology_pairs(FLOW_NODES, FLOW_EDGES);
    let ours = Topology::try_from_edges(FLOW_NODES, compact_edges(&pairs)).unwrap();
    let pet = pet_directed_with_size(FLOW_NODES, &pairs);
    let ours = measure(|| {
        black_box(
            maximum_flow(
                &ours,
                NodeIndex::new(0),
                NodeIndex::new(u32::try_from(FLOW_NODES - 1).unwrap()),
                |edge| weight(pairs[edge.index()]),
            )
            .unwrap(),
        )
    });
    let pet = measure(|| {
        black_box(dinics(
            &pet,
            petgraph::graph::NodeIndex::new(0),
            petgraph::graph::NodeIndex::new(FLOW_NODES - 1),
        ))
    });
    print_measurement("maximum-flow", "library=weavatrix-graph", &ours);
    print_measurement("maximum-flow", "library=petgraph", &pet);
}

fn compact_edges(pairs: &[(usize, usize)]) -> Vec<EdgeEndpoints> {
    pairs
        .iter()
        .map(|&(source, target)| {
            EdgeEndpoints::new(
                NodeIndex::new(u32::try_from(source).unwrap()),
                NodeIndex::new(u32::try_from(target).unwrap()),
            )
        })
        .collect()
}

fn pet_directed(pairs: &[(usize, usize)]) -> DiGraph<(), u64> {
    pet_directed_with_size(NODE_COUNT, pairs)
}

fn pet_directed_with_size(node_count: usize, pairs: &[(usize, usize)]) -> DiGraph<(), u64> {
    let mut graph = DiGraph::with_capacity(node_count, pairs.len());
    let nodes = (0..node_count)
        .map(|_| graph.add_node(()))
        .collect::<Vec<_>>();
    for &(source, target) in pairs {
        graph.add_edge(nodes[source], nodes[target], weight((source, target)));
    }
    graph
}

fn weight((source, target): (usize, usize)) -> u64 {
    u64::try_from((source * 31 + target * 17) % 97 + 1).unwrap()
}
