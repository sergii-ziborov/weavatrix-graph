#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod attribute;
mod error;
mod graph;
mod kind;
mod legacy;
mod model;

pub use attribute::{AttributeValue, FiniteF64};
pub use error::{GraphError, Result};
pub use graph::{Graph, GraphBuilder, GraphNodeIndex};
pub use kind::{EdgeKind, EvidenceKind, NodeKind};
pub use legacy::{LegacyGraph, LegacyLink, LegacyNode, LegacyPoint, LegacyRange};
pub use model::{Confidence, Edge, Node, NodeId, Provenance, SourcePosition, SourceSpan};
