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
        }
    }
}

impl std::error::Error for GraphError {}

pub type Result<T> = std::result::Result<T, GraphError>;
