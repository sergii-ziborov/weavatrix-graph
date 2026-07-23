use crate::{EdgeEndpoints, NodeIndex, Result, Topology};

/// Generates `0 -> 1 -> ... -> n-1`.
///
/// # Errors
///
/// Returns an error if the requested topology exceeds compact capacity.
pub fn path_topology(node_count: usize) -> Result<Topology> {
    Topology::try_from_edges(
        node_count,
        (1..node_count).map(|target| EdgeEndpoints::new(compact(target - 1), compact(target))),
    )
}

/// Generates one directed cycle, including a self-loop for one node.
///
/// # Errors
///
/// Returns an error if the requested topology exceeds compact capacity.
pub fn cycle_topology(node_count: usize) -> Result<Topology> {
    let edges = (node_count > 0)
        .then(|| {
            (0..node_count).map(|source| {
                EdgeEndpoints::new(compact(source), compact((source + 1) % node_count))
            })
        })
        .into_iter()
        .flatten();
    Topology::try_from_edges(node_count, edges)
}

/// Generates all directed edges between distinct nodes.
///
/// # Errors
///
/// Returns an error if the requested topology exceeds compact capacity.
pub fn complete_topology(node_count: usize) -> Result<Topology> {
    let edge_count = node_count.checked_mul(node_count.saturating_sub(1)).ok_or(
        crate::GraphError::IndexCapacityExceeded {
            category: "generated edges",
            count: usize::MAX,
        },
    )?;
    u32::try_from(edge_count).map_err(|_| crate::GraphError::IndexCapacityExceeded {
        category: "generated edges",
        count: edge_count,
    })?;
    Topology::try_from_edges(
        node_count,
        (0..node_count).flat_map(|source| {
            (0..node_count)
                .filter(move |&target| target != source)
                .map(move |target| EdgeEndpoints::new(compact(source), compact(target)))
        }),
    )
}

fn compact(index: usize) -> NodeIndex {
    NodeIndex::new(u32::try_from(index).unwrap_or(u32::MAX))
}
