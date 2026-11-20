# finish-line-review-1: where the project stands against the done criteria (Nov 20, 2026)

Measured on commit e17878d (the branch head, Nov 20 2026) by the item's measuring subagent,
for the criteria C1–C11 of docs/agents/finish-line-plan.md §1.2, with each criterion's own
"how measured" command run on today's code. The release tools were built once (8 min) and
every command ran from the worktree root with `CARGO_BUILD_JOBS=2`, one heavy job at a time.
Nothing in the engine, the tests or the tools was changed; regenerated reports went to
`target/` (and `docs/roundtrip-passing.txt`, which `roundtrip --write` rewrites, came out
byte-identical). Each command and its exact output lines are in
docs/agents/reports/finish-line-review-1-measurements.md.

An independent second reviewer then re-ran the criteria's commands on the same code, formed
its own numbers before reading these, and corrected this report where it was wrong (section
"Second reviewer" at the end lists what was re-measured, every disagreement and every
correction).

**After the measurement.** The tables give e17878d's numbers, as measured and verified. Before
this report landed: the user allowlisted the rules web sites (item web-research-1 issued,
9268b42); decisions-log-1 landed docs/DECISIONS.md (d85f2d0); and small-fixes-7 landed (cad6c67),
fixing the payment planner's repeatable mana abilities and "the next turn's upkeep" (C9's first
two gaps), with these effects by the committed files: round trip 28,618 → 28,620 passing (3,304
mismatch; Aang and Archangel Avacyn pass again), tiers after an 88-card partial sweep verified
19,827 → 19,835 (round-trip 8,895, compiled 3,346), register 203 → 205 entries and 13 → 14
provisional (new: `next-turns-upkeep-is-the-next-turns-first`). Its leftovers went to a new item,
small-fixes-8 (f944329): the planner's remaining gaps. The plan below takes these into account.

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
  (roundtrip-probe-1) was never started. With compiling the hand-written cards (17–22
  sessions), this is one of the two largest blocks of work left (about 21 sessions).
- **Audit (C3)**: all seven engine-audit-1 areas landed (Nov 10–13: 437 audit tests, 48 engine
  bugs, 1 compiler fix); judge-audit-2 has not started.
- **Properties (C4)**: the eight properties exist and all held on today's code (properties 1–4
  over 500 games, one replay game too slow, as before; properties 5–8 at the sizes the final run
  needs). The six new ones planned in property-tests-3 don't exist yet.
- **Mutation (C5)**: two of the five full runs have landed (simultaneous.rs, permissions.rs), two
  are running in other sessions (triggers.rs, its first half; replacement.rs), trigger_timing.rs
  and the eleven fresh samples haven't started.
- **Fuzz (C6)**: the Nov 20 sweep is clean except Charismatic Conqueror's two slow games (a known
  open loop). Today's random-deck fuzz, the first 10,000-game run on record: 10,202 games, 0
  panics, 0 hangs, 0 rules-check violations, 1 slow game (Phoenix Fleet Airship tokens doubling
  each turn a permanent is sacrificed).
- **Hand-written cards (C7)**: still 153; compiling them (17–22 sessions) hasn't started.
- **Register and log (C8)**: 203 entries, 13 provisional (11 of them are being decided right now
  by user-decisions-nov20b); 79 discrepancy entries, 77 "open for review", none reported (the
  user postponed that review to the end).
- **Engine gaps (C9)**: engine-gaps.md lists one open gap (the payment planner using a repeatable
  mana ability once), which small-fixes-7 fixed after the measurement, together with "the next
  turn's upkeep" (57 cards, 40 of them verified); the planner's remaining gaps went to
  small-fixes-8. Thirteen more confirmed gaps, nearly all on supported cards (several verified),
  are recorded only in item notes, in docs/rulings-uncovered.tsv or inside a closed
  engine-gaps.md entry.
- **Gate**: `cargo test --workspace` passes: 33,890 tests, 0 failed, 6 ignored (28,659 on Nov 9),
  14m37s of wall clock including a cold debug build (7m29s), no compiler warnings.
- Across all kinds of item, docs/agents/yield.tsv records 212 engine bugs in the 90 items
  landed since Nov 10 (counted as the yield rule counts them, whatever found them: nearly all
  were fixed by the item that found them, a few were left to a later item, such as the planner
  bug sweep-nov20 found, and some of those are counted again by the item that fixed them). The
  engine is still changing fast; the final measurement (C2 of §3, verified-refresh-final) is not
  close.

## The criteria, measured

Sizes are worker sessions (an implementer and a reviewer pass), as in the plan; "machine" means
hours of a whole machine. The commands and their exact outputs are in the appendix
(docs/agents/reports/finish-line-review-1-measurements.md); the notes below the table give the
details behind each row.

| # | Target | Today | Gap | Status | Closing item | Size |
|---|---|---|---|---|---|---|
| C1 | OPEN 0 except texts in docs/rulings-uncovered.tsv with their reason; the last 5 comparable items found no engine bug | 31,880 of 33,955 in-scope ruling texts cited (93.9%), 200 exempt, 11 open, 1,864 deferred; all 11 open texts are in docs/rulings-uncovered.tsv. The last 5 comparable items (the rulings residue) found 2, 2, 1, 5 and 7 engine bugs | coverage: none. Yield: 17 engine bugs in the last 5 comparable items, and no rulings are left to run | **partly** (coverage met, yield not met) | a user decision on the stopping signal (decision 1 below); fix items for the 5 open texts that are engine gaps | decision; 2–4 sessions for the gaps |
| C2 | 0 untriaged mismatches; the last 2 items on cards the probe didn't flag found no parser bug | 28,618 of 32,076 fully supported cards pass (89.2%); 3,306 mismatch; 152 hand-written cards not checked. About 2,880 mismatching cards are named in no item's "Remaining" section. One round-trip item has landed since Nov 9 (it found a parser bug, as did the one before it); the probe doesn't exist | about 2,800–2,880 cards; the yield signal not met, and unreadable without the probe | **not met** | roundtrip-probe-1, then roundtrip-pairs-7… | 2 + about 19 |
| C3 | 7 of 7 audit areas landed with their yield; judge audit 2 over 200 tests, 0 confirmed wrong | all 7 engine-audit-1 areas landed (Nov 10–13): 437 audit tests, 48 engine bugs, 1 compiler fix, 38 test errors. Judge audit 2 hasn't started | judge audit 2 | **partly** (audit met) | judge-audit-2 | 1 + the coordinator's workflow |
| C4 | the 8 properties plus property-tests-3's new ones hold on the final code at the recorded sizes | the 8 properties held on today's code: 1–4 over 500 games (one replay game too slow, as before), 5–8 at the final run's sizes. None of property-tests-3's 6 new properties exists | the 6 new properties; all of them at full size on the final code | **partly** | property-tests-3, then verified-refresh-final | 4–5, then the final run |
| C5 | every file ≥ 90% on a fresh sample, every survivor classified; full runs of simultaneous.rs, triggers.rs with trigger_timing.rs, permissions.rs, replacement.rs | 2 of the 5 full runs landed: simultaneous.rs 71.8% → 98.1% (Nov 19); permissions.rs 67.4% → 81.9%, every other survivor equivalent (Nov 20). triggers.rs (its first half) and replacement.rs are running in other sessions; trigger_timing.rs's full run and the 11 fresh samples haven't started | 3 full runs, 11 samples; as worded, permissions.rs can't reach 90% (its equivalent survivors count as missed) | **not met** | mutation-2 (the rest) | 4–6 machine-heavy sessions (6–12 h a file) |
| C6 | a final sweep with 0 failures written up in docs/EVERY_CARD.md; 10,000 random-deck games with 0 panics, recorded there | the Nov 20 sweep: 32,673 cards, 65,346 games, no panic, hang or rules violation, 2 slow games (Charismatic Conqueror's growing loop), written up in docs/EVERY_CARD.md. Random decks today: 10,202 games, no panic, hang or rules violation, 1 slow game; recorded only in this report | both again on the final code, recorded in docs/EVERY_CARD.md; Charismatic Conqueror's loop against "0 failures" | **partly** | verified-refresh-final (the loop: the post-finish loops item, D4) | 1 + about 4 h machine |
| C7 | no card depends on a `ManualAbility` | 153 cards with hand-written abilities (156 abilities, each tested; `mtg-tools manual-check`: 0 problems). No compile-manual item has started | 153 cards | **not met** | compile-manual-* (6 items, 3 waves; docs/agents/compile-manual-plan.md) | 17–22 |
| C8 | 0 provisional; every behavioral difference in an item's "Remaining" section registered or fixed; the user has read the deviations; the log's statuses set by the user | register: 203 entries, 13 provisional (user-decisions-nov20b, running, settles 11); 14 deviations, 4 with no record of the user's review. Discrepancy log: 79 entries, 77 open for review, 1 decided, 1 confirmed, 0 reported. The "Remaining" sections written since register-sweep-1 (Nov 10) haven't been swept | 2 provisional after nov20b; the sweep; the user's reviews (postponed) | **not met** | user-decisions-nov20b; the user's answers; register-sweep-2; the log review at the end (D2) | 1–2 + the user |
| C9 | no confirmed rules gap open for a supported card | docs/agents/engine-gaps.md: 1 open gap (the payment planner uses a repeatable mana ability once per payment: 26 cards, 10 of them verified). 14 more confirmed gaps are recorded only in items, docs/rulings-uncovered.tsv or a closed entry (list in the notes), among them "the next turn's upkeep" (57 cards, 40 of them verified) | 15 confirmed gaps, nearly all on supported cards | **not met** | small-fixes-7 (landed after the measurement: the planner and the next turn's upkeep); small-fixes-8 (the planner's remaining gaps); a gaps-inventory item and fixes | 1 + 1 + 3–5 |
| C10 | every rule cited, covered via subrules or exempt; every example covered | 3,040 cited, 74 via subrules, 52 exempt, 0 uncovered of 3,166 rules; 277 of 277 examples covered | none | **met** (a regression gate) | none | 0 |
| C11 | report it (D3) | verified 19,827 of 32,111 (61.7%); round-trip or better 28,728 (89.5%); compiled 32,076 (99.9%); 8,838 cards fail only the structure check | report only | **reported** (D3 undecided) | none until the user decides D3 | 0 |
| gate | `cargo test --workspace` passes | passes: 33,890 tests, 0 failed, 6 ignored; no compiler warnings | none | **met** | none | 0 |

### Notes per criterion

**C1.** Measured with `python3 scripts/rulings_batches.py summary` (33,955 texts in scope; 45
more are excluded, found only on Contraptions, host/augment cards, tokens or emblems: 662
card–text pairs) and `mtg-tools rulings-coverage --check` (exit 0, the same `rulings:` line).
The 11 open texts and their reasons (docs/rulings-uncovered.tsv): five deferred or
unplayable by scope (Tenacious Pup's boon, digital-only; Clown Car's and Attempted Murder's
infinite X, Drop Tower's Grand Marshal Macie, Lore Seeker's Agent of Acquisitions: acorn or
deferred draft cards), one that no supported card can set up (Comet, Stellar Pup), and five
**engine gaps on supported cards**: Word of Command (verified; no choice of lands to tap), Kathril,
Aspect Warper (compiled; no keyword counters for hexproof variants), Henzie "Toolbox" Torre
(round-trip; a modal double-faced card's back face can't be cast with blitz), Invoke the
Firemind (verified; X is announced before the modes, the provisional deviation
`x-announced-before-modes`), Tolarian Contempt (round-trip; other players aren't told top or
bottom). The file has 12 lines: it also lists Green Sun's Zenith, whose text rulings-residue-misc-2
has since cited (`tests/rulings/r_residue_misc_2b_control.rs`, Nov 20); the line is stale, and its
reason still holds: `Event::Shuffled` records only whose library was shuffled (events.rs; game.rs
`shuffle_library`), so the ruling's "You won't shuffle any library in this case" isn't something
the test can check. The test asserts one shuffle of the owner's library, which the event records
as the owner shuffling, the very thing shuffle triggers such as Psychogenic Probe read. The text
counts as cited while its last sentence is a known engine gap (C9, item 4); worth a look by
whoever fixes that gap.

Yield, per §1.1's rules (only residue items and batches whose report says most rulings were
interactions count; a bug counts whatever found it): the residue items (A3) are the comparable
ones, and every rulings item after the last A2 batch (P078) is one. In landing order the last
five are rulings-residue-keywords (2 engine bugs), -burn-freeze (2), -shapechange-typal (1),
-misc-1 (5) and -misc-2 (7): **17**. Every residue item but rulings-residue-counters found at
least one; the twelve found 30 in 1,677 rulings (1.8 per 100). Counting every rulings item since
Nov 10: 51 items, 6,071 rulings, 98 engine bugs and 24 compiler fixes (1.6 bugs per 100
rulings); 12 found none: 11 of the 39 A2 batches (P155, P101, P182, P181, P114, P093, P115, P084,
P164, P168, P105) and rulings-residue-counters. The longest run of rulings items without an
engine bug, in landing order, was three (P114, P093, P115, Nov 18–19), and the batch after them
(P095) found one.

**C2.** Measured with `mtg-tools roundtrip --list --limit 100000 --check --write
target/ROUNDTRIP.md`; `--check`: every card in docs/roundtrip-passing.txt still passes, and the
committed docs/ROUNDTRIP.md is current (the regenerated one is identical). Untriaged, by a scratch
script: the mismatching cards named in no `## Remaining` section of docs/agents/items/roundtrip-*.md
(whitespace collapsed, since the files are hard-wrapped; whole names only): 2,877 by full name
or a double-faced card's front face, 2,799 also counting the part of a name before its first
comma (item notes call Kalitas, Bloodchief of Ghet "Kalitas"); the second reviewer's own script
gave 2,878 and 2,805. The probe's absence matters twice: the plan reads the round-trip signal on
"cards the probe did not flag", and without the probe the next items have no way to take the
likely-behavioral mismatches first. The last two items: roundtrip-pairs-5 (The Deck of Many
Things: "its owner" read as target 0's owner) and roundtrip-pairs-6 (Rakdos, the Showstopper's
destroys one flip at a time, a compiler fix; Counterbalance revealing from an empty library, an
engine bug); roundtrip-pairs-6 is the only round-trip row in docs/agents/yield.tsv (Nov 10: 150
mismatching cards, 1 engine bug, 1 compiler fix, 1 test error).

**C3.** Measured from docs/agents/yield.tsv's seven engine-audit-1 rows (layers, casting-costs,
triggers, zones-lki, replacement, sba-copy, combat). docs/agents/reports/judge-audit-2.md doesn't
exist, and judge-audit-2 has no yield row or commit. No area's new tests all passed first time
(§1.1's audit signal): layers came closest, 64 of the implementer's 65 (the 65th failed on a
display-only bug). The audit's reports are its yield rows and, for four areas, a section in
docs/MUTATION.md; the per-area "Remaining" lists exist only in the yield rows (see C8 and C9).
Judge-audit-2's prerequisite (A1 landed) is met.

**C4.** Runs on today's code, each exit 0 (the first reviewer's; the second reviewer's smaller
runs agree, see the last section):

- `mtg-sim --properties --games 200 --jobs 3` (seed 1, 837 s): seat, snapshot, rollback (11,906
  reversed actions checked) and determinism 200 of 200 held; replay 199 held and 1 slow (game
  83, slow before and held with a longer limit, small-fixes-6); 0 violations, 0 panics.
- 300 fresh games, `--games 300 --seed 21 --jobs 3` (1,140 s): all five 300 of 300 held (17,678
  reversed actions checked), none slow.
- Properties 5–8 at verified-refresh-final's sizes, fresh seeds: copies over all 32,673 cards
  (182,438 copies, 718 not made in the fixture, 0 mismatches); copies-dynamic `--stride 1`
  (25,215 permanents, 79,934 steps, 0 differences); hidden 400 games (`--seed 5`, 0 violations);
  payment 200,000 boards (`--seed 5`, 73,216 payable, 0 violations); damage 20,000 combats
  (`--seed 5`, 234,851 assignments, 0 violations).

The latest recorded runs before today: 1–4 at 500 games on Nov 20 (small-fixes-6) and at 2,000
games in late October; 5–8 at scale on Oct 22. `mtg-sim --help` doesn't exist (the binary panics
on the unknown argument), so the list of properties comes from the code: `--property` takes
seat, snapshot, rollback, determinism, replay or all (`properties.rs`, `ALL`) and copies,
copies-dynamic, hidden, payment or damage (`exhaustive.rs`, `NAMES`), the eight properties of
docs/PROPERTIES.md (replay is property 4's second half). Of property-tests-3's six (legal-action
exactness, layer order-independence, ability and spell copies, multiplayer kinds, library
entitlement, extended payment), the nearest thing that exists is `crates/mtg-api/tests/exactness.rs`
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

**C5.** From docs/MUTATION.md's "Second round": simultaneous.rs full run (Nov 19) 71.8% → 98.1%;
permissions.rs full run (Nov 20) 67.4% → 81.9%, its 41 remaining survivors classified
equivalent. Running in other sessions, by their status lines: triggers.rs half 1 ("re-run at
26/58" at 20:13 UTC; 39 of 58 at 22:00 UTC, when the second reviewer looked) and replacement.rs
("mutants at 195/536" at 20:28 and 22:22 UTC). Not started: the second half of triggers.rs (no
session for it), trigger_timing.rs's full run, and the fresh samples of sba, turn, mana, eval,
stack, resolve, actions, layers, casting, combat and range.rs.

Per file, the latest numbers (kill rate after the new tests): simultaneous.rs 98.1% (full,
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

**C6.** The sweep's numbers (docs/EVERY_CARD.md, "Sweep for the card tiers (Nov 20)", engine at
cbf0113): cast or played 32,331 of 32,425 (99.7%), abilities used 28,422 of 29,847 (95.2%), fully
exercised 31,272 (95.7%); game kinds 32,081 standard, 344 commander, 108 planechase, 69
archenemy, 30 vanguard, 22 attractions, 18 conspiracy, 1 dungeon. Its two failed games are
Charismatic Conqueror's (card 4509), a loop that grows the game state, which the loop rules don't
end (register `growing-loops-run-to-the-limit`; loops are post-finish work, D4): the criterion's
"0 failures" can't be met on the final code unless that card's games are excused or the loops
work comes first. No document recorded a random-deck run of more than 300 games before today
(user-decisions-nov12b's yield row: "300 random-deck games clean"). The random-deck fuzz: 10,202
games today (200 at seed 1, 3,334 each at seeds 2, 3 and 4; 1.8–1.9 games a second a process;
average 26.4–27.3 turns; 32 draws), with no panic, hang or rules-check violation (the checks ran
every 4th decision). The one slow game (`mtg-sim --seed 2 --only 1148 --players 2 --random-decks
--max-turns 100 --log`) was stopped at the 60-second limit on turn 50 with a board of Phoenix
Fleet Airship tokens ("At the beginning of your end step, if you sacrificed a permanent this
turn, create a token that's a copy of this Vehicle.": every copy copies itself, so the tokens
double each turn a permanent is sacrificed): a game state that legitimately grows, like
Charismatic Conqueror's, not investigated further. It meets the criterion's size on today's
code; the criterion asks for it on the final code, so verified-refresh-final runs it again and
records it in docs/EVERY_CARD.md (no document records today's run but this report and its
appendix). The second reviewer replayed the slow game and ran 300 more games at another seed
(last section).

**C7.** The plan's command, `rg -o 'card: *"[^"]+"' crates/mtg-engine/src/cards | sort -u | wc -l`,
prints 154: rg prefixes each match with its file name, so the `card: "Trinisphere"` example in
mod.rs's doc comment counts; with `--no-filename` it prints 153. There are 157 matches in all:
Booby Trap, Ertai's Meddling and Swirl the Mists register two blocks each, and the doc example
is one, so 156 registrations, which `mtg-tools manual-check` reports as "156 hand-written
abilities, 0 problems"; 153 `tests/cards/m_*.rs` test them.

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
recorded as done. docs/DECISIONS.md (decisions-log-1, landed after the measurement) sorts the
203 entries by who decided them: 109 the user, 10 the sources, 71 the coordinator or a worker
(68 of them naming no decider), 13 provisional.

The criterion's second clause: the last sweep of the items' "Remaining" sections into the
register was register-sweep-1 (Nov 10); since then the audits' remainders (Uphill Battle, Thorn
Elemental, CR 510.4, ...) are in neither the register nor engine-gaps.md.

The discrepancy log by kind: template error 11, copying error 3, inconsistent 7, narrower than
stated 4, misleading 1, misattached 13, outdated 40 (79, against 48 on Nov 9). Statuses: open for
review 77, decided 1 (Grudge Keeper), confirmed 1 (Cleric Class and Reanimate), reported 0.
Seventeen of the open entries name a decision of the user's on how the engine follows them (Gnoll
War Band, Araumi, Arbiter of Knollridge, Fiery Annihilation, Headliner Scarlett, Chaotic
Transformation, the six battle entries of Nov 19, ...); "open for review" there stands for the
reporting decision, which the user postponed (D2). Eleven "not yet investigated" notes sit below
the entries (Seraph's moves to a template-error entry with user-decisions-nov20b).

**C9.** Everything the plan listed on Nov 9 is closed: G32's two unverified bits (verified by
gaps-residual-2), Inquisitor Eisenhorn's "this way" trigger (small-fixes-4), the hashed ability
links (gaps-residual-2). Open and confirmed today, on supported cards unless noted (only the
first has an open entry in engine-gaps.md; item 7's are notes inside G32's closed entry):
1. The payment planner plans a repeatable mana ability without {T} at most once per payment
   (engine-gaps.md, open; small-fixes-7 part 1). The entry lists 26 cards with such an ability:
   10 are **verified** (Ashnod's Altar, Krark-Clan Ironworks, Skirk Prospector, Squandered
   Resources, ...), 13 round-trip, 3 compiled; Inspired Ultimatum dropped from verified because
   no game could cast it. Fixed after the measurement by small-fixes-7 (cad6c67, the entry now
   closed); what it left (mana costs that tap other permanents or discard or exile from the
   hand, Squandered Resources, a cost's own sacrifices, convoke and improvise, Food Chain) is
   small-fixes-8.
2. "At the beginning of the next turn's upkeep" compiles like "the next upkeep" (register-sweep-1's
   note, sweep-nov20; small-fixes-7 part 2): the 57 cards worded "the next turn's upkeep" (all
   compiled or better, **40 of them verified**, among them Heal, Portent and Urza's Bauble) fire
   at a second upkeep of the same turn (Paradox Haze). Aang and Avacyn, worded "the next
   upkeep", play right but dropped to compiled: the renderer words both forms "the next turn's
   upkeep". Fixed after the measurement by small-fixes-7 (a delayed trigger of its own for the
   next turn's upkeep; register entry `next-turns-upkeep-is-the-next-turns-first`, provisional).
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

**C10, C11.** `mtg-tools cr-coverage`: the committed docs/CR_COVERAGE.md is stale (3,036 cited and
78 via subrules, Nov 5); verified-refresh-final regenerates it. `mtg-tools card-coverage --write
target/CARD_COVERAGE.md` gives a report identical to the committed docs/CARD_COVERAGE.md: the
32,076 fully supported cards are 31,923 compiled and 153 with hand-written abilities; 35
unsupported; failed checks: round-trip 3,348, structure 9,582, fuzz-cast 75, fuzz-clean 1
(Charismatic Conqueror, slow). docs/verified-cards.tsv: 32,111 rows with the same counts
(verified 19,827, round-trip 8,901, compiled 3,348, unsupported 35); 8,838 cards fail only the
structure check, 7,947 of them by exactly one unexercised structure. The committed
docs/CARD_COVERAGE.md and docs/ROUNDTRIP.md are current; docs/CR_COVERAGE.md,
docs/RULINGS_COVERAGE.md and docs/STRUCTURE_COVERAGE.md are stale (list under "Discrepancies").

**Gate.** `timeout 3000 cargo test --workspace`: exit 0; 33,890 passed, 0 failed, 6 ignored in 25
test binaries and 3 doc-test runs (28 `test result:` lines: rulings 22,017 in 134 s, cards 6,668
in 65 s, cr 2,987 in 49 s, keywords 1,678, actions 259, properties 19 in 38 s, engine unit 158,
smoke 10, mtg-api 51 with its doc-test, mtg-data 21, mtg-sim 16, mtg-tools 6); 14m37s wall with a
cold debug build (7m29s); no warnings. The passed count equals the tree's 33,890 `#[test]`
attributes.

## What's left to call the project done

### (a) Work a worker can do, in order

In the order to start them; items 2 to 8 can run side by side (they touch mostly different files),
mutation testing one file at a time with nothing else heavy on its machine, and
verified-refresh-final last, once no engine change is pending. The user's answer on the rulings
stopping signal may add an item (a Forge/XMage comparison).

1. Let the running items land: user-decisions-nov20b, and the triggers.rs and replacement.rs
   mutation runs (decisions-log-1 and small-fixes-7 landed after the measurement, d85f2d0 and
   cad6c67).
2. **web-research-1** (issued after the measurement, 9268b42, 1 session): the user allowlisted the
   rules sites on Nov 20, so a worker can now answer Q1–Q10 of
   docs/agents/research/web-questions-nov20.md; its contradictions and Q1
   (`that-opponent-fixed-as-it-triggers`) come back to the user.
3. **small-fixes-8** (issued after the measurement, f944329, 1 session): the payment planner's
   remaining gaps that small-fixes-7 left (C9 item 1).
4. **gaps-inventory** (new, 1 session): bring docs/agents/engine-gaps.md up to date from the items
   since Nov 10 (the C9 list above, G32's leftovers and fixed bullet), then fix them in
   small-fixes items (3–5 sessions), starting with the verified cards (Uphill Battle, The
   Mimeoplasm, Hardened Scales with tribute, Thorn Elemental, Word of Command, Green Sun's
   Zenith; Invoke the Firemind once the user has answered on `x-announced-before-modes`, since
   its gap is that deviation).
5. **register-sweep-2** (1–2 sessions): every behavioral difference named in a "Remaining" or
   "unless noted" passage since Nov 10 gets a register entry or a fix (C8's second clause).
6. **roundtrip-probe-1** (2 sessions), then **roundtrip-pairs-7 …** (about 19 sessions at 150 cards
   each), probe hits first. Two round-trip sessions side by side, as the plan advised.
7. **judge-audit-2** (1 session plus the coordinator's workflow).
8. **property-tests-3** (4–5 sessions).
9. **mutation-2**, the rest (triggers.rs's second half, trigger_timing.rs full, the 11 fresh
   samples, survivor triage; 4–6 machine-heavy sessions, one at a time).
10. **compile-manual-*** (6 items in 3 waves, 17–22 sessions).
11. **verified-refresh-final** (1 session plus about 4 hours of machine time; also regenerates
    docs/CR_COVERAGE.md, docs/RULINGS_COVERAGE.md, docs/STRUCTURE_COVERAGE.md and fixes
    docs/EVERY_CARD.md's "Latest full sweep" section).

About 55–65 worker sessions in all, plus a few days of machine time for mutation testing.

### (b) Decisions only the user can make

Each is set out under "Decisions only the user can make (§5), open today" below:

1. The rulings stopping signal: accept "exhausted" as done for C1, or add an oracle (the
   Forge/XMage comparison).
2. D3: the verified count's approach, and whether it is a criterion.
3. The provisional entries left after user-decisions-nov20b: `x-announced-before-modes`,
   `that-opponent-fixed-as-it-triggers`, and `next-turns-upkeep-is-the-next-turns-first`
   (added by small-fixes-7 after the measurement).
4. The four deviations with no record of the user's review.
5. The "Revisit later" list, before the project is called done.
6. How the final sweep treats Charismatic Conqueror's loop (D4 against C6).
7. D2 (the log's review and the report to Wizards) at the end, and D6 (when to refresh the data).

### (c) Outside the sandbox

- **What web-research-1 can't reach**: per its brief, www.mtgsalvation.com (a Cloudflare bot
  challenge) and web.archive.org (connection reset) are not usable from the sandbox, nor is any
  site that refuses automated access; those parts of the ten questions go to the user's local
  browser agent.
- **The data refresh** (D6): new Scryfall data and possibly a newer CR, by the user; then
  cr-diff-1 and a card-support round for the new sets (plan row C3).
- **The report to Wizards** of the discrepancy log (D2), and the Scryfall legality note (Spatial
  Contortion and two others) if the user wants it reported.

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
| Running now: two mutation runs, user-decisions-nov20b, decisions-log-1, small-fixes-7 | **held** (session list, about 20:45 UTC; at about 22:30 UTC the same, except that decisions-log-1 had landed, d85f2d0) |

## The plan's risks (§2): still current?

| Rank | Risk | What landed against it since Nov 9 | Still current? |
|---|---|---|---|
| 1 | the untriaged round-trip singletons | roundtrip-pairs-6 (150 cards: 1 compiler fix, 1 engine bug); about 2,800–2,880 untriaged now (about 3,100 then); no probe | **yes**, nearly unchanged |
| 2 | costs, X, casting permissions, the legal-action list | the casting-costs audit (6 bugs), the permissions.rs full run (5), small-fixes-2 ({X} costs), -4 (choice costs), -5 (Omniscience), batches P100, P101, P153–P155 | **yes**: the sweep found the planner bug in exactly this area on Nov 20 (small-fixes-7 fixed it after the measurement and found more, now small-fixes-8), and the legal-action property (property-tests-3 part 9) doesn't exist |
| 3 | "each player" simultaneity | the simultaneous.rs full run (71.8% → 98.1%; 8 engine bugs, 3 compiler fixes), small-fixes-3 (one event per "for each" instruction) | **reduced**; G32's per-player leftovers remain |
| 4 | triggers: timing, batching, reflexive, delayed | the triggers audit (4 bugs), ruling-conflicts-12, ruling-research-1 and -2; the triggers.rs full run is under way | **partly**: the full runs of triggers.rs and trigger_timing.rs are unfinished |
| 5 | last known information and departed objects | the zones-lki audit (18 bugs), gaps-residual-2's liveness assertions (3 engine bugs) | **reduced** |
| 6 | replacement and prevention | the replacement audit (9 bugs); the replacement.rs full run is under way | **partly**; the audit's own remainders are unrecorded gaps (C9) |
| 7 | copies and links | the sba-copy audit (6 bugs), gaps-residual-2's per-object links, small-fixes-1 and ruling-conflicts-11 (copy order and events) | **partly**: the property for copies of abilities and of spells (property-tests-3 part 11) doesn't exist |
| 8 | layers and the dependency shortcut | the layers audit (1 display-only bug) | **partly**: the order-independence property (part 10) doesn't exist |
| 9 | multiplayer variants the fuzz never plays | the combat audit's Grand Melee boards only | **yes**: `--kind` for the properties fuzz (part 12) doesn't exist |
| 10 | the hand-written cards and the EQUIVALENCES list | register-sweep-1 read the list (1 compiler fix) | **yes** for the cards: judge-audit-2 and compile-manual not started |
| 11 | judgment calls decided by workers | decision-audit-1 (81 entries: 16 fixed, 2 engine problems for the user), nine decision items (eight user-decisions items and ruling-conflicts-8; user-decisions-nov20b running); the register grew from 128 to 203 entries, 71 of them decided by the coordinator or a worker (docs/DECISIONS.md) | **yes**, and growing: 13 provisional, 4 deviations without a record of the user's review |
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
| B1 | mutation-2 | **in progress** | simultaneous.rs (a5441dc, Nov 19), permissions.rs (cf582a6, Nov 20) landed; triggers.rs half 1 and replacement.rs running in other sessions; triggers.rs's second half, trigger_timing.rs and 11 samples not started |
| B2 | property-tests-3 | **not started** | no code, no PROPERTIES.md section, no yield row |
| B3 | judge-audit-2 | **not started** | no report, no yield row |
| B4 | register-sweep-1 | **done** | 3a535f1, Nov 10 |
| B5 | gaps-residual-2 | **done** | e09e450, Nov 20 (part 4 moved to small-fixes-4, landed) |
| C1 | compile-manual-* | **not started** | no item file or commit; the user's condition ("after rulings coverage") is now met, the plan's (after A4) isn't |
| C2 | verified-refresh-final | **not started** | sweep-nov20 (4aba35f) refreshed the tiers in between, not the final measurement |
| C3 | data-refresh | **not started** | the user: no data refresh until the project is settled (D6) |
| C4 | loops (post-finish) | **not started** | post-finish by the user's decision (D4) |

Also running at the measurement, outside the plan's rows: small-fixes-7 (the planner bug and the
next turn's upkeep; it landed after the measurement, cad6c67), user-decisions-nov20b (calls
24–39), decisions-log-1 (a generated docs/DECISIONS.md; it landed after the measurement,
d85f2d0), and the two mutation runs. Issued after the measurement: web-research-1 (9268b42) and
small-fixes-8 (f944329).

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
   initiative; two engine changes to confirm); web-research-1's Q1 gathers the sources for it.
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
10. **`next-turns-upkeep-is-the-next-turns-first`** (new after the measurement, small-fixes-7,
    provisional): "the next turn's upkeep" is the first upkeep step of the turn after the one the
    delayed ability was created in, and if that turn has no upkeep step it never triggers.

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
  not 153 (file-name prefixes; see the C7 note).
- **docs/PROPERTIES.md**'s "Results" are October's (no date in it later than Oct 28): the November
  runs (small-fixes-6's 500 games on Nov 20, gaps-residual-2's 500 debug-build games, the smaller
  runs items report) are recorded only in item files, and it still gives the copies pool as
  31,181 cards (32,673 today).
- **`mtg-sim --help`** panics ("unknown argument --help"); the properties are listed only in
  docs/PROPERTIES.md and the source.
- **docs/DECISIONS.md**, named in commit 5cbc6d9's subject, didn't exist at the measured commit;
  decisions-log-1 landed it afterwards (d85f2d0).
- **docs/agents/finish-line-plan.md** §1.2's "today" column and §4 were Nov 9's (48 log entries,
  128 register entries, OPEN 6,125, ...); the second reviewer added a Nov 20 column to §1.2 from
  the numbers it verified.

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
  cascade only on the stack, a seat-independent entry order and deck shuffles, and in its review
  self-granted conditional keywords compiled only where they function); the committed tier file
  was checked instead (32,111 rows, counts identical to its header and docs/CARD_COVERAGE.md).
- **docs/STRUCTURE_COVERAGE.md** wasn't regenerated (it needs a structure-log test run); stale by
  its own numbers.

## Second reviewer

An independent second reviewer re-measured the criteria on the same code (the worktree at
e17878d plus this item's docs commit; release tools up to date), forming its own numbers before
reading the first reviewer's appendix, then compared. Its outputs went to its scratch directory,
not to docs/ (`roundtrip --write` rewrote docs/roundtrip-passing.txt byte-identically, as for the
first reviewer).

**Re-measured, and agreed:**

- **C1**: `rulings_batches.py summary` and `rulings-coverage --check` (exit 0) print the same
  `rulings: texts=33955 cited=31880 exempt=200 open=11 deferred=1864 excluded=45` line. The 11
  OPEN texts, listed batch by batch with `show PNNN --open`, are all in docs/rulings-uncovered.tsv;
  its 12th line, Green Sun's Zenith's, is CITED since rulings-residue-misc-2's 7c59e5f (Nov 20),
  and `Event::Shuffled` does carry only the library's owner (events.rs; game.rs:1570), so the
  stale line's reason still holds. The yield: the 5 rows chosen are the last 5 comparable items
  (every rulings item after P078, the last A2 batch, is a residue item), with 2, 2, 1, 5 and 7
  engine bugs; recomputed from docs/agents/yield.tsv: 51 rulings items since Nov 10, 6,071
  rulings, 98 engine bugs, 24 compiler fixes; the 12 residue items 30 bugs in 1,677 rulings; all
  90 items since Nov 10, 212 engine bugs; the longest zero-bug run P114, P093, P115.
- **C2**: `roundtrip --write` into the scratch directory: 28,618 of 32,076 pass, 3,306 mismatch,
  152 hand-written; identical to the committed docs/ROUNDTRIP.md; `roundtrip --check` passes.
  Untriaged, by its own script: a first count that read the "Remaining" sections line by line gave
  2,913, too high, since names wrap across lines in the hard-wrapped files; with whitespace
  collapsed and whole names only, 2,878 by full name (front face for double-faced cards) and 2,805
  with the part before a comma, against the first reviewer's 2,877 and 2,799 (per-file counts
  equal except roundtrip-pairs-4, 56 against 57). Either way about 2,880. The probe doesn't exist;
  roundtrip-pairs-5 and -6 each record a parser bug.
- **C3**: the seven audit rows give 437 tests, 48 engine bugs, 1 compiler fix, 38 test errors;
  layers' "64 of the implementer's 65 tests passed first time" is in its row; no judge-audit-2
  report, row or commit.
- **C4**: `mtg-sim --properties --games 50 --seed 33 --jobs 3` (177 s): seat, snapshot, rollback
  (3,104 reversed actions checked), determinism and replay 50 of 50 held; no violation, panic or
  slow game. `--property payment --games 50000 --seed 33`: 0 violations (18,371 payable);
  `--property damage --games 5000 --seed 33`: 0 violations (60,075 assignments, 71 skipped). The
  first reviewer's 500 games and its full-size runs of 5–8 were not repeated. property-tests-3
  has no code, section or row; the sizes the C4 note quotes are verified-refresh-final's.
- **C5**: docs/MUTATION.md's second-round table as quoted; the session list at about 22:30 UTC
  shows both runs still going (triggers.rs half 1: re-run at 39 of 58, updated 22:00; replacement.rs:
  195 of 536, updated 22:22) and no session for triggers.rs's second half, trigger_timing.rs or the
  samples; `git diff --shortstat` since the measured commits gives the same +466/−70, +37/−12,
  +282/−51 and +292/−33.
- **C6**: docs/EVERY_CARD.md's Nov 20 write-up as quoted; the largest earlier random-deck record
  is user-decisions-nov12b's 300 games. `mtg-sim --random-decks --games 300 --seed 7` (145 s,
  2.1 games a second): no panic, hang, rules-check violation or slow game, 0 draws, average 27.3
  turns. The first reviewer's slow game, replayed (`--random-decks --seed 2 --only 1148 --players
  2 --log`), was stopped as slow after 60.8 s on turn 50 with Phoenix Fleet Airship copies
  entering, as described. The 10,202-game run itself was not repeated; it is recorded only in the
  appendix.
- **C7**: the plan's command prints 154, `--no-filename` 153; 157 matches; `manual-check`: 156
  abilities, 0 problems; 153 `tests/cards/m_*.rs`.
- **C8**: `interpretations --check`: 203 entries (13 provisional), 0 problems; the 13 as listed;
  user-decisions-nov20b's brief settles all but `x-announced-before-modes` and
  `that-opponent-fixed-as-it-triggers`; the 14 deviations' Decision lines (8 the user's, 1
  provisional, 1 tied to loops, 4 with no record); ruling-research-2's four questions. The log:
  79 `### ` entries by kind as quoted, 77/1/1 statuses, 11 "not yet investigated" notes (plus 7
  "work item ... took up" pointers).
- **C9**: the planner entry's 26 cards (10 verified, 13 round-trip, 3 compiled, by the tier
  file); the tiers of every card the list calls verified (Word of Command, Invoke the Firemind,
  Green Sun's Zenith, Uphill Battle, The Mimeoplasm, Hardened Scales, Thorn Elemental, Heal);
  the replacement and combat audits' "Remaining" texts in their yield rows, and their absence from
  the register and engine-gaps.md; G32's copy-ordering tests exist. Three gaps checked in the
  code: Heal compiles to `DelayedTrigger { BeginningOf { step: Upkeep, whose: Any } }` (the plain
  next upkeep); Uphill Battle's replacement matches `ControlledBy(Opponent)` as the creature
  enters, not who played it; combat.rs offers Thorn Elemental's option only while the attacked
  object exists (`target.is_some()`).
- **C10, C11**: `cr-coverage` as quoted, and the committed docs/CR_COVERAGE.md differs from a
  regenerated one in exactly those two lines; `card-coverage` as quoted, its regenerated report
  identical to docs/CARD_COVERAGE.md; the tier file's counts, 8,838 and 7,947 as quoted.
- **Gate**: not re-run here (scripts/land.sh builds and tests when landing); the tree's
  `#[test]` count is 33,890, and the citation counts of the appendix reproduce.
- **§2–§5**: all 39 A2 batches have yield rows; the cited commits a5441dc, cf582a6, e09e450,
  ff476ba, 16d2eb7, 600fbb3, 4aba35f and cbf0113 exist (3a535f1 and e2157bf predate this shallow
  clone's history and are taken from their yield rows); decision-audit-1's "16 fixed" and two
  engine problems; the deviation and provisional statuses; the engine commits between cbf0113 and
  e17878d are small-fixes-6's two.

**Disagreements, all corrected above:**

1. **C1 note**: "11 found none, all of them A2 batches" (of the 51 rulings items since Nov 10).
   Twelve found none: the 11 A2 batches and rulings-residue-counters, an A3 item (the appendix had
   the A2 count right; the report's sentence over all 51 items didn't).
2. **C9, item 2** (and the headline): "the 57 'next turn's upkeep' cards (Heal is verified)". All
   57 are compiled or better and 40 of them are verified; the gap affects many more verified cards
   than the wording suggested.
3. **C8 note**: "Fifteen of the open entries mention a decision of the user's". Seventeen do, each
   naming the user's decision explicitly (the Nov 5 conflict decisions, the six battle entries of
   Nov 19, the pre-Dominaria policy of Nov 20, ...).
4. **(c)**: "docs/agents/research/web-questions-nov20.md, 8 questions": it has 10 (Q1–Q10), at the
   measured commit as now. Also changed since the measurement: the user allowlisted the rules
   sites and web-research-1 was issued (9268b42), so the web questions moved from (c) to worker
   work (a, item 2); (c) keeps only the sites web-research-1's brief says are still unusable.
5. **Headline**: "212 engine bugs fixed". The yield rows count engine bugs found, whatever found
   them; some were left open by the item that found them (sweep-nov20's planner bug,
   decision-audit-1's two) and counted again when a later item fixed them.
6. **Gate**: "in 27 test binaries and the doc-tests". The appendix's own list has 25 test
   binaries (8 mtg-api, 5 mtg-data, 8 mtg-engine, 2 mtg-sim, 2 mtg-tools) and 3 doc-test runs: 28
   `test result:` lines.

**Other corrections (wording, structure, plan):**

- The criteria table's cells are cut to their numbers (one or two plain sentences each); the
  commands and run details they held moved into the notes, nothing dropped (the exact outputs are
  in the appendix).
- "What's left to call the project done" moved up to follow the criteria, as the item brief
  orders (a table per criterion, then the plan); its (b) list is now self-contained.
- Plan (a): Invoke the Firemind's fix waits for the user's answer on `x-announced-before-modes`
  (its gap is that provisional deviation, so a worker can't fix it first); Thorn Elemental added to
  the verified cards to fix first; triggers.rs's second half named in the mutation work;
  web-research-1 added.
- Headline: the round trip is one of the two largest blocks of work (about 21 sessions), with the
  hand-written cards' 17–22, not "the largest".
- "What couldn't be measured": small-fixes-6's review commit also changed the compiler
  (self-granted conditional keywords compiled only where they function), after the sweep's engine.
- State after the measurement: decisions-log-1 landed docs/DECISIONS.md (d85f2d0), whose counts
  (109 decided by the user, 10 by the sources, 71 by the coordinator or a worker, 13 provisional)
  the C8 note and risk 11 now cite; small-fixes-7 landed while this review was landing (cad6c67)
  and small-fixes-8 was issued (f944329): the intro's "After the measurement" paragraph, C9's
  closing item and notes, the plan (a, b) and §5 now say so, with the committed files' numbers
  (round trip 28,620 passing, verified 19,835, register 205 entries and 14 provisional: read from
  docs/ROUNDTRIP.md, the tier file and docs/INTERPRETATIONS.md, not re-measured). The tables keep
  e17878d's numbers.

docs/agents/finish-line-plan.md §1.2 gained a "Today (Nov 20)" column with the numbers above that
the second reviewer reproduced or checked against their record, the Nov 9 column kept beside it.
