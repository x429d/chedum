//! Guards the documents that must ship inside every Chedum bundle.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri must live inside the repository")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

#[test]
fn license_text_is_present_and_is_mit() {
    let license = read("LICENSE");
    assert!(
        license.contains("MIT License"),
        "LICENSE must start with the MIT heading"
    );
    assert!(
        license.contains("Permission is hereby granted, free of charge"),
        "LICENSE must contain the full MIT grant"
    );
}

#[test]
fn third_party_notices_are_present() {
    let notices = read("docs/THIRD-PARTY.md");
    assert!(notices.contains("# Third-party notices"));
    assert!(notices.contains("Stockfish"));
    assert!(notices.contains("GPL-3.0"));
}

#[test]
fn bundle_resources_ship_license_and_notices() {
    let config = read("src-tauri/tauri.conf.json");
    let config: serde_json::Value =
        serde_json::from_str(&config).expect("tauri.conf.json must be valid JSON");
    let resources = &config["bundle"]["resources"];

    let sources: Vec<String> = match resources {
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(str::to_owned)
            .collect(),
        serde_json::Value::Object(map) => map.keys().cloned().collect(),
        _ => panic!("bundle.resources must be a list or a source-to-target map"),
    };

    for expected in ["../LICENSE", "../docs/THIRD-PARTY.md"] {
        assert!(
            sources.iter().any(|source| source == expected),
            "bundle.resources must include {expected}"
        );
    }
}
