use serde_json::{Value, json};
use weavatrix_graph::{
    AttributeValue, Confidence, Edge, EdgeKind, EvidenceKind, FiniteF64, Graph, LegacyGraph, Node,
    NodeId, NodeKind, Provenance,
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

    assert!(EdgeKind::custom(" leading-space").is_err());
    assert!(EvidenceKind::custom("trailing-space ").is_err());

    assert_eq!(
        serde_json::from_str::<EdgeKind>("\"re_exports\"").unwrap(),
        EdgeKind::ReExports
    );
    assert_eq!(
        serde_json::from_str::<EvidenceKind>("\"EXACT_LSP\"").unwrap(),
        EvidenceKind::ExactLsp
    );
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
        .with_language("rust");
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
    let wire: Value = serde_json::from_str(&encoded).unwrap();
    let keys = wire.as_object().unwrap().keys().collect::<Vec<_>>();
    assert_eq!(keys, ["edges", "nodes"]);
    let decoded: Graph = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, graph);
}

#[test]
fn graph_json_round_trip_preserves_structured_attributes() {
    let source = Node::new("src", "src", NodeKind::File)
        .unwrap()
        .with_attribute("physical_loc", 42_i64)
        .with_attribute("generated", false);
    let target = Node::new("target", "target", NodeKind::Function).unwrap();
    let edge = Edge::new(
        source.id.clone(),
        target.id.clone(),
        EdgeKind::References,
        Provenance::new("test", EvidenceKind::Extracted, Confidence::High).unwrap(),
    )
    .with_attribute("compileOnly", true)
    .with_attribute(
        "source_range",
        AttributeValue::Object(
            [("line".to_owned(), AttributeValue::Integer(7))]
                .into_iter()
                .collect(),
        ),
    );
    let graph = Graph::try_from_parts([source, target], [edge]).unwrap();

    let decoded: Graph = serde_json::from_str(&serde_json::to_string(&graph).unwrap()).unwrap();

    assert_eq!(
        decoded.nodes()[0].attributes.get("physical_loc"),
        Some(&AttributeValue::Integer(42))
    );
    assert_eq!(
        decoded.edges()[0].attributes.get("compileOnly"),
        Some(&AttributeValue::Bool(true))
    );

    let numeric: AttributeValue = serde_json::from_str("1.25").unwrap();
    assert_eq!(
        numeric,
        AttributeValue::Float(FiniteF64::new(1.25).unwrap())
    );
    let unsigned: AttributeValue = serde_json::from_str("18446744073709551615").unwrap();
    assert_eq!(unsigned, AttributeValue::Unsigned(u64::MAX));
}

#[test]
fn legacy_nodes_links_convert_without_losing_weavatrix_edge_details() {
    let legacy: LegacyGraph = serde_json::from_value(json!({
        "nodes": [
            { "id": "src/lib.rs", "label": "lib.rs", "source_file": "src/lib.rs" },
            {
                "id": "src/lib.rs#entry@3",
                "label": "entry()",
                "source_file": "src/lib.rs",
                "source_range": {
                    "start": { "line": 2, "character": 0 },
                    "end": { "line": 4, "character": 1 }
                },
                "callable": true,
                "complexity": { "cyclomatic": 1 }
            }
        ],
        "links": [
            {
                "source": "src/lib.rs",
                "target": "src/lib.rs#entry@3",
                "relation": "contains",
                "confidence": "EXTRACTED"
            },
            {
                "source": "src/lib.rs#entry@3",
                "target": "src/lib.rs",
                "relation": "references",
                "confidence": "INFERRED",
                "provenance": "RESOLVED",
                "line": 3,
                "compileOnly": true,
                "specifier": "crate::helper"
            }
        ],
        "edgeTypesV": 2,
        "edgeProvenanceV": 1
    }))
    .unwrap();

    let graph = legacy.into_graph("legacy-test").unwrap();

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 2);
    let reference = graph
        .edges()
        .iter()
        .find(|edge| edge.kind == EdgeKind::References)
        .unwrap();
    assert_eq!(
        reference.provenance.evidence,
        EvidenceKind::ResolvedCanonical
    );
    assert_eq!(reference.provenance.confidence, Confidence::Low);
    assert_eq!(
        reference.attributes.get("compileOnly"),
        Some(&AttributeValue::Bool(true))
    );
    assert_eq!(
        reference.attributes.get("specifier"),
        Some(&AttributeValue::String("crate::helper".into()))
    );
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
