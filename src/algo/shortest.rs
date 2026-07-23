use super::traversal::{Direction, for_each_adjacent};
use crate::IndexGraphView;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedPath<Node> {
    nodes: Vec<Node>,
    total_cost: u64,
}

impl<Node> WeightedPath<Node> {
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    #[must_use]
    pub const fn total_cost(&self) -> u64 {
        self.total_cost
    }

    #[must_use]
    pub fn into_nodes(self) -> Vec<Node> {
        self.nodes
    }
}

pub fn dijkstra<G, F>(
    graph: &G,
    source: G::Node,
    target: G::Node,
    mut edge_cost: F,
) -> Option<WeightedPath<G::Node>>
where
    G: IndexGraphView,
    F: FnMut(G::Edge) -> u64,
{
    dijkstra_filtered(graph, source, target, Direction::Outgoing, |edge| {
        Some(edge_cost(edge))
    })
}

pub fn dijkstra_filtered<G, F>(
    graph: &G,
    source: G::Node,
    target: G::Node,
    direction: Direction,
    mut edge_cost: F,
) -> Option<WeightedPath<G::Node>>
where
    G: IndexGraphView,
    F: FnMut(G::Edge) -> Option<u64>,
{
    if !graph.contains_node(source) || !graph.contains_node(target) {
        return None;
    }
    let bound = graph.node_bound();
    let mut nodes = vec![None; bound];
    for node in graph.node_indices() {
        nodes[G::node_slot(node)] = Some(node);
    }
    let mut costs = vec![u64::MAX; bound];
    let mut predecessor = vec![None; bound];
    let source_slot = G::node_slot(source);
    costs[source_slot] = 0;
    let mut queue = BinaryHeap::new();
    queue.push(Reverse((0_u64, source_slot)));

    while let Some(Reverse((cost, slot))) = queue.pop() {
        if cost != costs[slot] {
            continue;
        }
        let Some(node) = nodes[slot] else {
            continue;
        };
        if node == target {
            return Some(WeightedPath {
                nodes: reconstruct::<G>(source, target, &predecessor)?,
                total_cost: cost,
            });
        }
        relax_neighbors(
            graph,
            node,
            direction,
            cost,
            &mut edge_cost,
            &mut costs,
            &mut predecessor,
            &mut queue,
        );
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn relax_neighbors<G, F>(
    graph: &G,
    node: G::Node,
    direction: Direction,
    cost: u64,
    edge_cost: &mut F,
    costs: &mut [u64],
    predecessor: &mut [Option<G::Node>],
    queue: &mut BinaryHeap<Reverse<(u64, usize)>>,
) where
    G: IndexGraphView,
    F: FnMut(G::Edge) -> Option<u64>,
{
    for_each_adjacent(graph, node, direction, &mut |_| true, |edge, neighbor| {
        let Some(weight) = edge_cost(edge) else {
            return;
        };
        let Some(candidate) = cost.checked_add(weight) else {
            return;
        };
        let slot = G::node_slot(neighbor);
        if candidate < costs[slot] {
            costs[slot] = candidate;
            predecessor[slot] = Some(node);
            queue.push(Reverse((candidate, slot)));
        }
    });
}

fn reconstruct<G: IndexGraphView>(
    source: G::Node,
    target: G::Node,
    predecessor: &[Option<G::Node>],
) -> Option<Vec<G::Node>> {
    let mut path = vec![target];
    let mut cursor = target;
    while cursor != source {
        cursor = predecessor[G::node_slot(cursor)]?;
        path.push(cursor);
    }
    path.reverse();
    Some(path)
}
