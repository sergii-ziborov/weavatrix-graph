use weavatrix_graph::{
    Direction, EdgeEndpoints, GraphError, NodeIndex, Topology, astar, astar_filtered, bellman_ford,
    bellman_ford_filtered,
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

#[test]
fn astar_finds_the_weighted_path_and_honors_direction_and_filtering() {
    let graph = topology(5, &[(0, 1), (0, 2), (1, 3), (2, 3), (3, 4)]);
    let weights = [2_u64, 2, 2, 7, 1];
    let heuristic = [5_u64, 3, 7, 1, 0];
    let path = astar(
        &graph,
        NodeIndex::new(0),
        NodeIndex::new(4),
        |edge| weights[edge.index()],
        |node| heuristic[node.index()],
    )
    .unwrap();
    assert_eq!(path.nodes(), [0, 1, 3, 4].map(NodeIndex::new));
    assert_eq!(path.total_cost(), 5);

    let reverse = astar_filtered(
        &graph,
        NodeIndex::new(4),
        NodeIndex::new(0),
        Direction::Incoming,
        |edge| (edge.index() != 2).then_some(weights[edge.index()]),
        |_| 0,
    )
    .unwrap();
    assert_eq!(reverse.nodes(), [4, 3, 2, 0].map(NodeIndex::new));
    assert_eq!(reverse.total_cost(), 10);
    assert!(astar(&graph, NodeIndex::new(0), NodeIndex::new(9), |_| 1, |_| 0).is_none());
}

#[test]
fn bellman_ford_handles_negative_edges_and_reconstructs_paths() {
    let graph = topology(5, &[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)]);
    let weights = [4_i64, 5, -2, 6, 3];
    let result = bellman_ford(&graph, NodeIndex::new(0), |edge| weights[edge.index()])
        .unwrap()
        .unwrap();

    assert_eq!(result.source(), NodeIndex::new(0));
    assert_eq!(result.distance_to(NodeIndex::new(3)), Some(5));
    assert_eq!(
        result.path_to(NodeIndex::new(3)).unwrap().nodes(),
        [0, 1, 2, 3].map(NodeIndex::new)
    );
    assert_eq!(result.path_to(NodeIndex::new(3)).unwrap().total_cost(), 5);
    assert!(result.distance_to(NodeIndex::new(4)).is_none());
    assert!(
        bellman_ford(&graph, NodeIndex::new(9), |_| 1)
            .unwrap()
            .is_none()
    );
}

#[test]
fn bellman_ford_rejects_only_reachable_negative_cycles_and_overflow() {
    let graph = topology(5, &[(0, 1), (1, 2), (2, 1), (3, 4), (4, 3)]);
    let error = bellman_ford(&graph, NodeIndex::new(0), |edge| {
        [1_i64, -3, 1, -5, 1][edge.index()]
    })
    .unwrap_err();
    assert!(matches!(error, GraphError::NegativeCycle { .. }));

    let filtered = bellman_ford_filtered(&graph, NodeIndex::new(0), |edge| {
        (edge.index() != 1).then_some(1)
    })
    .unwrap()
    .unwrap();
    assert_eq!(filtered.distance_to(NodeIndex::new(2)), None);

    let overflow = topology(3, &[(0, 1), (1, 2)]);
    let error = bellman_ford(&overflow, NodeIndex::new(0), |edge| {
        [i64::MAX, 1][edge.index()]
    })
    .unwrap_err();
    assert!(matches!(error, GraphError::ArithmeticOverflow { .. }));
}
