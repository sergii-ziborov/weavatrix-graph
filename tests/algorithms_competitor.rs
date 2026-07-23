use petgraph::algo::{
    dijkstra as pet_dijkstra, dinics, has_path_connecting, is_cyclic_directed, kosaraju_scc,
    toposort,
};
use petgraph::graph::DiGraph;
use weavatrix_graph::{
    EdgeEndpoints, NodeIndex, Topology, dijkstra, has_cycle, maximum_flow, reachable,
    shortest_path, strongly_connected_components, topological_sort,
};

#[test]
fn algorithms_match_petgraph_across_deterministic_random_graphs() {
    let mut seed = 0x9e37_79b9_u64;
    for node_count in [1_usize, 2, 7, 32, 97] {
        for density in [1_u64, 5, 17] {
            let mut pairs = Vec::new();
            for source in 0..node_count {
                for target in 0..node_count {
                    seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                    if seed % 101 < density {
                        pairs.push((source, target));
                    }
                }
            }
            compare_case(node_count, &pairs);
        }
    }
}

fn compare_case(node_count: usize, pairs: &[(usize, usize)]) {
    let ours = Topology::try_from_edges(
        node_count,
        pairs.iter().map(|&(source, target)| {
            EdgeEndpoints::new(
                NodeIndex::new(u32::try_from(source).unwrap()),
                NodeIndex::new(u32::try_from(target).unwrap()),
            )
        }),
    )
    .unwrap();
    let mut pet = DiGraph::<(), u64>::with_capacity(node_count, pairs.len());
    let pet_nodes = (0..node_count)
        .map(|_| pet.add_node(()))
        .collect::<Vec<_>>();
    for &(source, target) in pairs {
        let weight = u64::try_from((source * 31 + target * 17) % 23 + 1).unwrap();
        pet.add_edge(pet_nodes[source], pet_nodes[target], weight);
    }

    assert_eq!(has_cycle(&ours), is_cyclic_directed(&pet));
    assert_eq!(
        topological_sort(&ours).is_some(),
        toposort(&pet, None).is_ok()
    );
    assert_eq!(normalize_ours(&ours), normalize_pet(&pet));
    if node_count > 1 {
        let ours_flow = maximum_flow(
            &ours,
            NodeIndex::new(0),
            NodeIndex::new(u32::try_from(node_count - 1).unwrap()),
            |edge| {
                let (source, target) = pairs[edge.index()];
                u64::try_from((source * 31 + target * 17) % 23 + 1).unwrap()
            },
        )
        .unwrap()
        .unwrap()
        .value();
        let pet_flow = dinics(&pet, pet_nodes[0], pet_nodes[node_count - 1]).0;
        assert_eq!(ours_flow, pet_flow);
    }

    for source in 0..node_count {
        let expected_costs = pet_dijkstra(&pet, pet_nodes[source], None, |edge| *edge.weight());
        for target in 0..node_count {
            let ours_source = NodeIndex::new(u32::try_from(source).unwrap());
            let ours_target = NodeIndex::new(u32::try_from(target).unwrap());
            let expected = has_path_connecting(&pet, pet_nodes[source], pet_nodes[target], None);
            assert_eq!(reachable(&ours, ours_source, ours_target), expected);
            assert_eq!(
                shortest_path(&ours, ours_source, ours_target).is_some(),
                expected
            );
            let ours_cost = dijkstra(&ours, ours_source, ours_target, |edge| {
                let (edge_source, edge_target) = pairs[edge.index()];
                u64::try_from((edge_source * 31 + edge_target * 17) % 23 + 1).unwrap()
            })
            .map(|path| path.total_cost());
            assert_eq!(ours_cost, expected_costs.get(&pet_nodes[target]).copied());
        }
    }
}

fn normalize_ours(graph: &Topology) -> Vec<Vec<usize>> {
    let mut components = strongly_connected_components(graph)
        .into_iter()
        .map(|component| {
            component
                .into_iter()
                .map(NodeIndex::index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    normalize(&mut components);
    components
}

fn normalize_pet(graph: &DiGraph<(), u64>) -> Vec<Vec<usize>> {
    let mut components = kosaraju_scc(graph)
        .into_iter()
        .map(|component| {
            component
                .into_iter()
                .map(petgraph::graph::NodeIndex::index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    normalize(&mut components);
    components
}

fn normalize(components: &mut [Vec<usize>]) {
    for component in &mut *components {
        component.sort_unstable();
    }
    components.sort_unstable();
}
