mod deterministic;
mod random;

pub use deterministic::{complete_topology, cycle_topology, path_topology};
pub use random::RandomGraphGenerator;
