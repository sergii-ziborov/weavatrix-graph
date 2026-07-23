use std::{env, fs, time::Instant};
use weavatrix_graph::{EdgeKind, EvidenceKind, Graph, LegacyGraph};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1).peekable();
    if args.peek().is_none() {
        eprintln!("usage: legacy_check <graph.json>...");
        std::process::exit(2);
    }

    for path in args {
        let started = Instant::now();
        let source = fs::read_to_string(&path)?;
        let read_ms = started.elapsed().as_secs_f64() * 1000.0;
        let parse_started = Instant::now();
        let legacy: LegacyGraph = serde_json::from_str(&source)?;
        let parse_ms = parse_started.elapsed().as_secs_f64() * 1000.0;
        let legacy_nodes = legacy.nodes.len();
        let legacy_links = legacy.links.len();
        let convert_started = Instant::now();
        let graph: Graph = legacy.into_graph("weavatrix.legacy_check")?;
        let convert_ms = convert_started.elapsed().as_secs_f64() * 1000.0;
        let unique_edges = graph.edge_count();
        let method_edges = graph
            .edges()
            .iter()
            .filter(|edge| edge.kind == EdgeKind::Method)
            .count();
        let exact_lsp_edges = graph
            .edges()
            .iter()
            .filter(|edge| edge.provenance.evidence == EvidenceKind::ExactLsp)
            .count();
        let attributed_edges = graph
            .edges()
            .iter()
            .filter(|edge| !edge.attributes.is_empty())
            .count();

        println!(
            "{path}\tnodes={legacy_nodes}->{nodes}\tedges={legacy_links}->{edges}\tunique_edges={unique_edges}\tduplicate_edges={duplicate_edges}\tmethod={method_edges}\texact_lsp={exact_lsp_edges}\tattributed_edges={attributed_edges}\tread_ms={read_ms:.2}\tparse_ms={parse_ms:.2}\tconvert_ms={convert_ms:.2}\ttotal_ms={total_ms:.2}",
            nodes = graph.node_count(),
            edges = graph.edge_count(),
            duplicate_edges = legacy_links - unique_edges,
            total_ms = started.elapsed().as_secs_f64() * 1000.0,
        );
    }

    Ok(())
}
