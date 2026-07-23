use petgraph::{Direction, Graph as PetGraph};
use weavatrix_graph::{EdgeEndpoints, EdgeIndex, NodeIndex, Topology};

const NODE_COUNT: usize = 128;
const EDGE_COUNT: usize = 1_024;

#[test]
fn dual_csr_matches_petgraph_on_deterministic_random_topology() {
    let endpoints = random_endpoints();
    let topology = Topology::try_from_edges(
        NODE_COUNT,
        endpoints.iter().map(|&(source, target)| {
            EdgeEndpoints::new(
                NodeIndex::new(source.try_into().unwrap()),
                NodeIndex::new(target.try_into().unwrap()),
            )
        }),
    )
    .unwrap();
    let petgraph = build_petgraph(&endpoints);

    for index in 0..NODE_COUNT {
        let node = NodeIndex::new(index.try_into().unwrap());
        let mut ours_out = topology
            .outgoing_neighbors(node)
            .map(NodeIndex::index)
            .collect::<Vec<_>>();
        let mut theirs_out = petgraph
            .neighbors_directed(petgraph::graph::NodeIndex::new(index), Direction::Outgoing)
            .map(petgraph::graph::NodeIndex::index)
            .collect::<Vec<_>>();
        ours_out.sort_unstable();
        theirs_out.sort_unstable();
        assert_eq!(ours_out, theirs_out);

        let mut ours_in = topology
            .incoming_neighbors(node)
            .map(NodeIndex::index)
            .collect::<Vec<_>>();
        let mut theirs_in = petgraph
            .neighbors_directed(petgraph::graph::NodeIndex::new(index), Direction::Incoming)
            .map(petgraph::graph::NodeIndex::index)
            .collect::<Vec<_>>();
        ours_in.sort_unstable();
        theirs_in.sort_unstable();
        assert_eq!(ours_in, theirs_in);
    }

    for (edge, expected) in endpoints.iter().copied().enumerate() {
        let endpoints = topology
            .edge_endpoints(EdgeIndex::new(edge.try_into().unwrap()))
            .unwrap();
        assert_eq!(
            (endpoints.source().index(), endpoints.target().index()),
            expected
        );
    }
}

fn build_petgraph(endpoints: &[(usize, usize)]) -> PetGraph<(), ()> {
    let mut graph = PetGraph::with_capacity(NODE_COUNT, EDGE_COUNT);
    let nodes = (0..NODE_COUNT)
        .map(|_| graph.add_node(()))
        .collect::<Vec<_>>();
    for &(source, target) in endpoints {
        graph.add_edge(nodes[source], nodes[target], ());
    }
    graph
}

fn random_endpoints() -> Vec<(usize, usize)> {
    let mut state = 0x4d59_5df4_d0f3_3173_u64;
    (0..EDGE_COUNT)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let source = usize::try_from(state % NODE_COUNT as u64).unwrap();
            state = state.rotate_left(23);
            let target = usize::try_from(state % NODE_COUNT as u64).unwrap();
            (source, target)
        })
        .collect()
}
