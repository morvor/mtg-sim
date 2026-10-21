# Verification plan (Oct 14) and the pause until the weekly usage reset

## Goals (user, Oct 14)

The simulation follows the game rules precisely, we are as sure of that as possible, and new
cards can be added as simply as possible.

## What the user chose

After the Oct 14 review of our approach, the user chose four adjustments to start with, plus a
sampled run of a fifth. All other work continues as before: card support, the Oracle round
trip, structure tests, fuzzing, mutation testing and rulings.

| Pick | What | Item(s) and brief | Worker |
|---|---|---|---|
| A | Only cards that pass every check count as verified, and a strict mode keeps the rest out of simulations by default | `verified-cards` (items/verified-cards.md) | compile |
| B | The Comprehensive Rules' 277 worked "Example:" paragraphs become tests | `cr-examples-1`..`7` (items/cr-examples.md); 2–7 need 1's tooling | rulings-1: 1, 3, 5, 7; rulings-2: 2, 4, 6 |
| D | One register of every rules judgment call (rule text, decision, alternatives, tests) | `interpretation-register` (items/interpretation-register.md) | structure |
| F | Property tests: metamorphic (seat symmetry, snapshots, rollback, replay), then exhaustive (copies of every card, hidden information, mana payment, damage assignment) | `property-tests-1`, `property-tests-2` (items/property-tests-1.md, -2.md) | mutation |
| E (sampled) | Blind audit: an agent that sees only the rules predicts what each of 60 randomly sampled tests should show, and every disagreement is adjudicated | run by the coordinator as a local workflow | coordinator |

Not chosen for now: C (comparing against Forge or XMage), G (a mutation-testing gate before
milestones), H (snapshots of every card's compiled form), I (a one-command intake for new
sets), J (writing long-tail cards as reviewed data), K (prioritizing by how often cards are played).

## Pause (user, Oct 14 ~20:15) and resume (user, Oct 21)

**Resumed Oct 21, 04:10 UTC.** At the user's request, every worker got the queue below. The hourly check-in is back on.

The week's usage is nearly spent. The rules until the user says to resume (after the weekly
reset, Oct 16 ~09:00 UTC):

- No new work is issued. Items already running finish and land; each worker then stops and reports.
- The A, B, D and F items have not started.
- The judge audit (E) was already running. It finished on Oct 14: 58 tests judged, 0 disagreements (reports/judge-audit-1.md).
- The hourly check-in is disabled.

## Queues after the reset

Pace: about 9 concurrent subagents. The card workers take at most 2 items at a time, every
other worker 1. While a local audit workflow is running, the card workers drop to 1 too.
Before starting a worker's queue, relaunch any item a usage limit interrupted, on its existing
worktree.

- **compile:** `verified-cards`, then its roundtrip items (items/roundtrip-followups.md).
- **structure:** `interpretation-register`, then `changeling-cda-only` (record the decision as register entry `changeling-cda-only`), then `cards2-triggers-delayed`.
- **mutation:** `property-tests-1`, then the remaining mutation-testing files (the 15 core files minus those already in docs/MUTATION.md), then `property-tests-2`.
- **rulings-1:** `cr-examples-1`, P104, `cr-examples-3`, P143, `cr-examples-5`, P080, `cr-examples-7`.
- **rulings-2:** P136, `cr-examples-2` (once `cr-examples-1` has landed; until then P212 first), P212, `cr-examples-4`, P094, `cr-examples-6`.
- **Rulings batches after those, in this order:** P056 P163 P126 P145 P176 P194 P141 P193 P142 P046 P111 P064 P185 P063 P147 P186 P089 P006 P052 P098 P151 P053 P031 P039 P183 P066 P074 P118 P152 P180 P010 P156 P096 P015 P134 P174 P097 P004 P075 P102 P127 P013 P054 P007 P132 P072 P150 P038 P123 P005 P091 P128 P198 P034 P124 P012 P001 P008 P025 P199 P140 P027 P003 P195 P110 P073 P159 P131 P178 P192 P179 P026 P051 P169 P024 P087 P022 P077 P181 P048 P014 P103 P168 P043 P011 P167 P081 P138 P101 P149 P036 P173 P144 P083 P200 P061 P084 P002 P095 P106 P059 P115 P158 P018 P049 P040 P157 P105 P182 P175 P170 P055 P135 P060 P196 P029 P197 P033 P139 P019 P078 P041 P090 P093 P164 P071 P047 P191 P099 P088 P100 P042 P086 P067 P069 P114 P154 P068 P153 P155 P070.
- **cards-1:** the unfinished ones of `cards2-filters-conditions` and `cards2-referents-values`, then `cards2-costs-replacements`, `cards2-choices-players`. Then re-check the round-1 texts in round2/pending-recheck.tsv whose items have landed.
- **cards-2:** `cards2-cast-permissions` if unfinished, then `cards2-zones-dig`, `cards2-copies-tokens-counters`, `cards-manual-defs-2`.

Already landed from round 2: `cards2-followups`, `cards2-statics-characteristics`, `cards2-combat`, `cards2-filters-conditions`, `cards2-referents-values` and `changeling-cda-only`. During the pause, rulings P219, P113 and P079 landed, as did mutation results for layers.rs and casting.rs: 14 of the 15 files, 81.8% caught overall, with combat.rs still to run.
Also landed: `roundtrip-tail-5`, `every-card-fuzz-2` and the three gap-residual items.

### Queue changes (Oct 21 evening)

- **structure:** `property-tests-2` moves here from the mutation worker, which is still busy with its survivor re-run. The structure worker landed `cards3-costs-mana-replacements` and is free.
- **cards-2:** `cards3-manual-defs` now that the dexterity cards are deferred, and `cards4-plan` (items/cards4-plan.md), which plans round 4 for the texts the landed round-3 items left unsupported. cards-1 keeps `cards3-followups-choices` and `cards3-game-rules-dice`.
- **mutation:** after the survivor re-run lands, rulings batches P064 and P185.
- **rulings-1:** P126, P176, P141, P142, P111. **rulings-2:** P194, P193, P046. Next in the pool: P063 P147 P186 P089 P006 ...

## The judge audit (E)

**Result (Oct 14).** The judge agreed with all 225 questions from 58 tests. See reports/judge-audit-1.md: method, limits, and two calls for the interpretation register.

**Sample.** 60 tests drawn at random with seed 20261014: 14 from tests/cr, 18 from
tests/rulings, 10 from tests/keywords, 12 from tests/cards and 6 from tests/actions, at commit
9b0af15a.

**Steps for each test:**
1. A describer reads the test and writes the game situation, with its assertions turned into
   questions. The answers are kept apart.
2. A judge answers the questions from the Comprehensive Rules, Oracle text and rulings only. It
   must not read the engine, its tests or docs.
3. An adjudicator with full access classifies each disagreement:
   - equivalent: the two answers say the same thing;
   - test wrong;
   - judge wrong;
   - situation misdescribed;
   - ambiguous.
4. Two more agents try to refute each "test wrong" verdict. A test error counts as confirmed
   only if neither can.

**After it finishes:**
1. Check from the judges' transcripts that they read only the rules data.
2. Write docs/agents/reports/judge-audit-1.md: the method, the agreement rate with a confidence
   interval, each confirmed test error, and the disputed and ambiguous cases.
3. Confirmed test errors become a fix item. Ambiguous cases become entries for the
   interpretation register (D).

If a usage limit interrupts the audit, resume its workflow run after the reset; finished
steps are reused. If the coordinator's machine has lost the run, run it again with the same
seed and sample.

## The user's decisions on the register (Oct 21)

The user reviewed the four provisional entries:
- `combat-tax-scaled-to-nothing`: follow CR 118.5, so a tax that comes to {0} is still a cost.
- `attacking-player-is-an-active-player`: implement 805.10c fully.
- `growing-loops-run-to-the-limit`: keep it; an unendable loop is a draw.
- `conspiracy-draft-needs-three-players`: keep rejecting two-player games.

Item `register-decisions-1` (items/register-decisions-1.md) applies them. It goes to rulings-2, next after cr-examples-2.

Later on Oct 21 the user accepted the three provisional entries added since then:
- `range-combat-trigger-events`;
- `illegal-action-reversal-restores-libraries`;
- `end-of-combat-on-your-next-turn-without-combat`.

The coordinator marks them decided. The 733.1 deviation also gets a scan test over every fully supported card, so a newly supported card that could reach the deviation fails the test.

## Other decisions (Oct 21)

- **Dexterity cards** (Chaos Orb, Falling Star): deferred (docs/DEFERRED.md). `cards3-manual-defs` goes ahead without them. It goes to cards-2, after `cards3-history-conditions`.
- **The verified count:** leave it as it is for now and verify later (see "Later"). The `structure-tests` stream stays stopped.

## Later

- **Loops and loop detection** (user, Oct 21). Once the rules work is settled, discuss how loops are recognized and handled: CR 104.4b and the 732 shortcut rules, recognizing growing loops early instead of at the action limit, and automating loops. A UI would need automated loops.
- **Raising the verified count** (user, Oct 21: leave it as it is for now, verify later). About 10,000 cards pass the round trip and the fuzz sweep but have exactly one structure that no test exercises. Mostly these are structures no other card shares: `structure-tests-2` verified about 3 cards per test. Options for when we pick it up:
  1. Test whole shapes: about 3,000 tests.
  2. Compositional coverage: every AST component and combination, at depth about 1, is exercised by some test. This was the recommendation.

## Resuming (coordinator)

1. The workers are the cloud sessions tagged `mtg-sim-worker`. Check each one. Resume any item a
   usage limit interrupted on its existing worktree, with a fresh implementer or reviewer told to
   continue from `git log`, `git status` and `git diff`.
2. Send each worker its queue from this file.
3. Re-enable the hourly check-in.
