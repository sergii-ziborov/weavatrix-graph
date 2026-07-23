use crate::IndexUndirectedGraphView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanningForest<Edge> {
    edges: Vec<Edge>,
    total_weight: u128,
    component_count: usize,
}

impl<Edge> SpanningForest<Edge> {
    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    #[must_use]
    pub const fn total_weight(&self) -> u128 {
        self.total_weight
    }

    #[must_use]
    pub const fn component_count(&self) -> usize {
        self.component_count
    }

    #[must_use]
    pub fn into_edges(self) -> Vec<Edge> {
        self.edges
    }
}

pub fn minimum_spanning_forest<G, F>(graph: &G, mut edge_weight: F) -> SpanningForest<G::Edge>
where
    G: IndexUndirectedGraphView,
    F: FnMut(G::Edge) -> u64,
{
    let mut weighted = graph
        .edge_indices()
        .map(|edge| (edge_weight(edge), G::edge_slot(edge), edge))
        .collect::<Vec<_>>();
    weighted.sort_unstable_by_key(|&(weight, slot, _)| (weight, slot));

    let mut sets = DisjointSets::new(graph.node_bound());
    let mut selected = Vec::with_capacity(graph.node_count().saturating_sub(1));
    let mut total_weight = 0_u128;
    let mut component_count = graph.node_count();
    for (weight, _, edge) in weighted {
        let Some(endpoints) = graph.edge_endpoints(edge) else {
            continue;
        };
        let source = G::node_slot(endpoints.source());
        let target = G::node_slot(endpoints.target());
        if sets.union(source, target) {
            selected.push(edge);
            total_weight += u128::from(weight);
            component_count -= 1;
        }
    }
    SpanningForest {
        edges: selected,
        total_weight,
        component_count,
    }
}

struct DisjointSets {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl DisjointSets {
    fn new(bound: usize) -> Self {
        Self {
            parent: (0..bound).collect(),
            rank: vec![0; bound],
        }
    }

    fn find(&mut self, mut node: usize) -> usize {
        let mut root = node;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        while self.parent[node] != node {
            let parent = self.parent[node];
            self.parent[node] = root;
            node = parent;
        }
        root
    }

    fn union(&mut self, left: usize, right: usize) -> bool {
        let mut left = self.find(left);
        let mut right = self.find(right);
        if left == right {
            return false;
        }
        if self.rank[left] < self.rank[right] {
            std::mem::swap(&mut left, &mut right);
        }
        self.parent[right] = left;
        if self.rank[left] == self.rank[right] {
            self.rank[left] = self.rank[left].saturating_add(1);
        }
        true
    }
}
