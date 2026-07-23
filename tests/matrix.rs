use weavatrix_graph::{DenseMatrix, GraphError, NodeIndex};

#[test]
fn dense_matrix_supports_directed_and_undirected_edges() {
    let mut matrix = DenseMatrix::try_new(3).unwrap();
    assert_eq!(
        matrix
            .insert(NodeIndex::new(0), NodeIndex::new(1), 7)
            .unwrap(),
        None
    );
    assert_eq!(
        matrix
            .insert(NodeIndex::new(0), NodeIndex::new(1), 8)
            .unwrap(),
        Some(7)
    );
    matrix
        .insert_undirected(NodeIndex::new(1), NodeIndex::new(2), 9)
        .unwrap();

    assert_eq!(matrix.edge_count(), 3);
    assert_eq!(matrix.get(NodeIndex::new(0), NodeIndex::new(1)), Some(&8));
    assert_eq!(
        matrix.outgoing(NodeIndex::new(1)).collect::<Vec<_>>(),
        vec![(NodeIndex::new(2), &9)]
    );
    assert_eq!(
        matrix.incoming(NodeIndex::new(1)).collect::<Vec<_>>(),
        vec![(NodeIndex::new(0), &8), (NodeIndex::new(2), &9)]
    );
    assert_eq!(matrix.remove(NodeIndex::new(2), NodeIndex::new(1)), Some(9));
    assert_eq!(matrix.edge_count(), 2);
    *matrix
        .get_mut(NodeIndex::new(0), NodeIndex::new(1))
        .unwrap() = 10;
    assert_eq!(matrix.edges().count(), 2);
    assert_eq!(matrix.get(NodeIndex::new(0), NodeIndex::new(1)), Some(&10));
    matrix
        .insert_undirected(NodeIndex::new(2), NodeIndex::new(2), 11)
        .unwrap();
    assert_eq!(matrix.get(NodeIndex::new(2), NodeIndex::new(2)), Some(&11));
}

#[test]
fn dense_matrix_rejects_invalid_indices_without_mutation() {
    let mut matrix = DenseMatrix::<u8>::try_new(2).unwrap();
    assert_eq!(
        matrix.insert(NodeIndex::new(2), NodeIndex::new(0), 1),
        Err(GraphError::InvalidNodeIndex {
            node: 2,
            node_count: 2
        })
    );
    assert_eq!(matrix.edge_count(), 0);
    assert_eq!(
        matrix.insert(NodeIndex::new(0), NodeIndex::new(2), 1),
        Err(GraphError::InvalidNodeIndex {
            node: 2,
            node_count: 2
        })
    );
    assert_eq!(matrix.outgoing(NodeIndex::new(9)).count(), 0);
    assert_eq!(matrix.incoming(NodeIndex::new(9)).count(), 0);
    assert_eq!(matrix.remove(NodeIndex::new(0), NodeIndex::new(1)), None);
}

#[test]
fn dense_matrix_deserialization_recomputes_derived_edge_count() {
    let json = r#"{"node_count":2,"edge_count":99,"cells":[null,4,null,null]}"#;
    let matrix: DenseMatrix<u8> = serde_json::from_str(json).unwrap();
    assert_eq!(matrix.edge_count(), 1);
    assert_eq!(matrix.get(NodeIndex::new(0), NodeIndex::new(1)), Some(&4));
    assert!(
        serde_json::from_str::<DenseMatrix<u8>>(
            r#"{"node_count":2,"edge_count":0,"cells":[null]}"#
        )
        .is_err()
    );
}

#[test]
fn empty_dense_matrix_iterates_without_division_by_zero() {
    let matrix = DenseMatrix::<u8>::try_new(0).unwrap();
    assert_eq!(matrix.edges().count(), 0);
}
