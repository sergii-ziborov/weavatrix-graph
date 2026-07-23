use crate::{Edge, GraphError, Node, NodeId, Result, SourceSpan};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    #[serde(skip)]
    outgoing_index: BTreeMap<NodeId, Vec<usize>>,
    #[serde(skip)]
    incoming_index: BTreeMap<NodeId, Vec<usize>>,
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
        let mut builder = GraphBuilder::new();
        for node in nodes {
            builder.add_node(node)?;
        }
        for edge in edges {
            builder.add_edge(edge)?;
        }
        builder.build()
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
        self.nodes
            .binary_search_by(|node| node.id.as_str().cmp(id))
            .ok()
            .map(|index| &self.nodes[index])
    }

    pub fn outgoing<'graph>(
        &'graph self,
        id: &'graph NodeId,
    ) -> impl Iterator<Item = &'graph Edge> {
        self.outgoing_index
            .get(id)
            .into_iter()
            .flatten()
            .map(|index| &self.edges[*index])
    }

    pub fn incoming<'graph>(
        &'graph self,
        id: &'graph NodeId,
    ) -> impl Iterator<Item = &'graph Edge> {
        self.incoming_index
            .get(id)
            .into_iter()
            .flatten()
            .map(|index| &self.edges[*index])
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
    nodes: BTreeMap<NodeId, Node>,
    edges: BTreeSet<Edge>,
}

impl GraphBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: BTreeSet::new(),
        }
    }

    /// Adds a node idempotently.
    ///
    /// # Errors
    ///
    /// Returns an error when the same identifier already has a different
    /// definition or the node contains an invalid source span.
    pub fn add_node(&mut self, node: Node) -> Result<&mut Self> {
        if let Some(span) = &node.span {
            validate_span(span)?;
        }
        if let Some(language) = &node.language {
            validate_language(language)?;
        }
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
        if edge.provenance.extractor.is_empty() {
            return Err(GraphError::EmptyExtractor);
        }
        if let Some(span) = &edge.provenance.span {
            validate_span(span)?;
        }
        self.edges.insert(edge);
        Ok(self)
    }

    /// Validates all endpoints and returns an immutable graph.
    ///
    /// # Errors
    ///
    /// Returns an error when an edge references a missing source or target.
    pub fn build(self) -> Result<Graph> {
        for edge in &self.edges {
            if !self.nodes.contains_key(&edge.source) {
                return Err(GraphError::MissingEdgeSource {
                    id: edge.source.to_string(),
                });
            }
            if !self.nodes.contains_key(&edge.target) {
                return Err(GraphError::MissingEdgeTarget {
                    id: edge.target.to_string(),
                });
            }
        }
        let edges = self.edges.into_iter().collect::<Vec<_>>();
        let (outgoing_index, incoming_index) = build_edge_indexes(&edges);
        Ok(Graph {
            nodes: self.nodes.into_values().collect(),
            edges,
            outgoing_index,
            incoming_index,
        })
    }
}

fn build_edge_indexes(
    edges: &[Edge],
) -> (BTreeMap<NodeId, Vec<usize>>, BTreeMap<NodeId, Vec<usize>>) {
    let mut outgoing = BTreeMap::<NodeId, Vec<usize>>::new();
    let mut incoming = BTreeMap::<NodeId, Vec<usize>>::new();
    for (index, edge) in edges.iter().enumerate() {
        outgoing.entry(edge.source.clone()).or_default().push(index);
        incoming.entry(edge.target.clone()).or_default().push(index);
    }
    (outgoing, incoming)
}

fn validate_language(language: &str) -> Result<()> {
    if language.is_empty() || language.trim() != language {
        return Err(GraphError::InvalidKind {
            category: "language",
            value: language.to_owned(),
        });
    }
    Ok(())
}

fn validate_span(span: &SourceSpan) -> Result<()> {
    if span.file.is_empty() {
        return Err(GraphError::InvalidSpan {
            file: span.file.clone(),
            reason: "file must not be empty",
        });
    }
    if span.start.line == 0 || span.start.column == 0 || span.end.line == 0 || span.end.column == 0
    {
        return Err(GraphError::InvalidSpan {
            file: span.file.clone(),
            reason: "positions are one-based",
        });
    }
    if span.end < span.start {
        return Err(GraphError::InvalidSpan {
            file: span.file.clone(),
            reason: "end precedes start",
        });
    }
    Ok(())
}
