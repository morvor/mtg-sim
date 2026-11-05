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
Done Nov 5 (implementer), on 9edfc568 merged with origin 19888498.

1. **Wastes for {C} costs** (`every_card.rs`, `wants_wastes`): the cost's symbols are checked
   (`ManaSymbol::Colorless`, `ColorlessHybrid`) instead of the Debug text. Unit test
   `every_card::tests::a_colorless_symbol_in_the_mana_cost_alone_gets_wastes` (Breaker of
   Creation yes, Eldrazi Temple yes, Grizzly Bears and Ulamog's Crusher no) fails on the old
   check. 18 cards in the data have {C} in a cost and not in their text (16 in the pool).
2. **Loop detection after a forced choice** (`game_end.rs`, `note_decision`): compares
   `loop_mark` with `actions_taken - forced_asks`, the units `check_mandatory_loop` keeps it in,
   and leaves single-answer decisions to `forced_asks` (bumping both would count them twice; the
   existing one-candidate test pins that). New tests in `tests/cr/r104_loop_with_targets.rs`
   (cite 104.4b, 732.4, 732.5 and `mandatory-loop-detection`): a five-Faceless-Devourer chain and
   a five-Hostage-Taker chain, each with a single-target choice first and two targets every time
   around; both are draws now and both run past 4,000 decisions without a result on the old code.
   **Charismatic Conqueror** is a different cause: no target choices are involved; with two
   Conquerors on each side every Vampire token triggers both opposing Conquerors, only one can
   tap it, so each token makes another and the state grows (about 1,500 tokens in 60 s, turn 10).
   That is the known deviation `growing-loops-run-to-the-limit` (the state never repeats, so the
   game runs to the action limit), not this bug; still slow after the fix, left as is.
3. **Recipe**: four chunks `--from 0|8500|17000|25500 --count 8500` (34,000 cards, ~40 min each;
   the pool is 32,672 now), `cat target/every-card-[0123].tsv`, in CLAUDE.md, `TIER_RECIPE` and
   `verified.rs`; CARD_COVERAGE.md regenerated (its note now says a pool past 34,000 needs a
   fifth chunk).
4. **Incremental refresh**: fresh structure log (`cargo test --workspace`, 15 log files, all
   passing), `--names` sweep of 28 names (the 18 {C}-cost cards, Hostage Taker, Charismatic
   Conqueror, the reviewer's 8; Colorless Ultimatum and Wowzer, the Aspirational aren't in the
   pool), 26 cards / 52 games, then `verified --fuzz-from` and `card-coverage`.

| Tier | Before | After |
|---|---:|---:|
| verified | 17,468 | **17,994** (+526) |
| round-trip | 11,085 | 10,565 |
| compiled | 3,507 | 3,517 |
| unsupported | 51 | 35 |
| fully supported | 32,060 | **32,076** (+16) |

Most of the gain is the fresh structure log (525 round-trip -> verified from tests landed since
the nov4 log); 1 verified -> round-trip (Veil of Assimilation: a structure no test exercises now).
Swept cards that changed tier: Breaker of Creation round-trip -> verified; Hostage Taker
round-trip -> verified (its games are clean); The Capitoline Triad compiled -> verified;
Uldaros Theorix unsupported -> verified; Baron Helmut Zemo, Chef's Kiss unsupported -> round-trip;
Captain America First Avenger, Chaos Moon, Toggo Goblin Weaponsmith, Vesuvan Shapeshifter
unsupported -> compiled. fuzz-cast cleared (same tier, other checks remain) for Devourer of
Destiny, Echoes of Eternity, Flayer of Loyalties, Kozilek the Great Distortion, Wastescape
Battlemage, Zhulodok Void Gorger. Still never cast, not the Wastes bug: Calamity of the Titans
(needs a colorless creature card in hand to reveal), Null Elemental Blast (needs a multicolored
spell or permanent), Chef's Kiss (needs a spell with a single target). Charismatic Conqueror
stays compiled with fuzz-clean (slow) (both games). fuzz-swept 9: cards that compile now but
weren't in the nov4 sweep (Backdraft, Glyph of Delusion, Glyph of Reincarnation, Guff Rewrites
History, Lim-Dûl's Vault, Rootweaver Druid, The Toymaker's Trap, Timesifter, Xanathar Guild
Kingpin); not swept here (outside this item's list), a `--names` sweep of them would settle them.
