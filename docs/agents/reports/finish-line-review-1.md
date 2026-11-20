# finish-line-review-1: where the project stands against the done criteria (Nov 20, 2026)

Measured on commit e17878d (the branch head, Nov 20 2026) by the item's measuring subagent,
for the criteria C1–C11 of docs/agents/finish-line-plan.md §1.2, with each criterion's own
"how measured" command run on today's code. The release tools were built once (8 min) and
every command ran from the worktree root with `CARGO_BUILD_JOBS=2`, one heavy job at a time.
Nothing in the engine, the tests or the tools was changed; regenerated reports went to
`target/` (and `docs/roundtrip-passing.txt`, which `roundtrip --write` rewrites, came out
byte-identical). Each command and its exact output lines are in
docs/agents/reports/finish-line-review-1-measurements.md.

## Headline

**The project is not done. Of the eleven criteria only C10 (CR coverage) is met, and C11
(verified cards) needs nothing more than reporting until the user decides D3; none of the three
correctness criteria is met. The rulings oracle is exhausted but its bug yield has not fallen;
the round trip, the second judge audit, the new properties, most of the mutation runs and the
hand-written cards are the work left.**

- **Rulings (C1)**: coverage is complete for practical purposes: 31,880 of 33,955 in-scope
  texts cited, 200 exempt, 11 open, and all 11 are listed with a reason in
  docs/rulings-uncovered.tsv. But the stopping signal is the opposite of what "done" needs:
  the last five comparable rulings items found 17 engine bugs, and since Nov 10 the rulings
  items found 98 engine bugs in 6,071 rulings (1.6 per 100, against about 0.7 per 100 in the 23
  batches of Nov 5–8 the plan measured: about 20 in 2,700). With no rulings left to test, this
  signal can't fall any more; the plan (§1.1) says
  that an exhausted oracle with a yield that hasn't fallen calls for another oracle (a
  Forge/XMage comparison, not chosen on Oct 14). That is a decision for the user.
- **Round trip (C2)**: 28,618 of 32,076 pass (89.2%); 3,306 mismatch, of which about 2,800–2,880
  have never been triaged. Only one round-trip item has landed since Nov 9 and the probe
  (roundtrip-probe-1) was never started. This is the largest block of work left (about 21
  sessions).
- **Audit (C3)**: all seven engine-audit-1 areas landed (Nov 10–13: 437 audit tests, 48 engine
  bugs, 1 compiler fix); judge-audit-2 has not started.
- **Properties (C4)**: the eight properties exist and all held on today's code (properties 1–4
  over 500 games, one replay game too slow, as before; properties 5–8 at the sizes the final run
  needs). The six new ones planned in property-tests-3 don't exist yet.
- **Mutation (C5)**: two of the five full runs have landed (simultaneous.rs, permissions.rs), two
  are running in other sessions (triggers.rs, replacement.rs), trigger_timing.rs and the eleven
  fresh samples haven't started.
- **Fuzz (C6)**: the Nov 20 sweep is clean except Charismatic Conqueror's two slow games (a known
  open loop). Today's random-deck fuzz, the first 10,000-game run on record: 10,202 games, 0
  panics, 0 hangs, 0 rules-check violations, 1 slow game (Phoenix Fleet Airship tokens doubling
  every turn).
- **Hand-written cards (C7)**: still 153; compiling them (17–22 sessions) hasn't started.
- **Register and log (C8)**: 203 entries, 13 provisional (11 of them are being decided right now
  by user-decisions-nov20b); 79 discrepancy entries, 77 "open for review", none reported (the
  user postponed that review to the end).
- **Engine gaps (C9)**: engine-gaps.md lists one open gap (the payment planner using a repeatable
  mana ability once), which small-fixes-7 is fixing together with "the next turn's upkeep". Thirteen
  more confirmed gaps, nearly all on supported cards (several verified), are recorded only in
  item notes, in docs/rulings-uncovered.tsv or inside a closed engine-gaps.md entry.
- **Gate**: `cargo test --workspace` passes: 33,890 tests, 0 failed, 6 ignored (28,659 on Nov 9),
  14m37s of wall clock including a cold debug build (7m29s), no compiler warnings.
- Across all kinds of item, docs/agents/yield.tsv records 212 engine bugs fixed in the 90 items
  landed since Nov 10. The engine is still changing fast; the final measurement (C2 of §3,
  verified-refresh-final) is not close.

## The criteria, measured

"Today" gives the command and its result. Sizes are worker sessions (an implementer and a
reviewer pass), as in the plan; "machine" means hours of a whole machine.

| # | Target | Today (command: result) | Gap | Status | Closing item | Size |
|---|---|---|---|---|---|---|
| C1 | OPEN 0 except texts in docs/rulings-uncovered.tsv with their reason; the last 5 comparable items found no engine bug | `python3 scripts/rulings_batches.py summary`: 33,955 texts, CITED 31,880 (93.9%), EXEMPT 200, OPEN 11, DEFERRED 1,864; 45 texts excluded (662 card-text pairs). The 11 OPEN texts are all in docs/rulings-uncovered.tsv (12 lines; the 12th, Green Sun's Zenith's, is CITED now). `mtg-tools rulings-coverage --check`: exit 0, the same `rulings:` line. Yield: the last 5 comparable items (residue items) found 2, 2, 1, 5 and 7 engine bugs | coverage half: none. Yield half: 17 engine bugs in the last 5 comparable items; no rulings are left to run | **partly** (coverage met, yield not met) | a user decision on the stopping signal (§5 below); fix items for the 5 open texts that are engine gaps | decision; 2–4 sessions for the gaps |
| C2 | 0 untriaged mismatches; the last 2 items on cards the probe didn't flag found no parser bug | `mtg-tools roundtrip --list --limit 100000 --check --write target/ROUNDTRIP.md`: 28,618 of 32,076 pass (89.2%), 3,306 mismatch, 152 with hand-written abilities not checked; `--check`: all cards in docs/roundtrip-passing.txt still pass. Untriaged (mismatching cards named in no roundtrip-*.md "Remaining" section, scratch script): 2,877 by full name, 2,799 counting legends' short names. The last 2 round-trip items (roundtrip-pairs-5, -6) each found a parser bug; no probe exists, so "cards the probe didn't flag" can't be read | about 2,800–2,880 cards; the yield signal not met | **not met** | roundtrip-probe-1, then roundtrip-pairs-7… | 2 + about 19 |
| C3 | 7 of 7 audit areas landed with their yield; judge audit 2 over 200 tests, 0 confirmed wrong | yield.tsv: all 7 areas, Nov 10–13: 437 audit tests, 48 engine bugs, 1 compiler fix, 38 test errors; no area's new tests all passed first time (layers came closest: 64 of 65). docs/agents/reports/judge-audit-2.md doesn't exist; no yield row or commit for judge-audit-2 | judge audit 2 | **partly** (audit met) | judge-audit-2 | 1 + the coordinator's workflow |
| C4 | the 8 properties plus property-tests-3's new ones hold on the final code at the recorded sizes | 8 exist (`crates/mtg-sim/src/properties.rs`: seat, snapshot, rollback, determinism, replay; `exhaustive.rs`: copies, copies-dynamic, hidden, payment, damage). None of property-tests-3's six (legal-action exactness, layer order-independence, ability and spell copies, multiplayer kinds, library entitlement, extended payment). Latest recorded runs: 1–4 at 500 games on Nov 20 (small-fixes-6), at 2,000 games in late Oct; 5–8 at scale on Oct 22. Today, `mtg-sim --properties --games 200 --jobs 3` (seed 1, 837 s): seat, snapshot, rollback (11,906 reversed actions checked) and determinism 200 of 200 held; replay 199 held and 1 slow (game 83, slow before and held with a longer limit, small-fixes-6); 0 violations, 0 panics. Then 300 fresh games, `--games 300 --seed 21 --jobs 3` (1,140 s): all five 300 of 300 held (17,678 reversed actions checked), none slow. Properties 5–8 at verified-refresh-final's sizes, fresh seeds: copies over all 32,673 cards (182,438 copies, 718 not made in the fixture, 0 mismatches); copies-dynamic `--stride 1` (25,215 permanents, 79,934 steps, 0 differences); hidden 400 games (`--seed 5`, 0 violations); payment 200,000 boards (`--seed 5`, 73,216 payable, 0 violations); damage 20,000 combats (`--seed 5`, 234,851 assignments, 0 violations) | the 6 new properties; full-size runs on the final code | **partly** | property-tests-3, then verified-refresh-final | 4–5, then the final run |
| C5 | every file ≥ 90% on a fresh sample, every survivor classified; full runs of simultaneous.rs, triggers.rs with trigger_timing.rs, permissions.rs, replacement.rs | docs/MUTATION.md "Second round": simultaneous.rs full run (Nov 19) 71.8% → 98.1%; permissions.rs full run (Nov 20) 67.4% → 81.9%, its 41 survivors classified equivalent. Running (other sessions, by their status at 20:13 and 20:28 UTC): triggers.rs half 1 ("re-run at 26/58"), replacement.rs ("mutants at 195/536"). Not started: trigger_timing.rs full run; fresh samples of sba, turn, mana, eval, stack, resolve, actions, layers, casting, combat and range.rs. Audit-time samples since Nov 10 (on code since changed): layers 88.7%, casting 89.4%, triggers 82.4% (17 of 55 reached), zones 84.6%, replacement 89.8%, sba 100%, copy 85.7%, combat 95.7% (49 of 100 run) | 3 full runs, 11 samples; permissions.rs below 90% (its survivors are all equivalent, so it can't reach 90%) | **not met** | mutation-2 (the rest) | 4–6 machine-heavy sessions (6–12 h a file) |
| C6 | a final sweep with 0 failures written up in docs/EVERY_CARD.md; 10,000 random-deck games with 0 panics, recorded there | sweep-nov20 (engine at cbf0113, Nov 20): 32,673 cards, 65,346 games, 141 min, 0 panics, hangs or rules violations, 2 slow games (Charismatic Conqueror); written up in docs/EVERY_CARD.md "Sweep for the card tiers (Nov 20)". No document records a 10,000-game random-deck run (the largest recorded is 300 games). Today, `mtg-sim --random-decks --games 200` (111 s), then `--random-decks --seed S --games 3334` for S = 2, 3, 4 in three processes at once (30 min): 10,202 two-player games, 0 panics, 0 hangs, 0 rules-check violations (checked every 4th decision), 1 slow game (seed 2, game 1148), 32 draws | the final sweep and the 10,000 games on the final code; the Conqueror loop | **partly** | verified-refresh-final (the loop: the post-finish loops item, D4) | 1 + about 4 h machine |
| C7 | no card depends on a `ManualAbility` | `rg -o 'card: *"[^"]+"' crates/mtg-engine/src/cards \| sort -u \| wc -l`: 154 (rg prefixes file names, so mod.rs's doc example counts; with `--no-filename`: 153 cards); 156 registrations (`mtg-tools manual-check`: "156 hand-written abilities, 0 problems"); 153 `tests/cards/m_*.rs`. No compile-manual item has started | 153 cards | **not met** | compile-manual-* (6 items, 3 waves; docs/agents/compile-manual-plan.md) | 17–22 |
| C8 | 0 provisional; every behavioral difference in an item's "Remaining" section registered or fixed; the user has read the deviations; the log's statuses set by the user | `mtg-tools interpretations --check`: "203 entries (13 provisional), 0 problems"; user-decisions-nov20b (running) decides 11 of the 13. docs/RULING_DISCREPANCIES.md: 79 entries, 77 open for review, 1 decided, 1 confirmed, 0 reported. The last sweep of "Remaining" sections was register-sweep-1 (Nov 10); since then the audits' remainders (Uphill Battle, Thorn Elemental, CR 510.4, ...) are in neither the register nor engine-gaps.md | 13 provisional (2 after nov20b); a sweep of 10 days of items; the user's review (postponed) | **not met** | user-decisions-nov20b; the user's answers; register-sweep-2; the log review at the end (D2) | 1–2 + the user |
| C9 | no confirmed rules gap open for a supported card | docs/agents/engine-gaps.md: one entry marked open (a repeatable mana ability planned at most once per payment: 26 cards listed, 10 of them verified); small-fixes-7 (running) fixes it and the next turn's upkeep (not in engine-gaps.md). Recorded elsewhere, not in engine-gaps.md: 5 engine gaps behind open rulings, the audits' remainders, G32's leftovers (list below) | 15 confirmed gaps (listed below; the planner's and the next turn's upkeep included), nearly all on supported cards | **not met** | small-fixes-7; a gaps-inventory item (engine-gaps.md updated from the items) and fixes | 1 + 3–5 |
| C10 | every rule cited, covered via subrules or exempt; every example covered | `mtg-tools cr-coverage`: 3,040 cited, 74 via subrules, 52 exempt, 0 uncovered of 3,166; examples 277 covered, 0 exempt, 0 open of 277 | none | **met** (a regression gate) | none (docs/CR_COVERAGE.md is stale: 3,036 + 78; regenerated by verified-refresh-final) | 0 |
| C11 | report it (D3) | `mtg-tools card-coverage --write target/CARD_COVERAGE.md`: verified 19,827 of 32,111 (61.7%); round-trip or better 28,728 (89.5%); compiled 32,076 (99.9%; compiled 31,923, hand-written 153); unsupported 35; failed checks: round-trip 3,348, structure 9,582, fuzz-cast 75, fuzz-clean 1. Regenerated report identical to the committed docs/CARD_COVERAGE.md. Tier file: 32,111 rows, the same counts; 8,838 cards fail only the structure check (7,947 by exactly one structure) | report only | **reported** (D3 undecided) | none until the user decides D3 | 0 |
| gate | `cargo test --workspace` passes | `timeout 3000 cargo test --workspace`: exit 0; 33,890 passed, 0 failed, 6 ignored in 27 test binaries and the doc-tests (rulings 22,017 in 134 s, cards 6,668 in 65 s, cr 2,987 in 49 s, keywords 1,678, actions 259, properties 19 in 38 s, engine unit 158, smoke 10, mtg-api 51 with its doc-test, mtg-data 21, mtg-sim 16, mtg-tools 6); 14m37s wall with a cold debug build (7m29s); no warnings. The passed count equals the tree's 33,890 `#[test]` attributes | none | **met** | none | 0 |

### Notes per criterion

**C1.** The 11 open texts and their reasons (docs/rulings-uncovered.tsv): five deferred or
unplayable by scope (Tenacious Pup's boon, digital-only; Clown Car's and Attempted Murder's
infinite X, Drop Tower's Grand Marshal Macie, Lore Seeker's Agent of Acquisitions: acorn or
deferred draft cards), one that no supported card can set up (Comet, Stellar Pup), and five
**engine gaps on supported cards**: Word of Command (verified; no choice of lands to tap), Kathril,
Aspect Warper (compiled; no keyword counters for hexproof variants), Henzie "Toolbox" Torre
(round-trip; a modal double-faced card's back face can't be cast with blitz), Invoke the
Firemind (verified; X is announced before the modes, the provisional deviation
`x-announced-before-modes`), Tolarian Contempt (round-trip; other players aren't told top or
bottom). The file also lists Green Sun's Zenith, whose text rulings-residue-misc-2 has since
cited (`tests/rulings/r_residue_misc_2b_control.rs`); the line is stale, and its reason still
holds: `Event::Shuffled` records only whose library was shuffled, so the ruling's "You won't
shuffle any library in this case" isn't something the test can check (the test asserts one
shuffle of the owner's library). Worth a look by the reviewer of that citation.

Yield, per §1.1's rules (only residue items and batches whose report says most rulings were
interactions count; a bug counts whatever found it): the residue items (A3) are the comparable
ones. In landing order the last five are rulings-residue-keywords (2 engine bugs),
-burn-freeze (2), -shapechange-typal (1), -misc-1 (5) and -misc-2 (7): **17**. Every residue
item but rulings-residue-counters found at least one; the twelve found 30 in 1,677 rulings (1.8
per 100). Counting every rulings item since Nov 10: 51 items, 6,071 rulings, 98 engine bugs and
24 compiler fixes (1.6 bugs per 100 rulings); 11 found none, all of them A2 batches (P155,
P101, P182, P181, P114, P093, P115, P084, P164, P168, P105); the longest run of rulings items
without an engine bug, in landing order, was three (P114, P093, P115, Nov 18–19), and the
batch after them (P095) found one.

**C2.** The probe's absence matters twice: the plan reads the round-trip signal on "cards the
probe did not flag", and without the probe the next items have no way to take the
likely-behavioral mismatches first. The last two items: roundtrip-pairs-5 (The Deck of Many
Things: "its owner" read as target 0's owner) and roundtrip-pairs-6 (Rakdos, the Showstopper's
destroys one flip at a time, a compiler fix; Counterbalance revealing from an empty library, an
engine bug). The committed docs/ROUNDTRIP.md is current (the regenerated one is identical).

**C3.** The audit's reports are its yield rows (docs/agents/yield.tsv) and, for four areas, a
section in docs/MUTATION.md; the per-area "Remaining" lists exist only in the yield rows (see
C8 and C9). Judge-audit-2's prerequisite (A1 landed) is met.

**C4.** `mtg-sim --help` doesn't exist (the binary panics on the unknown argument), so the list
comes from the code: `--property` takes seat, snapshot, rollback, determinism, replay or all
(`properties.rs`, `ALL`) and copies, copies-dynamic, hidden, payment or damage (`exhaustive.rs`,
`NAMES`), the eight properties of docs/PROPERTIES.md (replay is property 4's second half). Of
property-tests-3's six, the nearest thing that exists is `crates/mtg-api/tests/exactness.rs`
(the protocol's priority options are exact: every listed action can be taken and no other can,
CR 117.1, in four fixed situations and three random 8-turn games): a test, not property 9's
fuzz-wide check of the engine's own legal-action list.

How far today's evidence is from the target (verified-refresh-final's sizes, on the final code):
properties 5–8 ran today at those sizes and held; properties 1–4 ran at 500 games each against
2,000 (500 on Nov 20 by small-fixes-6, and 2,000 in late October on code about 2,000 commits
older); the six property-tests-3 properties have no runs at all (their stated sizes: property 9
inside a 4,000-card every-card chunk and 500 fuzz games, 10 over the same, 11 over every card,
12 at 300 games a variant, 13 at 400 games, 14 at 200,000 boards). Today's 200 games of seed 1
are the first 200 of small-fixes-6's 500 on nearly the same code, so they mostly re-confirm
that run; the 300 of seed 21 are new games.

**C5.** Per file, the latest numbers (kill rate after the new tests): simultaneous.rs 98.1% (full,
Nov 19), permissions.rs 81.9% (full, Nov 20), sba.rs 100% (17 sampled, Nov 12), combat.rs 95.7%
(49 of 100 sampled run, Nov 13), replacement.rs 89.8% (50 sampled, Nov 12), casting.rs 89.4%
(105 sampled, Nov 10), layers.rs 88.7% (53 sampled, Nov 10), copy.rs 85.7% (7 sampled, Nov 12),
zones.rs 84.6% (29 sampled, Nov 12), triggers.rs 82.4% (17 of 55 sampled reached, Nov 11);
only October samples: turn.rs 95.7%, eval.rs 100%, stack.rs 94.4%, trigger_timing.rs 93.8%,
resolve.rs 86.7%, actions.rs 85.7%, mana.rs 81.0%; never sampled: multiplayer/range.rs. The
sba.rs, copy.rs, replacement.rs and combat.rs samples are recorded only in their audits' yield
rows, not in docs/MUTATION.md. Unclassified survivors: whatever the two running full runs find;
triggers.rs's second sample left 38 of 55 mutants untested, combat.rs's left 51 of 100 (its
"rest run" was in progress when the area landed and no result is recorded). Code measured has
moved since: simultaneous.rs +466/−70 lines since the full run's commit, permissions.rs +37/−12,
triggers.rs +282/−51 since its Nov 11 sample, replacement.rs +292/−33 since its Nov 12 sample.
As worded, the target can't be met by permissions.rs: its 41 equivalent survivors count as
missed, which caps it at 81.9% (186 of 227), though every non-equivalent mutant is killed.
Counting the kill rate over non-equivalent mutants would make the target reachable (a wording
fix for the coordinator; mutation-2's brief has the same 90%).

**C6.** The sweep's numbers (docs/EVERY_CARD.md): cast or played 32,331 of 32,425 (99.7%),
abilities used 28,422 of 29,847 (95.2%), fully exercised 31,272 (95.7%);
game kinds 32,081 standard, 344 commander, 108 planechase, 69 archenemy, 30 vanguard, 22
attractions, 18 conspiracy, 1 dungeon. Its two failed games are Charismatic Conqueror's (card
4509), a loop that grows the game state, which the loop rules don't end (register
`growing-loops-run-to-the-limit`; loops are post-finish work, D4): the criterion's "0 failures"
can't be met on the final code unless that card's games are excused or the loops work comes
first. The random-deck fuzz: 10,202 games today (200 at seed 1, 3,334 each at seeds 2, 3 and 4;
1.8–1.9 games a second a process; average 26.4–27.3 turns), with no panic, hang or rules-check
violation. The one slow game (`mtg-sim --seed 2 --only 1148 --players 2 --random-decks
--max-turns 100 --log`) was stopped at the 60-second limit on turn 50 with a board of Phoenix
Fleet Airship tokens ("At the beginning of your end step, if you sacrificed a permanent this
turn, create a token that's a copy of this Vehicle.": every copy copies itself, so the tokens
double each turn a permanent is sacrificed): a game state that legitimately grows, like
Charismatic Conqueror's, not investigated further. It meets the criterion's size on today's
code; the criterion asks for it on the final code, so verified-refresh-final runs it again and
records it in docs/EVERY_CARD.md (no document records today's run but this report).

**C8.** Register by kind: interpretation 148 (11 provisional), deviation 14 (1), policy 41 (1).
The 13 provisional entries: `attack-total-cost-contains-x`, `chosen-permanents-to-destroy-can-be-destroyed`,
`own-permission-first`, `counter-choice-excludes-what-cant-have-them`,
`impossible-choice-options-not-offered`, `for-each-tokens-counters-life-one-event`,
`x-announced-before-modes`, `that-opponent-fixed-as-it-triggers`,
`copied-ability-duration-starts-as-the-copy-is-created`,
`phasing-out-ends-a-waiting-for-as-long-as-duration`,
`duration-about-another-object-ends-while-the-ability-waits`,
`duration-the-resolution-sets-up-is-checked-as-the-effect-begins`,
`looked-at-face-down-permanent-stays-seen`. user-decisions-nov20b (running) marks all but
`x-announced-before-modes` (the user asked for more explanation) and
`that-opponent-fixed-as-it-triggers` (ruling-research-2's four questions to the user) decided.
Deviations: 14 (9 on Nov 9); 8 of them say the user decided them, `x-announced-before-modes` is
provisional, `mill-until-stops-when-nothing-moves` is tied to the loops work the user postponed,
and 4 have no record of the user's review (`default-card-name`, `unbounded-number-ceiling`,
`concede-with-priority`, `division-after-cost-amount`): D5's "read the deviations" isn't
recorded as done.

The discrepancy log by kind: template error 11, copying error 3, inconsistent 7, narrower than
stated 4, misleading 1, misattached 13, outdated 40 (79, against 48 on Nov 9). Statuses: open for
review 77, decided 1 (Grudge Keeper), confirmed 1 (Cleric Class and Reanimate), reported 0. Fifteen
of the open entries mention a decision of the user's on how the engine follows them (Gnoll War
Band, Araumi, Arbiter of Knollridge, Fiery Annihilation, Headliner Scarlett, Chaotic
Transformation, ...); "open for review" there stands for the reporting decision, which the user
postponed (D2). Eleven "not yet investigated" notes sit below the entries (Seraph's moves to a
template-error entry with user-decisions-nov20b).

**C9.** Everything the plan listed on Nov 9 is closed: G32's two unverified bits (verified by
gaps-residual-2), Inquisitor Eisenhorn's "this way" trigger (small-fixes-4), the hashed ability
links (gaps-residual-2). Open and confirmed today, on supported cards unless noted (only the
first has an open entry in engine-gaps.md; item 7's are notes inside G32's closed entry):
1. The payment planner plans a repeatable mana ability without {T} at most once per payment
   (engine-gaps.md, open; small-fixes-7 part 1). The entry lists 26 cards with such an ability:
   10 are **verified** (Ashnod's Altar, Krark-Clan Ironworks, Skirk Prospector, Squandered
   Resources, ...), 13 round-trip, 3 compiled; Inspired Ultimatum dropped from verified because
   no game could cast it.
2. "At the beginning of the next turn's upkeep" compiles like "the next upkeep" (register-sweep-1's
   note, sweep-nov20; small-fixes-7 part 2): the 57 "next turn's upkeep" cards (Heal is verified)
   fire at a second upkeep of the same turn (Paradox Haze). Aang and Avacyn, worded "the next
   upkeep", play right but dropped to compiled: the renderer words both forms "the next turn's
   upkeep".
3. The five engine gaps behind open rulings (C1 note): Word of Command and Invoke the Firemind
   are verified.
4. Green Sun's Zenith's shuffling player (no record of who shuffles; Green Sun's Zenith is verified).
5. From the replacement audit's "Remaining" (yield row of Nov 12): Uphill Battle's "played by your
   opponents" reads the entering controller (wrong under Gather Specimens or Xantcha; Uphill
   Battle is verified); The Mimeoplasm's entering copy ordered as 616.1e, not 616.1c (verified);
   Hardened Scales adding one per player for tribute (verified).
6. From the combat audit's "Remaining" (Nov 13): Thorn Elemental's "as though unblocked" isn't
   offered after the attacked planeswalker left (verified; it would deal 0).
7. G32's leftovers (engine-gaps.md, inside the closed entry): a "for each" part made of several
   actions performed per player (Tempt with Bunnies, compiled), token copies of several objects
   one creation per kind, and Sacred Boon's delayed triggers (one per point of damage; compiled).
   G32's copy-ordering bullet is fixed (`tests/cr/r405_copies_order.rs`,
   `tests/cards/copies_ordered.rs`) but not marked so.

Possible, with no card or realistic case found (the audits' "Remaining"): CR 510.4 for an existing
creature gaining first strike after the first-strike step (combat); state-based actions performed
before the check's zone changes, which could change last known information, and legends with
several names grouped transitively (sba-copy). Finer limitations sit in other items' "Remaining
gaps" lines, for example ruling-research-1's on "for as long as" periods (reflexive, delayed and
state triggers start theirs as they're put on the stack); the gaps-inventory item of the plan
below should sweep them all into engine-gaps.md.

Seat-symmetry items found but not fixed: none open (games 175 and 405 were fixed by
small-fixes-6). Note that "verified" doesn't exclude a card with a known gap: the tier checks
compile, render, exercise and fuzz a card; none compares its play with the rules.

**C10, C11.** See the table. The committed docs/CARD_COVERAGE.md and docs/ROUNDTRIP.md are
current; docs/CR_COVERAGE.md, docs/RULINGS_COVERAGE.md and docs/STRUCTURE_COVERAGE.md are stale
(list under "Discrepancies").

## The item brief's "known recent state", checked

| Claim | Held? |
|---|---|
| Rulings: 31,880 cited, 200 exempt, 11 open listed in docs/rulings-uncovered.tsv | **held** (the file lists 12; the 12th is cited now) |
| sweep-nov20: verified 19,827 of 32,111; 0 panics or hangs; 2 slow Charismatic Conqueror games | **held** (tier file and docs/EVERY_CARD.md) |
| Mutation full runs done for simultaneous.rs and permissions.rs, running for triggers.rs half 1 and replacement.rs | **held** (docs/MUTATION.md; the two sessions' status lines) |
| The G-list closed by gaps-residual-2 except the planner bug and the next turn's upkeep trigger | **partly**: the planner bug is engine-gaps.md's only open entry, but the next turn's upkeep isn't in engine-gaps.md at all (register-sweep-1.md, sweep-nov20.md, small-fixes-7.md), G32 keeps unmarked leftovers, and other confirmed gaps are recorded only in items (C9) |
| The register's provisional count | 13 (11 being decided by user-decisions-nov20b) |
| C3: engine-audit-1's seven areas, judge audit 2 | areas **held** (all landed); judge audit 2 not started |
| C7: 153 hand-written cards | **held** (153 cards, 156 registrations) |
| C4: the properties' sizes | latest at scale before today: 500 games for 1–4 (Nov 20), Oct 22 for 5–8; today's runs above |
| C6: random-deck 10,000 games | never recorded before today; run today: 10,202 games, 0 panics, 1 slow game |
| Running now: two mutation runs, user-decisions-nov20b, decisions-log-1, small-fixes-7 | **held** (session list, about 20:45 UTC) |

## The plan's risks (§2): still current?

| Rank | Risk | What landed against it since Nov 9 | Still current? |
|---|---|---|---|
| 1 | the untriaged round-trip singletons | roundtrip-pairs-6 (150 cards: 1 compiler fix, 1 engine bug); about 2,800–2,880 untriaged now (about 3,100 then); no probe | **yes**, nearly unchanged |
| 2 | costs, X, casting permissions, the legal-action list | the casting-costs audit (6 bugs), the permissions.rs full run (5), small-fixes-2 ({X} costs), -4 (choice costs), -5 (Omniscience), batches P100, P101, P153–P155 | **yes**: the sweep found the planner bug in exactly this area on Nov 20, and the legal-action property (property-tests-3 part 9) doesn't exist |
| 3 | "each player" simultaneity | the simultaneous.rs full run (71.8% → 98.1%; 8 engine bugs, 3 compiler fixes), small-fixes-3 (one event per "for each" instruction) | **reduced**; G32's per-player leftovers remain |
| 4 | triggers: timing, batching, reflexive, delayed | the triggers audit (4 bugs), ruling-conflicts-12, ruling-research-1 and -2; the triggers.rs full run is under way | **partly**: the full runs of triggers.rs and trigger_timing.rs are unfinished |
| 5 | last known information and departed objects | the zones-lki audit (18 bugs), gaps-residual-2's liveness assertions (3 engine bugs) | **reduced** |
| 6 | replacement and prevention | the replacement audit (9 bugs); the replacement.rs full run is under way | **partly**; the audit's own remainders are unrecorded gaps (C9) |
| 7 | copies and links | the sba-copy audit (6 bugs), gaps-residual-2's per-object links, small-fixes-1 and ruling-conflicts-11 (copy order and events) | **partly**: the property for copies of abilities and of spells (property-tests-3 part 11) doesn't exist |
| 8 | layers and the dependency shortcut | the layers audit (1 display-only bug) | **partly**: the order-independence property (part 10) doesn't exist |
| 9 | multiplayer variants the fuzz never plays | the combat audit's Grand Melee boards only | **yes**: `--kind` for the properties fuzz (part 12) doesn't exist |
| 10 | the hand-written cards and the EQUIVALENCES list | register-sweep-1 read the list (1 compiler fix) | **yes** for the cards: judge-audit-2 and compile-manual not started |
| 11 | judgment calls decided by workers | decision-audit-1 (81 entries: 16 fixed, 2 engine problems for the user), nine decision items (eight user-decisions items and ruling-conflicts-8; user-decisions-nov20b running); the register grew from 128 to 203 entries | **yes**, and growing: 13 provisional, 4 deviations without a record of the user's review |
| 12 | the rulings never tested | 93.9% cited, 11 open | **closed as coverage**; but the yield it measured is still high (C1) |
| 13 | loops, hidden information, structure-blocked cards | nothing new for loops (D4 is post-finish); Charismatic Conqueror still slow; the library-entitlement check (part 13) doesn't exist | **yes**, low impact |

## The plan's rows (§3)

| Row | Item | Status | Evidence |
|---|---|---|---|
| A1 | engine-audit-1, seven areas | **done** | yield rows Nov 10–13 (layers, casting-costs, triggers, zones-lki, replacement, sba-copy, combat); the combat area's last commits ff476ba, 16d2eb7 |
| A2 | the never-worked batches | **done** | all 39 batches the row lists have yield rows (Nov 10–19); the row says "40" but lists 39 |
| A3 | the rulings residue | **done** | 12 items, Nov 19–20 (rulings-residue-copy-creature … -misc-2, last 600fbb3) |
| A4 | roundtrip-pairs-6 and successors | **in progress, stalled** | roundtrip-pairs-6 landed Nov 10 (e2157bf); no successor started |
| A5 | roundtrip-probe-1 | **not started** | no hits file, no yield row, no `--probe` option in `mtg-tools roundtrip` |
| B1 | mutation-2 | **in progress** | simultaneous.rs (a5441dc, Nov 19), permissions.rs (cf582a6, Nov 20) landed; triggers.rs half 1 and replacement.rs running in other sessions; trigger_timing.rs and 11 samples not started |
| B2 | property-tests-3 | **not started** | no code, no PROPERTIES.md section, no yield row |
| B3 | judge-audit-2 | **not started** | no report, no yield row |
| B4 | register-sweep-1 | **done** | 3a535f1, Nov 10 |
| B5 | gaps-residual-2 | **done** | e09e450, Nov 20 (part 4 moved to small-fixes-4, landed) |
| C1 | compile-manual-* | **not started** | no item file or commit; the user's condition ("after rulings coverage") is now met, the plan's (after A4) isn't |
| C2 | verified-refresh-final | **not started** | sweep-nov20 (4aba35f) refreshed the tiers in between, not the final measurement |
| C3 | data-refresh | **not started** | the user: no data refresh until the project is settled (D6) |
| C4 | loops (post-finish) | **not started** | post-finish by the user's decision (D4) |

Also running now, outside the plan's rows: small-fixes-7 (the planner bug and the next turn's
upkeep), user-decisions-nov20b (calls 24–39), decisions-log-1 (a generated docs/DECISIONS.md,
which doesn't exist yet), and the two mutation runs.

## The discrepancy log's human review (§4)

79 entries (48 on Nov 9): 77 open for review, 1 decided, 1 confirmed, 0 reported. The user
postponed the review and the report to Wizards until the project is done (D2, Nov 10). The
remaining release-notes read (about 1,100 changed passages, D2's second question) hasn't been
done. Eleven "not yet investigated" notes wait for the coordinator.

## Decisions only the user can make (§5), open today

1. **New: the stopping signal for rulings.** The rulings pool is exhausted (11 open texts, all
   with a reason) while the yield is higher than on Nov 9. Accept "exhausted" as done for C1,
   or add an oracle: the plan names a Forge/XMage comparison, not chosen on Oct 14. (A second
   independent audit pass, such as judge-audit-2 over a larger sample, is the cheaper option
   within the current methods.)
2. **D3** (the verified count: which approach, and whether it is a criterion): undecided.
3. **`x-announced-before-modes`**: the user asked for more explanation (call 37).
4. **`that-opponent-fixed-as-it-triggers`**: ruling-research-2's four questions (fixed as it
   triggers or read on resolution; whether the Mutiny decision was meant generally; the
   initiative; two engine changes to confirm).
5. **D2** (the log's review and the report to Wizards): postponed to the end, still open.
6. **The "Revisit later" list** (§5, user Nov 12–20): buyback and alternative-cost reductions
   (two entries); the gaps in returned-card values; a phased-out source's characteristics; loops
   (range of influence, Helm of Obedience with Wheel of Sun and Moon); counter modifiers on an
   entering permanent (Vorinclex); the combat requirement search budget; "N or more" block
   triggers; the subtype and base power and toughness set on a noncreature. user-decisions-nov20b
   adds two: Ixidron's entering effects (call 24) and the four "for as long as" entries (25–28).
7. **Four deviations with no record of the user's review** (C8 note), as D5 proposed.
8. **D6** (the data refresh) stays "when the project is settled"; the 189 unreleased-set cards
   come with it.
9. **D4 against C6**: with loops left to post-finish work (D4), Charismatic Conqueror's growing
   loop will make the final sweep's games fail as slow again, so C6's "0 failures" needs either
   that card's games excused as a registered known loop or the loops work done first.

## What's left to call the project done

### (a) Work a worker can do, in order

In the order to start them; items 2 to 6 can run side by side (they touch mostly different files),
mutation testing one file at a time with nothing else heavy on its machine, and
verified-refresh-final last, once no engine change is pending. The user's answer on the rulings
stopping signal may add an item (a Forge/XMage comparison).

1. Let the running items land: small-fixes-7, user-decisions-nov20b, decisions-log-1, and the
   triggers.rs and replacement.rs mutation runs.
2. **gaps-inventory** (new, 1 session): bring docs/agents/engine-gaps.md up to date from the items
   since Nov 10 (the C9 list above, G32's leftovers and fixed bullet), then fix them in
   small-fixes items (3–5 sessions), starting with the verified cards (Uphill Battle, The
   Mimeoplasm, Hardened Scales with tribute, Word of Command, Invoke the Firemind, Green Sun's
   Zenith).
3. **register-sweep-2** (1–2 sessions): every behavioral difference named in a "Remaining" or
   "unless noted" passage since Nov 10 gets a register entry or a fix (C8's second clause).
4. **roundtrip-probe-1** (2 sessions), then **roundtrip-pairs-7 …** (about 19 sessions at 150 cards
   each), probe hits first. Two round-trip sessions side by side, as the plan advised.
5. **judge-audit-2** (1 session plus the coordinator's workflow).
6. **property-tests-3** (4–5 sessions).
7. **mutation-2**, the rest (trigger_timing.rs full, the 11 fresh samples, survivor triage; 4–6
   machine-heavy sessions, one at a time).
8. **compile-manual-*** (6 items in 3 waves, 17–22 sessions).
9. **verified-refresh-final** (1 session plus about 4 hours of machine time; also regenerates
   docs/CR_COVERAGE.md, docs/RULINGS_COVERAGE.md, docs/STRUCTURE_COVERAGE.md and fixes
   docs/EVERY_CARD.md's "Latest full sweep" section).

About 55–65 worker sessions in all, plus a few days of machine time for mutation testing.

### (b) Decisions only the user can make

The nine items of the previous section: the rulings stopping signal (and whether to add the
Forge/XMage oracle), D3, the two provisional entries left after user-decisions-nov20b, D2 at the
end, the "Revisit later" list, the four unreviewed deviations, when to refresh the data (D6),
and how the final sweep treats Charismatic Conqueror's loop (D4 against C6).

### (c) Outside the sandbox

- **The web questions** (docs/agents/research/web-questions-nov20.md, 8 questions): judge and
  Wizards sources the sandbox can't open (magic.wizards.com, media.wizards.com,
  apps.magicjudges.org, cardmarket.com, reddit through the search tool), for an agent with a
  browser; Q1 bears on `that-opponent-fixed-as-it-triggers`.
- **The data refresh** (D6): new Scryfall data and possibly a newer CR, by the user; then
  cr-diff-1 and a card-support round for the new sets (plan row C3).
- **The report to Wizards** of the discrepancy log (D2), and the Scryfall legality note (Spatial
  Contortion and two others) if the user wants it reported.

## Discrepancies between committed docs and today's measurements

- **docs/rulings-uncovered.tsv** lists 12 texts; 11 are OPEN. Green Sun's Zenith's text is CITED
  now (rulings-residue-misc-2), though the reason the line gives still holds (C1 note).
- **docs/RULINGS_COVERAGE.md** is stale: 26,013 cited, 186 exempt, 5,892 open (register-sweep-1,
  Nov 10); today 31,880, 200, 11.
- **docs/CR_COVERAGE.md** is stale: 3,036 cited and 78 via subrules (Nov 5); today 3,040 and 74
  (regenerated into target/, those are the only two lines that differ). Its text, which the tool
  writes, still says the exemptions are in `docs/cr-exemptions.tsv`; they are in
  `docs/cr-exemptions/*.tsv`.
- **docs/STRUCTURE_COVERAGE.md** is stale: "31667 of 32159 cards are supported", 17,640
  structure-verified; today 32,076 fully supported and about 22,500 that pass the structure check.
- **docs/EVERY_CARD.md**'s "Latest full sweep" still describes the sweep at fb94ed0 (29,534
  cards); the Nov 20 sweep is written up under "Sweep for the card tiers (Nov 20)".
- **docs/MUTATION.md** lacks the audit-time samples of replacement.rs, sba.rs, copy.rs (Nov 12)
  and combat.rs (Nov 13), which only the audits' yield rows record; triggers.rs's second sample
  (17 of 55) and combat.rs's (49 of 100) were never finished.
- **docs/agents/engine-gaps.md** lacks the next turn's upkeep gap and the other C9 items; G32's
  copy-ordering bullet is fixed but reads as open.
- **docs/agents/finish-line-plan.md**: A2 says 40 batches and lists 39; C7's command prints 154,
  not 153 (file-name prefixes; see the table).
- **docs/PROPERTIES.md**'s "Results" are October's (no date in it later than Oct 28): the November
  runs (small-fixes-6's 500 games on Nov 20, gaps-residual-2's 500 debug-build games, the smaller
  runs items report) are recorded only in item files, and it still gives the copies pool as
  31,181 cards (32,673 today).
- **`mtg-sim --help`** panics ("unknown argument --help"); the properties are listed only in
  docs/PROPERTIES.md and the source.
- **docs/DECISIONS.md**, named in commit 5cbc6d9's subject, doesn't exist yet (decisions-log-1 is
  writing it).
- **docs/agents/finish-line-plan.md** §1.2's "today" column and §4 are Nov 9's (48 log entries,
  128 register entries, OPEN 6,125, ...); this report has today's, for the reviewer to carry over.

## What couldn't be measured here, and why

- **C5 mutation runs**: not run (6–12 hours a file, and two files are being run in other
  sessions); the numbers are docs/MUTATION.md's and the yield rows', with the running sessions'
  progress as their status lines report it.
- **C3 judge audit 2**: nothing to measure (not started).
- **C4 at the target sizes**: properties 1–4 at 2,000 games each would take about 2.2 hours on 3
  cores at today's speed (500 games took 1,977 s); run at 500 games here (200 of seed 1 and 300
  of seed 21). They belong to verified-refresh-final on the final code anyway. The six new
  properties can't be run: they don't exist.
- **C6's every-card sweep**: not re-run (about 2.5 hours; sweep-nov20 ran it today on the engine
  at cbf0113, and since then only small-fixes-6's two commits changed engine code: a self-granted
  cascade only on the stack, a seat-independent entry order and deck shuffles); the committed
  tier file was checked instead (32,111 rows, counts identical to its header and
  docs/CARD_COVERAGE.md).
- **docs/STRUCTURE_COVERAGE.md** wasn't regenerated (it needs a structure-log test run); stale by
  its own numbers.
