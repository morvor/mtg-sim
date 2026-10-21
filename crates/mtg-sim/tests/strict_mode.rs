//! Strict mode end to end: `mtg-sim` rejects a deck with a card below the `--cards` bar
//! (verified by default) before any game starts, and plays it with a lower bar.

use std::path::PathBuf;
use std::process::Command;

fn scratch(name: &str, text: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mtg-sim-strict-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(name);
    std::fs::write(&p, text).unwrap();
    p
}

fn sim(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_mtg-sim"))
        .args(args)
        .output()
        .expect("run mtg-sim");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stderr).into_owned() + &String::from_utf8_lossy(&out.stdout),
    )
}

#[test]
fn a_deck_with_an_unverified_card_is_rejected_unless_the_bar_is_lowered() {
    let tiers = scratch(
        "tiers.tsv",
        "name\ttier\tfailed\tnotes\nForest\tverified\t\t\nGrizzly Bears\tverified\t\t\nHill Giant\tcompiled\tround-trip\t\nMountain\tverified\t\t\n",
    );
    let deck = scratch(
        "deck.txt",
        "20 Forest\n20 Mountain\n10 Grizzly Bears\n10 Hill Giant\n",
    );
    let (tiers, deck) = (tiers.to_str().unwrap(), deck.to_str().unwrap());
    let base = [
        "--tiers",
        tiers,
        "--deck",
        deck,
        "--deck",
        deck,
        "--games",
        "1",
        "--max-turns",
        "2",
    ];
    let (code, out) = sim(&base);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("below the verified tier"), "{out}");
    assert!(
        out.contains("Hill Giant: compiled; fails round-trip"),
        "{out}"
    );
    assert!(!out.contains("Grizzly Bears:"), "{out}");
    assert!(!out.contains("1 games in"), "no game was played: {out}");

    let mut lowered = base.to_vec();
    lowered.extend(["--cards", "compiled"]);
    let (code, out) = sim(&lowered);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("1 games in"), "{out}");
}
