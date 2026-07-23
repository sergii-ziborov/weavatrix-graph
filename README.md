# Weavatrix Graph

[![CI](https://github.com/sergii-ziborov/weavatrix-graph/actions/workflows/ci.yml/badge.svg)](https://github.com/sergii-ziborov/weavatrix-graph/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/weavatrix-graph.svg)](https://crates.io/crates/weavatrix-graph)
[![docs.rs](https://docs.rs/weavatrix-graph/badge.svg)](https://docs.rs/weavatrix-graph)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/sergii-ziborov/weavatrix-graph/blob/main/LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](https://github.com/sergii-ziborov/weavatrix-graph/blob/main/Cargo.toml)

`weavatrix-graph` is a small Rust library for deterministic, typed,
evidence-carrying graphs. It is the graph foundation of Weavatrix, but it is
usable by repository analyzers, architecture tools, dependency explorers, and
other applications without depending on Weavatrix itself.

The crate owns graph integrity and serialization. It does **not** walk files,
parse programming languages, execute commands, access the network, or provide
an MCP/CLI transport.

## Properties

- typed nodes and edges with custom extension kinds;
- strongly typed, non-empty node identifiers;
- source spans, optional language labels, evidence kind, confidence, and extractor
  provenance;
- structured node and edge attributes for parser-specific metadata;
- deterministic node and edge order independent of insertion order;
- compact numeric endpoints with incoming and outgoing CSR indexes;
- a mutable insertion-order graph with generation-stable node and edge keys;
- BFS, DFS, reachability, unweighted and weighted shortest paths, SCC, cycle
  discovery, topological sort, MST, and Dinic maximum flow;
- edge-kind, evidence, extractor, confidence, and caller-defined traversal
  filters;
- undirected incidence CSR, a generic dense matrix, and deterministic random
  graph generators;
- idempotent insertion of identical nodes and edges;
- rejection of conflicting nodes, dangling edges, and invalid source spans;
- validated deserialization that cannot bypass graph invariants;
- compatibility conversion from Weavatrix's legacy `{ nodes, links }` graph;
- no unsafe code and one runtime dependency: `serde`.

## Layered graph contracts

The crate keeps storage contracts separate instead of making one graph type pay
for every feature:

| Type | Purpose | Ordering and validation |
| --- | --- | --- |
| `Topology` | Immutable directed numeric graph | Preserves edge order, validates compact endpoints, builds outgoing and incoming CSR |
| `WorkingGraph` | Fast rich mutation and incremental extraction | Preserves insertion order, validates local invariants once, uses generation-stable keys |
| `Graph` | Immutable evidence snapshot and wire format | Sorts, deduplicates, validates, and emits canonical output |
| `UndirectedTopology` | General-purpose undirected algorithms | Compact incidence CSR with parallel-edge and self-loop support |
| `DenseMatrix<T>` | Small dense graphs | Fixed-size O(1) edge lookup without sparse-graph overhead |

`WorkingGraph::freeze()` is the explicit boundary between extraction and
publication. It returns the canonical `Graph` plus a stable-to-compact index
map. `Graph::try_from_sorted_nodes` avoids rebuilding the node-id map when an
extractor already emits unique sorted nodes, while
`Graph::try_from_sorted_parts` is the fastest fully canonical input path.

## Example

```rust
use weavatrix_graph::{
    Confidence, Edge, EdgeKind, EvidenceKind, GraphBuilder, Node, NodeKind,
    Provenance,
};

let repository = Node::new("repo:demo", "demo", NodeKind::Repository)?;
let file = Node::new("file:src/lib.rs", "src/lib.rs", NodeKind::File)?;

let mut builder = GraphBuilder::new();
builder.add_node(repository.clone())?;
builder.add_node(file.clone())?;
builder.add_edge(Edge::new(
    repository.id,
    file.id,
    EdgeKind::Contains,
    Provenance::new("example", EvidenceKind::Parsed, Confidence::High)?,
))?;

let graph = builder.build()?;
assert_eq!(graph.node_count(), 2);
assert_eq!(graph.edge_count(), 1);
# Ok::<(), weavatrix_graph::GraphError>(())
```

Algorithms use `GraphView`/`IndexGraphView`, so the same call works with a
canonical `Graph`, a numeric `Topology`, or a mutable `WorkingGraph`.
Filtering stays outside the topology and can inspect the evidence payload:

```rust
use weavatrix_graph::{
    Confidence, Direction, EdgeFilter, EdgeKind, EvidenceKind, Graph,
    bfs_filtered,
};

# fn inspect(graph: &Graph) -> Result<(), weavatrix_graph::GraphError> {
let start = graph.node_index("repo:demo").unwrap();
let filter = EdgeFilter::new()
    .with_kind(EdgeKind::Contains)
    .with_evidence(EvidenceKind::Parsed)
    .with_minimum_confidence(Confidence::High);

let reachable = bfs_filtered(graph, start, Direction::Outgoing, |index| {
    graph.edge_at(index).is_some_and(|edge| filter.matches(edge))
});
assert!(!reachable.is_empty());
# Ok(())
# }
```

## Extension Kinds

Known relation and node kinds are enum variants. Ecosystem-specific kinds remain
forward-compatible through `Custom` values. Language taxonomies intentionally
belong to analyzers, not the graph core; nodes carry language as a validated
string label.

```rust
use weavatrix_graph::NodeKind;

let kind = NodeKind::custom("terraform_resource")?;
assert_eq!(kind.as_str(), "terraform_resource");
# Ok::<(), weavatrix_graph::GraphError>(())
```

## Weavatrix compatibility

The core graph format intentionally keeps canonical `nodes` and `edges`, but the
crate can ingest the current JavaScript Weavatrix `{ nodes, links }` shape:

```rust
use weavatrix_graph::{Graph, LegacyGraph};

let legacy: LegacyGraph = serde_json::from_str(r#"{
  "nodes": [
    { "id": "src/lib.rs", "label": "lib.rs" },
    { "id": "src/lib.rs#entry@1", "label": "entry()" }
  ],
  "links": [
    {
      "source": "src/lib.rs",
      "target": "src/lib.rs#entry@1",
      "relation": "contains",
      "confidence": "EXTRACTED"
    }
  ],
  "edgeTypesV": 2,
  "edgeProvenanceV": 1
}"#)?;

let graph: Graph = legacy.into_graph("weavatrix-js")?;
assert_eq!(graph.edge_count(), 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Legacy metadata such as `line`, `compileOnly`, `typeOnly`, `specifier`,
`usage`, `source_range`, and unknown extension fields is preserved as structured
attributes.

## Benchmarks

The repository includes benchmark harnesses for graph construction, indexed
queries, JSON serialization, validated deserialization, and dev-only
comparisons with `petgraph 0.8.3` and `graaf 0.112.0`:

```sh
cargo bench --locked
```

Each workload runs two warmups and 11 measured iterations. The tables below use
the median of five independent harness medians on Windows 11 with Rust 1.97.1.
They compare equal contracts where possible and label preprocessing explicitly.

### Rich evidence construction

10,000 nodes and 30,000 evidence-carrying edges:

| Mode | Library | Median |
| --- | --- | ---: |
| Unsorted canonical snapshot | weavatrix-graph `Graph` | 32.862 ms |
| Sorted canonical snapshot | weavatrix-graph `Graph` | 19.059 ms |
| Validated mutable append | weavatrix-graph `WorkingGraph` | 19.551 ms |
| Payload append, no canonicalization | petgraph adapter | 20.215 ms |
| Mutable append plus canonical `freeze()` | weavatrix-graph | 45.495 ms |

The petgraph adapter resolves string ids and clones the same payload but does
not validate, sort, or deduplicate it. `WorkingGraph` remains slightly faster
while validating local invariants. `freeze()` is reported separately because it
adds canonical sorting, evidence deduplication, and immutable CSR construction.

### Compact dual CSR

10,000 numeric nodes and 30,000 edges:

| Mode | Library | Median |
| --- | --- | ---: |
| Arbitrary input, endpoint validation, both CSR directions | weavatrix-graph | 0.365 ms |
| Two CSR builds from caller-provided pre-sorted directions | petgraph | 0.463 ms |
| Sorting/dedup plus both CSR builds | petgraph | 1.699 ms |

The pre-sorted petgraph row deliberately excludes preparing two differently
sorted edge arrays. It is retained because that narrower contract can be useful
when a caller already owns both orders.

### Algorithms

10,000 nodes and 30,000 edges, except maximum flow at 1,000/5,000:

| Algorithm | weavatrix-graph | petgraph |
| --- | ---: | ---: |
| BFS | 0.095 ms | 0.125 ms |
| Strongly connected components | 0.333 ms | 0.606 ms |
| Dijkstra to one target | 0.783 ms | 1.036 ms |
| Minimum spanning forest | 1.042 ms | 1.599 ms |
| Dinic maximum flow | 0.276 ms | 0.284 ms |

Deterministic randomized differential tests also compare reachability, shortest
path existence and cost, SCC partitions, cycle status, topological feasibility,
MST weight, and maximum-flow value against petgraph.

Incoming and outgoing indexes are rebuilt during graph construction and
deserialization. They are intentionally excluded from JSON, so the canonical
wire format remains only `nodes` and `edges`. Resolve a stable string id once
with `node_index`, then use `node_at`, `outgoing_at`, `incoming_at`,
`out_degree`, and `in_degree` in repeated graph algorithms.

Extractors that already emit sorted nodes can use
`Graph::try_from_sorted_nodes`; fully canonical input can use
`Graph::try_from_sorted_parts`. Both keep validation, endpoint checks,
deduplication, and both indexes. Unordered input safely falls back to the
canonicalizing constructor.

`petgraph` and `graaf` are dev-dependencies only. The runtime dependency budget
remains unchanged.

Timing varies by allocator, CPU, and build toolchain. Run the included harnesses
on the deployment target before using these figures for capacity planning.

## Quality Gates

Local checks:

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo doc --locked --no-deps
cargo llvm-cov --workspace --all-features --fail-under-lines 85
cargo bench --locked
```

The test suite includes architecture and duplicate-contract ratchets: every Rust
source stays at or below 300 lines, domain facades remain small, runtime
dependencies remain limited, and canonical kind strings cannot collide.

CI also runs measured Rust coverage with `cargo-llvm-cov`, emits `lcov.info`
for analyzer import, and fails below 85% line coverage. Weavatrix architecture
verification is backed by `.weavatrix/architecture.json`. The current local
LLVM report measures 93.16% of lines and 91.13% of functions.

## License

MIT
