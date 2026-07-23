use petgraph::algo::{
    condensation as pet_condensation, connected_components, is_cyclic_directed, kosaraju_scc,
    toposort,
};
use petgraph::graph::DiGraph;
use petgraph::unionfind::UnionFind;
use petgraph::visit::EdgeRef;
use std::collections::{BTreeMap, BTreeSet};
use weavatrix_graph::{
    EdgeEndpoints, GraphView, NodeIndex, Topology, condensation, condensation_filtered,
    has_cycle_filtered, strongly_connected_components_filtered, topological_sort_filtered,
    weakly_connected_components, weakly_connected_components_filtered,
};

#[test]
fn component_algorithms_match_petgraph_on_random_graphs() {
    let mut seed = 0xa076_1d64_78bd_642f_u64;
    for node_count in [0_usize, 1, 2, 9, 41] {
        for density in [1_u64, 11, 37] {
            let mut pairs = Vec::new();
            for source in 0..node_count {
                for target in 0..node_count {
                    seed = seed
                        .wrapping_mul(2_862_933_555_777_941_757)
                        .wrapping_add(3_037_000_493);
                    if seed % 101 < density {
                        pairs.push((source, target));
                    }
                }
            }
            compare_case(node_count, &pairs);
        }
    }
}

fn compare_case(node_count: usize, pairs: &[(usize, usize)]) {
    let ours = ours_graph(node_count, pairs);
    let pet = pet_graph(node_count, pairs);

    assert_eq!(
        normalize_ours(weakly_connected_components(&ours)),
        weak_pet(&pet)
    );
    assert_eq!(
        connected_components(&pet),
        weakly_connected_components(&ours).len()
    );
    compare_condensation(&ours, &pet);

    let allows = |index: usize| index % 3 != 1;
    let filtered_pairs = pairs
        .iter()
        .enumerate()
        .filter_map(|(index, &pair)| allows(index).then_some(pair))
        .collect::<Vec<_>>();
    let filtered_pet = pet_graph(node_count, &filtered_pairs);

    assert_eq!(
        normalize_ours(strongly_connected_components_filtered(&ours, |edge| {
            allows(edge.index())
        })),
        normalize_pet(kosaraju_scc(&filtered_pet))
    );
    assert_eq!(
        has_cycle_filtered(&ours, |edge| allows(edge.index())),
        is_cyclic_directed(&filtered_pet)
    );
    assert_eq!(
        topological_sort_filtered(&ours, |edge| allows(edge.index())).is_some(),
        toposort(&filtered_pet, None).is_ok()
    );
    assert_eq!(
        normalize_ours(weakly_connected_components_filtered(&ours, |edge| {
            allows(edge.index())
        })),
        weak_pet(&filtered_pet)
    );
    compare_condensation_filtered(&ours, &filtered_pet, allows);
}

fn compare_condensation(ours: &Topology, pet: &DiGraph<usize, ()>) {
    let ours = condensation(ours).unwrap();
    let pet = pet_condensation(pet.clone(), true);
    assert_condensation_parity(&ours, &pet);
}

fn compare_condensation_filtered(
    ours: &Topology,
    pet: &DiGraph<usize, ()>,
    allows: impl Fn(usize) -> bool,
) {
    let ours = condensation_filtered(ours, |edge| allows(edge.index())).unwrap();
    let pet = pet_condensation(pet.clone(), true);
    assert_condensation_parity(&ours, &pet);
}

fn assert_condensation_parity(
    ours: &weavatrix_graph::Condensation<NodeIndex>,
    pet: &DiGraph<Vec<usize>, ()>,
) {
    let ours_components = ours
        .components()
        .iter()
        .map(|component| component.iter().map(|node| node.index()).collect())
        .collect::<Vec<Vec<_>>>();
    let pet_components = pet.node_weights().cloned().collect::<Vec<_>>();
    assert_eq!(
        normalize(ours_components),
        normalize(pet_components),
        "component membership differs"
    );
    assert_eq!(ours.topology().edge_count(), pet.edge_count());
    assert_eq!(canonical_ours_edges(ours), canonical_pet_edges(pet));
}

fn canonical_ours_edges(
    graph: &weavatrix_graph::Condensation<NodeIndex>,
) -> BTreeSet<(Vec<usize>, Vec<usize>)> {
    graph
        .topology()
        .edge_indices()
        .map(|edge| {
            let endpoints = graph.topology().edge_endpoints(edge).unwrap();
            (
                component_key(graph.components(), endpoints.source().index()),
                component_key(graph.components(), endpoints.target().index()),
            )
        })
        .collect()
}

fn canonical_pet_edges(graph: &DiGraph<Vec<usize>, ()>) -> BTreeSet<(Vec<usize>, Vec<usize>)> {
    graph
        .edge_references()
        .map(|edge| {
            let mut source = graph[edge.source()].clone();
            let mut target = graph[edge.target()].clone();
            source.sort_unstable();
            target.sort_unstable();
            (source, target)
        })
        .collect()
}

fn component_key(components: &[Vec<NodeIndex>], index: usize) -> Vec<usize> {
    let mut component = components[index]
        .iter()
        .map(|node| node.index())
        .collect::<Vec<_>>();
    component.sort_unstable();
    component
}

fn weak_pet(graph: &DiGraph<usize, ()>) -> Vec<Vec<usize>> {
    let mut sets = UnionFind::new(graph.node_count());
    for edge in graph.edge_references() {
        sets.union(edge.source().index(), edge.target().index());
    }
    let mut components = BTreeMap::<usize, Vec<usize>>::new();
    for node in graph.node_indices() {
        components
            .entry(sets.find(node.index()))
            .or_default()
            .push(node.index());
    }
    normalize(components.into_values().collect())
}

fn normalize_ours(components: Vec<Vec<NodeIndex>>) -> Vec<Vec<usize>> {
    normalize(
        components
            .into_iter()
            .map(|component| component.into_iter().map(NodeIndex::index).collect())
            .collect(),
    )
}

fn normalize_pet(components: Vec<Vec<petgraph::graph::NodeIndex>>) -> Vec<Vec<usize>> {
    normalize(
        components
            .into_iter()
            .map(|component| {
                component
                    .into_iter()
                    .map(petgraph::graph::NodeIndex::index)
                    .collect()
            })
            .collect(),
    )
}

fn normalize(mut components: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    for component in &mut components {
        component.sort_unstable();
    }
    components.sort_unstable();
    components
}

fn ours_graph(node_count: usize, pairs: &[(usize, usize)]) -> Topology {
    Topology::try_from_edges(
        node_count,
        pairs.iter().map(|&(source, target)| {
            EdgeEndpoints::new(
                NodeIndex::new(u32::try_from(source).unwrap()),
                NodeIndex::new(u32::try_from(target).unwrap()),
            )
        }),
    )
    .unwrap()
}

fn pet_graph(node_count: usize, pairs: &[(usize, usize)]) -> DiGraph<usize, ()> {
    let mut graph = DiGraph::with_capacity(node_count, pairs.len());
    let nodes = (0..node_count)
        .map(|index| graph.add_node(index))
        .collect::<Vec<_>>();
    for &(source, target) in pairs {
        graph.add_edge(nodes[source], nodes[target], ());
    }
    graph
}
