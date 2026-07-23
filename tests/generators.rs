use weavatrix_graph::{
    GraphError, NodeIndex, RandomGraphGenerator, complete_topology, cycle_topology, path_topology,
};

#[test]
fn deterministic_generators_cover_standard_shapes() {
    let path = path_topology(4).unwrap();
    assert_eq!(path.edge_count(), 3);
    assert_eq!(
        path.outgoing_neighbors(NodeIndex::new(1))
            .collect::<Vec<_>>(),
        vec![NodeIndex::new(2)]
    );

    let cycle = cycle_topology(4).unwrap();
    assert_eq!(cycle.edge_count(), 4);
    assert_eq!(
        cycle
            .outgoing_neighbors(NodeIndex::new(3))
            .collect::<Vec<_>>(),
        vec![NodeIndex::new(0)]
    );
    assert_eq!(cycle_topology(1).unwrap().edge_count(), 1);
    assert_eq!(cycle_topology(0).unwrap().edge_count(), 0);

    let complete = complete_topology(5).unwrap();
    assert_eq!(complete.edge_count(), 20);
    assert_eq!(complete.out_degree(NodeIndex::new(2)), Some(4));
}

#[test]
fn seeded_random_generators_are_reproducible_and_bounded() {
    let mut first = RandomGraphGenerator::new(42);
    let mut second = RandomGraphGenerator::new(42);
    assert_eq!(
        first.directed(20, 1, 5).unwrap(),
        second.directed(20, 1, 5).unwrap()
    );

    assert_eq!(
        RandomGraphGenerator::new(1)
            .directed(8, 0, 1)
            .unwrap()
            .edge_count(),
        0
    );
    assert_eq!(
        RandomGraphGenerator::new(1)
            .directed(8, 1, 1)
            .unwrap()
            .edge_count(),
        56
    );
    assert_eq!(
        RandomGraphGenerator::new(1)
            .undirected(8, 1, 1)
            .unwrap()
            .edge_count(),
        28
    );
}

#[test]
fn random_generators_reject_invalid_probabilities() {
    assert_eq!(
        RandomGraphGenerator::new(0).directed(5, 2, 1),
        Err(GraphError::InvalidProbability {
            numerator: 2,
            denominator: 1
        })
    );
    assert_eq!(
        RandomGraphGenerator::new(0).undirected(5, 0, 0),
        Err(GraphError::InvalidProbability {
            numerator: 0,
            denominator: 0
        })
    );
    assert!(matches!(
        complete_topology(100_000),
        Err(GraphError::IndexCapacityExceeded {
            category: "generated edges",
            ..
        })
    ));
}
