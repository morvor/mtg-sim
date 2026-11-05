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

### Hold for the weekly usage limit (Oct 21, 21:45 UTC)

The workers' rate-limit status turned to a warning on the weekly window, which resets Fri Oct 23 09:00 UTC. As with the user's choice on Oct 14, every worker finishes and lands the items it is running, then stops and reports. Queued items wait until the coordinator sends "resume": after the reset, or sooner if the user says so. Running when the hold began:
- compile: `roundtrip-pairs-1`;
- cards-1: `cards3-triggers-durations` (review) and `cards3-followups-choices`;
- cards-2: `cards3-manual-defs` and `cards4-plan`;
- structure: `property-tests-2`;
- mutation: the review of the survivor re-run;
- rulings-1: P176;
- rulings-2: P194.

Waiting:
- cards-1: `cards3-game-rules-dice`;
- mutation: P064, P185;
- rulings-1: P141, P142, P111;
- rulings-2: P193, P046;
- compile: the rest of its round-trip items.

### The 733.1 entry (user, Oct 21 evening)

The scan test written for `illegal-action-reversal-restores-libraries` found that 16 supported cards do reach 733.1's library exceptions, through replacement effects. Its review and the newer card pool later raised the count to 22; the user's decision covers them all. When the user accepted the entry, the claim was that none could. The user chose to fix it: item `reversal-library-exceptions` (items/reversal-library-exceptions.md). It goes to the structure worker after `property-tests-2`, once the hold ends.

### Resumed Oct 28 (user)

The user's decisions:
- The three provisional entries from the hold are accepted and marked decided: `trample-reassignment-counts-only-assigned-damage`, `copy-values-fixed-when-determined` and `venture-limit-looks-at-whole-turn`.
- The five Conspiracy draft cards are deferred (recorded in items/cards3-game-rules-dice.md, which applies the deferral).
- The tools' paper bug is to be fixed, and the 117 texts it hid are to be worked on. True digital-only cards and digital versions of cards stay out, but every paper card must be counted: items `paper-scope-1` and `cards4-paper-plan`.

Queues (pace about 9 concurrent subagents; card workers at most 2 items, the others 1):
- **compile:** `paper-scope-1`, then its round-trip items (items/roundtrip-followups.md).
- **cards-1:** `cards3-game-rules-dice` (with the Conspiracy deferral) and `cards4-cast-zones`, then `cards4-triggers-durations` and `cards4-manual-defs`.
- **cards-2:** `cards4-paper-plan` and `cards4-referents-statics`, then `cards4-costs-mana-replacements` and the `cards4-paper-*` items the plan creates.
- **structure:** `reversal-library-exceptions`.
- **mutation:** P064, P185.
- **rulings-1:** P141, P142, P111. **rulings-2:** P193, P046.
- Next in the rulings pool: P063 P147 P186 P089 P006 P052 P098 P151 P053 P031 ...

### Round 5 (Oct 29)

`cards5-plan` landed (ccfd9945): docs/agents/card-support-plan-5.md, 4 items, 112 texts, no decisions needed. cards4-triggers-durations and cards4-manual-defs have landed; cards4-paper-effects and cards4-costs-mana-replacements are still in flight with cards-2.

Queues from the 06:42 check-in:
- **cards-1:** `cards5-choices-players` and `cards5-referents-statics`, together because they share the Builder's referent tracking and the optional as-enters choice.
- **structure:** re-place cards4-manual-defs' leftover texts (an update section in plan 5), then `cards5-game-rules-dice`. It also re-places the leftovers of cards-2's round-4 items when they land, and re-probes Mr. House after costs-mana.
- **cards-2:** its two round-4 items, then `cards5-manual-defs` (Celebr-8000, Curse of Leeches and the hooks first).
- **Change at 10:43:** round 4 has fully landed (cards4-costs-mana-replacements, 15183001). cards-2 landed the first part of cards5-manual-defs and was free, so it took over the re-placement of its two round-4 items' leftovers and Mr. House's re-probe from the structure worker. It places them in new `cards5-<area>` items and implements them. River Song, Deification and Vedalken Squirrel-Whacker wait for their cards5 items.
- **Change at 13:04:** cards5-game-rules-dice landed (20 cards, with Mr. House). The re-plan added `cards5-mana-replacements` and `cards5-costs-sets` (cards-2 runs both) and two one-offs to cards5-manual-defs (Lodestone Bauble, North Star). The structure worker, now idle, takes the rest of cards5-manual-defs: Squirrel-Whacker, Lodestone Bauble and North Star now, River Song and Deification when their items land.
- **Hold at 14:20 (weekly usage warning; the window resets Fri Oct 30 09:00 UTC).** cards5-choices-players has landed (8aec96aa). Each worker finishes and lands the item it is on, then stops:
  - compile: roundtrip-pairs-4;
  - cards-1: cards5-referents-statics;
  - cards-2: cards5-mana-replacements and cards5-costs-sets;
  - structure: Squirrel-Whacker, Lodestone Bauble, North Star and Deification;
  - mutation: P010;
  - rulings-1: P156;
  - rulings-2: P066.

  Queued for after the hold: River Song (structure) and P096 (rulings-2); the rulings pool continues at P015. The hourly check-in is disabled during the hold, with one hold check at 18:45 UTC. Workers resume when the user says so.
- **API-billed worker (Nov 1, user).** The user has about $200 of API credits that expire in a few days, and approved spending them on extra rulings batches through the usual pipeline (implementer, reviewer, scripts/land.sh).
  - A launcher session, session_019p6spxpwd25URaDvPYLJ9P, runs each batch as a headless Claude Code child process billed to the key in the environment variable `MTG_SIM_KEY`. The child's environment is cleaned of the platform's ANTHROPIC_*/CLAUDE* variables, and the launcher checks apiKeySource on a trial run.
  - Budget: $35 per run, $190 in total.
  - Batches, in order, until the budget runs out: P015 P134 P174 P097 P004 P075 P102 P127 P013 P054.
  - When the subscription workers resume, the rulings pool continues at P007, plus any of those batches the launcher reports unfinished.
  - **05:40 update.** Six batches landed in 4.5 hours for $57, about $9.50 a batch: P015, P134, P174, P097, P004 and P075. The children run at Claude Code's default effort; the cloud sessions run with CLAUDE_EFFORT=max. The list was extended within the same $190 cap: P007 P132 P072 P150 P038 P123 P005 P091 P128 P198. The subscription pool resumes after the last batch the launcher starts, which it reports in its final message.
  - **Finished (15:34).** $183.31 spent. 16 batches landed, with no unfinished items: P015 P134 P174 P097 P004 P075 P102 P127 P013 P054 P007 P132 P072 P150 P038 P123. Rulings cited went from 17,998 (53.0%) to 19,800 (58.3%). Two provisional register entries were added: qualified-landwalk-is-that-landwalk and trigger-x-before-targets. **The subscription rulings pool resumes at P005:** P005 P091 P128 P198 P034 P124 P012 P001 P008 P025 ..., after rulings-2's queued P096.

### Resumed Nov 4 (user)

The user resumed normal work, at the normal pace.

**Register decisions:**
- **Accepted (25 entries), marked decided "Decided by the user, Nov 4":** every provisional entry except the nine below.
- **To fix (item `register-fixes-1`):** `library-arranged-by-owner-not-instructing-player`, `linked-reveal-trigger-as-reflexive`, `nihiloor-taps-one-opponent-at-a-time`.
- **To investigate (item `ruling-conflicts-1`).** The user: "The CR and rulings should NOT contradict. If they seem to, we need to examine these cases much more closely." This covers:
  - the six entries where the CR or the Oracle text and a ruling seem to disagree: `put-onto-battlefield-blocking-retriggers-becomes-blocked-by`, `history-counts-players-who-left`, `self-shuffle-gone-library-still-shuffled`, `room-door-unlock-timestamp`, `ice-cauldron-last-exiled`, `cyclopean-tomb-land-choice`;
  - the rulings exempted earlier as superseded by the CR: Krosan Adaptation, Jade Statue, Deranged Assistant, Mossfire Egg, Oculus Whelp, Corruption of Towashi;
  - any other case its inventory finds.

  Conclusions go to the user. Until then, these entries stay provisional.

**Queues:**
- **compile:** its round-trip queue (items/roundtrip-followups.md).
- **cards-1:** `cards6-plan`, then the round-6 items it creates.
- **cards-2:** `register-fixes-1`, which includes River Song.
- **structure:** `ruling-conflicts-1`.
- **rulings-2:** P096, P005. **rulings-1:** P091, P128. **mutation:** P198, P034.
- **Next in the rulings pool:** P124 P012 P001 P008 P025 P199 P140 P027 P003 P195 ...

**Verified-count refresh:** approved by the user on Nov 4 evening. The compile worker runs it (`verified-refresh-nov4`) after roundtrip-pairs-5, then does rulings P001 and P008.

### Priorities (user, Nov 4 evening)

- **Deferred categories stay as they are.** The sticker sheets, Contraptions, dexterity cards, digital-only mechanics and the Conspiracy face-down draft cards stay deferred, so round 6 ends at about 99.8% of paper cards.
- **First: resolve every case where the CR seems to contradict a ruling** (`ruling-conflicts-1`, the structure worker). Its conclusions go to the user. Any new case a rulings batch finds goes to the same investigation.
- **Then: maximize rulings coverage.**
  - cards-1 finishes round 6 (the five cards6 items).
  - Every other worker moves to rulings batches when its current item lands: compile after roundtrip-pairs-5; cards-2 after register-fixes-1, two batches at a time; the structure worker after ruling-conflicts-1.
  - The round-trip queue pauses after roundtrip-pairs-5.
- **No new card data for now.** Once everything works properly, the user will update the data stores in data/ themselves, and then new cards get supported. Until then, no Scryfall refresh.
- **Later:** compile every hand-written card, and support new cards after the user's data update ("Later", below).

### Hold (user, Nov 5, 17:45 UTC)

The weekly usage window showed a warning from about 15:40 (reset Fri Nov 6, 09:00 UTC). The user: "Each worker should land its batch then stop, but wait on my go ahead to resume them."

- Each worker lands the item it is on, then stops.
- cards-1 and cards-2 had already finished their queues, so they get nothing new.
- **Don't resume or top up any worker until the user says go**, even after the reset. Check-ins during the hold only confirm the landings and log them.
- On the user's go, resume each worker with its remaining queue: compile P048, P049; rulings-2 P055; rulings-1 after P042; mutation and structure after their current items; cards-1 and cards-2 from the pool (P059 P060 P061 P067 ...).

### Worker roster

The coordinator keeps this current. Every session is a cloud session tagged `mtg-sim-worker`. "Handing off" means the session lands its current item, stops, and is then replaced by a fresh session.

| Role | Session | State (Nov 5, 09:45) | Queue for the next session |
|---|---|---|---|
| compile | session_011rMBWnWVZubEfnTBPyDyne | keeps its session (compacted Nov 4, 23:40) | roundtrip-pairs-5, then verified-refresh-nov4, rulings P001, P008 |
| cards-1 | session_01JuEc757NG5yjUWzpG55Mo4 | round 6 done (cards6-per-player-results 78a67e68) | round-6 result note, then rulings P014+P018 |
| cards-2 | session_01UgktDvDn9k5EGhmSRYhCMB | keeps its session (compacted Nov 4) | register-fixes-1, then P025+P199, then P140+P027 |
| structure | session_019p6sq1QYmQYJv7NuQZDt2A (new Nov 5 08:00; old session archived 09:45 after P073) | register-decisions-nov5, ruling-conflicts-3, P087, P010 |
| mutation | session_01HW8apRLHq3Lrn9uCc5uTbe | keeps its session (compacted Nov 4, 23:40) | P198, then P034, P003 |
| rulings-1 | session_019p6spxfiVy5mFUashMDSDX (new Nov 5 01:44; old session_016AqVwbbcGHLPnsbE5QiWBy archived after P128, P091) | P124, P012 |
| rulings-2 | session_019p6sq1M5uLvZMf7G2RJ9mB (new Nov 5 01:44; old session archived 02:45 after f3ca81bb) | P195, p005-flagged-bugs, P159 |

### Decisions (user, Nov 5, morning)

The user accepted every recommendation from ruling-conflicts-1 and ruling-conflicts-2, and the Grip of Chaos reading:

> "It looks like some rulings are applied as a template, and are therefore occasionally incorrect. This is the case for Homicide Investigator and the similar cases you mentioned. I agree with all of your recommendations. We should still keep a detailed account of all of the instances though, so they can be checked by a human, and reported to Wizards of the Coast if erroneous. For the Grip of Chaos question, I believe you are correct in that it does not trigger ward and 'whenever this becomes the target' abilities again."

Item `register-decisions-nov5` records this:
- the register entries become decided;
- the policy "the card's text wins over a ruling that contradicts it" is recorded and applied;
- so is the policy for misattached rulings;
- `docs/RULING_DISCREPANCIES.md` starts as a log of every erroneous ruling, for human review and reporting to Wizards. Every worker adds to it from now on.

### Effort and fresh sessions (user, Nov 4, late evening)

The user decided:
- Implementers run at effort `high`.
- Reviewers, rules investigations and register judgment calls stay at `max`. This is now in worker.md.
- The long-running worker sessions are replaced by fresh ones, because their 300–760k-token contexts were re-read on every step.
- The user turned Ultracode off in the coordinator's session.
- No A/B measurement.

Transition: each old worker lands its current item and stops. The coordinator then starts a fresh session for that role, with the rest of the queue, and archives the old one. cards-2 keeps its session, which compacted to 94k on Nov 4.

New-session prompt (fill in ROLE, QUEUE and N):
> You are a worker session for morvor/mtg-sim, a Rust Magic: The Gathering rules engine. You replace an earlier session for the same role, whose context had grown large.
> - Read CLAUDE.md and docs/agents/worker.md (the worker guide: the per-item flow, the implementer and reviewer briefs, the rulings batch brief), and follow it, including its effort rule.
> - Role: ROLE. Queue, in order, at most N item(s) at a time: QUEUE.
> - Pull origin claude/mtg-rules-engine-lkr27h first.
> - When an item lands, report: the landed commit, the counts, the remaining gaps, any provisional register entries added, and any ruling that seems to contradict the CR or the Oracle text.
> - Keep working autonomously.

Rulings queues from Nov 4 22:55:
- **rulings-1:** P128, P124, P012.
- **rulings-2:** P096, P005, P195.
- **mutation:** P198, P034, P003.
- **compile:** P001, P008.
- **cards-2:** P025, P199, P140, P027.
- **structure, after ruling-conflicts-1:** P110, P073.
- **Next in the pool (Nov 5, 09:45):** the old lists are done; take open batches in id order: P059 P060 P061 P067 P068 P069 ... Assigned by 13:45: compile P008, P048, P049; rulings-2 P040, P050, P055; cards-2 P029+P033; rulings-1 P041, P042; mutation P019, P022; cards-1 P043+P047.
- **compile:** `roundtrip-gaps-5`.
- **mutation:** P031, P180. **rulings-1:** P186, P074, P118, P152. **rulings-2:** P052, P039, P183, P066.
- Next in the rulings pool: P010 P156 P096 P015 P134 P174 P097 P004 P075 P102 ...
- When round 4 has fully landed: a verified regeneration (CLAUDE.md, incremental or full sweep), when the user wants the verified count raised.

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
- **New cards** (user, Nov 4). When everything works properly, the user updates the data stores (Scryfall bulk data, rulings, tags; possibly a newer CR). Then the cards from newer sets are planned and supported like a card-support round.
- **Compile every hand-written card** (user, Nov 4). Eventually, every card with a hand-written ability must be supported by the compiler instead, so that no card depends on a `ManualAbility`. That is 151 cards on Nov 4, with River Song and Vesuvan Shapeshifter in flight (list: `rg -o 'card: *"[^"]+"' crates/mtg-engine/src/cards`).
  - Plan it like a card-support round: probe each hand-written text, then group the texts by the grammar or engine construct each needs.
  - The rules hooks the hand-written cards added (the `custom_*` hooks, `draws_from_bottom`, the loyalty-removal hook, ...) become compiled constructs any card can use.
  - It comes after rulings coverage.

## Resuming (coordinator)

0. **Refresh long-running worker sessions** (user, Nov 4). When a worker session's context passes about 300k tokens, refresh it at an item boundary: the worker lands its current item and stops, and a new session takes over its role and the rest of its queue. The new session gets the worker guide and its queue, and the old one is archived. The template is in the "Effort and fresh sessions" section above.

1. The workers are the cloud sessions tagged `mtg-sim-worker`. Check each one. Resume any item a
   usage limit interrupted on its existing worktree, with a fresh implementer or reviewer told to
   continue from `git log`, `git status` and `git diff`.
2. Send each worker its queue from this file.
3. Re-enable the hourly check-in.
