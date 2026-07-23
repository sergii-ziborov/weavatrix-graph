# Weavatrix Graph

[![CI](https://github.com/sergii-ziborov/weavatrix-graph/actions/workflows/ci.yml/badge.svg)](https://github.com/sergii-ziborov/weavatrix-graph/actions/workflows/ci.yml)

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

## Quality Gates

Local checks:

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo doc --locked --no-deps
```

The test suite includes architecture and duplicate-contract ratchets: source
files stay below 300 lines, `model` and `kind` remain focused folder modules,
runtime dependencies remain limited, and canonical kind strings cannot collide.

CI also runs measured Rust coverage with `cargo-tarpaulin` and fails below 85%.
Weavatrix architecture verification is backed by `.weavatrix/architecture.json`.

## License

MIT
