use petgraph::algo::dominators::simple_fast;
use petgraph::algo::tred::{
    dag_to_toposorted_adjacency_list, dag_transitive_reduction_closure as pet_transitive,
};
use petgraph::graph::{DiGraph, IndexType};
use std::collections::BTreeSet;
use weavatrix_graph::{
    EdgeEndpoints, NodeIndex, Topology, dag_transitive_reduction_closure, dominators, page_rank,
};

#[test]
fn dominators_and_transitive_results_match_petgraph_on_seeded_graphs() {
    let mut seed = 0xa076_1d64_78bd_642f_u64;
    for node_count in [1_usize, 2, 9, 37, 97] {
        for density in [3_u64, 13, 31] {
            let mut dag_edges = Vec::new();
            let mut cfg_edges = Vec::new();
            for source in 0..node_count {
                for target in 0..node_count {
                    seed = next(seed);
                    if source != target && seed % 101 < density {
                        cfg_edges.push((source, target));
                        if source < target {
                            dag_edges.push((source, target));
                        }
                    }
                }
            }
            compare_dominators(node_count, &cfg_edges);
            compare_transitive(node_count, &dag_edges);
        }
    }
}

#[test]
fn page_rank_matches_an_independent_standard_reference() {
    let edges = [(0_usize, 1_usize), (0, 1), (1, 2), (2, 0), (2, 1)];
    let graph = ours_graph(4, &edges);
    let actual = page_rank(&graph, 0.85, 80)
        .unwrap()
        .into_iter()
        .map(|(_, rank)| rank)
        .collect::<Vec<_>>();
    let expected = reference_page_rank(4, &edges, 0.85, 80);
    for (actual, expected) in actual.iter().zip(expected) {
        assert!((actual - expected).abs() < 1.0e-12);
    }
}

fn compare_dominators(node_count: usize, edges: &[(usize, usize)]) {
    let ours = ours_graph(node_count, edges);
    let (pet, pet_nodes) = pet_graph(node_count, edges);
    let actual = dominators(&ours, node(0)).unwrap();
    let expected = simple_fast(&pet, pet_nodes[0]);
    for (target, &pet_target) in pet_nodes.iter().take(node_count).enumerate() {
        assert_eq!(
            actual
                .immediate_dominator(node(target))
                .map(NodeIndex::index),
            expected
                .immediate_dominator(pet_target)
                .map(petgraph::graph::NodeIndex::index)
        );
    }
}

fn compare_transitive(node_count: usize, edges: &[(usize, usize)]) {
    let ours = dag_transitive_reduction_closure(&ours_graph(node_count, edges)).unwrap();
    let (pet, pet_nodes) = pet_graph(node_count, edges);
    let (ordered, _) = dag_to_toposorted_adjacency_list::<_, u32>(&pet, &pet_nodes);
    let (pet_reduction, pet_closure) = pet_transitive(&ordered);

    assert_eq!(
        endpoints(ours.reduction_edges()),
        pet_endpoints(&pet_reduction)
    );
    assert_eq!(endpoints(ours.closure_edges()), pet_endpoints(&pet_closure));
}

fn reference_page_rank(
    node_count: usize,
    edges: &[(usize, usize)],
    damping: f64,
    iterations: usize,
) -> Vec<f64> {
    let mut degree = vec![0_usize; node_count];
    for &(source, _) in edges {
        degree[source] += 1;
    }
    let count = usize_as_f64(node_count);
    let mut ranks = vec![1.0 / count; node_count];
    for _ in 0..iterations {
        let dangling = (0..node_count)
            .filter(|&node| degree[node] == 0)
            .map(|node| ranks[node])
            .sum::<f64>();
        let mut next = vec![(1.0 - damping + damping * dangling) / count; node_count];
        for &(source, target) in edges {
            next[target] += damping * ranks[source] / usize_as_f64(degree[source]);
        }
        let sum = next.iter().sum::<f64>();
        for rank in &mut next {
            *rank /= sum;
        }
        ranks = next;
    }
    ranks
}

fn ours_graph(node_count: usize, edges: &[(usize, usize)]) -> Topology {
    Topology::try_from_edges(
        node_count,
        edges
            .iter()
            .map(|&(source, target)| EdgeEndpoints::new(node(source), node(target))),
    )
    .unwrap()
}

fn pet_graph(
    node_count: usize,
    edges: &[(usize, usize)],
) -> (DiGraph<(), ()>, Vec<petgraph::graph::NodeIndex>) {
    let mut graph = DiGraph::with_capacity(node_count, edges.len());
    let nodes = (0..node_count)
        .map(|_| graph.add_node(()))
        .collect::<Vec<_>>();
    for &(source, target) in edges {
        graph.add_edge(nodes[source], nodes[target], ());
    }
    (graph, nodes)
}

fn endpoints(edges: &[EdgeEndpoints<NodeIndex>]) -> BTreeSet<(usize, usize)> {
    edges
        .iter()
        .map(|edge| (edge.source().index(), edge.target().index()))
        .collect()
}

fn pet_endpoints(graph: &petgraph::adj::UnweightedList<u32>) -> BTreeSet<(usize, usize)> {
    graph
        .edge_indices()
        .map(|edge| {
            let (source, target) = graph.edge_endpoints(edge).unwrap();
            (source.index(), target.index())
        })
        .collect()
}

fn node(index: usize) -> NodeIndex {
    NodeIndex::new(u32::try_from(index).unwrap())
}

fn next(seed: u64) -> u64 {
    seed.wrapping_mul(2_862_933_555_777_941_757)
        .wrapping_add(3_037_000_493)
}

#[allow(clippy::cast_precision_loss)]
fn usize_as_f64(value: usize) -> f64 {
    value as f64
}
