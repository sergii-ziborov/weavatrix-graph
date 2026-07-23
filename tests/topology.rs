use weavatrix_graph::{EdgeEndpoints, EdgeIndex, GraphError, GraphView, NodeIndex, Topology};

fn node(index: u32) -> NodeIndex {
    NodeIndex::new(index)
}

#[test]
fn dual_csr_indexes_edges_and_neighbors_in_both_directions() {
    let topology = Topology::try_from_edges(
        4,
        [
            EdgeEndpoints::new(node(0), node(1)),
            EdgeEndpoints::new(node(0), node(2)),
            EdgeEndpoints::new(node(2), node(0)),
            EdgeEndpoints::new(node(1), node(2)),
        ],
    )
    .unwrap();

    assert_eq!(topology.node_count(), 4);
    assert_eq!(topology.edge_count(), 4);
    assert_eq!(
        topology.outgoing_edges(node(0)).collect::<Vec<_>>(),
        [EdgeIndex::new(0), EdgeIndex::new(1)]
    );
    assert_eq!(
        topology.incoming_edges(node(2)).collect::<Vec<_>>(),
        [EdgeIndex::new(1), EdgeIndex::new(3)]
    );
    assert_eq!(
        topology.outgoing_neighbors(node(0)).collect::<Vec<_>>(),
        [node(1), node(2)]
    );
    assert_eq!(
        topology.incoming_neighbors(node(2)).collect::<Vec<_>>(),
        [node(0), node(1)]
    );
    assert_eq!(topology.out_degree(node(3)), Some(0));
    assert_eq!(topology.in_degree(node(3)), Some(0));
    assert_eq!(topology.out_degree(node(4)), None);
}

#[test]
fn endpoints_and_compact_indices_have_a_stable_public_contract() {
    let topology = Topology::try_from_edges(2, [EdgeEndpoints::new(node(0), node(1))]).unwrap();
    let endpoints = topology.edge_endpoints(EdgeIndex::new(0)).unwrap();

    assert_eq!(endpoints.source(), node(0));
    assert_eq!(endpoints.target(), node(1));
    assert_eq!(node(1).get(), 1);
    assert_eq!(node(1).index(), 1);
    assert!(topology.contains_node(node(1)));
    assert!(topology.contains_edge(EdgeIndex::new(0)));
    assert!(!topology.contains_edge(EdgeIndex::new(1)));
}

#[test]
fn invalid_endpoints_are_rejected_on_build_and_deserialization() {
    let error = Topology::try_from_edges(1, [EdgeEndpoints::new(node(0), node(1))]).unwrap_err();
    assert_eq!(
        error,
        GraphError::InvalidTopologyEndpoint {
            edge: 0,
            node: 1,
            node_count: 1,
        }
    );

    let invalid = r#"{"node_count":1,"endpoints":[{"source":0,"target":2}]}"#;
    assert!(serde_json::from_str::<Topology>(invalid).is_err());
}

#[test]
fn topology_round_trips_without_serializing_derived_csr_storage() {
    let topology = Topology::try_from_edges(
        3,
        [
            EdgeEndpoints::new(node(2), node(0)),
            EdgeEndpoints::new(node(0), node(1)),
        ],
    )
    .unwrap();
    let encoded = serde_json::to_string(&topology).unwrap();
    let decoded: Topology = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, topology);
    assert!(!encoded.contains("outgoing"));
    assert!(!encoded.contains("incoming"));
}

#[test]
fn graph_view_exposes_topology_without_concrete_type_coupling() {
    fn degree_sum(
        graph: &impl GraphView<Node = NodeIndex, Edge = EdgeIndex>,
        node: NodeIndex,
    ) -> usize {
        graph.outgoing_edges(node).count() + graph.incoming_edges(node).count()
    }

    let topology = Topology::try_from_edges(2, [EdgeEndpoints::new(node(0), node(1))]).unwrap();
    assert_eq!(degree_sum(&topology, node(0)), 1);
}
