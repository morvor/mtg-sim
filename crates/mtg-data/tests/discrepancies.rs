//! The log of erroneous rulings (`docs/RULING_DISCREPANCIES.md`) is complete and exact:
//! every rulings exemption that sets a ruling aside as outdated, inconsistent, misattached
//! or contradicting the card's Oracle text has an entry, and every entry quotes real
//! rulings verbatim with their dates and names files, tests and register ids that exist.

#[test]
fn ruling_discrepancies_log_is_complete() {
    let root = mtg_data::interpretations::repo_root();
    let problems = mtg_data::discrepancies::check(&root);
    assert!(
        problems.is_empty(),
        "docs/RULING_DISCREPANCIES.md problems:\n{}",
        problems.join("\n")
    );
}
