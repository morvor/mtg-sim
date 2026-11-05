# verified-refresh-bugs: the two bugs verified-refresh-nov4 found, and the sweep recipe

Queued by the coordinator (Nov 5) before rulings P008. Details of both bugs are in the
"Review" part of docs/agents/items/verified-refresh-nov4.md.

1. `crates/mtg-sim/src/every_card.rs` (the Wastes rule in `setup`) looks for "Colorless" in
   `format!("{:?}")` of the mana cost, which prints "{C}", so cards with {C} in their cost but
   not their text (Breaker of Creation, Kozilek the Great Distortion, Devourer of Destiny,
   Flayer of Loyalties, Zhulodok, Wastescape Battlemage, Calamity of the Titans, Echoes of
   Eternity, Null Elemental Blast) never get Wastes and are never cast. Check the cost's
   symbols properly (ManaSymbol::Colorless / ColorlessHybrid, whatever the type has), with a
   test.
2. Mandatory-loop detection (`crates/mtg-engine/src/game_end.rs`): `check_mandatory_loop`
   keeps `loop_mark` as `actions_taken - forced_asks`, but `note_decision` compares it with
   `actions_taken`, so once any earlier decision was forced, a looping trigger with two or more
   candidates resets detection every time (Hostage Taker chain never draws, CR 104.4b).
   Fix it, with a regression test that makes a forced choice before the loop, failing on the
   old code, citing the loop rules (CR 104.4b and CR 732, whatever subrules it exercises).
   Check whether Charismatic Conqueror's slow sweep game has the same cause
   (`mtg-sim --every-card --from 4505 --count 1 --game 0 --max-turns 30 --log`) and say.
3. The sweep recipe covers only 30,300 of the 32,656-card pool: update CLAUDE.md's "Commands"
   section, mtg-tools' `TIER_RECIPE` (feeds docs/CARD_COVERAGE.md) and `verified.rs` so the
   chunks cover the whole pool (e.g. four chunks, or counts that sum past the pool; keep each
   chunk around 45 min with 3 threads), and regenerate docs/CARD_COVERAGE.md's recipe text if
   it is generated (no full sweep).
4. Then an incremental refresh of the affected cards only: fresh structure log
   (`rm -rf target/structlog && MTG_STRUCTURE_LOG=$PWD/target/structlog cargo test --workspace`),
   `--names` sweep of the 9 {C} cards, Hostage Taker, Charismatic Conqueror and the 8 cards the
   nov4 reviewer listed (Baron Helmut Zemo, Captain America First Avenger, Chaos Moon, Chef's
   Kiss, Toggo Goblin Weaponsmith, Uldaros Theorix, Vesuvan Shapeshifter, The Capitoline Triad),
   then `verified --logs target/structlog --fuzz-from docs/verified-cards.tsv --report
   target/every-card-partial.tsv --write` and `card-coverage --write docs/CARD_COVERAGE.md`.
   Report before/after counts (before: 17,468 verified, 32,060 fully supported).

## Result
(filled in by the implementer)
