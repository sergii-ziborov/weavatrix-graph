use serde_json::{Value, json};
use weavatrix_graph::{
    Confidence, Edge, EdgeKind, EvidenceKind, Graph, Language, Node, NodeId, NodeKind, Provenance,
};

#[test]
fn known_and_custom_kinds_have_a_stable_string_contract() {
    let known = serde_json::to_string(&NodeKind::KubernetesResource).unwrap();
    assert_eq!(known, "\"kubernetes_resource\"");
    assert_eq!(
        serde_json::from_str::<NodeKind>(&known).unwrap(),
        NodeKind::KubernetesResource
    );

    let custom = NodeKind::custom("terraform_resource").unwrap();
    let encoded = serde_json::to_string(&custom).unwrap();
    assert_eq!(encoded, "\"terraform_resource\"");
    assert_eq!(serde_json::from_str::<NodeKind>(&encoded).unwrap(), custom);

    assert!(Language::custom("").is_err());
    assert!(EdgeKind::custom(" leading-space").is_err());
    assert!(EvidenceKind::custom("trailing-space ").is_err());
}

#[test]
fn empty_node_ids_are_rejected_by_construction_and_deserialization() {
    assert!(NodeId::new("").is_err());
    assert!(serde_json::from_str::<NodeId>("\"\"").is_err());
}

#[test]
fn graph_json_round_trip_preserves_the_canonical_graph() {
    let repository = Node::new("repo:demo", "demo", NodeKind::Repository).unwrap();
    let file = Node::new("file:src/lib.rs", "src/lib.rs", NodeKind::File)
        .unwrap()
        .with_language(Language::Rust);
    let graph = Graph::try_from_parts(
        [file.clone(), repository.clone()],
        [Edge::new(
            repository.id,
            file.id,
            EdgeKind::Contains,
            Provenance::new("test", EvidenceKind::Parsed, Confidence::Exact).unwrap(),
        )],
    )
    .unwrap();

    let encoded = serde_json::to_string(&graph).unwrap();
    let decoded: Graph = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, graph);
}

#[test]
fn deserialization_cannot_bypass_dangling_edge_validation() {
    let value = json!({
        "nodes": [{
            "id": "source",
            "label": "source",
            "kind": "file"
        }],
        "edges": [{
            "source": "source",
            "target": "missing",
            "kind": "references",
            "provenance": {
                "extractor": "test",
                "evidence": "parsed",
                "confidence": "high"
            }
        }]
    });

    let error = serde_json::from_value::<Graph>(value).unwrap_err();
    assert!(error.to_string().contains("edge target does not exist"));
}

#[test]
fn deserialization_cannot_bypass_conflicting_node_validation() {
    let value: Value = json!({
        "nodes": [
            { "id": "same", "label": "first", "kind": "file" },
            { "id": "same", "label": "second", "kind": "function" }
        ],
        "edges": []
    });

    let error = serde_json::from_value::<Graph>(value).unwrap_err();
    assert!(error.to_string().contains("conflicting definitions"));
}
