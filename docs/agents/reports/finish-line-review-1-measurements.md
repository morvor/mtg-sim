# finish-line-review-1: raw measurements (Nov 20, 2026, commit e17878d)

The commands behind docs/agents/reports/finish-line-review-1.md, each with its exact output
lines. Run from the worktree root (branch wip/finish-line-review-1 at e17878d, the branch head)
with `CARGO_BUILD_JOBS=2`, one heavy job at a time, on a 4-core, 15 GB machine with nothing
else running. The release tools were built once and their binaries run directly
(`./target/release/mtg-tools X` is what `cargo run --release -p mtg-tools -- X` runs). Reports
that the tools write were written to `target/`; `git status --short` showed no tracked file
changed after any step (`roundtrip --write` rewrites docs/roundtrip-passing.txt: byte-identical).

## Build

`cargo build --release -p mtg-tools -p mtg-sim` (cold, 20:27:25 to 20:35:29 UTC):

```
    Finished `release` profile [optimized + debuginfo] target(s) in 8m 04s
```

## C1: rulings

`python3 scripts/rulings_batches.py summary` (6 s):

```
ruling texts 33955 in 227 batches: CITED 31880 (93.9%), EXEMPT 200, OPEN 11, DEFERRED (digital-only, sticker sheets and their cards, not legal anywhere, dexterity cards, face-down draft cards) 1864; excluded 45 texts found only on Contraptions, host/augment cards, tokens or emblems (662 (card, text) pairs on out-of-scope cards)
rulings: texts=33955 cited=31880 exempt=200 open=11 deferred=1864 excluded=45
```

`python3 scripts/rulings_batches.py list`: batches with open texts: P005 1, P074 1, P076 1, P101 1,
P115 1, P124 1, P142 1, P150 1, P165 1, P190 2 (the other 217 batches: open 0).
`python3 scripts/rulings_batches.py show PNNN --open` for each: the 11 OPEN texts are

| Batch | Card | Text begins | In docs/rulings-uncovered.tsv |
|---|---|---|---|
| P005 | Lore Seeker | "If you use Agent of Acquisitions to draft a booster pack containing Lore Seeker" | yes |
| P074 | Attempted Murder | "In an Un- game, if X is actually infinite" | yes |
| P076 | Tenacious Pup | "A one-time boon is a single-use triggered ability with no source." | yes |
| P101 | Clown Car | "In an Un- game, if X is actually infinite" | yes |
| P115 | Word of Command | "You do get to choose which lands get tapped." | yes |
| P124 | Kathril, Aspect Warper | "If cards in your graveyard have multiple hexproof abilities" | yes |
| P142 | Comet, Stellar Pup | "In some unusual cases, Comet may gain loyalty abilities other than the one he normally has." | yes |
| P150 | Drop Tower | "You can choose Drop Tower’s effect for Grand Marshal Macie’s last ability." | yes |
| P165 | Henzie "Toolbox" Torre | "If the back face of a modal double-faced card is a creature with mana value 4 or greater" | yes |
| P190 | Tolarian Contempt | "Every player gets to know whether a card is going to the top or bottom of any library." | yes |
| P190 | Invoke the Firemind | "As you cast Invoke the Firemind, first you choose the mode" | yes |

docs/rulings-uncovered.tsv has 12 data lines; the 12th, Green Sun's Zenith ("but an opponent
casts it (due to Knowledge Pool's effect, for example)"), is CITED
(`python3 scripts/rulings_batches.py card "Green Sun's Zenith"`: `CITED  P168  If you own Green
Sun's Zenith, but an opponent casts it ...`; the test is
`tests/rulings/r_residue_misc_2b_control.rs`,
`green_suns_zenith_cast_by_an_opponent_searches_their_library_and_shuffles_into_yours`).

`mtg-tools rulings-coverage --check` (exit 0, 1.8 s):

```
Rulings cited: 32852 (card, ruling) pairs; 31880 of 33955 ruling texts in scope cited (93.89%), 200 exempt (0.59%), 11 open, 1864 deferred (every card of the text is deferred: docs/DEFERRED.md); 45 texts excluded (found only on Contraptions, host/augment cards, tokens or emblems); 0 citations and 0 exemptions unmatched
rulings: texts=33955 cited=31880 exempt=200 open=11 deferred=1864 excluded=45
```

The same `rulings:` line as the script. `mtg-tools rulings-coverage --write
target/RULINGS_COVERAGE.md` against the committed docs/RULINGS_COVERAGE.md (9,289 diff lines),
its head:

```
< - (card, ruling) pairs cited by tests: **26775**
< - Ruling texts covered by tests: **26013** (76.61%)
< - Ruling texts exempt: **186** (0.55%)
< - Ruling texts open: **5892**
---
> - (card, ruling) pairs cited by tests: **32852**
> - Ruling texts covered by tests: **31880** (93.89%)
> - Ruling texts exempt: **200** (0.59%)
> - Ruling texts open: **11**
```

Yield (docs/agents/yield.tsv, 93 lines: the header, the column names, 91 rows): the rulings rows
since Nov 10 are 51 items, 6,071 rulings, 98 engine bugs, 24 compiler fixes. Of them, the 39 A2
batches: 4,394 rulings, 68 engine bugs, 13 compiler fixes (11 batches with no engine bug: P155,
P101, P182, P181, P114, P093, P115, P084, P164, P168, P105); the 12 residue items: 1,677
rulings, 30 engine bugs, 11 compiler fixes. The residue rows in order, engine bugs:
copy-creature 4, counters 0, enchant-equip 3, reflexive-refund 3, type-change 1,
hate-attack-block 1, copy-instant 1, keywords 2, burn-freeze 2, shapechange-typal 1, misc-1 5,
misc-2 7. The rulings rows in landing order, engine bugs: P081 3, P153 1, P049 2, P154 3, P100 5,
P155 0, P158 1, P144 2, P196 3, P101 0, P138 4, P157 5, P182 0, P090 2, P175 2, P197 1, P200 1,
P139 2, P181 0, P135 1, P114 0, P093 0, P115 0, P095 1, P084 0, P083 4, P086 5, P164 0, P167 3,
P168 0, P170 1, P173 4, P103 1, P105 0, P106 1, P191 2, P149 3, P088 3, P078 2, then the twelve
residue items above (the longest run without an engine bug: P114, P093, P115). All kinds since
Nov 10: 90 rows, 212 engine bugs.

## C2: round trip

`mtg-tools roundtrip --list --limit 100000 --check --write target/ROUNDTRIP.md` (1m28s):

```
Round trip: 28618 of 32076 fully supported cards pass (89.2%); 3306 mismatch; 152 with hand-written abilities not checked
...
all cards in docs/roundtrip-passing.txt still pass
wrote target/ROUNDTRIP.md and docs/roundtrip-passing.txt
```

`diff docs/ROUNDTRIP.md target/ROUNDTRIP.md`: no difference; `git status`: docs/roundtrip-passing.txt
unchanged. The list holds 3,306 `== <name> — MISMATCH` lines.

Untriaged, by a scratch script (not in the repo): take the 3,306 names; take every
`docs/agents/items/roundtrip-*.md` section from a `## Remaining` heading to the next `## ` heading
or the end, with whitespace collapsed (the files are hard-wrapped); a card is named when its full
name, or a double-faced card's front face, appears with no letter, digit, apostrophe or hyphen on
either side; the lenient count also accepts the part of a name before its first comma (item notes
call Kalitas, Bloodchief of Ghet "Kalitas").

```
[strict] mismatching 3306; named in a Remaining section 429; untriaged 2877
[lenient] mismatching 3306; named in a Remaining section 507; untriaged 2799
```

Named, per file (strict): clusters-3 19, clusters-4 7, gaps-1 2, gaps-2 23, gaps-3 9, gaps-4 30,
gaps-5 52, pairs-1 30, pairs-2 41, pairs-3 55, pairs-4 57, pairs-5 86, pairs-6 56, tail-4 9,
tail-5 1, tail-6 6. Round-trip rows in docs/agents/yield.tsv: one (roundtrip-pairs-6, Nov 10:
150 mismatching cards, 1 engine bug, 1 compiler fix, 1 test error, e2157bf).

## C3, C5: from the record

engine-audit-1, the seven yield rows: layers (Nov 10) 66 tests, 1 engine bug, 0 compiler fixes, 1
test error; casting-costs (Nov 10) 64, 6, 0, 6; triggers (Nov 11) 81, 4, 0, 5; zones-lki (Nov 12)
40, 18, 0, 4; replacement (Nov 12) 66, 9, 0, 6; sba-copy (Nov 12) 49, 6, 0, 7; combat (Nov 13) 71,
4, 1, 9. Total 437 tests, 48 engine bugs, 1 compiler fix, 38 test errors.
`ls docs/agents/reports/`: no judge-audit-2.md.

Mutation, docs/MUTATION.md "Second round, Nov 2026": `simultaneous.rs (full run) | 279 | 191 | 75 |
0 | 13 | 71.8% (191/266) | 98.1% (261/266)`; `permissions.rs (full run) | 238 | 153 | 74 | 0 | 11 |
67.4% (153/227) | 81.9% (186/227)`. The running sessions' status lines (`list_sessions`, read at
about 20:45 UTC): "mtg-sim worker: mutation-2, triggers.rs half 1", running, "triggers.rs mutation
re-run at 26/58; new tests catching misses" (updated 20:13 UTC); "mtg-sim worker: mutation-2,
replacement.rs", running, "replacement.rs mutants at 195/536; batch 3 triage launching" (updated
20:28 UTC). Changes since the measured commits (`git diff --stat <commit> HEAD -- <file>`):
simultaneous.rs since e08e631 `466 insertions(+), 70 deletions(-)`; permissions.rs since 43fcf29
`37 insertions(+), 12 deletions(-)`; triggers.rs since 7ee1be9 `282 insertions(+), 51
deletions(-)`; replacement.rs since 3df9139 `292 insertions(+), 33 deletions(-)`.

## C6: fuzz

docs/EVERY_CARD.md, "Sweep for the card tiers (Nov 20)", its "All" row: 32,673 cards, 65,346
games, 141 min, cast or played 32,331 / 32,425 (99.7%), abilities used 28,422 / 29,847 (95.2%),
fully exercised 31,272 (95.7%), failed games 2; and "No panic, hang or rules violation; the two
failed games are Charismatic Conqueror's own (card 4509, both games stopped as slow". Random-deck
results recorded anywhere in docs/ (a grep for `--random-decks` followed by `--games N`, and for
"random-decks ... N games"): none larger than 300 games; the one `--games 10000` is
verified-refresh-final's instruction.

Today's random-deck fuzz (two players, random 60-card decks of every fully supported card, up to
100 turns, the rules checks every 4th decision, a game stopped as slow after 60 s):

`./target/release/mtg-sim --random-decks --games 200` (seed 1; 21:19:16 to 21:21:07 UTC, exit 0):

```
200 games in 110.58s (1.8 games/s)
  player 1: 107 wins (53.5%)
  player 2: 92 wins (46.0%)
  draws: 1
  avg turns: 26.4
```

Then three processes at once, `./target/release/mtg-sim --random-decks --seed S --games 3334` for
S = 2, 3, 4 (21:21:20 to 21:51:44 UTC, each exit 0):

```
--- seed 2
SLOW: game 1148 (seed 2001154) stopped after 60s, on turn 50
  deck 1: 24 Plains, 3 Magma Mine, 3 Paladin en-Vec, 3 Together Forever, 3 Rocket-Powered Goblin Glider, 3 Guardian Idol, 3 Infested Fleshcutter, 3 Flickering Ward, 3 Pre-War Formalwear, 3 Charmed Pendant, 3 Relentless X-ATM092, 3 Herald of Dromoka, 3 Elspeth, Sun's Champion
  deck 2: 24 Swamp, 3 Jar of Eyeballs, 3 Mortis Dogs, 3 Conductor of Cacophony, 3 Demonlord of Ashmouth, 3 Visions of Dread, 3 Necrotic Plague, 3 Urza's Bauble, 3 Parallax Dementia, 3 Avarice Amulet, 3 Gluttonous Guest, 3 Phoenix Fleet Airship, 3 Balustrade Spy
  reproduce: mtg-sim --seed 2 --only 1148 --players 2 --random-decks --max-turns 100 --log
    [T50] Phoenix Fleet Airship #832, Phoenix Fleet Airship #834 crew Phoenix Fleet Airship #801
    ...
3334 games in 1823.11s (1.8 games/s)
  player 1: 1614 wins (48.4%)
  player 2: 1713 wins (51.4%)
  draws: 6
  avg turns: 26.8
  slow (stopped): 1 games [1148]
--- seed 3
3334 games in 1712.51s (1.9 games/s)
  player 1: 1669 wins (50.1%)
  player 2: 1653 wins (49.6%)
  draws: 12
  avg turns: 26.8
--- seed 4
3334 games in 1795.44s (1.9 games/s)
  player 1: 1682 wins (50.4%)
  player 2: 1639 wins (49.2%)
  draws: 13
  avg turns: 27.3
```

In all, 10,202 games: no `PANIC`, `HANG` or `RULES` line; 1 slow game; 32 draws.

## C4: properties

`mtg-sim --help` panics: `thread 'main' (2594) panicked at crates/mtg-sim/src/main.rs:154:22:
unknown argument --help`. The property names are `ALL` in crates/mtg-sim/src/properties.rs
(`["seat", "snapshot", "rollback", "determinism", "replay"]`) and `NAMES` in
crates/mtg-sim/src/exhaustive.rs (`["copies", "copies-dynamic", "hidden", "payment", "damage"]`).

`./target/release/mtg-sim --properties --games 200 --jobs 3` (seed 1 by default; 20:57:12 to
21:11:09 UTC, exit 0):

```
SLOW: replay, game 83 (seed 1000086): stopped at the time limit
  reproduce: mtg-sim --properties --property replay --seed 1 --only 83 --players 4 --max-turns 40
837.19s
  determinism: 200 games (438s), 200 held, 456550 decisions; violations [], panics [], slow []
  replay: 200 games (1083s), 199 held, 144437 decisions; violations [], panics [], slow [83]
  rollback: 200 games (252s), 200 held, 236852 decisions, 11906 reversed actions checked (0 keeping library actions); violations [], panics [], slow []
  seat: 200 games (305s), 200 held, 450190 decisions; violations [], panics [], slow []
  snapshot: 200 games (400s), 200 held, 456550 decisions; violations [], panics [], slow []
real	13m57.242s
```

(Game 83 of seed 1 is the replay small-fixes-6 also found slow and re-checked with `--slow 300`:
"game 83 held (718 s, 4,001 requests)".)

Fresh games: `./target/release/mtg-sim --properties --games 300 --seed 21 --jobs 3` (21:52:16 to
22:11:15 UTC, exit 0):

```
1139.77s
  determinism: 300 games (669s), 300 held, 691484 decisions; violations [], panics [], slow []
  replay: 300 games (1238s), 300 held, 225073 decisions; violations [], panics [], slow []
  rollback: 300 games (361s), 300 held, 354987 decisions, 17678 reversed actions checked (0 keeping library actions); violations [], panics [], slow []
  seat: 300 games (464s), 300 held, 681845 decisions; violations [], panics [], slow []
  snapshot: 300 games (651s), 300 held, 691484 decisions; violations [], panics [], slow []
real	18m59.826s
```

Properties 5–8, each `./target/release/mtg-sim --properties <args>`, all exit 0:

```
--property copies --jobs 3
copies: 32673 cards, 182438 copies compared, 718 not made in the fixture; 0 mismatches on 0 cards
81.31s
--property copies-dynamic --stride 1 --jobs 3
copies-dynamic: 32673 cards sampled, 25215 permanents checked, 79934 steps compared, 0 differences
116.18s
--property hidden --games 400 --seed 5 --jobs 3
hidden: 400 games, 400 held (89259 decisions compared with a copy, 12738987 objects changed in the copies), 0 violations
191.06s
--property payment --games 200000 --seed 5 --jobs 3
payment: 200000 cases, 200000 held (73216 payable), 0 violations
30.37s
--property damage --games 20000 --seed 5 --jobs 3
damage: 20000 cases, 20000 held (234851 assignments compared; 285 skipped), 0 violations
26.15s
```

The copies run's 718 "not made" lines, grouped: a Clone, Copy Enchantment, Sculpting Steel or
Vesuva that didn't enter 69; no token copy created 69; a permanent spell copy that didn't
resolve to a permanent 4; a spell that can't be cast in the fixture (costs it can't pay there,
timing, prohibitions) 574; a spell that can't be copied 2. (October's runs: 743 and 745 not
made, over 31,181 and 31,310 cards.)

## C7: hand-written cards

```
$ rg -o 'card: *"[^"]+"' crates/mtg-engine/src/cards | sort -u | wc -l
154
$ rg -o --no-filename 'card: *"[^"]+"' crates/mtg-engine/src/cards | sort -u | wc -l
153
$ rg -o 'card: *"[^"]+"' crates/mtg-engine/src/cards | wc -l
157
$ ls crates/mtg-engine/src/cards/*.rs | wc -l
154
$ ls crates/mtg-engine/tests/cards/m_*.rs | wc -l
153
$ ./target/release/mtg-tools manual-check
156 hand-written abilities, 0 problems
```

Booby Trap, Ertai's Meddling and Swirl the Mists register two blocks each; the 157th match is the
`card: "Trinisphere"` example in mod.rs's doc comment.

## C8: register and log

`mtg-tools interpretations --check` (exit 0): `203 entries (13 provisional), 0 problems`.
docs/INTERPRETATIONS.md: `| interpretation | 137 | 11 |`, `| deviation | 13 | 1 |`, `| policy |
40 | 1 |`. docs/RULING_DISCREPANCIES.md (entries are `### ` headings under the kind sections):
template errors 11, copying errors 3, inconsistent 7, narrower than stated 4, misleading 1,
misattached 13, outdated 40; `- Status:` lines: open for review 77, confirmed 1 (Cleric Class and
Reanimate), decided 1 (Grudge Keeper); "Not yet investigated": 11 bullets.

## C10, C11

`mtg-tools cr-coverage` (0.3 s):

```
CR coverage: 3040 cited, 74 via subrules, 52 exempt, 0 uncovered of 3166 (100.0% covered)
CR examples: 277 covered, 0 exempt, 0 open of 277 (100.0% covered or exempt)
```

The committed docs/CR_COVERAGE.md: `| Cited by tests | 3036 | 95.9% |`, `| Covered via subrules |
78 | 2.5% |`. `mtg-tools cr-coverage --write target/CR_COVERAGE.md` differs from it only there
(`| Cited by tests | 3040 | 96.0% |`, `| Covered via subrules | 74 | 2.3% |`).

`mtg-tools card-coverage --write target/CARD_COVERAGE.md` (53 s):

```
Verified: 19827/32111 (61.7%); round-trip or better: 28728 (89.5%); compiled (fully supported): 32076 (99.9%)
Cards fully supported: 32076/32111 (99.9%); abilities supported: 60150/60188 (99.9%)
Fully supported: 32076 (compiled: 31923, with hand-written abilities: 153)
Deferred, not counted: 48 sticker sheets (see docs/DEFERRED.md)
wrote target/CARD_COVERAGE.md
```

`diff docs/CARD_COVERAGE.md target/CARD_COVERAGE.md`: no difference. Its failed-check table:
round-trip 3348, manual-check 0, structure 9582, fuzz-swept 0, fuzz-cast 75, fuzz-clean 1
(Charismatic Conqueror: `compiled	round-trip; fuzz-clean (slow)`). docs/verified-cards.tsv, parsed:
32,111 rows and 32,111 distinct names; verified 19,827, round-trip 8,901, compiled 3,348,
unsupported 35 (as its header and docs/CARD_COVERAGE.md); failed checks structure 9,582,
round-trip 3,348, fuzz-cast 75, compiled 35, fuzz-clean 1; 8,838 fully supported cards fail only
the structure check, 7,947 of them by exactly one unexercised structure.

## Gate

`timeout 3000 cargo test --workspace` (20:42:09 to 20:56:46 UTC; exit 0):

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7m 29s
real	14m37.096s
```

28 `test result:` lines, all `ok`: 33,890 passed, 0 failed, 6 ignored. Per binary: mtg-api unit 0,
card_tiers 4, exactness 5, hidden_info 22, hidden_information 6, protocol 7, replay 3 (33.6 s),
roundtrip 3, doc-tests 1; mtg-data unit 7, discrepancies 1, interpretations 5, load 3 (1 ignored),
paper 5, doc-tests 1 ignored; mtg-engine unit 158, actions 259, cards 6,668 (64.8 s), cr 2,987
(48.6 s), keywords 1,678 (8.3 s), properties 19 (38.2 s), rulings 22,017 (134.3 s), smoke 10
(31.8 s), doc-tests 4 ignored; mtg-sim unit 15, strict_mode 1; mtg-tools unit 5, rulings_counts 1.
No `warning` lines. Tree counts: `grep -rho '#\[test\]' crates | wc -l` 33,890 (cr 2,987,
keywords 1,678, actions 259, rulings 22,017, cards 6,668, properties 19, smoke 10, engine src
158); citations `cr!(` 33,425, `ruling!(` 33,722, `interp!(` 704, `cr_example!(` 291; the engine's
source 422,608 lines in 1,570 files; registries src/kw 328, src/kwa 32, src/oracle/patterns 781,
src/cards 154.
