use weavatrix_graph::{
    Confidence, Edge, EdgeKind, EvidenceKind, Graph, GraphError, IndexGraphView, Node, NodeKind,
    Provenance, WorkingGraph,
};

fn node(id: &str) -> Node {
    Node::new(id, id, NodeKind::File).unwrap()
}

fn edge(source: &Node, target: &Node, detail: &str) -> Edge {
    Edge::new(
        source.id.clone(),
        target.id.clone(),
        EdgeKind::References,
        Provenance::new("working-test", EvidenceKind::Parsed, Confidence::High)
            .unwrap()
            .with_detail(detail),
    )
}

#[test]
fn stable_keys_survive_updates_and_reject_reused_slots() {
    let mut graph = WorkingGraph::new();
    let alpha = graph.insert_node(node("alpha")).unwrap();
    let updated = Node::new("alpha", "updated", NodeKind::Function).unwrap();
    assert_eq!(
        graph.replace_node(alpha, updated.clone()).unwrap(),
        Some(node("alpha"))
    );
    assert_eq!(graph.node(alpha), Some(&updated));

    assert_eq!(graph.remove_node(alpha), Some(updated));
    let beta = graph.insert_node(node("beta")).unwrap();
    assert_eq!(beta.slot(), alpha.slot());
    assert_eq!(beta.generation(), alpha.generation() + 1);
    assert!(graph.node(alpha).is_none());
    assert_eq!(graph.node(beta), Some(&node("beta")));
}

#[test]
fn mutations_keep_adjacency_and_endpoint_ids_consistent() {
    let mut graph = WorkingGraph::new();
    let alpha_node = node("alpha");
    let beta_node = node("beta");
    let gamma_node = node("gamma");
    let alpha = graph.insert_node(alpha_node.clone()).unwrap();
    let beta = graph.insert_node(beta_node.clone()).unwrap();
    let gamma = graph.insert_node(gamma_node.clone()).unwrap();
    let relation = graph
        .insert_edge(edge(&alpha_node, &beta_node, "first"))
        .unwrap();

    let renamed = Node::new("renamed", "alpha", NodeKind::File).unwrap();
    graph.replace_node(alpha, renamed.clone()).unwrap();
    assert!(graph.node_key("alpha").is_none());
    assert_eq!(graph.node_key("renamed"), Some(alpha));
    assert_eq!(graph.edge(relation).unwrap().source, renamed.id);

    let replacement = edge(&gamma_node, &renamed, "moved");
    graph.replace_edge(relation, replacement.clone()).unwrap();
    assert_eq!(graph.outgoing_edges(alpha).len(), 0);
    assert_eq!(graph.incoming_edges(beta).len(), 0);
    assert_eq!(
        graph.outgoing_edges(gamma).collect::<Vec<_>>(),
        vec![relation]
    );
    assert_eq!(
        graph.incoming_edges(alpha).collect::<Vec<_>>(),
        vec![relation]
    );
    assert_eq!(graph.edge(relation), Some(&replacement));
}

#[test]
fn removing_a_node_cascades_incident_edges() {
    let mut graph = WorkingGraph::new();
    let alpha_node = node("alpha");
    let beta_node = node("beta");
    let alpha = graph.insert_node(alpha_node.clone()).unwrap();
    graph.insert_node(beta_node.clone()).unwrap();
    let relation = graph
        .insert_edge(edge(&alpha_node, &beta_node, "incident"))
        .unwrap();

    graph.remove_node(alpha);
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 0);
    assert!(graph.edge(relation).is_none());
}

#[test]
fn freeze_matches_canonical_graph_and_maps_duplicate_edges() {
    let mut working = WorkingGraph::new();
    let beta_node = node("beta");
    let alpha_node = node("alpha");
    let beta = working.insert_node(beta_node.clone()).unwrap();
    let alpha = working.insert_node(alpha_node.clone()).unwrap();
    let relation = edge(&alpha_node, &beta_node, "same");
    let first = working.insert_edge(relation.clone()).unwrap();
    let duplicate = working.insert_edge(relation.clone()).unwrap();

    let frozen = working.freeze().unwrap();
    let expected = Graph::try_from_parts([beta_node, alpha_node], [relation]).unwrap();
    assert_eq!(frozen.graph(), &expected);
    assert_eq!(frozen.indices().node(alpha).unwrap().index(), 0);
    assert_eq!(frozen.indices().node(beta).unwrap().index(), 1);
    assert_eq!(
        frozen.indices().edge(first),
        frozen.indices().edge(duplicate)
    );
}

#[test]
fn working_graph_implements_the_shared_graph_view() {
    fn inspect<G>(view: &G, node: G::Node, edge: G::Edge)
    where
        G: IndexGraphView,
    {
        assert_eq!(view.node_count(), view.node_indices().count());
        assert_eq!(view.edge_count(), view.edge_indices().count());
        assert!(view.contains_node(node));
        assert!(view.contains_edge(edge));
        assert!(view.edge_endpoints(edge).is_some());
        assert_eq!(view.outgoing_edges(node).count(), 1);
        assert_eq!(view.incoming_edges(node).count(), 0);
        assert!(G::node_slot(node) < view.node_bound());
        assert!(G::edge_slot(edge) < view.edge_bound());
    }

    let mut graph = WorkingGraph::new();
    let alpha_node = node("alpha");
    let beta_node = node("beta");
    let alpha = graph.insert_node(alpha_node.clone()).unwrap();
    graph.insert_node(beta_node.clone()).unwrap();
    let relation = graph
        .insert_edge(edge(&alpha_node, &beta_node, "view"))
        .unwrap();
    inspect(&graph, alpha, relation);
}

#[test]
fn working_insertions_validate_local_invariants_once() {
    let mut graph = WorkingGraph::new();
    graph.insert_node(node("alpha")).unwrap();
    assert_eq!(
        graph.insert_node(Node::new("alpha", "other", NodeKind::File).unwrap()),
        Err(GraphError::ConflictingNode { id: "alpha".into() })
    );

    let missing = edge(&node("alpha"), &node("missing"), "dangling");
    assert_eq!(
        graph.insert_edge(missing),
        Err(GraphError::MissingEdgeTarget {
            id: "missing".into()
        })
    );
}

#[test]
fn direct_edge_removal_reuses_slots_with_a_new_generation() {
    let mut graph = WorkingGraph::new();
    assert!(graph.is_empty());
    let alpha = node("alpha");
    let beta = node("beta");
    graph.insert_node(alpha.clone()).unwrap();
    graph.insert_node(beta.clone()).unwrap();
    let first = graph.insert_edge(edge(&alpha, &beta, "first")).unwrap();
    assert_eq!(graph.nodes().count(), 2);
    assert_eq!(graph.edges().count(), 1);
    assert!(graph.remove_edge(first).is_some());
    assert!(graph.edge(first).is_none());
    let second = graph.insert_edge(edge(&alpha, &beta, "second")).unwrap();
    assert_eq!(second.slot(), first.slot());
    assert_eq!(second.generation(), first.generation() + 1);
}
