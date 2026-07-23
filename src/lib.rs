#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod algo;
mod attribute;
mod error;
mod filter;
mod generator;
mod graph;
mod kind;
mod legacy;
mod matrix;
mod model;
mod topology;
mod undirected;
mod working;

pub use algo::{
    Condensation, Direction, MaxFlow, SpanningForest, WeightedPath, bfs, bfs_filtered,
    condensation, condensation_filtered, dfs, dfs_filtered, dijkstra, dijkstra_filtered,
    find_cycle, find_cycle_filtered, has_cycle, has_cycle_filtered, maximum_flow,
    minimum_spanning_forest, reachable, reachable_filtered, shortest_path, shortest_path_filtered,
    strongly_connected_components, strongly_connected_components_filtered, topological_sort,
    topological_sort_filtered, weakly_connected_components, weakly_connected_components_filtered,
};
pub use attribute::{AttributeValue, FiniteF64};
pub use error::{GraphError, Result};
pub use filter::EdgeFilter;
pub use generator::{RandomGraphGenerator, complete_topology, cycle_topology, path_topology};
pub use graph::{Graph, GraphBuilder, GraphNodeIndex};
pub use kind::{EdgeKind, EvidenceKind, NodeKind};
pub use legacy::{LegacyGraph, LegacyLink, LegacyNode, LegacyPoint, LegacyRange};
pub use matrix::DenseMatrix;
pub use model::{Confidence, Edge, Node, NodeId, Provenance, SourcePosition, SourceSpan};
pub use topology::{EdgeEndpoints, EdgeIndex, GraphView, IndexGraphView, NodeIndex, Topology};
pub use undirected::{IndexUndirectedGraphView, UndirectedGraphView, UndirectedTopology};
pub use working::{FreezeMap, FrozenGraph, StableEdgeKey, StableNodeKey, WorkingGraph};
