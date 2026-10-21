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

#[test]
fn citations_are_found_across_lines_and_outside_comments() {
    let src = "fn t() {\r\n    interp!(\"a-b\");\r\n    // interp!(\"commented\")\r\n    interp!(\r\n        \"c\",\r\n        \"d\", // two ids\r\n    );\r\n    let s = \"http://x\"; interp!(\"e\");\r\n}\r\n";
    let found = mtg_data::interpretations::citations_in(src);
    let ids: Vec<(usize, Vec<String>)> = found;
    assert_eq!(
        ids,
        vec![
            (2, vec!["a-b".to_string()]),
            (4, vec!["c".to_string(), "d".to_string()]),
            (8, vec!["e".to_string()]),
        ]
    );
}

#[test]
fn register_parser_handles_crlf_and_a_byte_order_mark() {
    let text = "\u{feff}## crlf-id: Title\r\n- Kind: interpretation\r\n- Status: decided\r\n";
    let e = mtg_data::interpretations::parse_file(std::path::Path::new("a.md"), text);
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].id, "crlf-id");
    assert_eq!(e[0].title, "Title");
    assert_eq!(e[0].kind, "interpretation");
    assert_eq!(e[0].status, "decided");
}

#[test]
fn interpretations_index_is_current() {
    let root = mtg_data::interpretations::repo_root();
    let entries = mtg_data::interpretations::load(&root.join("docs/interpretations"));
    let index = mtg_data::interpretations::render_index(&entries);
    let current = std::fs::read_to_string(root.join("docs/INTERPRETATIONS.md"))
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert!(
        current == index,
        "docs/INTERPRETATIONS.md is out of date: run \
         `cargo run --release -p mtg-tools -- interpretations --write`"
    );
}
