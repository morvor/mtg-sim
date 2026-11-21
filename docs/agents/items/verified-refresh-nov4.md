# verified-refresh-nov4: regenerate the card tiers and card coverage

Approved by the user (Nov 4). The full regeneration from CLAUDE.md's "Commands" section, run as
the machine's one heavy job, with nothing else heavy alongside.

Before (origin a4103945, docs/verified-cards.tsv from the Oct 21 sweep + paper-scope-1):
15,769 verified; 31,667 fully supported (3,743 compiled + 12,155 round-trip + 15,769 verified);
444 unsupported; 32,111 cards.

1. `rm -rf target/structlog && MTG_STRUCTURE_LOG=$PWD/target/structlog cargo test --workspace`
2. The every-card sweep in three chunks (`--threads 3 --from 0|10100|20200 --count 10100`,
   reports target/every-card-{0,1,2}.tsv), then `cat target/every-card-[012].tsv > target/every-card.tsv`.
3. `mtg-tools verified --logs target/structlog --report target/every-card.tsv --write` and
   `mtg-tools card-coverage --write docs/CARD_COVERAGE.md`.
4. Commit docs/verified-cards.tsv and docs/CARD_COVERAGE.md (and this brief) as
   `verified-refresh-nov4: ...` with before/after verified and fully supported counts.
5. Report the counts and every card the sweep found panicking or hanging, with its repro command.

## Result
Done Nov 5 on a4103945 (no engine changes; regenerated docs/verified-cards.tsv and docs/CARD_COVERAGE.md).

| Tier | Before (Oct 21 + paper-scope-1) | After (this sweep) |
|---|---:|---:|
| verified | 15,769 | **17,468** (+1,699) |
| round-trip | 12,155 | 11,085 |
| compiled | 3,743 | 3,507 |
| unsupported | 444 | 51 |
| fully supported (all but unsupported) | 31,667 | **32,060** (+393) |
| cards | 32,111 | 32,111 |

Per-card moves: 1,366 round-trip -> verified, 300 compiled -> verified, 111 unsupported -> verified,
268 unsupported -> compiled, 14 unsupported -> round-trip, 206 compiled -> round-trip; down:
76 verified -> round-trip, 2 verified -> compiled. The 78 drops: 65 structure (an ability
structure no test exercises now, mostly fingerprints that changed as the compiler changed),
10 fuzz-cast: 9 "never legal to cast" (Blue/Red Elemental Blast, Breaker of Creation, Coiling
Rebirth, Dark Withering, Devout Decree, Expunge, Noxious Grasp, Wild Ricochet) and Myr
Superion "offered but never chosen" (the random games never set up a legal target or never
chose the cast; see the review below), 2 round-trip (Remove Enchantments, Shrouded Lore),
1 fuzz-clean (Hostage Taker).

Failed checks among non-verified cards (a card can fail several): structure 12,240,
round-trip 3,507, fuzz-cast 83, compiled (unsupported) 51, fuzz-clean 2, manual-check 0, fuzz-swept 0.

Sweep failures (no panics, no hangs; two slow games, 60 s cap):
- SLOW: Charismatic Conqueror game 0, stopped after 60 s on turn 19 (the log shows its
  trigger creating Vampire tokens over and over; both decks run 8 copies; it was already
  fuzz-clean (slow) in the previous tier file). Repro: `mtg-sim --every-card --from 4505 --count 1 --game 0 --max-turns 30 --log`
- SLOW: Hostage Taker game 1, stopped after 60 s on turn 21 (verified before; new this sweep).
  Repro: `mtg-sim --every-card --from 13129 --count 1 --game 1 --max-turns 30 --log`

Notes on the run:
- The pool is now 32,656 cards (sweep pool, including cards outside the 32,111 paper-legal
  set), so CLAUDE.md's three chunks (`--from 0|10100|20200 --count 10100`) stop at index 30,300.
  I ran a fourth chunk, `--from 30300 --count 10100` (2,356 cards, ~15 min), and concatenated
  target/every-card-[0123].tsv. fuzz-swept is 0, so every card is in the report. The CLAUDE.md
  recipe needs a fourth chunk (or larger counts) from now on.
- The machine restarted a few minutes into the first attempt at chunk 0 (3 threads). I reran
  all chunks with `--threads 2` under a 12 GB `ulimit -v` and a 2-second memory watchdog.
  Peak sweep RSS was about 0.77 GB and available memory never fell below 15.2 GB, so the sweep
  is unlikely to have caused the restart. With 2 threads each 10,100-card chunk took about 70 min.
- The structure logs come from a complete `cargo test --workspace` run (15 log files, all test
  binaries passed) on the same commit.

## Review (Nov 5)
- **The tier file's cards.** The four chunk reports concatenate byte for byte into
  target/every-card.tsv, whose outcome lines have indices 0-32,655, each once, with 32,656
  distinct names. The tier file lists exactly the 32,111 counted cards (recomputed independently
  from the Scryfall data with the `is_paper` rule: the same names), the same set as before:
  32,060 are in the sweep pool, and the other 51 are exactly the unsupported cards (the pool holds
  only fully supported ones). The pool's other 596 cards are legal in no format (373 normal
  layouts, mostly Un- and playtest cards, 108 planes, 68 schemes, 30 vanguards, ...), which the
  tier file doesn't list.
- **unsupported 444 -> 51** is compiler progress, not a scope or loader change: the old file
  was written Oct 28 (paper-scope-1), before most of cards4-* and all of cards5-* landed. The 51
  are exactly `mtg-tools unsupported --paper --by-card`, no card went down to unsupported, and 14
  sampled cards that left it (Tawnos's Coffin, Ensnared by the Mara, Island Sanctuary,
  Wonderscape Sage, Telekinesis, Turf War, Gaze of Pain, Pox, Autumn Willow, Rasputin
  Dreamweaver, Saruman of Many Colors, Chaotic Transformation, Snowfall, Jaya's Phoenix) all
  compile now, each in the text list of a cards4/cards5 item that landed after Oct 28.
- **"Never legal to cast" is not an engine casting bug.** A `legal_actions` probe offers each of
  the nine exactly when a legal target and the mana exist, and not otherwise. The sweep builds
  both decks in the card's color identity, so a color hoser (Blue/Red Elemental Blast, Devout
  Decree, Noxious Grasp, Dark Withering, Expunge) gets a target only by luck (in 24 games Devout
  Decree was cast once, Blue Elemental Blast never); Coiling Rebirth and Wild Ricochet need a
  creature card in your graveyard with five mana open, or a spell on the stack with {2}{R}{R}
  open, which their two random games never had. Which such cards get cast changes from sweep to
  sweep (the indices, and so the seeds, moved: 28 cards left fuzz-cast, 33 joined it).
  **Breaker of Creation ({6}{C}{C}) is a sweep bug**: `every_card.rs` adds Wastes when the text
  has "{c}" or when `format!("{m:?}")` of a mana cost contains "Colorless", but `ManaCost`'s
  Debug prints the symbols ("{6}{C}{C}"), so the second test never holds and its games had no
  colorless source. 9 of the 16 pool cards with {C} in a mana cost and none in their text were
  never cast (Breaker of Creation, Kozilek, the Great Distortion, Devourer of Destiny, Flayer of
  Loyalties, Zhulodok, Void Gorger, Wastescape Battlemage, Calamity of the Titans, Echoes of
  Eternity, Null Elemental Blast), against 1 of the 14 with "{c}" in their text. Not fixed here
  (it changes the sweep): test the cost's symbols (`ManaSymbol::Colorless`, `ColorlessHybrid`),
  then re-sweep those cards with `--names`.
- **Remove Enchantments, Shrouded Lore** are no behavior regression: cards4-paper-effects
  (0afbd176, Oct 29) replaced their hand-written definitions, which counted as round-tripping,
  with compiled ones. The compiled ASTs are faithful (Shrouded Lore's choice excludes the cards
  already chosen and the last chosen card is returned) and their tests pass, but the renderer
  doesn't yet render "you both own and control" or "can't choose a card already chosen", so both
  are compiled-tier until a round-trip item covers those phrasings.
- **Hostage Taker's slow game is a loop, and an engine bug keeps it from being recognized.** On
  turn 21 the only creatures or artifacts are Hostage Takers, each holding an exiled Hostage
  Taker, so every enters trigger must exile one of them, which returns another, whose trigger
  must exile another, ...: each step follows the rules, the chain can't stop, and CR 104.4b makes
  the game a draw. With a 900 s cap the game ends after about 440 s, still on turn 21 with no
  loop logged, so at the 30,000-decision limit (9,859 trigger resolutions, slower as the objects
  pile up); the 60 s watchdog calls it slow. The detector (`mandatory-loop-detection`,
  `game_end.rs`) misses it because of a bookkeeping mismatch: `check_mandatory_loop` keeps
  `end.loop_mark` in units of `actions_taken - forced_asks`, but `note_decision`, which lets the
  required target choice of a looping trigger not interrupt the stretch, compares `loop_mark`
  with `actions_taken`. Once any decision of the game has had a single possible answer
  (`forced_asks > 0`, as in nearly every real game), a looping trigger with two or more possible
  targets interrupts the stretch every time around. A TestGame probe shows it: the same
  five-Hostage-Taker chain (two targets each time, both continuing it) is a draw after 18
  decisions when no earlier decision was forced, and still running after 9,000 when one earlier
  target choice had a single candidate. Not fixed here (an engine change for an engine item):
  compare `loop_mark` with `actions_taken - forced_asks` in `note_decision`, with a regression
  test that makes a forced choice before such a loop (the existing
  `a_loop_of_targeted_triggered_abilities_is_a_draw` has one candidate each time).
- **Structure drops**, spot-checked (Acidic Sliver, Righteous War, Way of the Healer, Hierophant
  Bio-Titan): each has an ability structure no test exercises now (`structure-coverage --card`;
  Hierophant Bio-Titan's only test checks that it compiles).
- The recipe in CLAUDE.md, `mtg-tools`' `TIER_RECIPE` (CARD_COVERAGE.md) and `verified.rs` still
  says three chunks, which stop at index 30,300 of the 32,656-card pool; a run that follows it
  leaves 2,356 cards out of the report (they fail fuzz-swept). Left for the user to update
  (CLAUDE.md).
