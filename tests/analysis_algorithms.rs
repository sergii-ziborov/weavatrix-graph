use std::collections::BTreeSet;
use weavatrix_graph::{
    EdgeEndpoints, GraphError, NodeIndex, Topology, dag_transitive_reduction_closure,
    dag_transitive_reduction_closure_filtered, dominators, dominators_filtered, page_rank,
    page_rank_filtered,
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
fn page_rank_is_normalized_deterministic_and_filterable() {
    let cycle = topology(3, &[(0, 1), (1, 2), (2, 0)]);
    let ranks = page_rank(&cycle, 0.85, 20).unwrap();
    assert_eq!(ranks.len(), 3);
    for (_, rank) in &ranks {
        assert!((*rank - 1.0 / 3.0).abs() < 1.0e-12);
    }
    assert!((ranks.iter().map(|(_, rank)| rank).sum::<f64>() - 1.0).abs() < 1.0e-12);

    let graph = topology(4, &[(0, 1), (0, 2), (1, 2), (2, 0)]);
    let filtered = page_rank_filtered(&graph, 0.85, 50, |edge| edge.index() != 1).unwrap();
    assert_eq!(filtered.len(), 4);
    assert!((filtered.iter().map(|(_, rank)| rank).sum::<f64>() - 1.0).abs() < 1.0e-12);
    assert!((filtered[0].1 - filtered[1].1).abs() < 1.0e-12);
    assert!((filtered[1].1 - filtered[2].1).abs() < 1.0e-12);
    assert!(filtered[0].1 > filtered[3].1);
    assert!(page_rank(&graph, f64::NAN, 1).is_err());
    assert!(matches!(
        page_rank(&graph, 1.1, 1),
        Err(GraphError::InvalidAlgorithmParameter { .. })
    ));
}

#[test]
fn dominators_model_control_flow_and_ignore_unreachable_nodes() {
    let graph = topology(7, &[(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (3, 5), (4, 5)]);
    let result = dominators(&graph, NodeIndex::new(0)).unwrap();

    assert_eq!(
        result.immediate_dominator(NodeIndex::new(1)),
        Some(NodeIndex::new(0))
    );
    assert_eq!(
        result.immediate_dominator(NodeIndex::new(3)),
        Some(NodeIndex::new(0))
    );
    assert_eq!(
        result.immediate_dominator(NodeIndex::new(4)),
        Some(NodeIndex::new(3))
    );
    assert_eq!(
        result.immediate_dominator(NodeIndex::new(5)),
        Some(NodeIndex::new(3))
    );
    assert!(result.dominates(NodeIndex::new(3), NodeIndex::new(5)));
    assert!(!result.dominates(NodeIndex::new(4), NodeIndex::new(5)));
    assert!(result.dominators(NodeIndex::new(6)).is_none());
    assert_eq!(
        result
            .strict_dominators(NodeIndex::new(5))
            .unwrap()
            .collect::<Vec<_>>(),
        [NodeIndex::new(3), NodeIndex::new(0)]
    );
    assert_eq!(
        result.immediately_dominated_by(NodeIndex::new(3)),
        [NodeIndex::new(4), NodeIndex::new(5)]
    );
    assert!(dominators(&graph, NodeIndex::new(9)).is_none());

    let filtered =
        dominators_filtered(&graph, NodeIndex::new(0), |edge| edge.index() != 1).unwrap();
    assert_eq!(
        filtered.immediate_dominator(NodeIndex::new(3)),
        Some(NodeIndex::new(1))
    );
}

#[test]
fn dag_transitive_results_are_canonical_and_reject_cycles() {
    let graph = topology(4, &[(0, 1), (1, 2), (0, 2), (2, 3), (0, 3), (0, 2)]);
    let result = dag_transitive_reduction_closure(&graph).unwrap();

    assert_eq!(
        endpoints(result.reduction_edges()),
        BTreeSet::from([(0, 1), (1, 2), (2, 3)])
    );
    assert_eq!(
        endpoints(result.closure_edges()),
        BTreeSet::from([(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)])
    );
    let (reduction, closure) = result.into_parts();
    assert_eq!(reduction.len(), 3);
    assert_eq!(closure.len(), 6);

    let cycle = topology(2, &[(0, 1), (1, 0)]);
    assert!(dag_transitive_reduction_closure(&cycle).is_none());
    let filtered =
        dag_transitive_reduction_closure_filtered(&cycle, |edge| edge.index() == 0).unwrap();
    assert_eq!(
        endpoints(filtered.reduction_edges()),
        BTreeSet::from([(0, 1)])
    );
}

fn endpoints(edges: &[EdgeEndpoints<NodeIndex>]) -> BTreeSet<(usize, usize)> {
    edges
        .iter()
        .map(|edge| (edge.source().index(), edge.target().index()))
        .collect()
}
