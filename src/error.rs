use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GraphError {
    EmptyNodeId,
    InvalidKind {
        category: &'static str,
        value: String,
    },
    ConflictingNode {
        id: String,
    },
    MissingEdgeSource {
        id: String,
    },
    MissingEdgeTarget {
        id: String,
    },
    EmptyExtractor,
    InvalidSpan {
        file: String,
        reason: &'static str,
    },
    IndexCapacityExceeded {
        category: &'static str,
        count: usize,
    },
    InvalidTopologyEndpoint {
        edge: usize,
        node: usize,
        node_count: usize,
    },
    ArithmeticOverflow {
        operation: &'static str,
    },
    InvalidNodeIndex {
        node: usize,
        node_count: usize,
    },
    InvalidProbability {
        numerator: u64,
        denominator: u64,
    },
    InvalidAlgorithmParameter {
        algorithm: &'static str,
        parameter: &'static str,
        value: String,
    },
    NegativeCycle {
        algorithm: &'static str,
    },
}

impl Display for GraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyNodeId => formatter.write_str("node id must not be empty"),
            Self::InvalidKind { category, value } => {
                write!(formatter, "invalid {category} kind: {value:?}")
            }
            Self::ConflictingNode { id } => {
                write!(formatter, "node id {id} has conflicting definitions")
            }
            Self::MissingEdgeSource { id } => {
                write!(formatter, "edge source does not exist: {id}")
            }
            Self::MissingEdgeTarget { id } => {
                write!(formatter, "edge target does not exist: {id}")
            }
            Self::EmptyExtractor => formatter.write_str("provenance extractor must not be empty"),
            Self::InvalidSpan { file, reason } => {
                write!(formatter, "invalid source span for {file:?}: {reason}")
            }
            Self::IndexCapacityExceeded { category, count } => {
                write!(
                    formatter,
                    "{category} count {count} exceeds u32 index capacity"
                )
            }
            Self::InvalidTopologyEndpoint {
                edge,
                node,
                node_count,
            } => write!(
                formatter,
                "edge {edge} references node index {node}, but node count is {node_count}"
            ),
            Self::ArithmeticOverflow { operation } => {
                write!(formatter, "arithmetic overflow while computing {operation}")
            }
            Self::InvalidNodeIndex { node, node_count } => {
                write!(
                    formatter,
                    "node index {node} is outside matrix node count {node_count}"
                )
            }
            Self::InvalidProbability {
                numerator,
                denominator,
            } => write!(
                formatter,
                "probability {numerator}/{denominator} must have a nonzero denominator and be at most one"
            ),
            Self::InvalidAlgorithmParameter {
                algorithm,
                parameter,
                value,
            } => write!(formatter, "invalid {parameter} for {algorithm}: {value}"),
            Self::NegativeCycle { algorithm } => {
                write!(formatter, "{algorithm} found a reachable negative cycle")
            }
        }
    }
}

impl std::error::Error for GraphError {}

pub type Result<T> = std::result::Result<T, GraphError>;
