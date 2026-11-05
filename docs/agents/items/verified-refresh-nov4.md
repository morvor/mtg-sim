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
10 fuzz-cast "never legal to cast" (Blue/Red Elemental Blast, Breaker of Creation, Coiling
Rebirth, Dark Withering, Devout Decree, Expunge, Noxious Grasp, Wild Ricochet) plus Myr
Superion "offered but never chosen" (the random games never set up a legal target or never
chose the cast), 2 round-trip (Remove Enchantments, Shrouded Lore), 1 fuzz-clean (Hostage Taker).

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
