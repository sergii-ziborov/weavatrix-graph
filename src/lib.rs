#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod error;
mod graph;
mod kind;
mod model;

pub use error::{GraphError, Result};
pub use graph::{Graph, GraphBuilder};
pub use kind::{EdgeKind, EvidenceKind, Language, NodeKind};
pub use model::{Confidence, Edge, Node, NodeId, Provenance, SourcePosition, SourceSpan};
