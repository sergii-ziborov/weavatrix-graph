use weavatrix_graph::{
    Confidence, Edge, EdgeKind, EvidenceKind, Graph, GraphBuilder, GraphError, Node, NodeKind,
    Provenance, SourcePosition, SourceSpan,
};

fn node(id: &str) -> Node {
    Node::new(id, id, NodeKind::File).unwrap()
}

fn provenance(detail: &str) -> Provenance {
    Provenance::new("test.extractor", EvidenceKind::Parsed, Confidence::High)
        .unwrap()
        .with_detail(detail)
}

fn edge(source: &Node, target: &Node, detail: &str) -> Edge {
    Edge::new(
        source.id.clone(),
        target.id.clone(),
        EdgeKind::References,
        provenance(detail),
    )
}

#[test]
fn insertion_order_does_not_change_the_graph() {
    let alpha = node("alpha");
    let beta = node("beta");
    let gamma = node("gamma");

    let first = Graph::try_from_parts(
        [gamma.clone(), alpha.clone(), beta.clone()],
        [edge(&beta, &gamma, "second"), edge(&alpha, &beta, "first")],
    )
    .unwrap();
    let second = Graph::try_from_parts(
        [beta.clone(), gamma.clone(), alpha.clone()],
        [edge(&alpha, &beta, "first"), edge(&beta, &gamma, "second")],
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.nodes()[0].id.as_str(), "alpha");
    assert_eq!(first.nodes()[2].id.as_str(), "gamma");
}

#[test]
fn sorted_fast_path_matches_canonical_builder_and_falls_back_safely() {
    let alpha = node("alpha");
    let beta = node("beta");
    let gamma = node("gamma");
    let nodes = vec![alpha.clone(), beta.clone(), gamma.clone()];
    let mut edges = vec![edge(&alpha, &beta, "first"), edge(&beta, &gamma, "second")];
    edges.sort_unstable();

    let expected = Graph::try_from_parts(nodes.clone(), edges.clone()).unwrap();
    let sorted = Graph::try_from_sorted_parts(nodes.clone(), edges.clone()).unwrap();
    assert_eq!(sorted, expected);

    edges.reverse();
    let sorted_nodes = Graph::try_from_sorted_nodes(nodes.clone(), edges.clone()).unwrap();
    assert_eq!(sorted_nodes, expected);
    let fallback = Graph::try_from_sorted_parts(nodes, edges).unwrap();
    assert_eq!(fallback, expected);
}

#[test]
fn identical_nodes_are_idempotent_but_conflicts_are_rejected() {
    let original = node("same");
    let mut builder = GraphBuilder::new();
    builder.add_node(original.clone()).unwrap();
    builder.add_node(original).unwrap();
    assert_eq!(builder.build().unwrap().node_count(), 1);

    let mut builder = GraphBuilder::new();
    builder.add_node(node("same")).unwrap();
    let conflict = Node::new("same", "different", NodeKind::Function).unwrap();
    assert_eq!(
        builder.add_node(conflict).unwrap_err(),
        GraphError::ConflictingNode { id: "same".into() }
    );
}

#[test]
fn edges_may_precede_nodes_but_dangling_endpoints_are_rejected() {
    let source = node("source");
    let target = node("target");
    let relation = edge(&source, &target, "call site");

    let mut valid = GraphBuilder::new();
    valid.add_edge(relation.clone()).unwrap();
    valid.add_node(target.clone()).unwrap();
    valid.add_node(source.clone()).unwrap();
    assert_eq!(valid.build().unwrap().edge_count(), 1);

    let mut missing_source = GraphBuilder::new();
    missing_source.add_node(target.clone()).unwrap();
    missing_source.add_edge(relation.clone()).unwrap();
    assert_eq!(
        missing_source.build().unwrap_err(),
        GraphError::MissingEdgeSource {
            id: "source".into()
        }
    );

    let mut missing_target = GraphBuilder::new();
    missing_target.add_node(source).unwrap();
    missing_target.add_edge(relation).unwrap();
    assert_eq!(
        missing_target.build().unwrap_err(),
        GraphError::MissingEdgeTarget {
            id: "target".into()
        }
    );
}

#[test]
fn identical_edges_are_idempotent_but_distinct_evidence_is_preserved() {
    let source = node("source");
    let target = node("target");
    let first = edge(&source, &target, "line 1");
    let second = edge(&source, &target, "line 2");

    let graph =
        Graph::try_from_parts([source, target], [first.clone(), first, second.clone()]).unwrap();

    assert_eq!(graph.edge_count(), 2);
    assert!(graph.edges().contains(&second));
}

#[test]
fn incoming_outgoing_and_lookup_are_consistent() {
    let source = node("source");
    let target = node("target");
    let graph = Graph::try_from_parts(
        [source.clone(), target.clone()],
        [edge(&source, &target, "reference")],
    )
    .unwrap();

    assert_eq!(graph.node("target"), Some(&target));
    assert!(graph.node("missing").is_none());
    assert_eq!(graph.outgoing(&source.id).count(), 1);
    assert_eq!(graph.incoming(&target.id).count(), 1);
    assert_eq!(graph.outgoing(&target.id).count(), 0);
    let source_index = graph.node_index("source").unwrap();
    let target_index = graph.node_index("target").unwrap();
    assert_eq!(graph.node_at(source_index), Some(&source));
    assert_eq!(source_index.index(), 0);
    assert_eq!(graph.outgoing_at(source_index).count(), 1);
    assert_eq!(graph.incoming_at(target_index).count(), 1);
    assert_eq!(graph.out_degree(source_index), Some(1));
    assert_eq!(graph.in_degree(target_index), Some(1));
}

#[test]
fn invalid_spans_and_empty_extractors_are_rejected() {
    let invalid_span = SourceSpan::new(
        "src/lib.rs",
        SourcePosition::new(2, 1),
        SourcePosition::new(1, 1),
    );
    let invalid_node = node("node").with_span(invalid_span.clone());
    assert!(matches!(
        GraphBuilder::new().add_node(invalid_node),
        Err(GraphError::InvalidSpan { .. })
    ));

    assert_eq!(
        Provenance::new("", EvidenceKind::Parsed, Confidence::High).unwrap_err(),
        GraphError::EmptyExtractor
    );

    let source = node("source");
    let target = node("target");
    let invalid_edge = Edge::new(
        source.id.clone(),
        target.id.clone(),
        EdgeKind::Calls,
        Provenance {
            extractor: "test".into(),
            evidence: EvidenceKind::Parsed,
            confidence: Confidence::High,
            span: Some(invalid_span),
            detail: None,
        },
    );
    assert!(matches!(
        GraphBuilder::new().add_edge(invalid_edge),
        Err(GraphError::InvalidSpan { .. })
    ));
}
