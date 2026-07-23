use std::collections::BTreeSet;
use weavatrix_graph::{
    EdgeEndpoints, GraphView, NodeIndex, Topology, condensation, condensation_filtered,
    find_cycle_filtered, has_cycle, has_cycle_filtered, strongly_connected_components_filtered,
    topological_sort_filtered, weakly_connected_components, weakly_connected_components_filtered,
};

fn topology(node_count: usize, edges: &[(u32, u32)]) -> Topology {
    Topology::try_from_edges(
        node_count,
        edges.iter().map(|&(source, target)| {
            EdgeEndpoints::new(NodeIndex::new(source), NodeIndex::new(target))
        }),
    )
    .unwrap()
}

fn normalize(mut components: Vec<Vec<NodeIndex>>) -> Vec<Vec<NodeIndex>> {
    for component in &mut components {
        component.sort_unstable();
    }
    components.sort_unstable();
    components
}

#[test]
fn filtered_directed_algorithms_share_the_same_semantic_subgraph() {
    let graph = topology(4, &[(0, 1), (1, 0), (1, 2), (2, 3), (3, 2)]);
    let allows = |edge: weavatrix_graph::EdgeIndex| !matches!(edge.index(), 1 | 4);

    assert!(has_cycle(&graph));
    assert_eq!(
        normalize(strongly_connected_components_filtered(&graph, allows)),
        [0, 1, 2, 3].map(|node| vec![NodeIndex::new(node)]).to_vec()
    );
    assert!(!has_cycle_filtered(&graph, allows));
    assert!(find_cycle_filtered(&graph, allows).is_none());

    let order = topological_sort_filtered(&graph, allows).unwrap();
    let position = |node| {
        order
            .iter()
            .position(|candidate| *candidate == NodeIndex::new(node))
            .unwrap()
    };
    assert!(position(0) < position(1));
    assert!(position(1) < position(2));
    assert!(position(2) < position(3));
}

#[test]
fn weak_components_ignore_direction_and_honor_filters() {
    let graph = topology(5, &[(0, 1), (2, 1), (3, 4)]);
    assert_eq!(
        normalize(weakly_connected_components(&graph)),
        vec![
            [0, 1, 2].map(NodeIndex::new).to_vec(),
            [3, 4].map(NodeIndex::new).to_vec(),
        ]
    );
    assert_eq!(
        normalize(weakly_connected_components_filtered(&graph, |edge| {
            edge.index() != 1
        })),
        vec![
            [0, 1].map(NodeIndex::new).to_vec(),
            vec![NodeIndex::new(2)],
            [3, 4].map(NodeIndex::new).to_vec(),
        ]
    );
}

#[test]
fn condensation_deduplicates_cross_component_edges_into_a_dag() {
    let graph = topology(
        6,
        &[
            (0, 1),
            (1, 0),
            (1, 2),
            (0, 2),
            (2, 3),
            (3, 2),
            (3, 4),
            (4, 5),
        ],
    );
    let condensed = condensation(&graph).unwrap();

    assert_eq!(condensed.components().len(), 4);
    assert!(!has_cycle(condensed.topology()));
    assert_eq!(condensed.topology().edge_count(), 3);
    assert_eq!(
        condensed.component_of(NodeIndex::new(0)),
        condensed.component_of(NodeIndex::new(1))
    );
    assert_eq!(
        condensed.component_of(NodeIndex::new(2)),
        condensed.component_of(NodeIndex::new(3))
    );
    assert_ne!(
        condensed.component_of(NodeIndex::new(1)),
        condensed.component_of(NodeIndex::new(2))
    );

    let expected = [(1_u32, 2_u32), (3, 4), (4, 5)]
        .map(|(source, target)| {
            (
                condensed.component_of(NodeIndex::new(source)).unwrap(),
                condensed.component_of(NodeIndex::new(target)).unwrap(),
            )
        })
        .into_iter()
        .collect::<BTreeSet<_>>();
    let actual = condensed
        .topology()
        .edge_indices()
        .map(|edge| {
            let endpoints = condensed.topology().edge_endpoints(edge).unwrap();
            (endpoints.source(), endpoints.target())
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);

    let component = condensed.component_of(NodeIndex::new(0)).unwrap();
    assert_eq!(
        condensed.component(component).unwrap(),
        [0, 1].map(NodeIndex::new)
    );
}

#[test]
fn filtered_condensation_uses_one_consistent_edge_predicate() {
    let graph = topology(4, &[(0, 1), (1, 0), (1, 2), (2, 3)]);
    let condensed = condensation_filtered(&graph, |edge| edge.index() != 1).unwrap();

    assert_eq!(condensed.components().len(), 4);
    assert_eq!(condensed.topology().edge_count(), 3);
    assert!(!has_cycle(condensed.topology()));
}

#[test]
fn empty_condensation_round_trips_into_empty_parts() {
    let graph = topology(0, &[]);
    let (components, topology) = condensation(&graph).unwrap().into_parts();
    assert!(components.is_empty());
    assert_eq!(topology.node_count(), 0);
    assert_eq!(topology.edge_count(), 0);
}
