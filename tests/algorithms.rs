use weavatrix_graph::{
    Confidence, Direction, Edge, EdgeEndpoints, EdgeFilter, EdgeKind, EvidenceKind, Graph, Node,
    NodeIndex, NodeKind, Provenance, Topology, bfs, bfs_filtered, dfs, dijkstra, dijkstra_filtered,
    find_cycle, has_cycle, maximum_flow, reachable, shortest_path, strongly_connected_components,
    topological_sort,
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
fn breadth_depth_reachability_and_shortest_path_are_deterministic() {
    let graph = topology(6, &[(0, 1), (0, 2), (1, 3), (2, 4), (3, 5), (4, 5)]);
    let zero = NodeIndex::new(0);
    let five = NodeIndex::new(5);

    assert_eq!(bfs(&graph, zero), [0, 1, 2, 3, 4, 5].map(NodeIndex::new));
    assert_eq!(dfs(&graph, zero), [0, 1, 3, 5, 2, 4].map(NodeIndex::new));
    assert!(reachable(&graph, zero, five));
    assert!(!reachable(&graph, five, zero));
    assert_eq!(
        shortest_path(&graph, zero, five).unwrap(),
        [0, 1, 3, 5].map(NodeIndex::new)
    );
}

#[test]
fn direction_and_edge_filters_limit_traversal() {
    let nodes = ["a", "b", "c"].map(|id| Node::new(id, id, NodeKind::File).unwrap());
    let provenance = Provenance::new("parser", EvidenceKind::Parsed, Confidence::High).unwrap();
    let graph = Graph::try_from_parts(
        nodes.clone(),
        [
            Edge::new(
                nodes[0].id.clone(),
                nodes[1].id.clone(),
                EdgeKind::Calls,
                provenance.clone(),
            ),
            Edge::new(
                nodes[1].id.clone(),
                nodes[2].id.clone(),
                EdgeKind::References,
                provenance,
            ),
        ],
    )
    .unwrap();
    let a = graph.node_index("a").unwrap();
    let b = graph.node_index("b").unwrap();
    let c = graph.node_index("c").unwrap();

    let filter = EdgeFilter::new()
        .with_kind(EdgeKind::Calls)
        .with_evidence(EvidenceKind::Parsed)
        .with_extractor("parser")
        .with_minimum_confidence(Confidence::High);
    let calls_only = bfs_filtered(&graph, a, Direction::Outgoing, |index| {
        graph
            .edge_at(index)
            .is_some_and(|edge| filter.matches(edge))
    });
    assert_eq!(calls_only, vec![a, b]);
    assert_eq!(
        bfs_filtered(&graph, c, Direction::Incoming, |_| true),
        vec![c, b, a]
    );
}

#[test]
fn components_cycles_and_topological_order_share_one_view_contract() {
    let cyclic = topology(6, &[(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 3)]);
    let mut components = strongly_connected_components(&cyclic)
        .into_iter()
        .map(|mut component| {
            component.sort_unstable();
            component
        })
        .collect::<Vec<_>>();
    components.sort_unstable();
    assert_eq!(
        components,
        vec![
            vec![NodeIndex::new(0), NodeIndex::new(1), NodeIndex::new(2)],
            vec![NodeIndex::new(3), NodeIndex::new(4)],
            vec![NodeIndex::new(5)]
        ]
    );
    assert!(has_cycle(&cyclic));
    assert_eq!(
        find_cycle(&cyclic).unwrap(),
        [0, 1, 2, 0].map(NodeIndex::new)
    );
    assert!(topological_sort(&cyclic).is_none());

    let dag = topology(5, &[(0, 2), (1, 2), (2, 3), (2, 4)]);
    let order = topological_sort(&dag).unwrap();
    assert!(find_cycle(&dag).is_none());
    let position = |node| {
        order
            .iter()
            .position(|candidate| *candidate == node)
            .unwrap()
    };
    for (source, target) in [(0, 2), (1, 2), (2, 3), (2, 4)] {
        assert!(position(NodeIndex::new(source)) < position(NodeIndex::new(target)));
    }
}

#[test]
fn missing_or_stale_indices_are_safe() {
    let graph = topology(1, &[]);
    let missing = NodeIndex::new(7);
    assert!(bfs(&graph, missing).is_empty());
    assert!(!reachable(&graph, NodeIndex::new(0), missing));
    assert!(shortest_path(&graph, NodeIndex::new(0), missing).is_none());
}

#[test]
fn dijkstra_supports_weights_filters_and_overflow_safe_relaxation() {
    let graph = topology(4, &[(0, 1), (0, 2), (2, 1), (1, 3), (2, 3)]);
    let weights = [10_u64, 2, 1, 2, 20];
    let path = dijkstra(&graph, NodeIndex::new(0), NodeIndex::new(3), |edge| {
        weights[edge.index()]
    })
    .unwrap();
    assert_eq!(path.nodes(), [0, 2, 1, 3].map(NodeIndex::new).as_slice());
    assert_eq!(path.total_cost(), 5);

    let filtered = dijkstra_filtered(
        &graph,
        NodeIndex::new(0),
        NodeIndex::new(3),
        Direction::Outgoing,
        |edge| (edge.index() != 2).then_some(weights[edge.index()]),
    )
    .unwrap();
    assert_eq!(filtered.total_cost(), 12);
    assert_eq!(
        filtered.clone().into_nodes().first(),
        Some(&NodeIndex::new(0))
    );

    let overflow = topology(3, &[(0, 1), (1, 2)]);
    assert!(
        dijkstra(&overflow, NodeIndex::new(0), NodeIndex::new(2), |edge| {
            [u64::MAX, 1][edge.index()]
        })
        .is_none()
    );
}

#[test]
fn dinic_reports_edge_flows_and_the_min_cut_source_partition() {
    let graph = topology(
        6,
        &[
            (0, 1),
            (0, 2),
            (1, 2),
            (2, 1),
            (1, 3),
            (2, 4),
            (3, 2),
            (4, 3),
            (3, 5),
            (4, 5),
        ],
    );
    let capacities = [16_u64, 13, 10, 4, 12, 14, 9, 7, 20, 4];
    let flow = maximum_flow(&graph, NodeIndex::new(0), NodeIndex::new(5), |edge| {
        capacities[edge.index()]
    })
    .unwrap()
    .unwrap();

    assert_eq!(flow.value(), 23);
    assert_eq!(flow.edge_flows().len(), capacities.len());
    assert_eq!(flow.source_side(), [0, 1, 2, 4].map(NodeIndex::new));
    for &(edge, value) in flow.edge_flows() {
        assert!(value <= capacities[edge.index()]);
    }
    assert_eq!(
        maximum_flow(&graph, NodeIndex::new(0), NodeIndex::new(0), |_| 1)
            .unwrap()
            .unwrap()
            .value(),
        0
    );
}
