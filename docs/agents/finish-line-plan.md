# Finish-line plan: what "done" means for this engine, and the shortest sound path to it

Written Nov 9, 2026 by the planner session for item `finish-line-plan`
(docs/agents/items/finish-line-plan.md). It changes no engine code. The project's aim, from the
user: simulate Magic **precisely** by the Comprehensive Rules, be as sure of that as possible,
and make adding cards simple.

Everything measured here was measured on commit ad23a7a (the branch head on Nov 9); the
inputs were read by six subagents, the tool reports were re-run on that commit and compared
with the committed docs (docs/agents/reports/finish-line-measurements.md: every number in
§1.2 matches the committed doc unless its cell says otherwise), and the plan was challenged by
an independent reviewer before landing (its changes are the Nov 9 review commits). Where a
number comes from a committed report rather than a fresh run, the report's date is given.

## 1. Done criteria

### 1.1 What "done" can honestly mean

No finite set of tests proves that the engine plays every supported card by the CR. What we
can show is that **every independent oracle we have has been exhausted and has stopped finding
bugs**. We have five oracles that are independent of the implementer's reading of the rules:

1. **Wizards' rulings** (34,000 texts): someone else wrote down the outcome.
2. **The Oracle text through the round trip**: the compiled ability must render back to the
   card's words, by a renderer written separately from the parser (in this project, by the
   same kind of agent, so a misreading both share would pass).
3. **The CR's own worked examples** (277): done, 100% covered.
4. **Brute-force oracles written from the rules** (payment, damage assignment) and
   **metamorphic properties** (seat symmetry, snapshots, rollback, determinism, copies, hidden
   information): they catch whole classes of bugs without anyone predicting an outcome.
5. **A blind judge**: an agent that sees only the rules predicts what a test should assert.

Plus one method that does not need an oracle: **mutation testing**, which measures whether the
tests would notice a change in the core files at all.

What this cannot show, said plainly:

- a rule applied wrongly but consistently on cards no ruling discusses: the metamorphic
  properties and the fuzz see only inconsistency and crashes, so such a bug stays until a
  ruling, the audit or a human reader finds it;
- a misreading the parser and the renderer share, or a difference one of the renderer's
  equivalence rules normalizes away (B4 reads that list);
- a misreading the blind judge shares with a test's author: the judge is the same kind of
  agent, and its agreement bounds the error rate only for errors it can detect;
- a hand-written card whose definition and test follow the same wrong reading, until the card
  is compiled and round-tripped (C7);
- that any 100% figure means "right": every coverage number counts paths touched, not outcomes
  checked, except the three correctness criteria.

The criteria below are of two kinds. **Correctness criteria** compare the engine with one of
those oracles; reaching them is evidence the engine is right. **Bookkeeping criteria** measure
how much of the engine the tests touch; reaching them is necessary, and makes regressions
visible, but a touched path is not a right path. The honest headline is the correctness set.

The **stopping signal** is the measured bug yield. Today the rulings batches find about one
engine bug per 100 rulings processed (0.7–1.0 core engine bugs and about 0.5 compiler fixes per
batch of 100–140 in the 23 batches landed Nov 5–8) and the round trip finds a parser bug in
about 3% of the mismatching cards it triages. "Done" is when the oracles are exhausted **and**
the yield has fallen: five consecutive rulings items with no engine fix (at today's rate of
about 0.9 a batch, five zero-fix batches in a row would happen by chance about 1% of the time),
two consecutive round-trip items with no parser bug, and an audit area whose new tests all
passed first time. Until the yield drops, the engine is not done however good the coverage
numbers look. If the oracles are exhausted and the yield has not fallen, the next oracle to
consider is the one method not chosen on Oct 14, a comparison with Forge or XMage (the last
paragraph of §5); the plan does not need it yet.

Three conditions make the signal honest:

- **It must be measurable.** Today it is not: item reports go to the coordinator, commit bodies
  carry no counts, and the yields above were reconstructed from commit subjects
  (docs/agents/reports/finish-line-rulings-pool.md §3). From now on the reviewer of every
  rulings, round-trip, audit, mutation and property item appends one row to
  docs/agents/yield.tsv before landing (date, item, kind, units processed, engine bugs, compiler
  fixes, test errors, commit) and puts the same counts in the landing commit's message as a
  `Yield:` line (worker.md's reviewer brief says so). The signal is read from that file; the
  23 reconstructed batches are its first, approximate row.
- **It must be read on comparable items.** Phase A works the interaction-heavy batches first and
  the restatement-heavy ones last, so the yield falls along the queue partly by construction.
  The rulings signal counts only on the residue items (A3, the hardest, worked last) and on
  batches whose report says most of their rulings were interactions; a zero yield on a
  restatement batch is not evidence. Likewise the round-trip probe (A5) puts the likely
  behavioral mismatches first, so the signal there is two clean items on cards the probe did
  not flag, read together with the probe's measured recall on the known bugs: a clean tail
  behind a probe with low recall means little.
- **A bug counts whatever found it.** A parser bug a rulings batch finds counts as a compiler
  fix; a reading a reviewer or the user reverses counts as an engine bug; a test rewritten
  because it asserted a wrong outcome counts as a test error for C3.

### 1.2 The criteria, with today's values and targets

| # | Criterion | Kind | Today (Nov 9) | Target | How it is measured |
|---|---|---|---|---|---|
| C1 | **Rulings**: every in-scope ruling text is CITED by a test, EXEMPT with a reason, or UNCOVERED for a named reason (an unsupported card, a deferred mechanic) | correctness | CITED 25,777 of 33,955 (75.9%); EXEMPT 189; OPEN 6,125; DEFERRED 1,864; out of scope 662 | OPEN 0, except texts named in docs/rulings-uncovered.tsv with their reason (the script has no UNCOVERED status; today that file holds the one text whose only card is unsupported for a digital mechanic, Tenacious Pup's); and the last 5 comparable items found no engine bug (§1.1) | `python3 scripts/rulings_batches.py summary` (OPEN) against the lines of docs/rulings-uncovered.tsv |
| C2 | **Round trip**: every fully supported card's compiled abilities render back to its Oracle text, or the card is listed with a reason (a renderer gap, or a CR-justified equivalence) | correctness | 28,460 of 32,076 pass (88.7%); 3,464 mismatch, about 3,100 of them never triaged; 152 hand-written cards unchecked | 0 untriaged mismatches: every mismatching card either passes or is named in a roundtrip item's "Remaining" section with its reason (a renderer gap, or an equivalence a CR rule justifies); the last 2 items on cards the probe did not flag found no parser bug (§1.1) | `mtg-tools roundtrip --write`; untriaged = the mismatching cards not named in any docs/agents/items/roundtrip-*.md "Remaining" section (`grep`) |
| C3 | **Independent audit**: the seven core areas of `engine-audit-1` (layers, replacement, triggers, zones/LKI, combat, casting/costs, SBA/copies) audited adversarially, each with reviewer sign-off; and a second blind judge audit with 0 confirmed wrong tests | correctness | audit: 0 of 7 areas (the item was issued Nov 9). Judge audit 1 (Oct 14): 58 tests, 225/225 questions agreed, so the wrong-test rate is below about 5% | 7 of 7 areas landed, each report giving its yield (tests added, bugs found); judge audit 2 over 200 tests including every hand-written card's test, 0 confirmed wrong | the areas' reports and docs/agents/yield.tsv; docs/agents/reports/judge-audit-2.md |
| C4 | **Properties**: all metamorphic and exhaustive properties hold on the final code | correctness (consistency) | 8 properties, all held on their last runs (2,000 games each for 1–4; every card for copies; 1.2M boards; 200k combats; 2,400 hidden-information games), but on code of Oct 22–28, about 2,000 commits ago | the 8 plus the new ones in §3 (legal-action exactness, layer order-independence, ability copies, multiplayer kinds) hold on the final code at the same sizes | `mtg-sim --properties`, docs/PROPERTIES.md |
| C5 | **Mutation score**: the tests notice changes to the core files | test strength | 459 of 520 viable sampled mutants killed (88.3%; 551 sampled, 1 in 10 of 5,281, Oct 14–21) across the 15 core files, now 33,113 lines (simultaneous.rs 55.6%, triggers.rs 75.0%, permissions.rs 78.6%, mana.rs 81.0%, replacement.rs 82.4%); several files rewritten since | every file ≥ 90% on a fresh sample, every survivor classified (equivalent, or killed); full runs, not samples, of simultaneous.rs, triggers.rs and trigger_timing.rs, permissions.rs and replacement.rs | `scripts/mutants.sh`, docs/MUTATION.md |
| C6 | **Fuzz**: every-card sweep and random-deck fuzz on the final code with no panic, hang or invariant violation | crash and invariant check | last full sweep Nov 5: 32,656 cards, 1 fuzz-unclean card (recorded only in the tier file and docs/CARD_COVERAGE.md's sweep list; docs/EVERY_CARD.md has no write-up of that sweep, its latest is an older one: 0 panics or hangs, 95% of abilities used); no document records any `--random-decks` result | a final sweep with 0 failures, written up in docs/EVERY_CARD.md, and 10,000 random-deck games with 0 panics, recorded there too | `mtg-sim --every-card`, `--random-decks` |
| C7 | **Hand-written cards**: no card depends on a `ManualAbility` | correctness (removes the only unaudited path) | 153 cards (154 registrations, 156 ability texts), all tested; the round trip skips their hand-written abilities (152 of the cards have nothing else it checks) | 0 (the user's standing "later" item; see §3 phase C) | `rg -o 'card: *"[^"]+"' crates/mtg-engine/src/cards \| sort -u \| wc -l` |
| C8 | **Register and discrepancy log**: every judgment call registered and decided; every erroneous ruling logged and reviewed by a human | bookkeeping of judgment | 128 register entries, 0 provisional; 48 discrepancy entries, all "open for review", 0 reported | 0 provisional; every behavioral difference named in an item's "Remaining" section has a register entry or, where the CR is plain, a fix (B4 sweeps for them); the user has read the 9 deviations; the log's statuses set by the user | `mtg-tools interpretations --check`, docs/RULING_DISCREPANCIES.md |
| C9 | **Engine gaps**: no confirmed rules gap open for a supported card | bookkeeping | the 33-gap inventory is closed as a feature list; open: two unverified bits of G32 (graveyard counts of tokens, meld on a copy), Inquisitor Eisenhorn's "this way" trigger never firing for a card drawn while casting (CR 121.8 reveals the card when the spell becomes cast, so the trigger should fire then; noted only in roundtrip-tail-5's "Remaining", a bug to fix, not a reading to register), hashed ability links that could collide (CR 607.5) | 0 | docs/agents/engine-gaps.md |
| C10 | **CR coverage**: every numbered rule cited, covered via subrules or exempt; every worked example covered | bookkeeping | 3,038 cited + 76 via subrules + 52 exempt, 0 uncovered of 3,166 (measured Nov 9; the committed doc of Nov 5 says 3,036 + 78); 277/277 examples | stays at 0 uncovered (a regression gate, nothing to do) | `mtg-tools cr-coverage` |
| C11 | **Verified cards**: cards passing every check | bookkeeping | 18,113 of 32,111 paper cards (56.4%); round-trip or better 28,570 (89.0%); compiled 32,076 (99.9%); 11,522 cards blocked only by an unexercised structure | report it; raising it is decision D3 below (not a correctness gate: "exercised" is not "asserted") | `mtg-tools card-coverage` |

Housekeeping found while measuring: the committed docs/RULINGS_COVERAGE.md (1,558 unique
texts cited, 4.6%) was never regenerated and is stale; `mtg-tools rulings-coverage --check`
itself agrees with the batches script on the cited count (25,777) and differs only in the
denominator (34,000 texts against the script's 33,955 in-scope ones) and the exempt count (166
against 189). Item `register-sweep-1` regenerates the doc, reconciles the two denominators and
adds a check that keeps them in step, so the standard update has one number.

Today's baseline (measured Nov 9, details in docs/agents/reports/finish-line-measurements.md):
`cargo test --workspace` passes, 28,659 tests, 0 failed, 6 ignored, 14 minutes of wall clock
including a cold debug build (about 6.5 minutes of test execution); citations `cr!` 28,155,
`ruling!` 27,008, `interp!` 340, `cr_example!` 290; 5,776 commits since Sep 24, about 108 a
day over the last 14 days.

### 1.3 Which criteria measure correctness

- **C1, C2, C3** are the correctness evidence. Each compares the engine with words Wizards or
  the renderer produced independently of the implementer. Their *yield* (bugs per batch, bugs
  per item, disagreements per audit) is the real measure of how wrong the engine still is.
- **C4** finds inconsistency, not wrongness: a rule applied wrongly but consistently passes
  every property. The exhaustive ones (payment, damage) do compare with the rules, inside the
  catalog they cover.
- **C5** measures the tests, not the engine; but a surviving mutant is a behavior nothing pins
  down, and the one engine bug mutation testing found (multi-searcher moves, CR 701.23i) came
  from such a survivor.
- **C6** finds crashes, hangs and ledger inconsistencies only. Nine engine bugs over five
  sweeps, none of them a wrong amount, target or timing.
- **C7–C11** are bookkeeping. C7 matters because the hand-written cards are the one place
  no independent oracle has looked. C11's structure check records that a code path *ran* in
  some test, possibly a random one; the structure-tests stream found about one engine bug per
  600 tests, against one per 100 rulings.

## 2. Where the remaining risk is

Ranked by evidence that the engine is *wrong* there, not merely untested. Full detail with
file and section citations is in docs/agents/reports/finish-line-risks.md (summarized here; the reviewer
checked the numbers).

| Rank | Area | Evidence | Cheapest way to find out |
|---|---|---|---|
| 1 | **The ~3,100 untriaged round-trip singletons** | 69 parser bugs came from triaging about 2,000–2,800 mismatching cards (2.5–3.5% per card); ~20 were referent misbindings ("it", "that player", "another"), ~15 dropped clauses ("other", "unless", "reveal", "your graveyard"). Singletons are unusual text, where these failure modes live. Expect 75–100 more cards that play wrong. | `roundtrip-pairs-N` behaviour-first; and a referent probe over the AST (item `roundtrip-probe-1`) |
| 2 | **Costs, X, casting permissions, the legal-action list** (casting.rs, permissions.rs, spell_costs.rs) | the largest cluster of engine fixes in the 23 visible batches (~12 of ~45): castability judged without continuous effects, default X ignoring X-dependent costs, X = 0 under an alternative activation cost, free casts and additional costs, riders. casting.rs is the biggest file (841 mutants, 90% unsampled). No property sees over- or under-offered actions. | a legal-action exactness property inside the fuzz (`property-tests-3`); audit area 6 |
| 3 | **"Each player" simultaneity and per-player referents** (simultaneous.rs) | lowest mutation kill rate (55.6%), six real survivors; the one mutation-found engine bug; six rulings fixes in 23 batches (searcher inside "for each player", Kefka, Breeches, "named twice"); every probe into this file found something; rewritten since the sample. | full mutation run of simultaneous.rs (`mutation-2`); residue batch of "each player" rulings |
| 4 | **Triggers: timing within resolutions, batching, reflexive and delayed** (triggers.rs, trigger_timing.rs) | 75% kill rate, seven survivors deleting EventInfo fields no test reads; G09 rebuilt in a 17-mutant file carrying a lot; six rulings fixes (reflexive once per N counters, Phelia, Codie, 603.2c batching). Five methods found trigger bugs in Oct–Nov. | audit area 3; `RERUN=1` plus a fresh shard of both files |
| 5 | **Last known information and departed objects** (CR 400.7, 608.2h) | eight rulings fixes (abilities of a departed object activatable, target checks on what an Equipment was attached to, Gonti, Reanimate's MV, an exiled card's power); 2 of 9 fuzz bugs were stale ids. Liveness is enforced by convention (`game.current(id)`), not types. | debug-assert liveness in the actions.rs primitives and run one fuzz chunk with assertions on (`gaps-residual-2`); audit area 4 |
| 6 | **Replacement and prevention** (replacement.rs, prevention.rs, draw_rules.rs) | 82.4% kill, survivors unkillable so far; five rulings fixes (Stagger, shields per recipient, draw replacements, 616.1 choice) and four round-trip bugs. Wrong-but-consistent: invisible to every property and to the fuzz ledger. | audit area 2; a residue batch filtered to "instead"/"prevent" |
| 7 | **Copies and links** (copy.rs, ability_grants.rs) | property 5 found nine bugs on its first run over exactly its paths; the paths it skips (copies of activated and triggered abilities, non-permanent spell copies resolving, planeswalkers and battles copied only as tokens) have had no equivalent scrutiny; hashed links can collide. | extend `properties::copies` to those paths; per-object link counter (`property-tests-3`, `gaps-residual-2`); audit area 7 |
| 8 | **Layers with copy, type and ability changes; the dependency shortcut** (layers.rs) | well tested where sampled (87.5%, no killable survivor; eval.rs 100%); zero rulings fixes. But nobody has looked at dependency loops, 613.7 timestamps after control changes, or the cubic-to-linear shortcut's ordering. | a property: recompute characteristics with the active statics shuffled before dependency analysis (`property-tests-3`); audit area 1 |
| 9 | **Multiplayer variants the fuzz never plays** (2HG, Emperor, Grand Melee, range of influence) | the sweep's kinds are standard, commander, planechase, archenemy, vanguard, conspiracy, attractions, dungeon; properties 1–4 play 2–4 player free-for-all; seat symmetry's seven bugs were all multiplayer edge paths; range.rs was rewritten this round. | `--kind` for those variants in the properties fuzz, a few hundred games each (`property-tests-3`) |
| 10 | **The 153 hand-written cards and the renderer's EQUIVALENCES list** | no oracle checks the hand-written definitions beyond their author's test; the equivalence list (~150 rewrite rules) is where a real difference could be normalized away. | judge audit 2 over every manual-card test; one session reading EQUIVALENCES against the CR (`judge-audit-2`, `register-sweep-1`) |
| 11 | **Judgment calls decided by workers** | of the two tested behaviors the user has been asked about so far, one was reversed ("you and that player each draw" is one instruction, Nov 7). 128 entries are decided; one behavioral difference (Eisenhorn, 121.8) lives only in an item's "Remaining" section (a bug, not a reading: B5 fixes it). | the user reads the 9 deviations (D5); sweep every "unless noted" remainder into the register (`register-sweep-1`) |
| 12 | **The 95% of rulings never tested by name** | the only method with a measured, repeatable yield on semantics; the open pool is 75% interactions (sample of 60) against 25% restatements. Expect 45–60 core engine bugs and about 30 compiler fixes from the 6,125 open texts. | it is the check (phase A) |
| 13 | Loop detection, hidden information in the protocol, structure-blocked cards | each has had one or two bugs and a fix; low card impact or already covered by a property | report; `property-tests-3` adds a library-entitlement check to the hidden-information property |

Which method finds what, from the record so far:

| Method | Finds | Blind to | Yield |
|---|---|---|---|
| Rulings batches | interaction semantics: cost and X arithmetic, LKI, per-player referents, replacement choice, trigger timing, illegal targets | anything no ruling mentions; a worker's misreading encoded in the test | ~1 engine bug per 100 rulings, plus ~0.5 compiler fixes |
| Round trip | parser misreads: referents, dropped clauses, ownership, "this way" scope, trigger zone | engine semantics once the AST is right; hand-written cards; differences the equivalence list absorbs | ~3% of triaged mismatching cards |
| Mutation testing | missing tests on core paths; one engine bug | the 90% of mutants never run | 1 bug and ~30 real missing tests per 551 mutants |
| Metamorphic properties | seat-order leaks, players leaving together, subgames, shared ids | any rule applied wrongly but consistently | 7 bugs in the first 2,300 games, 0 since |
| Exhaustive properties | links and copy exceptions, Phyrexian and pain-land ordering, trample lethal sharing, hidden ids | outside their catalog | 15 bugs on first runs, 0 since |
| Every-card fuzz | stale state and ids, tapping departed objects, unending loops, performance cliffs | wrong amounts, targets and timing | 9 over 5 sweeps |
| Blind judge | wrong test assertions | shared misreadings | 0 of 225 |
| Structure tests | compiler omissions on shared constructs | single-card structures; outcomes nobody asserted | ~1 bug per 600 tests |

## 3. The plan

Ordered, in three phases. Phase A runs now, in parallel across the sessions the user has.
Phase B starts as sessions free up; its items are the new verification work this plan adds.
Phase C is the user's standing "later" work and the final measurement. Effort is in
worker sessions (one implementer plus one reviewer pass, 1–3 hours of wall clock each on the
current model at the measured rate). "Judgment" items need the strongest model throughout;
"routine" items run implementers at high and reviewers at max, per worker.md.

### Phase A: exhaust the two oracles that keep finding bugs (now)

| # | Item | Fixes or proves | Effort | Judgment |
|---|---|---|---|---|
| A1 | `engine-audit-1` (issued Nov 9, nothing landed yet): layers, replacement, triggers, zones-LKI, combat, casting-costs, SBA-copy. Its brief lists layers first; the risk table says to take the areas in the order casting-costs, triggers, zones-LKI, replacement, SBA-copy, combat, layers, so that if the budget runs out the riskiest areas were audited (re-order the auditor's queue unless it has already begun layers) | the direct check on risks 2, 4, 5, 6, 7, 8; each area lands with tests and a mutation run of its main file | 7–10 sessions, one per area, serial | **judgment, max** |
| A2 | Rulings: the 40 never-worked batches (4,526 open texts), interaction-heavy first: P154 P153 P155 P100 P101 (mana value, X, costs), P158 P157 (self-replacement, "without paying"), P144 P090 (impulse exile permissions and costs: the family that found ~10 bugs in P067–P071), P196 P197 (indefinite effects, 611.2), P138 P139 (removal-sacrifice: triggers, LKI, 2HG), P182 P181 (tutor to battlefield: Auras, simultaneous entry), P175 P114 P200 P115 P135; then the cheaper ones (P093 P095 P049 P081 P083 P084 P086 P164 P167 P168 P170 P173 P103 P105 P106 P191 P149 P088 P078) | risk 12; expect 35–45 engine bugs here | 40 sessions | routine |
| A3 | Rulings residue: the 1,599 open texts left in 114 worked batches, regrouped into themed items (docs/agents/items/rulings-residue.md): copy-creature (P020 P021 P023), Enchant/Equip keywords (P207–P213), counters (P035 P120 P119 P121), reflexive and refund (P129 P130), type change (P187 P188), hate-attacker/blocker (P079 P080 P082), copy-instant (P028 P030 P032), keyword leftovers (P217 P214 P205 P226 P222 P215), burn P009, freeze P062, shapechange (P160 P161), typal P184, and two misc sweeps (about 770 texts) | risks 3, 6, 7 (what earlier rounds left was the harder part) | ~14 sessions | routine |
| A4 | Round trip: `roundtrip-pairs-6` (running) and its successors over the ~3,100 untriaged mismatches, behaviour-first, ~150 cards an item | risk 1; expect 75–100 parser bugs | ~20 sessions | routine, with the parser-bug triage at max |
| A5 | `roundtrip-probe-1` (docs/agents/items/roundtrip-probe-1.md): an automated probe over every mismatching card's AST that lists referents bound to something other than the text's nearest antecedent, dropped qualifiers ("other", "another", "you own", zone words) and dropped clauses; its hits are triaged first by A4 | makes A4 find the behavioral mismatches before the wording ones | 2 sessions | judgment to design, routine to run |

Pace: at an estimated 10–14 batches a day from two rulings sessions (the measured rate is
1.1–1.7 hours per batch per session, and the Nov 8 run landed 7 batches in 8 hours from two
sessions), A2+A3 is 4–5 days; with the API launcher's two children alongside, 2–3 days. A4 at
one session is about three weeks; at two, ten days. A1 is serial by design.

### Phase B: the verification this plan adds (as sessions free up; B1–B3 can run in parallel)

| # | Item | Fixes or proves | Effort | Judgment |
|---|---|---|---|---|
| B1 | `mutation-2` (docs/agents/items/mutation-2.md): full runs (not 1 in 10) of simultaneous.rs, triggers.rs with trigger_timing.rs, permissions.rs and replacement.rs; fresh 1-in-10 samples of the other 11 core files on current code; every survivor classified, the real ones killed | C5; risks 3, 4, 6 | 4–5 sessions, machine-heavy (6–12 h a file; one at a time, nothing else heavy alongside) | survivor classification needs judgment; the rest routine |
| B2 | `property-tests-3` (docs/agents/items/property-tests-3.md): (a) legal-action exactness inside the every-card fuzz: every offered cast or activation succeeds with the engine's own payment, and sampled non-offered ones fail for a CR reason; (b) layer order-independence: characteristics agree when the active statics are shuffled before dependency analysis; (c) copies of activated and triggered abilities and of non-permanent spells resolve like the original; (d) `--kind 2hg\|emperor\|grand-melee\|range` in the properties fuzz; (e) the hidden-information check compares library cards a decision shows with what the effect entitles (701.20, 701.23); (f) cost reductions and one alternative cost in the payment oracle | C4; risks 2, 7, 8, 9, 13 | 4–5 sessions | **judgment** to design each oracle from the rules; routine to run |
| B3 | `judge-audit-2` (docs/agents/items/judge-audit-2.md): the blind audit again over 200 tests: every hand-written card's test (153) plus 47 drawn at random from rulings and cr tests written since Oct 14; disagreements adjudicated and refuted as in audit 1 | C3; risk 10, 11 | 1 session plus the coordinator's workflow (audit 1: 118 agent runs in 18 minutes) | **judgment, max** |
| B4 | `register-sweep-1` (docs/agents/items/register-sweep-1.md): register every behavioral deviation named in an item's "Remaining" or "unless noted" section (Eisenhorn 121.8 first); read the renderer's EQUIVALENCES list against the CR; regenerate docs/RULINGS_COVERAGE.md and make `rulings-coverage` and the batches script agree on their denominators and exempt counts, with a check | C8; risks 10, 11 | 1–2 sessions | judgment for the register entries |
| B5 | `gaps-residual-2` (docs/agents/items/gaps-residual-2.md): G32's two unverified bits (tokens in graveyard counts, meld on a copy), the hashed ability links replaced by a per-object counter (607.5), and liveness debug-assertions in the actions.rs primitives with one fuzz chunk run under them | C9; risk 5 | 1–2 sessions | routine |

### Phase C: the user's standing items and the final measurement

| # | Item | Fixes or proves | Effort | Judgment |
|---|---|---|---|---|
| C1 | `compile-manual-*`: compile every hand-written card, grouped by construct (docs/agents/compile-manual-plan.md: the groups, the constructs each needs, the order) | criterion C7; the custom hooks become constructs any card can use | 6 items in 3 waves, 17–22 sessions (the plan) | judgment for the constructs that need new AST; routine for the rest |
| C2 | `verified-refresh-final` (docs/agents/items/verified-refresh-final.md): structure log, every-card sweep in four chunks, the tier file, docs/CARD_COVERAGE.md, docs/STRUCTURE_COVERAGE.md, plus a full run of every property and 10,000 random-deck games, all on the final code | criteria C4, C6, C11 | 1 session plus ~4 hours of machine time, nothing else heavy alongside | routine |
| C3 | `data-refresh` (docs/agents/items/data-refresh.md; after the user refreshes data/): `cr-diff-1` (renumbered or changed rules: fix `cr!` citations, re-read every changed rule against the engine) then a card-support round 7 for the new sets, planned like rounds 1–6 | keeps the engine current | planning 1 session; the round depends on the set count (189 unreleased-set cards are in the data already) | the CR diff needs judgment; the round is routine |
| C4 | Loops (post-finish, decision D4): early recognition of growing loops; proposed shortcuts (CR 732.2) in the agent protocol | a UI would need it | 3–5 sessions | judgment |

The order within each phase is the order to start items; across phases, start B as soon as a
session frees from A (B1 and B2 need no A results; B3 should run after A1's areas have landed
so it audits the audit's tests too). C1 waits for A2–A4 as the user said (compile every
hand-written card comes after rulings coverage). C2 is last, once no engine change is
pending; C3 starts when the user says the data is refreshed.

### What the user's sessions do next (a suggested re-plan of the queues)

- **auditor**: A1, area by area.
- **rulings-1, rulings-2** (and the API launcher's children when credits allow): A2 in the
  interaction-first order above, then A3.
- **roundtrip**: A5 first (two sessions of tooling), then A4 with the probe's hits first. A
  second round-trip session is worth more per session than a third rulings session: an item of
  150 mismatches has found 4–5 parser bugs, a rulings batch about 1.4 fixes (an engine bug does
  reach more cards than a parser bug, but both pools must be exhausted either way, so run them
  side by side).
- **planner** (this session) or a fresh session: B4 now (it is small and unblocks the user's
  review), then B2.
- A **mutation** session when the auditor has landed an area: B1, one file at a time, since
  its machine load forbids a second heavy job.

## 4. Human review: the ruling discrepancy log

docs/RULING_DISCREPANCIES.md holds 48 entries, every one "open for review"; nothing has been
reported to Wizards yet. By kind: template error 6, copying error 2, inconsistent 5, narrower
than stated 4, misattached 6 (15 rulings), outdated 25 (36 rulings; most are the 73 modal
double-faced cards and about 60 Siege, craft and transform cards whose "can't be transformed"
rulings the 2025-09-19 rules change reversed). The exemption files agree with the log (41
policy-naming exemption lines against the entries that need them; three outdated entries and
Share the Spoils have no exemption by design).

The ten most consequential entries, where a player following the published ruling would play
the card wrong, ranked by how wrong and how often the card is played. Every quote is verbatim
from data/.

1. **Combat Celebrant** (template error). Ruling (2017-04-18): "Combat Celebrant's ability
   untaps all of your creatures, not just the ones that are attacking." Oracle: "... untap all
   other creatures you control ...". The engine leaves the exerted Celebrant tapped. Likely
   adapted from Lightning Runner's ruling, whose card lacks "other" (an inference; the
   contradiction stands regardless).
2. **Homicide Investigator** (template error). Ruling (2024-04-12): "If Homicide Investigator
   dies at the same time as one or more other nontoken creatures you control, Homicide
   Investigator's ability triggers for each of them." Oracle: "Whenever one or more nontoken
   creatures you control die, investigate. This ability triggers only once each turn." Word
   for word Midnight Reaper's ruling (2018-10-05) with the name swapped; G'raha Tia's ruling on
   the same template (2025-06-06) says the opposite. A board wipe gives one Clue.
3. **Run Out of Town** (copying error). Ruling (2022-04-29) says "That permanent's controller
   chooses whether to put it on the top or bottom of the library" and, of a permanent made of
   several cards, "that permanent's controller puts all of them on the top or all of them on the
   bottom"; the Oracle text ("The owner of target nonland permanent puts it on their choice of
   the top or bottom of their library.") and the Streets of New Capenna release notes
   (2022-04-20) say the owner. Matters whenever a
   stolen permanent is targeted.
4. **Share the Spoils** (inconsistent; the one entry still awaiting the user, decision D1).
   Ruling (2021-07-23): "If you control an opponent's Share the Spoils and that player loses the
   game, Share the Spoils leaves the game with them and the first ability won't trigger."
   Against CR 603.10f and the Sanctum of Serra, Thieving Amalgam and Grismold rulings.
5. **Sakashima's Protege** (template error). Ruling (2020-11-10): "... You may choose only a
   creature that's already on the battlefield and entered the battlefield this turn." Oracle:
   "... enter as a copy of any permanent that entered this turn." The Clone template with a
   clause appended; the card's own same-day ruling about copying a planeswalker contradicts it.
6. **Acolyte Hybrid** (template error). Ruling (2022-10-07): "... you won't draw a card. If it
   is destroyed but put into a zone other than a graveyard, you'll draw a card." Oracle: "If an
   artifact is destroyed this way, its controller draws a card." Noxious Gearhulk's template
   with "you" kept; the published ruling says the opposite of the card's drawback.
7. **Deranged Assistant** (outdated). Ruling (2025-01-24) describes a mana ability activated
   while casting; CR 605.1a (revised 2026-08-07 by the log's dating; data/ holds only the
   current CR) makes an ability whose cost moves a card to or from a library not a mana ability, and Wizards issued new rulings for Millikin, Chromatic
   Sphere and Selvala but left this one (and the Eggs' 2008 "This is a mana ability", Charmed
   Pendant's 2004 one).
8. **Valakut Awakening and 72 other modal double-faced cards** (outdated). Ruling (2020-09-25
   to 2024-11-08): "A modal double-faced card can't be transformed or be put onto the
   battlefield transformed. ..." Reversed by the 2025-09-19 rules change (CR 701.27a, 712.14a;
   Gwen Stacy's ruling of 2025-09-19 says so). The same change outdates about 60 Siege, craft
   and transform-copy rulings logged with it.
9. **Araumi of the Dead Tide** (inconsistent). Ruling (2020-11-10): "If the creature card has
   {X} in its mana cost, you can choose any value for X, but the value you choose won't apply to
   the tokens that are created. X will be 0 for those tokens." CR 107.3h makes X 0 for a cost
   paid off the stack; Varolz's scavenge ruling (2013-04-15) agrees. The engine uses 0. Wizards
   published it deliberately, so it may reflect intent: ask which is meant.
10. **Sanguine Spy** (copying error). Ruling (2022-04-29) says "among cards you control" where
    the card and the release notes (2022-04-20) say "among cards in your graveyard".

Next tier: Rimescale Dragon (Rimefeather Owl's ruling with the name swapped), Fiery
Annihilation (decided Nov 7), Lich (its 2010 ruling still says "leaves the battlefield" though
Wizards reworded that text in 2011 and the Oracle text has said "put into a graveyard from the
battlefield" since 2010), Citanul Druid.

Read most carefully (the evidence is an inference, not a document): Combat Celebrant's origin,
Araumi, Share the Spoils, Gnoll War Band, Fiery Annihilation, Arbiter of Knollridge, Citanul
Druid, Stone Calendar, Scuttletide, Astarion. Solid (release notes, an identical sibling ruling
or a dated CR change): Homicide Investigator, Run Out of Town, Sanguine Spy, Sakashima's
Protege, Acolyte Hybrid, Rimescale Dragon, Deranged Assistant and the Eggs, every MDFC, Siege,
craft and transform entry, and the misattached Clive, Squall and Gut entries.

"Not yet investigated" (not entries): a Scryfall legality note on Spatial Contortion, Circle of
Flame and Hada Freeblade (a report would go to Scryfall); Dragon-Kami's Egg's "hatchling"
typo (Wizards' own, in the release notes too); Geier Reach Sanitarium's "When you activate"
and Kami of the Crescent Moon's "just after the normal draw" (wording that puts an event at
the wrong moment, outcome unaffected). The recommendation is that none becomes a conflict case
(the engine is unaffected by all of them); say so if you want any of them reported. Also: ruling-conflicts-3 compared 69 release-notes files
(2013–2022) against the data by word changes and hand-read about 25; the other ~1,100 changed
notes were not read one by one, so more copying errors may exist. A one-session item could
finish that read if the user wants the log complete before reporting.

The full summary with every quote is docs/agents/reports/ruling-discrepancies-review.md; the user decides
whether and where to send it (D2).

## 5. Decisions only the user can make

The register has 0 provisional entries and every conflict case but one is decided. The
decisions below are the one open case and the five policy choices this plan turns on. Every
rule and ruling is quoted verbatim from data/ (CR effective 2026-09-25; rulings and Oracle
texts as of 2026-09-24).

### D1. Share the Spoils: set the ruling aside as inconsistent with CR 603.10f?

**Question.** A player controls an opponent's Share the Spoils and that opponent loses the
game. Does "whenever an opponent loses the game" trigger (the engine, CR 603.10f) or not (the
ruling)?

**Recommendation.** Set the ruling aside as inconsistent, as Gnoll War Band's and Arbiter of
Knollridge's were on Nov 5: an exemption line citing this decision in a new
docs/rulings-exemptions/rulings-P068.tsv (P068 has none yet); the log entry stays open for
review and goes to Wizards with the others. The engine already follows the
CR and is tested (`tests/rulings/r_p068_players.rs`, `tests/cr/r600_look_back.rs`). The
alternative is to follow the ruling: make the leave-the-game procedure (800.4a) remove the
object before the loss is checked for triggers; that contradicts 603.10f and three other
rulings and would need a register deviation.

**Oracle, Share the Spoils:** "When this enchantment enters and whenever an opponent loses the
game, exile the top card of each player's library. / During each player's turn, that player may
play a land or cast a spell from among cards exiled with this enchantment, and they may spend
mana as though it were mana of any color to cast that spell. When they do, exile the top card
of their library."

**Ruling, Share the Spoils (2021-07-23):** "If you control an opponent's Share the Spoils and
that player loses the game, Share the Spoils leaves the game with them and the first ability
won't trigger."

**CR 603.10:** "Normally, objects that exist immediately after an event are checked to see if
the event matched any trigger conditions, and continuous effects that exist at that time are
used to determine what the trigger conditions are and what the objects involved in the event
look like. However, some triggered abilities are exceptions to this rule; the game “looks back
in time” to determine if those abilities trigger, using the existence of those abilities and
the appearance of objects immediately prior to the event. The list of exceptions is as
follows:"

**CR 603.10f:** "Abilities that trigger when a player loses the game look back in time."

**CR 603.3a:** "A triggered ability is controlled by the player who controlled its source at
the time it triggered, unless it’s a delayed triggered ability. ..."

**CR 800.4a:** "When a player leaves the game, all objects (see rule 109) owned by that player
leave the game and any effects which give that player control of any objects or players end.
Then, if that player controlled any objects on the stack not represented by cards, those
objects cease to exist. Then, if there are any objects still controlled by that player, those
objects are exiled. This is not a state-based action. It happens as soon as the player leaves
the game. ..."

**Rulings the other way** (an object's own look-back trigger when 800.4a removes it):
- Sanctum of Serra (2009-10-01): "If you planeswalk away from Sanctum of Serra because the
  player who owns Sanctum of Serra leaves the game, its first ability will still trigger and
  resolve."
- Thieving Amalgam (2019-08-23): "If a creature you control but don't own dies at the same time
  that its owner leaves the game, Thieving Amalgam's last ability triggers. No player loses 2
  life, but you gain 2 life."
- Grismold, the Dreadsower (2019-08-23): "In a multiplayer game, if a creature token dies at
  the same time its controller loses the game, Grismold's last ability triggers."

**Evidence gathered by rulings-P068 (investigator and skeptic):** 603.10f dates from the CR of
2016-11-11 (by the investigation's reading of the archived CR texts; data/ holds only the
current CR), so the ruling is not outdated; the Forgotten Realms release notes (2021-07-12) have
the same words, so it is not a copying error; it names its own card and doesn't contradict the
Oracle text, so it is neither misattached nor a template error. "the first ability won't
trigger" appears on no other card.

### D2. The discrepancy log: review and report now?

**Question.** Does the user want to read the 48 entries (§4 gives the ten most consequential)
and set their statuses now, and should a worker draft the report to Wizards? Also: should the
remaining ~1,100 changed release-notes passages be read one by one first (one session), so the
log is complete before anything is sent?

**Recommendation.** Review the ten entries of §4 now and mark them; have a worker draft the
report once the user has marked what to send; run the release-notes read as part of the
rulings residue (A3) rather than before reporting.

### D3. The verified count: which approach, and is it a done criterion?

**Question.** Today 18,113 of 32,111 cards (56.4%) are verified; 11,522 compiled cards fail
only the structure check, about 9,100 of them by exactly one unexercised structure, and no
unexercised structure is shared by more than four cards, so each new test verifies one to
three cards. Options (from the Oct 21 "Later" note): (1) test whole shapes, about 3,000 tests,
routine, many sessions; (2) compositional coverage: every AST component and combination at
depth about one is exercised by some test, and a card counts as structure-verified when all of
its components are; this changes what "verified" means and needs a register policy entry;
(3) leave the count as it is and report it.

**Recommendation.** (3) for the finish line, and (2) afterwards if the user wants the headline
raised: the structure-tests stream found about one engine bug per 600 tests, so option (1)
buys bookkeeping, not correctness, and the three correctness criteria (C1–C3) are where the
sessions should go. No CR text turns on this.

### D4. Loops: how far before "done"?

**Question.** Today the engine fingerprints the game state at each priority and calls a loop of
mandatory actions a draw on the third repeat after at least eight passes (register entries
`mandatory-loop-detection`, `growing-loops-run-to-the-limit`); a fragmented loop (732.3) is
broken by the active player; a loop that grows the state (two Charismatic Conquerors a side)
runs to the action limit and is then a draw. Options: (1) keep that for the finish line; (2)
also recognize growing loops early (a model of "the same sequence of events", not states);
(3) automate proposed shortcuts (CR 732.2), which a UI would need and which needs an
agent-protocol extension.

**Recommendation.** (1) for the finish line; (2) and (3) as the post-finish item C4.

**CR 104.4b:** "If a game that’s not using the limited range of influence option (including a
two-player game) somehow enters a “loop” of mandatory actions, repeating a sequence of events
with no way to stop, the game is a draw. Loops that contain an optional action don’t result in
a draw."

**CR 732.1b:** "Occasionally the game gets into a state in which a set of actions could be
repeated indefinitely (thus creating a “loop”). In that case, the shortcut rules can be used
to determine how many times those actions are repeated without having to actually perform
them, and how the loop is broken."

**CR 732.2a (first sentences):** "At any point in the game, the player with priority may
suggest a shortcut by describing a sequence of game choices, for all players, that may be
legally taken based on the current game state and the predictable results of the sequence of
choices. This sequence may be a non-repetitive series of choices, a loop that repeats a
specified number of times, multiple loops, or nested loops, and may even cross multiple turns.
It can’t include conditional actions, where the outcome of a game event determines the next
action a player takes. ..."

**CR 732.2b:** "Each other player, in turn order starting after the player who suggested the
shortcut, may either accept the proposed sequence, or shorten it by naming a place where they
will make a game choice that’s different than what’s been proposed. (The player doesn’t need
to specify at this time what the new choice will be.) This place becomes the new ending point
of the proposed sequence."

**CR 732.2c:** "Once the last player has either accepted or shortened the shortcut proposal,
the shortcut is taken. The game advances to the last proposed ending point, with all game
choices contained in the shortcut proposal having been taken. If the shortcut was shortened
from the original proposal, the player who now has priority must make a different game choice
than what was originally proposed for that player."

**CR 732.3:** "Sometimes a loop can be fragmented, meaning that each player involved in the
loop performs an independent action that results in the same game state being reached
multiple times. If that happens, the active player (or, if the active player is not involved
in the loop, the first player in turn order who is involved) must then make a different game
choice so the loop does not continue."

**CR 732.4:** "If a loop contains only mandatory actions, the game is a draw. (See rules 104.4b
and 104.4f.)"

### D5. Accept the done criteria of §1, and read the nine deviations

**Question.** Are C1–C11 with their targets the definition of done, with C7 (compile every
hand-written card) and C11 (verified) as the user prefers: C7 a gate or a post-finish item,
C11 reported only (D3)? And will the user read the nine registered deviations from the CR
(docs/INTERPRETATIONS.md, kind "deviation") as the human check on the judgment calls workers
made? Of the two tested behaviors the user has reviewed so far, one was reversed (Nov 7), so
this read is the cheapest correctness check left.

**Recommendation.** Accept C1–C6 and C8–C10 as gates; C7 a gate (it is the only unaudited
path, and the user already asked for it); C11 reported. Read the nine deviations.

### D6. When to refresh the data

**Question.** When does the user refresh data/ (Scryfall cards, rulings and tags; possibly a
newer CR)? The refresh by itself brings the 189 cards of sets unreleased on 2026-09-24 into
scope (today deferred as legal in no format) and may renumber or reword CR rules, which breaks
`cr!` citations until item C3 fixes them. The deferrals themselves (digital-only cards 855,
cards legal in no format about 2,050 by docs/DEFERRED.md's count, the 50 sticker sheets plus 24
cards, Contraptions and host/augment, dexterity cards 2, Conspiracy draft cards 5; 1,864
deferred ruling texts plus 662 out of scope) were reaffirmed on Nov 4 and are not re-asked: the
plan takes them as standing for "done".

**Recommendation.** Refresh the data after phase B has landed, so the new cards are supported
by a compiler that has stopped changing under them (item C3). "Done" (item C2) is measured on
the data of 2026-09-24 that the engine was built against; C3 then re-runs the incremental
refresh for the new cards.

### The user's answers (Nov 10)

- **D1:** follow the ruling. The user: "Thieving Amalgam and Grismold the Dreadsower are permanents YOU own, trigger from something happening to a creature your opponent owns as they leave the game. Whereas with Share the Spoils, the permanent WITH the trigger is the one that is leaving the game." It is not a discrepancy. Item user-decisions-nov10, part A.
- **D2:** postponed until the project is done. Review the log and report to Wizards then.
- **D3:** the user asked what "verified" means here (answered in chat on Nov 10); undecided, report the count meanwhile.
- **D4:** loops are an open action item. Build a robust system later (post-finish item C4); keep the current behavior until then.
- **D5:** the definition of done (§1) is accepted.
- **D6:** no data refresh until the project is in a settled state.
- **Also decided:**
  - Plasma Caster (`target-restriction-source-attachment-as-announced`): accepted.
  - Tarmogoyf under Dress Down (`star-without-defining-ability-is-zero`): accepted.
  - Stinging Study: the player chooses the commander (item user-decisions-nov10, part B).

### Revisit later (user, Nov 12)

Readings the user accepted for now, but wants to look at again before calling the project done:
- **Buyback and alternative-cost reductions.** The user: "I agree with the recommendations about buyback for now, but could discover that I am wrong about this in the future, so let's make sure it is noted somehow."
  - Entry `keyword-cost-reduction-on-an-additional-cost-within-it`: Memory Crystal is bounded by the buyback cost.
  - Entry `alternative-keyword-cost-reduction-reduces-additional-costs`: Catalyst Stone's reduction also comes off a buyback paid with flashback.
- **The gaps in returned-card values.** The user: "I accept the recommendations and will try to resolve the gaps later." These are in entry `returned-card-values-by-wording`:
  - "its power/toughness" after a graveyard return;
  - a card that never arrives (Containment Priest with Reanimate);
  - Deicide's searched name for a copied God.
- **Loops:** a robust system later (D4, Nov 10).

A question raised by a decision, not yet asked:
- **Torpor Orb and Panharmonicon on a reanimation** (entry `ability-triggered-cause-is-the-event`). The Nov 12 answer ("those are two separate things, and desecrated tomb should still trigger") was given for Strict Proctor. The statics of the same template still treat the zone change as one event. So Torpor Orb ("Creatures entering don't cause abilities to trigger.") stops Desecrated Tomb's ability when a creature card is put onto the battlefield from your graveyard, and Panharmonicon makes it trigger twice. Should they follow the same reading? Hushbringer's ruling on "from anywhere" abilities reads its "dying" the current way.

### Already decided, not re-asked

All 128 register entries (Oct 21, Nov 4, 5, 7 and 9); the Nov 5 conflict cases (template and
copying errors give way to the Oracle text; misattached rulings tested on the card they
describe; Gnoll War Band, Arbiter of Knollridge and Araumi set aside; Grip of Chaos); the
scope decisions (deferrals, paper cards counted, no new data until the user refreshes it);
the methods not chosen on Oct 14 (Forge/XMage comparison, a mutation gate, compiled-form
snapshots, set intake, long-tail cards as data, play-frequency priority); effort levels and
fresh sessions. This plan re-raises none of them. One note on the Forge/XMage comparison: it
is the only remaining oracle independent of both Wizards' words and our reading, and the
roadmap above does not need it; if phase A's yield has not fallen by the time A2–A4 are
exhausted, it is the next thing to consider.
