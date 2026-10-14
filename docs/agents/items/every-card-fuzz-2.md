# every-card-fuzz-2: re-run the every-card sweep on the current engine and fix what it finds

WHY: since the last full sweep (docs/EVERY_CARD.md), card support grew from about 22,700 to about 28,600 fully supported cards and the engine changed a lot. Every newly supported card needs to be played in real games under the rules checks.

1. On the latest origin PR branch, build in release (`CARGO_BUILD_JOBS=2`) and run the full sweep: `cargo run --release -p mtg-sim -- --every-card --threads 3` (4 cores; leave one for the system). Record every failure: panics, hangs (watchdog), and rules violations from the invariant checks, each with its replay command.
2. Fix every failure in the engine generally (no card-name special cases), each with a regression test in crates/mtg-engine/tests (real cards through TestGame). Re-run the affected chunk to confirm.
3. Look at the cards whose abilities were never used and improve the deck setup or RandomAgent choices where a common reason shows up.
4. Update docs/EVERY_CARD.md (cards, games, exercised share, failures fixed, remaining failures with causes).

DONE when a full sweep over every supported card runs clean, or the remaining failures are listed with causes.
