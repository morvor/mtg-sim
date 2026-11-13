//! Guards for hand-written card abilities (`src/cards/`, see its module docs): none is
//! stale or duplicated, none is parsed by the compiler on its own (compile-first policy),
//! and every manual card has a test in this directory.

#[test]
fn manual_definitions_are_current_compiled_first_and_tested() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cards");
    let problems = mtg_engine::cards::manual_check(&dir);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
