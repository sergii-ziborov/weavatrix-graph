use crate::{Edge, GraphError, Node, NodeId, Result};
use std::collections::HashMap;
use std::ops::Range;

pub(super) type OutgoingIndex = Vec<Range<usize>>;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AdjacencyIndex {
    offsets: Vec<usize>,
    edges: Vec<usize>,
}

impl AdjacencyIndex {
    pub(super) fn get(&self, node: usize) -> Option<&[usize]> {
        let start = *self.offsets.get(node)?;
        let end = *self.offsets.get(node + 1)?;
        self.edges.get(start..end)
    }

    pub(super) fn len(&self, node: usize) -> Option<usize> {
        let start = *self.offsets.get(node)?;
        let end = *self.offsets.get(node + 1)?;
        Some(end - start)
    }
}

pub(super) fn canonicalize_edges(
    nodes: &[Node],
    edges: Vec<Edge>,
) -> Result<(Vec<Edge>, OutgoingIndex, AdjacencyIndex)> {
    let positions = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (&node.id, index))
        .collect::<HashMap<&NodeId, usize>>();
    let mut mapped = Vec::with_capacity(edges.len());
    let mut counts = vec![0_usize; nodes.len()];
    for edge in edges {
        let source = position(&positions, &edge.source, true)?;
        let target = position(&positions, &edge.target, false)?;
        counts[source] += 1;
        mapped.push((source, edge, target));
    }
    let mut buckets = counts
        .into_iter()
        .map(Vec::with_capacity)
        .collect::<Vec<Vec<(Edge, usize)>>>();
    for (source, edge, target) in mapped {
        buckets[source].push((edge, target));
    }

    let edge_count = buckets.iter().map(Vec::len).sum();
    let mut canonical = Vec::with_capacity(edge_count);
    let mut targets = Vec::with_capacity(edge_count);
    let mut outgoing_offsets = Vec::with_capacity(nodes.len() + 1);
    outgoing_offsets.push(0);
    for mut bucket in buckets {
        bucket.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        bucket.dedup_by(|left, right| left.0 == right.0);
        for (edge, target) in bucket {
            canonical.push(edge);
            targets.push(target);
        }
        outgoing_offsets.push(canonical.len());
    }
    let outgoing = outgoing_offsets
        .windows(2)
        .map(|window| window[0]..window[1])
        .collect();
    let incoming = incoming_index(nodes.len(), &targets);
    Ok((canonical, outgoing, incoming))
}

pub(super) fn index_canonical_edges(
    nodes: &[Node],
    edges: &[Edge],
) -> Result<(OutgoingIndex, AdjacencyIndex)> {
    let positions = node_positions(nodes);
    let mut outgoing_counts = vec![0_usize; nodes.len()];
    let mut targets = Vec::with_capacity(edges.len());
    for edge in edges {
        let source = position(&positions, &edge.source, true)?;
        let target = position(&positions, &edge.target, false)?;
        outgoing_counts[source] += 1;
        targets.push(target);
    }
    let offsets = prefix_offsets(&outgoing_counts);
    let outgoing = offsets
        .windows(2)
        .map(|window| window[0]..window[1])
        .collect();
    Ok((outgoing, incoming_index(nodes.len(), &targets)))
}

fn node_positions(nodes: &[Node]) -> HashMap<&NodeId, usize> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (&node.id, index))
        .collect()
}

fn prefix_offsets(counts: &[usize]) -> Vec<usize> {
    let mut offsets = Vec::with_capacity(counts.len() + 1);
    offsets.push(0);
    for &count in counts {
        offsets.push(offsets.last().copied().unwrap_or(0) + count);
    }
    offsets
}

fn incoming_index(node_count: usize, targets: &[usize]) -> AdjacencyIndex {
    let mut offsets = vec![0_usize; node_count + 1];
    for &target in targets {
        offsets[target + 1] += 1;
    }
    for node in 1..offsets.len() {
        offsets[node] += offsets[node - 1];
    }
    let mut cursors = offsets[..node_count].to_vec();
    let mut edges = vec![0_usize; targets.len()];
    for (edge, &target) in targets.iter().enumerate() {
        edges[cursors[target]] = edge;
        cursors[target] += 1;
    }
    AdjacencyIndex { offsets, edges }
}

fn position(positions: &HashMap<&NodeId, usize>, id: &NodeId, source: bool) -> Result<usize> {
    positions.get(id).copied().ok_or_else(|| {
        if source {
            GraphError::MissingEdgeSource { id: id.to_string() }
        } else {
            GraphError::MissingEdgeTarget { id: id.to_string() }
        }
    })
}
