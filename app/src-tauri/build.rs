fn main() {
    expose_dune_server_service_version();
    rerun_if_bundled_binaries_change();
    tauri_build::build();
}

/// Tauri's resource-copy step only fires when Cargo decides build.rs needs to
/// re-run, which by default doesn't watch arbitrary files. Without these
/// `rerun-if-changed` lines, refreshing the bundled `dune-server-service`
/// binary or its systemd/openrc units in `binaries/` after a previous build
/// produces a stale `target/release/binaries/` copy — the running exe then
/// pushes the OLD binary on Install/Update, with no visible signal.
fn rerun_if_bundled_binaries_change() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries");
    // Watch the directory itself so file additions/deletions also trigger a rerun.
    println!("cargo:rerun-if-changed={}", dir.display());
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Skip README, .gitignore, and similar bookkeeping files.
            if matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some("README.md") | Some(".gitignore")
            ) {
                continue;
            }
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn expose_dune_server_service_version() {
    let cargo_toml = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/dune-server-service/Cargo.toml");
    println!("cargo:rerun-if-changed={}", cargo_toml.display());
    let contents = std::fs::read_to_string(&cargo_toml)
        .unwrap_or_else(|err| panic!("reading {}: {err}", cargo_toml.display()));
    let version = parse_package_version(&contents).unwrap_or_else(|| {
        panic!(
            "could not find [package].version in {}",
            cargo_toml.display()
        )
    });
    println!("cargo:rustc-env=DUNE_SERVER_SERVICE_VERSION={version}");
}

fn parse_package_version(toml: &str) -> Option<String> {
    let mut in_package = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("version") {
            let rest = rest.trim_start();
            let rest = rest.strip_prefix('=')?.trim_start();
            let rest = rest.trim_start_matches('"');
            let end = rest.find('"')?;
            return Some(rest[..end].to_string());
        }
    }
    None
}
