mod support;

use petgraph::{
    algo::{condensation as pet_condensation, kosaraju_scc, toposort},
    graph::DiGraph,
    unionfind::UnionFind,
    visit::{EdgeFiltered, EdgeRef},
};
use std::hint::black_box;
use support::{measure_batched, measure_batched_with_setup, print_measurement, topology_pairs};
use weavatrix_graph::{
    EdgeEndpoints, NodeIndex, Topology, condensation, strongly_connected_components_filtered,
    topological_sort_filtered, weakly_connected_components,
};

const NODE_COUNT: usize = 10_000;
const EDGE_COUNT: usize = 30_000;
const BATCH_SIZE: u32 = 64;

fn main() {
    println!("statistic=median runs=11 warmups=2 batch={BATCH_SIZE}");
    let pairs = topology_pairs(NODE_COUNT, EDGE_COUNT);
    let ours = Topology::try_from_edges(NODE_COUNT, compact_edges(&pairs)).unwrap();
    let pet = pet_graph(&pairs);
    compare_filtered_scc(&ours, &pet);
    compare_filtered_toposort(&ours, &pet);
    compare_weak_components(&ours, &pet);
    compare_condensation(&ours, &pet);
}

fn compare_filtered_scc(ours: &Topology, pet: &DiGraph<usize, ()>) {
    let ours = measure_batched(BATCH_SIZE, || {
        black_box(strongly_connected_components_filtered(ours, |edge| {
            allows(edge.index())
        }))
    });
    let filtered = EdgeFiltered::from_fn(pet, |edge| allows(edge.id().index()));
    let pet = measure_batched(BATCH_SIZE, || black_box(kosaraju_scc(&filtered)));
    print_measurement("filtered-scc", "library=weavatrix-graph", &ours);
    print_measurement("filtered-scc", "library=petgraph", &pet);
}

fn compare_filtered_toposort(ours: &Topology, pet: &DiGraph<usize, ()>) {
    let ours = measure_batched(BATCH_SIZE, || {
        black_box(topological_sort_filtered(ours, |edge| allows(edge.index())))
    });
    let filtered = EdgeFiltered::from_fn(pet, |edge| allows(edge.id().index()));
    let pet = measure_batched(BATCH_SIZE, || black_box(toposort(&filtered, None)));
    print_measurement("filtered-toposort", "library=weavatrix-graph", &ours);
    print_measurement("filtered-toposort", "library=petgraph", &pet);
}

fn compare_weak_components(ours: &Topology, pet: &DiGraph<usize, ()>) {
    let ours = measure_batched(BATCH_SIZE, || black_box(weakly_connected_components(ours)));
    let pet = measure_batched(BATCH_SIZE, || black_box(pet_weak_components(pet)));
    print_measurement(
        "weak-components",
        "library=weavatrix-graph contract=memberships",
        &ours,
    );
    print_measurement(
        "weak-components",
        "library=petgraph contract=memberships",
        &pet,
    );
}

fn compare_condensation(ours: &Topology, pet: &DiGraph<usize, ()>) {
    let ours = measure_batched(BATCH_SIZE, || black_box(condensation(ours).unwrap()));
    let pet = measure_batched_with_setup(
        BATCH_SIZE,
        || pet.clone(),
        |graph| black_box(pet_condensation(graph, true)),
    );
    print_measurement(
        "condensation-dag",
        "library=weavatrix-graph clones=excluded",
        &ours,
    );
    print_measurement("condensation-dag", "library=petgraph clones=excluded", &pet);
}

fn pet_weak_components(graph: &DiGraph<usize, ()>) -> Vec<Vec<usize>> {
    let mut sets = UnionFind::new(graph.node_count());
    for edge in graph.edge_references() {
        sets.union(edge.source().index(), edge.target().index());
    }
    let mut grouped = vec![Vec::new(); graph.node_count()];
    for node in graph.node_indices() {
        grouped[sets.find(node.index())].push(node.index());
    }
    let mut components = grouped
        .into_iter()
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();
    components.sort_unstable_by_key(|component| component[0]);
    components
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

fn pet_graph(pairs: &[(usize, usize)]) -> DiGraph<usize, ()> {
    let mut graph = DiGraph::with_capacity(NODE_COUNT, EDGE_COUNT);
    let nodes = (0..NODE_COUNT)
        .map(|index| graph.add_node(index))
        .collect::<Vec<_>>();
    for &(source, target) in pairs {
        graph.add_edge(nodes[source], nodes[target], ());
    }
    graph
}

const fn allows(edge: usize) -> bool {
    edge % 3 != 1
}
