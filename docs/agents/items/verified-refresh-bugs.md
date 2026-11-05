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

## Review (Nov 5)
- **The loop fix holds.** With `game_end.rs` reverted to origin, both new chain tests fail (no
  result after 4,000 decisions); with the comparison fixed but without the single-answer skip,
  `a_loop_of_targeted_triggered_abilities_is_a_draw` fails, so the skip is pinned.
- **It also turned loops a player could stop into draws.** Since every target choice and
  trigger order now keeps a stretch of mandatory actions going in every game (before, only in
  games with no earlier single-answer choice), a Faceless Devourer chain with a Soltari Foot
  Soldier around (exiling it instead ends the chain) was a draw after 21 decisions while P0 kept
  choosing Devourers. CR 104.4b ("with no way to stop") and 732.5 (a player can be made to take
  an action an object involved in the loop calls for) make that no draw. Fixed in
  `note_decision`: a target choice keeps the stretch going only for a fixed number of targets
  among candidates that are alike (`alike`: one zone, the same loop-fingerprint key, now
  `Game::loop_object_key`, and the same linked cards, CR 607, so a Devourer holding a Soltari
  differs from one holding a Devourer), and a trigger order only when the triggered abilities are
  identical. Other choices interrupt the stretch, as they did before in games with a forced
  choice. The Devourer and Hostage Taker chains (alike candidates) are still draws. The cost: a
  loop whose different choices all lead back into it (Hostage Takers of different controllers)
  runs to the action limit again, which `growing-loops-run-to-the-limit` accepts. New tests in
  `tests/cr/r104_loop_with_targets.rs`, each failing on the implementer's commit:
  `a_loop_another_target_would_end_is_not_a_draw`,
  `a_loop_through_a_devourer_holding_another_card_is_not_a_draw`,
  `a_loop_another_order_of_its_triggers_would_end_is_not_a_draw`; and
  `a_chain_of_single_target_choices_that_changes_the_game_state_is_not_a_draw` (one target each
  time while Soul Warden gains life: the state never repeats, no draw). Register entry
  `mandatory-loop-detection` updated (still decided: it follows 732.5's exception).
- **Draw rates** (release build of the merged head): `mtg-sim --games 100`: 51/49, 0 draws.
  `--random-decks --games 300`: 147/152, 1 draw, game 74 at the 100-turn limit (a Witch's
  Oven/Food stall); no game ended by the loop rule. `--properties --games 30`: all five
  properties held (no violations, panics or slow games; that harness doesn't count draws).
- **Charismatic Conqueror** is the other cause, as the Result says: its game 0 (index 4509 now)
  created 1,499 Vampire tokens on turn 10 in 60 s, with no target choices (the trigger's tap is a
  "may"); the state grows and never repeats.
- **Wastes.** `ManaSymbol` has two colorless forms, `Colorless` and `ColorlessHybrid` (a {C/W}
  can also be paid with W; Wastes are harmless there). {C} in ability text (activation and
  alternative costs, mana a card adds) is caught by the text check, unchanged for those cards;
  the unit test now also has Deepfathom Skulker ("{3}{C}: ..."). No card has a {C/X} hybrid in
  its text. The one {C/P} cost is That Which Was Compleated (Unknown Event, legal nowhere; the
  engine's mana parser rejects {C/P}). The unit test fails on the old Debug-text check.
- **Recipe.** Four chunks of 8,500 from 0 cover 34,000 cards (the last chunk's end is clamped to
  the pool, 32,672 today); CLAUDE.md, `TIER_RECIPE` and `verified.rs` agree, and the CLAUDE.md
  change is limited to the recipe. They now give today's pool size (32,672).
- **Tier refresh at the merged head** (origin f07d4c19 merged): a fresh structure log
  (`cargo test --workspace`, 15 files, 26,582 tests passing) plus one file from a run of the new
  Veil of Assimilation test; `roundtrip --write docs/ROUNDTRIP.md` (11 cards round-trip that the
  passing list didn't have yet, from items landed since: Agonizing Memories, Faerie Artisans,
  Martial Coup, Munda, Ambush Leader, Petals of Insight, Priority Boarding, Stomping Slabs,
  Storybook Ride, Vesuvan Duplimancy, White Sun's Twilight, Zinnia, Valley's Voice); a `--names`
  sweep (`target/every-card-partial-review.tsv`) of the 9 unswept cards plus Hostage Taker and
  Faceless Devourer (22 games, no failures; Glyph of Delusion was never legal to cast: it needs a
  Wall that blocked this turn); `verified --fuzz-from`; `card-coverage --write`.

| Tier | Before the item | Implementer | Review |
|---|---:|---:|---:|
| verified | 17,468 | 17,994 | **18,113** |
| round-trip | 11,085 | 10,565 | 10,457 |
| compiled | 3,507 | 3,517 | 3,506 |
| unsupported | 51 | 35 | 35 |
| fully supported | 32,060 | 32,076 | **32,076** |

  Since the implementer's file: 110 round-trip -> verified (Backdraft and Glyph of Reincarnation
  from the sweep, the rest from the fresh structure log), 8 compiled -> verified and 3 compiled
  -> round-trip (the newly round-tripping cards), Veil of Assimilation round-trip -> verified.
  fuzz-swept 0, fuzz-cast 78, fuzz-clean 1 (Charismatic Conqueror).
- **Veil of Assimilation's demotion** was real: no test names it, and its one structure had been
  exercised only by the random card pools of the property and API tests, which the larger
  verified set changed (as in paper-scope-1). New test
  `tests/cards/structure_random_pools_nov5.rs`. Spot-checked promotions (Abattoir Ghoul,
  Accursed Horde, Aang, A Lot to Learn, Abrade, Accumulated Knowledge, Martial Coup): each
  structure is exercised by a test of that card or of one with the same structure.
