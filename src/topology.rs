mod core;
pub(crate) mod csr;
mod index;
mod view;

pub use core::Topology;
pub use index::{EdgeIndex, NodeIndex};
pub use view::{EdgeEndpoints, GraphView, IndexGraphView};
