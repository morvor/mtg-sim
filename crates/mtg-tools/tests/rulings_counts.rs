//! The two rulings counters agree: `mtg-tools rulings-coverage` and
//! `scripts/rulings_batches.py summary` count the same texts as in scope, cited, exempt,
//! open, deferred and excluded (docs/agents/finish-line-plan.md, criterion C1), each
//! printing the same `rulings: texts=... cited=... exempt=... open=... deferred=...
//! excluded=...` line. The tool's scope predicates (`out_of_scope_reason`,
//! `deferred_reason`, the exemption matching) and the script's mirror each other; this
//! test is what keeps them in step on the committed data.

use std::process::Command;

/// The Python interpreter, if one is available.
fn python() -> Option<String> {
    ["python3", "python"]
        .into_iter()
        .find(|p| {
            Command::new(p)
                .arg("--version")
                .output()
                .is_ok_and(|o| o.status.success())
        })
        .map(str::to_string)
}

fn rulings_line(output: &[u8]) -> Option<String> {
    String::from_utf8_lossy(output)
        .lines()
        .find(|l| l.starts_with("rulings: "))
        .map(str::to_string)
}

#[test]
fn the_tool_and_the_batches_script_count_the_same_rulings() {
    let Some(py) = python() else {
        eprintln!("python3 not found: skipping the rulings count comparison");
        return;
    };
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let tool = Command::new(env!("CARGO_BIN_EXE_mtg-tools"))
        .arg("rulings-coverage")
        .current_dir(root)
        .output()
        .expect("run mtg-tools rulings-coverage");
    assert!(
        tool.status.success(),
        "mtg-tools rulings-coverage failed:\n{}",
        String::from_utf8_lossy(&tool.stdout)
    );
    let script = Command::new(py)
        .args(["scripts/rulings_batches.py", "summary"])
        .current_dir(root)
        .output()
        .expect("run scripts/rulings_batches.py summary");
    assert!(
        script.status.success(),
        "scripts/rulings_batches.py summary failed:\n{}",
        String::from_utf8_lossy(&script.stderr)
    );
    let tool_line = rulings_line(&tool.stdout).expect("the tool prints a `rulings:` line");
    let script_line = rulings_line(&script.stdout).expect("the script prints a `rulings:` line");
    assert_eq!(
        tool_line, script_line,
        "mtg-tools rulings-coverage and scripts/rulings_batches.py disagree on the rulings counts"
    );
}
