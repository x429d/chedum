fn main() {
    // The bundled Stockfish sidecar is named after the Rust target triple, so
    // runtime discovery needs the same value the bundler used.
    println!(
        "cargo:rustc-env=CHEDUM_TARGET={}",
        std::env::var("TARGET").unwrap_or_default()
    );
    tauri_build::build()
}
