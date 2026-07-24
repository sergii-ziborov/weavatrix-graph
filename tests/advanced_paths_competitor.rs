use petgraph::algo::{astar as pet_astar, bellman_ford as pet_bellman_ford};
use petgraph::graph::DiGraph;
use weavatrix_graph::{EdgeEndpoints, NodeIndex, Topology, astar, bellman_ford, dijkstra};

#[test]
fn advanced_shortest_paths_match_petgraph_on_seeded_dags() {
    let mut seed = 0xd1b5_4a32_d192_ed03_u64;
    for node_count in [1_usize, 2, 8, 31, 89] {
        for density in [5_u64, 17, 43] {
            let mut edges = Vec::new();
            for source in 0..node_count {
                for target in (source + 1)..node_count {
                    seed = next(seed);
                    if seed % 101 < density {
                        let signed = i64::try_from(seed % 31).unwrap() - 10;
                        edges.push((source, target, signed));
                    }
                }
            }
            compare_case(node_count, &edges);
        }
    }
}

#[test]
fn bellman_ford_matches_petgraph_with_cycles_and_negative_weights() {
    let mut seed = 0x243f_6a88_85a3_08d3_u64;
    for node_count in [2_usize, 5, 12, 29] {
        for _ in 0..12 {
            let mut edges = Vec::new();
            for source in 0..node_count {
                for target in 0..node_count {
                    seed = next(seed);
                    if source != target && seed % 101 < 19 {
                        let weight = i64::try_from((seed >> 16) % 17).unwrap() - 6;
                        edges.push((source, target, weight));
                    }
                }
            }
            compare_bellman_cycle_case(node_count, &edges);
        }
    }
}

fn compare_case(node_count: usize, edges: &[(usize, usize, i64)]) {
    let ours = Topology::try_from_edges(
        node_count,
        edges
            .iter()
            .map(|&(source, target, _)| EdgeEndpoints::new(node(source), node(target))),
    )
    .unwrap();
    let mut signed_pet = DiGraph::<(), f64>::with_capacity(node_count, edges.len());
    let signed_nodes = (0..node_count)
        .map(|_| signed_pet.add_node(()))
        .collect::<Vec<_>>();
    let mut positive_pet = DiGraph::<(), u64>::with_capacity(node_count, edges.len());
    let positive_nodes = (0..node_count)
        .map(|_| positive_pet.add_node(()))
        .collect::<Vec<_>>();
    for &(source, target, signed) in edges {
        signed_pet.add_edge(
            signed_nodes[source],
            signed_nodes[target],
            f64::from(i32::try_from(signed).unwrap()),
        );
        positive_pet.add_edge(
            positive_nodes[source],
            positive_nodes[target],
            signed.unsigned_abs() + 1,
        );
    }

    for source in 0..node_count {
        compare_bellman(&ours, edges, &signed_pet, &signed_nodes, source, node_count);
        for target in 0..node_count {
            compare_astar(&ours, edges, &positive_pet, &positive_nodes, source, target);
        }
    }
}

fn compare_bellman(
    ours: &Topology,
    edges: &[(usize, usize, i64)],
    pet: &DiGraph<(), f64>,
    pet_nodes: &[petgraph::graph::NodeIndex],
    source: usize,
    node_count: usize,
) {
    let ours = bellman_ford(ours, node(source), |edge| edges[edge.index()].2)
        .unwrap()
        .unwrap();
    let expected = pet_bellman_ford(pet, pet_nodes[source]).unwrap();
    for (target, expected_distance) in expected.distances.iter().take(node_count).enumerate() {
        let actual = ours.distance_to(node(target));
        if expected_distance.is_infinite() {
            assert_eq!(actual, None);
        } else {
            assert_eq!(actual.map(as_f64), Some(*expected_distance));
        }
    }
}

fn compare_bellman_cycle_case(node_count: usize, edges: &[(usize, usize, i64)]) {
    let ours = Topology::try_from_edges(
        node_count,
        edges
            .iter()
            .map(|&(source, target, _)| EdgeEndpoints::new(node(source), node(target))),
    )
    .unwrap();
    let mut pet = DiGraph::<(), f64>::with_capacity(node_count, edges.len());
    let nodes = (0..node_count)
        .map(|_| pet.add_node(()))
        .collect::<Vec<_>>();
    for &(source, target, weight) in edges {
        pet.add_edge(nodes[source], nodes[target], as_f64(weight));
    }
    let actual = bellman_ford(&ours, node(0), |edge| edges[edge.index()].2);
    let expected = pet_bellman_ford(&pet, nodes[0]);
    assert_eq!(actual.is_err(), expected.is_err());
    if let (Ok(Some(actual)), Ok(expected)) = (actual, expected) {
        for (target, expected) in expected.distances.iter().enumerate() {
            if expected.is_infinite() {
                assert_eq!(actual.distance_to(node(target)), None);
            } else {
                assert_eq!(
                    actual.distance_to(node(target)).map(as_f64),
                    Some(*expected)
                );
            }
        }
    }
}

fn compare_astar(
    ours: &Topology,
    edges: &[(usize, usize, i64)],
    pet: &DiGraph<(), u64>,
    pet_nodes: &[petgraph::graph::NodeIndex],
    source: usize,
    target: usize,
) {
    let edge_cost = |index: usize| edges[index].2.unsigned_abs() + 1;
    let actual = astar(
        ours,
        node(source),
        node(target),
        |edge| edge_cost(edge.index()),
        |_| 0,
    );
    let expected = pet_astar(
        pet,
        pet_nodes[source],
        |node| node == pet_nodes[target],
        |edge| *edge.weight(),
        |_| 0,
    );
    assert_eq!(
        actual
            .as_ref()
            .map(weavatrix_graph::WeightedPath::total_cost),
        expected.as_ref().map(|(cost, _)| *cost)
    );
    assert_eq!(
        actual
            .as_ref()
            .map(weavatrix_graph::WeightedPath::total_cost),
        dijkstra(ours, node(source), node(target), |edge| edge_cost(
            edge.index()
        ))
        .map(|path| path.total_cost())
    );
}

fn node(index: usize) -> NodeIndex {
    NodeIndex::new(u32::try_from(index).unwrap())
}

fn next(seed: u64) -> u64 {
    seed.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

#[allow(clippy::cast_precision_loss)]
fn as_f64(value: i64) -> f64 {
    value as f64
}
