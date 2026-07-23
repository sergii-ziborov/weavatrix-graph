mod support;

use graaf::{AddArc, AdjacencyList, Empty};
use petgraph::{Directed, Graph as PetGraph, csr::Csr};
use support::{measure, print_measurement, topology_pairs};
use weavatrix_graph::{EdgeEndpoints, NodeIndex, Topology};

const NODE_COUNT: usize = 10_000;
const EDGE_COUNT: usize = 30_000;

fn main() {
    println!("statistic=median runs=11 warmups=2");
    let endpoints = topology_pairs(NODE_COUNT, EDGE_COUNT);
    compare_frozen_topology(&endpoints);
    compare_mutable_append(&endpoints);
}

fn compare_frozen_topology(endpoints: &[(usize, usize)]) {
    let compact = endpoints
        .iter()
        .map(|&(source, target)| {
            EdgeEndpoints::new(
                NodeIndex::new(source.try_into().unwrap()),
                NodeIndex::new(target.try_into().unwrap()),
            )
        })
        .collect::<Vec<_>>();
    let mut forward = endpoints.to_vec();
    forward.sort_unstable();
    forward.dedup();
    let mut reverse = forward
        .iter()
        .map(|&(source, target)| (target, source))
        .collect::<Vec<_>>();
    reverse.sort_unstable();
    reverse.dedup();

    let ours = measure(|| Topology::try_from_edges(NODE_COUNT, compact.iter().copied()).unwrap());
    let petgraph = measure(|| {
        let outgoing = Csr::<(), (), Directed, usize>::from_sorted_edges(&forward).unwrap();
        let incoming = Csr::<(), (), Directed, usize>::from_sorted_edges(&reverse).unwrap();
        assert_eq!(outgoing.edge_count(), EDGE_COUNT);
        assert_eq!(incoming.edge_count(), EDGE_COUNT);
        (outgoing, incoming)
    });
    let petgraph_with_preprocessing = measure(|| {
        let mut outgoing_edges = endpoints.to_vec();
        outgoing_edges.sort_unstable();
        outgoing_edges.dedup();
        let mut incoming_edges = outgoing_edges
            .iter()
            .map(|&(source, target)| (target, source))
            .collect::<Vec<_>>();
        incoming_edges.sort_unstable();
        incoming_edges.dedup();
        let outgoing = Csr::<(), (), Directed, usize>::from_sorted_edges(&outgoing_edges).unwrap();
        let incoming = Csr::<(), (), Directed, usize>::from_sorted_edges(&incoming_edges).unwrap();
        (outgoing, incoming)
    });
    print_measurement("dual-csr-build", "library=weavatrix-graph", &ours);
    print_measurement(
        "dual-csr-build",
        "library=petgraph-two-presorted-csr",
        &petgraph,
    );
    print_measurement(
        "dual-csr-build",
        "library=petgraph-two-csr-with-preprocessing",
        &petgraph_with_preprocessing,
    );
}

fn compare_mutable_append(endpoints: &[(usize, usize)]) {
    let petgraph = measure(|| {
        let mut graph: PetGraph<(), ()> = PetGraph::with_capacity(NODE_COUNT, EDGE_COUNT);
        let nodes = (0..NODE_COUNT)
            .map(|_| graph.add_node(()))
            .collect::<Vec<_>>();
        for &(source, target) in endpoints {
            graph.add_edge(nodes[source], nodes[target], ());
        }
        graph
    });
    let graaf = measure(|| {
        let mut graph = AdjacencyList::empty(NODE_COUNT);
        for &(source, target) in endpoints {
            graph.add_arc(source, target);
        }
        graph
    });
    print_measurement("mutable-append", "library=petgraph", &petgraph);
    print_measurement("mutable-append", "library=graaf", &graaf);
}
