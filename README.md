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
- source spans, evidence kind, confidence, and extractor provenance;
- deterministic node and edge order independent of insertion order;
- idempotent insertion of identical nodes and edges;
- rejection of conflicting nodes, dangling edges, and invalid source spans;
- validated deserialization that cannot bypass graph invariants;
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

## Extension kinds

Known Weavatrix kinds are enum variants. Ecosystem-specific kinds remain
forward-compatible through `Custom` values:

```rust
use weavatrix_graph::NodeKind;

let kind = NodeKind::custom("terraform_resource")?;
assert_eq!(kind.as_str(), "terraform_resource");
# Ok::<(), weavatrix_graph::GraphError>(())
```

## License

MIT
