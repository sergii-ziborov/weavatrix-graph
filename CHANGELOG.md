# Changelog

All notable changes to this project will be documented in this file.

## 0.1.1 - 2026-07-23

- Add deterministic incoming and outgoing adjacency indexes.
- Reduce a 10,000-node/30,000-edge full adjacency workload from seconds to
  milliseconds without changing the serialized graph contract.
- Add repeatable build, query, JSON serialization, and validated
  deserialization benchmarks.
- Document benchmark methodology and sample results.

## 0.1.0 - 2026-07-22

- Initial typed graph model and builder.
- Deterministic ordering and idempotent insertion.
- Evidence provenance and source spans.
- Extensible node, edge, and evidence kinds.
- Language remains a validated node label instead of a graph-core taxonomy.
- Canonical Weavatrix relation/provenance values, including `method`,
  `implements`, `re_exports`, `EXACT_LSP`, `EXTRACTED`, `RESOLVED`,
  `INFERRED`, and `CONFLICT`.
- Structured node and edge attributes without adding a runtime JSON dependency.
- Compatibility conversion from Weavatrix's legacy `{ nodes, links }` graph.
- Integrity-checked serialization boundary.
