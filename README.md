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
- indexed node lookup plus incoming and outgoing adjacency queries;
- idempotent insertion of identical nodes and edges;
- rejection of conflicting nodes, dangling edges, and invalid source spans;
- validated deserialization that cannot bypass graph invariants;
- compatibility conversion from Weavatrix's legacy `{ nodes, links }` graph;
- no unsafe code and one runtime dependency: `serde`.

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

Each workload runs two warmups and 11 measured iterations, then reports the
median and minimum. Sample result on Windows 11 with Rust 1.97.1:

| Workload | Graph size | Median |
| --- | ---: | ---: |
| Validated build | 10,000 nodes / 30,000 edges | 28.2 ms |
| 10,000 node lookups + 20,000 adjacency walks | 10,000 / 30,000 | 3.2 ms |
| JSON serialization | 5,000 / 15,000, 2.86 MB | 3.1 ms |
| Validated JSON deserialization | 5,000 / 15,000, 2.86 MB | 21.2 ms |

Competitor sample from the same machine and workload:

| Mode | Library | Median |
| --- | --- | ---: |
| Same `Node`/`Edge` payload build | weavatrix-graph | 33.1 ms |
| Same `Node`/`Edge` payload build | petgraph adapter | 16.4 ms |
| Bare topology build | petgraph | 0.175 ms |
| Bare topology build | graaf | 1.423 ms |
| Sum in/out degree for 10,000 nodes | weavatrix-graph | 0.026 ms |
| Sum in/out degree for 10,000 nodes | petgraph | 0.049 ms |
| Sum in/out degree for 10,000 nodes | graaf | 273.6 ms |

These rows expose different contracts. `petgraph` appends numeric topology to
preallocated vectors in O(1); its adapter row does not canonicalize, deduplicate,
or validate the evidence payload. Weavatrix Graph performs those checks and
uses source buckets plus compact incoming/outgoing indexes. Bare topology is
therefore not a claim of parity, while repeated bidirectional degree queries
are a directly comparable hot path.

Incoming and outgoing indexes are rebuilt during graph construction and
deserialization. They are intentionally excluded from JSON, so the canonical
wire format remains only `nodes` and `edges`. Resolve a stable string id once
with `node_index`, then use `node_at`, `outgoing_at`, `incoming_at`,
`out_degree`, and `in_degree` in repeated graph algorithms.

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
cargo bench --locked
```

The test suite includes architecture and duplicate-contract ratchets: source
files stay below 300 lines, `model` and `kind` remain focused folder modules,
runtime dependencies remain limited, and canonical kind strings cannot collide.

CI also runs measured Rust coverage with `cargo-tarpaulin` and fails below 85%.
Weavatrix architecture verification is backed by `.weavatrix/architecture.json`.

## License

MIT
