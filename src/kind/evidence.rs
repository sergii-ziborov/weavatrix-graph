string_kind!(
    /// Origin of the evidence supporting an edge.
    EvidenceKind,
    "evidence",
    {
        ExactLsp => "EXACT_LSP",
        Extracted => "EXTRACTED",
        ResolvedCanonical => "RESOLVED",
        Inferred => "INFERRED",
        Conflict => "CONFLICT",
        Parsed => "parsed",
        Resolved => "resolved",
        Manifest => "manifest",
        Literal => "literal",
        Toolchain => "toolchain",
        Runtime => "runtime",
    }
);
