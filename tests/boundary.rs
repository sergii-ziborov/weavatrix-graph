use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn library_has_no_process_network_filesystem_or_unsafe_path() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    collect_rust_sources(&root.join("src"), &mut sources);
    let banned = [
        "std::process",
        "std::net",
        "std::fs",
        "unsafe {",
        "reqwest",
        "tokio",
        "tree_sitter",
        "petgraph",
    ];
    for path in sources {
        let source = fs::read_to_string(&path).unwrap();
        for marker in banned {
            assert!(
                !source.contains(marker),
                "{} contains forbidden graph marker {marker}",
                path.display()
            );
        }
    }
}

#[test]
fn runtime_dependency_budget_contains_only_serde() {
    let manifest =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("[dev-dependencies]")
        .next()
        .unwrap();
    let entries = dependencies
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].starts_with("serde ="));
}

fn collect_rust_sources(directory: &Path, output: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .unwrap()
        .collect::<std::io::Result<Vec<_>>>()
        .unwrap();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(&path, output);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            output.push(path);
        }
    }
}
