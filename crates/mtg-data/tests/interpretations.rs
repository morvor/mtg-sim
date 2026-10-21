//! The interpretation register (`docs/interpretations/`) is well formed and every entry is
//! pinned by a test that cites it.

#[test]
fn interpretation_register_is_consistent() {
    let root = mtg_data::interpretations::repo_root();
    let problems = mtg_data::interpretations::check(&root);
    assert!(
        problems.is_empty(),
        "interpretation register problems:\n{}",
        problems.join("\n")
    );
}

#[test]
fn register_parser_reads_fields() {
    let text = "# Area\n\n## some-id: A title\n- Kind: Policy\n- Status: provisional\n- Rules: CR 1\n\n## other: B\n- Kind: deviation\n";
    let e = mtg_data::interpretations::parse_file(std::path::Path::new("x/area.md"), text);
    assert_eq!(e.len(), 2);
    assert_eq!(e[0].id, "some-id");
    assert_eq!(e[0].title, "A title");
    assert_eq!(e[0].kind, "policy");
    assert_eq!(e[0].status, "provisional");
    assert_eq!(e[0].area, "area");
    assert_eq!(e[1].kind, "deviation");
    assert!(e[1].status.is_empty());
}
