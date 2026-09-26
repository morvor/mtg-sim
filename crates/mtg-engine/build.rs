//! Generates module declarations for directories that agents extend by adding files,
//! so that adding a keyword implementation, an oracle pattern file, or a test module
//! never requires editing a shared `mod` list.

use std::path::{Path, PathBuf};

fn gen(dir: &Path, skip: &[&str], out: &Path, vis: &str) {
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut names: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "rs") {
                let stem = p.file_stem().unwrap().to_string_lossy().to_string();
                if !skip.contains(&stem.as_str()) {
                    names.push((stem, p.canonicalize().unwrap()));
                }
            }
        }
    }
    names.sort();
    let mut s = String::new();
    for (n, p) in names {
        s.push_str(&format!(
            "#[path = {:?}]\n{vis}mod {n};\n",
            p.display().to_string()
        ));
    }
    std::fs::write(out, s).unwrap();
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    gen(
        &root.join("src/kw"),
        &["mod"],
        &out.join("kw_mods.rs"),
        "pub ",
    );
    gen(
        &root.join("src/kwa"),
        &["mod"],
        &out.join("kwa_mods.rs"),
        "pub ",
    );
    gen(
        &root.join("src/oracle/patterns"),
        &["mod"],
        &out.join("oracle_pattern_mods.rs"),
        "pub ",
    );
    for t in ["cr", "keywords", "rulings", "cards", "actions"] {
        gen(
            &root.join("tests").join(t),
            &["main"],
            &out.join(format!("test_mods_{t}.rs")),
            "",
        );
    }
}
