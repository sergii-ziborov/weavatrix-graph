use crate::{Edge, GraphError, Node, NodeId, Result};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::collections::HashMap;

mod index;
mod validate;

use index::{AdjacencyIndex, OutgoingIndex, canonicalize_edges, index_canonical_edges};
use validate::{validate_edge, validate_node};

/// Compact position for repeated graph traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GraphNodeIndex(usize);

impl GraphNodeIndex {
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    #[serde(skip)]
    outgoing_index: OutgoingIndex,
    #[serde(skip)]
    incoming_index: AdjacencyIndex,
}

impl Graph {
    /// Creates a validated graph and canonicalizes its ordering.
    ///
    /// # Errors
    ///
    /// Returns an error for conflicting nodes, dangling edges, empty extractor
    /// identities, or invalid source spans.
    pub fn try_from_parts(
        nodes: impl IntoIterator<Item = Node>,
        edges: impl IntoIterator<Item = Edge>,
    ) -> Result<Self> {
        let nodes = nodes.into_iter();
        let edges = edges.into_iter();
        let mut builder = GraphBuilder::with_capacity(nodes.size_hint().0, edges.size_hint().0);
        for node in nodes {
            builder.add_node(node)?;
        }
        for edge in edges {
            builder.add_edge(edge)?;
        }
        builder.build()
    }

    /// Builds faster when nodes and edges are already in canonical order.
    ///
    /// Unordered input safely falls back to [`Self::try_from_parts`].
    ///
    /// # Errors
    ///
    /// Returns the same validation errors as [`Self::try_from_parts`].
    pub fn try_from_sorted_parts(nodes: Vec<Node>, mut edges: Vec<Edge>) -> Result<Self> {
        let nodes_are_sorted = nodes.windows(2).all(|pair| pair[0].id < pair[1].id);
        if !nodes_are_sorted || !edges.is_sorted() {
            return Self::try_from_parts(nodes, edges);
        }
        for node in &nodes {
            validate_node(node)?;
        }
        for edge in &edges {
            validate_edge(edge)?;
        }
        edges.dedup();
        let (outgoing_index, incoming_index) = index_canonical_edges(&nodes, &edges)?;
        Ok(Self {
            nodes,
            edges,
            outgoing_index,
            incoming_index,
        })
    }

    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    #[must_use]
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.node_index(id).and_then(|index| self.node_at(index))
    }

    pub fn outgoing<'graph>(&'graph self, id: &NodeId) -> impl Iterator<Item = &'graph Edge> {
        self.node_index(id.as_str())
            .and_then(|index| self.outgoing_index.get(index.0).cloned())
            .into_iter()
            .flatten()
            .map(|index| &self.edges[index])
    }

    pub fn incoming<'graph>(&'graph self, id: &NodeId) -> impl Iterator<Item = &'graph Edge> {
        self.node_index(id.as_str())
            .and_then(|index| self.incoming_index.get(index.0))
            .into_iter()
            .flatten()
            .map(|&index| &self.edges[index])
    }

    /// Resolves a stable id once for repeated indexed traversal.
    #[must_use]
    pub fn node_index(&self, id: &str) -> Option<GraphNodeIndex> {
        self.nodes
            .binary_search_by(|node| node.id.as_str().cmp(id))
            .ok()
            .map(GraphNodeIndex)
    }

    #[must_use]
    pub fn node_at(&self, index: GraphNodeIndex) -> Option<&Node> {
        self.nodes.get(index.0)
    }

    pub fn outgoing_at(&self, index: GraphNodeIndex) -> impl Iterator<Item = &Edge> {
        self.outgoing_index
            .get(index.0)
            .cloned()
            .into_iter()
            .flatten()
            .map(|edge| &self.edges[edge])
    }

    pub fn incoming_at(&self, index: GraphNodeIndex) -> impl Iterator<Item = &Edge> {
        self.incoming_index
            .get(index.0)
            .into_iter()
            .flatten()
            .map(|&edge| &self.edges[edge])
    }

    #[must_use]
    pub fn out_degree(&self, index: GraphNodeIndex) -> Option<usize> {
        self.outgoing_index.get(index.0).map(std::ops::Range::len)
    }

    #[must_use]
    pub fn in_degree(&self, index: GraphNodeIndex) -> Option<usize> {
        self.incoming_index.len(index.0)
    }

    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub const fn edge_count(&self) -> usize {
        self.edges.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    #[must_use]
    pub fn into_parts(self) -> (Vec<Node>, Vec<Edge>) {
        (self.nodes, self.edges)
    }
}

#[derive(Deserialize)]
struct GraphWire {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl<'de> Deserialize<'de> for Graph {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = GraphWire::deserialize(deserializer)?;
        Self::try_from_parts(wire.nodes, wire.edges).map_err(D::Error::custom)
    }
}

#[derive(Debug, Default)]
pub struct GraphBuilder {
    nodes: HashMap<NodeId, Node>,
    edges: Vec<Edge>,
}

impl GraphBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Creates a builder with storage sized for the expected graph.
    #[must_use]
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(nodes),
            edges: Vec::with_capacity(edges),
        }
    }

    /// Adds a node idempotently.
    ///
    /// # Errors
    ///
    /// Returns an error when the same identifier already has a different
    /// definition or the node contains an invalid source span.
    pub fn add_node(&mut self, node: Node) -> Result<&mut Self> {
        validate_node(&node)?;
        if let Some(existing) = self.nodes.get(&node.id) {
            if existing == &node {
                return Ok(self);
            }
            return Err(GraphError::ConflictingNode {
                id: node.id.to_string(),
            });
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(self)
    }

    /// Adds an edge idempotently. Endpoint existence is validated by `build`,
    /// so callers may insert edges before nodes.
    ///
    /// # Errors
    ///
    /// Returns an error when provenance or its source span is invalid.
    pub fn add_edge(&mut self, edge: Edge) -> Result<&mut Self> {
        validate_edge(&edge)?;
        self.edges.push(edge);
        Ok(self)
    }

    /// Validates all endpoints and returns an immutable graph.
    ///
    /// # Errors
    ///
    /// Returns an error when an edge references a missing source or target.
    pub fn build(self) -> Result<Graph> {
        let mut nodes = self.nodes.into_values().collect::<Vec<_>>();
        nodes.sort_unstable_by(|left, right| left.id.cmp(&right.id));
        let (edges, outgoing_index, incoming_index) = canonicalize_edges(&nodes, self.edges)?;
        Ok(Graph {
            nodes,
            edges,
            outgoing_index,
            incoming_index,
        })
    }
}
