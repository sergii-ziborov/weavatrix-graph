use super::traversal::{Direction, for_each_neighbor};
use crate::IndexGraphView;
use std::collections::VecDeque;

#[must_use]
pub fn strongly_connected_components<G>(graph: &G) -> Vec<Vec<G::Node>>
where
    G: IndexGraphView,
{
    let mut seen = vec![false; graph.node_bound()];
    let mut finish = Vec::with_capacity(graph.node_count());
    for start in graph.node_indices() {
        finish_from(graph, start, &mut seen, &mut finish);
    }

    seen.fill(false);
    let mut components = Vec::new();
    for start in finish.into_iter().rev() {
        if seen[G::node_slot(start)] {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![start];
        seen[G::node_slot(start)] = true;
        while let Some(node) = stack.pop() {
            component.push(node);
            for_each_neighbor(
                graph,
                node,
                Direction::Incoming,
                &mut |_| true,
                |neighbor| {
                    let slot = G::node_slot(neighbor);
                    if !seen[slot] {
                        seen[slot] = true;
                        stack.push(neighbor);
                    }
                },
            );
        }
        components.push(component);
    }
    components
}

#[must_use]
pub fn topological_sort<G>(graph: &G) -> Option<Vec<G::Node>>
where
    G: IndexGraphView,
{
    let mut indegree = vec![0_usize; graph.node_bound()];
    for edge in graph.edge_indices() {
        if let Some(endpoints) = graph.edge_endpoints(edge) {
            indegree[G::node_slot(endpoints.target())] += 1;
        }
    }
    let mut ready = graph
        .node_indices()
        .filter(|node| indegree[G::node_slot(*node)] == 0)
        .collect::<VecDeque<_>>();
    let mut order = Vec::with_capacity(graph.node_count());
    while let Some(node) = ready.pop_front() {
        order.push(node);
        for edge in graph.outgoing_edges(node) {
            let Some(endpoints) = graph.edge_endpoints(edge) else {
                continue;
            };
            let slot = G::node_slot(endpoints.target());
            indegree[slot] -= 1;
            if indegree[slot] == 0 {
                ready.push_back(endpoints.target());
            }
        }
    }
    (order.len() == graph.node_count()).then_some(order)
}

#[must_use]
pub fn has_cycle<G>(graph: &G) -> bool
where
    G: IndexGraphView,
{
    topological_sort(graph).is_none()
}

#[must_use]
pub fn find_cycle<G>(graph: &G) -> Option<Vec<G::Node>>
where
    G: IndexGraphView,
{
    let mut adjacency = vec![Vec::new(); graph.node_bound()];
    for edge in graph.edge_indices() {
        if let Some(endpoints) = graph.edge_endpoints(edge) {
            adjacency[G::node_slot(endpoints.source())].push(endpoints.target());
        }
    }
    let mut color = vec![0_u8; graph.node_bound()];
    let mut parent = vec![None; graph.node_bound()];
    for start in graph.node_indices() {
        if color[G::node_slot(start)] != 0 {
            continue;
        }
        color[G::node_slot(start)] = 1;
        let mut stack = vec![(start, 0_usize)];
        while let Some((node, next)) = stack.last_mut() {
            let slot = G::node_slot(*node);
            let Some(&target) = adjacency[slot].get(*next) else {
                color[slot] = 2;
                stack.pop();
                continue;
            };
            *next += 1;
            let target_slot = G::node_slot(target);
            if color[target_slot] == 0 {
                parent[target_slot] = Some(*node);
                color[target_slot] = 1;
                stack.push((target, 0));
            } else if color[target_slot] == 1 {
                return reconstruct_cycle::<G>(*node, target, &parent);
            }
        }
    }
    None
}

fn reconstruct_cycle<G>(
    node: G::Node,
    target: G::Node,
    parent: &[Option<G::Node>],
) -> Option<Vec<G::Node>>
where
    G: IndexGraphView,
{
    let mut cycle = vec![node];
    let mut cursor = node;
    while cursor != target {
        cursor = parent[G::node_slot(cursor)]?;
        cycle.push(cursor);
    }
    cycle.reverse();
    cycle.push(target);
    Some(cycle)
}

fn finish_from<G>(graph: &G, start: G::Node, seen: &mut [bool], finish: &mut Vec<G::Node>)
where
    G: IndexGraphView,
{
    if seen[G::node_slot(start)] {
        return;
    }
    let mut stack = vec![(start, false)];
    while let Some((node, exiting)) = stack.pop() {
        let slot = G::node_slot(node);
        if exiting {
            finish.push(node);
            continue;
        }
        if seen[slot] {
            continue;
        }
        seen[slot] = true;
        stack.push((node, true));
        for edge in graph.outgoing_edges(node).rev() {
            let Some(target) = graph.edge_endpoints(edge).map(crate::EdgeEndpoints::target) else {
                continue;
            };
            if !seen[G::node_slot(target)] {
                stack.push((target, false));
            }
        }
    }
}
