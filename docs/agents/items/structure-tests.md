# structure-tests: test the compiled ability structures no test exercises

WHY: abilities that compile to the same structure (the AST with numbers, names and mana amounts abstracted; `crates/mtg-engine/src/structure.rs`) run the same engine code, so one test covers all of them. docs/STRUCTURE_COVERAGE.md (Oct 2, before about 6,000 more cards became supported) had 3,746 of 17,550 distinct structures exercised by some test (21.3%), and 35.5% of supported cards structure-verified (every ability's structure exercised). An unexercised structure is engine behavior for real cards that no test has ever run.

This is a stream of items, `structure-tests-<N>`. Each item takes one or more ability kinds (activated, triggered, spell, static, replacement, mana, keyword) so that items running in parallel never take the same structures; the queue message says which kinds.

For each item:
1. Measure on the latest origin PR branch (with `CARGO_BUILD_JOBS=2`):
   `rm -rf target/structlog && MTG_STRUCTURE_LOG=$PWD/target/structlog cargo test --workspace`, then
   `cargo run --release -p mtg-tools -- structure-coverage --write docs/STRUCTURE_COVERAGE.md`.
2. Take the top ~60 unexercised structures of the item's kinds by the number of cards that share them (`structure-coverage --show-fingerprints`, `--card NAME`).
3. For each structure, write a test with a real card that has it, through `TestGame`, asserting that the in-game outcome is exactly what the card's text says: the right objects and players affected and not others, the amounts, the duration (check it ends), conditions both true and false, targets legal and illegal. Not just "it resolves". Cite `cr!` for the rules the test exercises and `ruling!` for any ruling of that card it checks. Use compact table-driven tests where several structures share a setup. Files: `crates/mtg-engine/tests/cards/structure_<kind>_<N>.rs`.
4. A card that misbehaves is the most valuable result: fix the engine generally (no card-name special cases) and keep the test as a regression test. If a card's compiled ability doesn't mean what its text says, it's a parser bug: fix the parser or make the text unsupported.
5. Re-run step 1 and report before and after: structures exercised, card abilities exercised, cards structure-verified, bugs found and fixed. Commit the regenerated docs/STRUCTURE_COVERAGE.md.

DONE per item when its ~60 structures are exercised (or each skipped one has a reason) and the numbers are recorded.
