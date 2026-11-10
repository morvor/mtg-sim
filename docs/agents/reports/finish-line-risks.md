# Risk inventory for the finish-line plan (Nov 9, 2026)

Read by a subagent of the `finish-line-plan` planner from the docs at commit ad23a7a and `rg` checks of the engine; summarized in docs/agents/finish-line-plan.md §2. Kept here as the evidence behind that section.


Read-only reading of the docs at HEAD (Nov 9) plus `rg` checks of the engine. Sources are cited as
file:section. "Untested" and "wrong" are kept apart throughout: most of the evidence below shows the
first; the second is inferred from measured bug yields.

## 0. Baseline numbers (for the denominators)

| What | Value | Source |
|---|---|---|
| Fully supported cards | 32,076 of 32,111 (35 unsupported) | docs/CARD_COVERAGE.md |
| Verified (round-trip + structure + fuzz-clean) | 18,113 (56.4%) | docs/CARD_COVERAGE.md |
| Round-trip passing / mismatching / manual (unchecked) | 28,460 / 3,464 / 152 | docs/ROUNDTRIP.md header |
| Compiler bugs the round trip found (bug table rows) | 69 | docs/ROUNDTRIP.md "Compiler bugs" (counted) |
| Distinct structures exercised | 13,192 / 28,232 (46.7%); activated 33.3%, triggered 44.2%, mana 47.6%, spell 50.6%, static 61.1%, keyword 83.0% | docs/STRUCTURE_COVERAGE.md |
| Cards failing only the structure check | 11,522 | docs/CARD_COVERAGE.md |
| Mutants sampled / kill rate | 551 of 5,281 (every 10th); 82.1% before, 88.3% after new tests; 61 survivors left | docs/MUTATION.md "Results" |
| Rulings: unique texts cited | 1,558 of 34,000 (4.58%), 905 cards | docs/RULINGS_COVERAGE.md |
| CR rules cited / worked examples covered | 3,036 (95.9%) + 78 via subrules + 52 exempt, 0 uncovered; 277/277 examples | docs/CR_COVERAGE.md |
| Tests in crates/mtg-engine/tests | 28,410 `#[test]`s (rulings 1,857 files, cards 711, cr 343, keywords 226) | counted |
| Register | 128 entries: 85 interpretations, 9 deviations, 34 policies, 0 provisional | docs/INTERPRETATIONS.md |
| Judge audit | 58 tests, 225/225 questions agreed (error rate bound about 5%) | docs/agents/reports/judge-audit-1.md |
| Properties 1–4 latest | 2,000 games each, 0 violations; earlier runs found 7 seat-symmetry bugs, 0 from snapshot/rollback/determinism | docs/PROPERTIES.md "Results" |
| Properties 5–8 latest | copies 173,906 copies, payment 1.2M boards, damage 200k combats, hidden 2,400 games: 0 violations; first runs found 9+1+1+4 bugs | docs/PROPERTIES.md "Properties 5–8" |
| Every-card sweep | 32,656 cards (Nov 5); 0 panics/hangs; 95% abilities used; 9 engine problems found over all sweeps | docs/EVERY_CARD.md |

## 1. Status of the engine-gap inventory (G01–G33)

docs/agents/engine-gaps.md (base 85f020d) grouped 33 gaps into 14 work items; all 14 and
`gap-residual-1` (R1–R7) have landed. The gap-*.md files are briefs without result sections and
docs/agents/reports/ holds only three reports, so status was checked in the code with `rg`:

| Gap | Status at HEAD | Evidence |
|---|---|---|
| G29 open-ended repetition | landed | `Effect::RepeatProcess`/`RepeatThisProcess`, `tests/cr/r608_repeat_process.rs` (residual brief says Primal Surge, Ad Nauseam compile) |
| G22 X/X tokens | landed | `TokenSpec` doc "Power and toughness given by values" (ability.rs:2611) |
| G03 ability cost modification | landed | `ability_total_cost_with`, `CostChange` with Training Grounds floor (ability.rs) |
| G04 activation timing permissions | landed | `ActivationPermission { instant_timing, loyalty_per_turn, as_though_haste }` (ability.rs:3637) |
| G18/G31 play-grant choice | landed | permissions.rs `PermissionUse`, `choose_cast_permission` (mutation-tested) |
| G21 foretell faces | landed | kw/foretell.rs uses `casting::castable_faces` |
| G09 trigger detection at event time | landed (new) | trigger_timing.rs (only 17 mutants: a small file; "atomic action" boundaries) |
| G13 bushido X at resolution | landed | kw/bushido.rs `x_determined_on_resolution` |
| G30 first draw-step draw | landed | kw/draw_step_draws.rs, kw/first_draw_except_draw_step.rs |
| G06 target groups | landed | target_groups.rs (`group_ok`, `holds_on_resolution`; the latter had a bug fixed Nov 6 in ruling-conflicts-4) |
| G07 X-dependent target counts | landed | `count: Value` on target specs (ability.rs:509–532) |
| G08/G16 event causes | landed | `Event::CountersAdded { by, origin, as_put }`, `MoveCause`, event_causes.rs |
| G01/G02/G20 external spell costs | landed | spell_costs.rs `alternative_cost_allowed`, `spells_change_applies` |
| G28 Auras on cards in graveyards | landed | attach.rs:121 "Enchant creature card in a graveyard" |
| G23 "instead" destinations | landed | `ReplacementAction::MoveInstead(Destination)`, `RevealAndMoveInstead` |
| G24/G25 fabricate, empty shuffle | landed | kw/fabricate.rs doc; shuffle_rules.rs "even if none" (701.24d) |
| G10 each-player simultaneity | landed, weakest tested | simultaneous.rs `plannable` now includes `GainControl` (R1); mutation kill rate 55.6% |
| G05/G19 mana X limits, spend triggers | mostly landed | `ManaRestriction::{XCostsOnly, AbilitiesOnly, ...}`; `TriggerCond::ManaSpent` exists; whether spend triggers fire for abilities not verified here |
| G11/G12/G15 ability grants, hexproof from, linked referents | landed | `AddAbilitiesOf { terms: GainedTerms }` (R4), `HexproofFrom(Filter)`, `Sel::LinkedNoted`/`Effect::NoteLinked` |
| G14 rule statics | landed (8 pieces) | rule_statics/{cant_be_copied, counters_remain, cleanup_damage, sacrifice_causes, face_up, turns_taken, payment}.rs; "as though infect" is a manual card (cards/phyrexian_unlife.rs) |
| G17 combat permissions | landed | `Restriction::AttackAsThoughHaste`, "block as though untapped" (combat.rs:426), only the active player declares (ability.rs:1363) |
| G26 Grand Melee / range | landed | multiplayer/range.rs; the "known simplifications" comment in grand_melee.rs is gone |
| G27 commander eligibility | landed | commander_rules.rs "Cards from outside the game (CR 903.11)" |
| G33 ante | landed | ante.rs `exchange_ownership` |
| G32 small bugs | 1 of 3 verified | `copy::order_copies` (CR 405.3) exists; graveyard-size counting tokens (eval.rs:1623) and meld-on-a-copy (merge.rs:882 still `meld_pair(...).is_some()`) not verified: S-size each |
| R1–R7 residual | landed | `SeatSide` (R2), granted_by `GRANTER` (R3), `GainedTerms` (R4), `Value::PlayerTally` (R5), `PILE_C`/`PileAction` (R6), `CastInfo` permission (R7) |

**Still open as rules gaps, by the docs:** G32's two unverified bits (S each); the Charismatic
Conqueror growing loop (deviation `growing-loops-run-to-the-limit`, docs/EVERY_CARD.md); links of
abilities continuous effects grant are hashes that "could, very rarely" collide (docs/PROPERTIES.md
"Remaining gaps"; S); Inquisitor Eisenhorn: a card drawn while casting is revealed only after the
cast (CR 121.8) so the "this way" ability doesn't trigger then (roundtrip-tail-5 "Remaining"; a
behavioral deviation that is **not** in the register; S). The headline: the inventory's *missing
features* are closed; the remaining risk is features that exist and are wrong.

## 2. Ranked risk areas

Ranking weighs (a) measured bug yield of the methods that have looked there, (b) how much of the
area no method has looked at, and (c) how many cards it touches.

### R1. The 3,100 untriaged round-trip singletons (parser misreads that play wrong)
- **Evidence.** docs/ROUNDTRIP.md: 3,464 mismatching cards; the "Mismatch clusters" table's 200
  rows cover only 330 cards (20 `gap:` rows, 49 cards, are constructs the renderer can't word at
  all), so about 3,100 mismatches are unique differences nobody has read. The 69-row bug table came
  from triaging roughly 2,000–2,800 cards in the roundtrip-* items (each took 84–153 cards; pairs-5
  passed 65 of 151 taken, pairs-2 69 of 147): a yield of about 2.5–3.5% parser bugs per mismatching
  card. The remaining-section notes "plays as the text says unless noted" apply only to taken cards.
- **Why wrong, not just untested.** About 20 of the 69 bugs are referent misbindings ("it", "that
  player", "that card", "another"), another 15 are dropped clauses (reveal, "other", "unless",
  ownership "your graveyard"): systematic parser failure modes that recur on unusual text. Singletons
  are by definition unusual text. Expect 75–100 more parser bugs in the untriaged set.
- **Cheapest check.** Continue roundtrip-pairs-N (pairs-6 is queued) but behaviour-first, as its brief
  says; cheaper still, an automated probe for the dominant failure class: for every mismatching card
  whose AST binds a referent differently from the text's nearest antecedent (`roundtrip --ast`), list
  them first. Also the 152 manual cards are "not checked" by the round trip at all (R10).

### R2. "Each player" simultaneity and per-player referents (simultaneous.rs)
- **Evidence.** docs/MUTATION.md "simultaneous.rs": 55.6% kill rate after new tests (10/18), the
  lowest of 15 files; six survivors are real untested behaviour, not equivalents (lines 249 choice
  attachment, 283 `Sel::Union` choosers, 557 plannable `&&`, 628 links of each-player moves,
  1061/1081 per-player damage totals). The one engine bug mutation testing found was here
  (multi-searcher moves not simultaneous, CR 701.23i). Rulings batches keep finding bugs in the same
  area: P067 (a searcher inside "for each player" searched the wrong library; Opposition Agent
  across zones), P070 (Kefka one per opponent), P071 (Breeches one per damaged opponent, 603.2c),
  P061 (cards each player drew), P060 ("you and that player each draw", reversed by the user Nov 7).
  gap-residual-1 found the `vars::CREATED` per-frame pitfall and the control-change ordering bug by
  probing.
- **Why wrong.** Every probe into this file has found something; R1 (planned GainControl) landed days
  ago with no mutation run since.
- **Cheapest check.** A full (not 1/10) mutation run of simultaneous.rs: 171 mutants at about 2 min
  is 6 hours of machine time, then kill the (b) survivors; plus one rulings batch chosen for
  "each player ... their/that player's" texts (rulings_batches.py can select it).

### R3. Costs, X, casting permissions and the legal-action list (casting.rs, permissions.rs, spell_costs.rs)
- **Evidence.** The largest cluster of engine fixes in the 23 visible rulings batches (about 12 of
  about 45; see section 3): P005's three bugs (castability judged without continuous effects on the
  would-be spell; default X ignoring X-dependent costs and target counts; flashback offered when the
  only target is the card itself), P033 (X is 0 under an alternative activation cost; hybrid halves),
  P068/P069/P070 (free casts and additional costs, "an X the text defines is the only value offered",
  Primeval Spawn's total fitted, Jodah under a reduction), P071 (riders count when listing castable
  cards). casting.rs is the biggest file (841 mutants, 90% unsampled); permissions.rs is 78.6%.
  The payment property excludes exactly the hard part: "no cost changes, convoke, delve or improvise,
  no 'as though mana of any color', at most five sources" (docs/PROPERTIES.md "Remaining gaps").
  gap-activation (G03) rewrote ability costs recently with no mutation run since.
- **Why wrong.** Legal-action over/under-offering is invisible to every property (the fuzz only
  plays offered actions; rollback checks reversal, not that the offer was right) and to the round
  trip (the AST is right; the engine's cost arithmetic is not).
- **Cheapest check.** A legal-actions exactness property, run inside the every-card fuzz: every
  offered cast/activation must succeed when attempted with the engine's own payment, and a sample of
  non-offered casts (cards in hand with mana available) must fail for a reason the CR gives. Then
  extend the payment oracle with one cost reduction and one alternative cost.

### R4. Triggers: timing within resolutions, batching, reflexive and delayed triggers, EventInfo referents
- **Evidence.** triggers.rs 75% (36/48), second lowest; seven survivors delete `EventInfo` fields
  (Attacks objects, IsDealtDamage source, SpellCopied, TokenCreated player, TappedForManaOfType,
  Mills, chapter lki) that "no card was found" to read; line 269's look-back case (a source still on
  the battlefield that lost the ability between event and check) is unpinned. G09 (detection at
  event time, not after the whole resolution) was rebuilt in trigger_timing.rs this round: 17
  mutants, so a small mechanism carrying a lot of weight. Rulings fixes: P050 (a reflexive trigger
  after N counters triggers once per N; count used up before saving), P059 (Phelia's intervening
  condition belongs to the delayed return), P069 (Codie's trigger resolves first), P071 (Breeches,
  603.2c batching), Shirei's one-shot delayed trigger waited for a later end step (round-trip bug
  table, 603.7b), Bloodthirsty Adversary's reflexive copy order (603.12).
- **Why wrong.** Five independent methods (mutation, rulings, round trip, structure tests' vacuous
  "doesn't trigger" lesson, gap work) each found trigger-timing bugs in Oct–Nov; the area changed
  structurally since the mutation sample.
- **Cheapest check.** engine-audit-1 area 3 (brief exists, not yet run), then re-sample triggers.rs
  and trigger_timing.rs with `RERUN=1` plus a fresh 1/10 shard.

### R5. Last known information, new objects (CR 400.7), and operations on departed objects
- **Evidence.** Rulings fixes: P042 (abilities of an object that left its zone could be activated),
  P047 (target checks used what an Equipment *was* attached to), P008 (cost-discarded cards into a
  hidden zone), P022 (links to a creator that left), P069 (Gonti's card put back is a new object;
  Reanimate's life loss from the graveyard's mana value, 608.2h), P077 (exiled card's power from the
  graveyard). Fuzz: Sight Beyond Sight's stale library id (docs/EVERY_CARD.md), `Game::tap` tapping a
  permanent that had left the battlefield. Mutation: actions.rs survivors are equivalents only, but
  the 400.7 argument ("an object that left has a new id, so its old id is never live") is used to
  dismiss two survivors, which is only as good as every caller following `game.current(id)`.
- **Why wrong.** Two of the nine fuzz-found engine bugs and six rulings fixes are stale-id or LKI
  bugs; the invariant is enforced by convention (CLAUDE.md "Key invariants"), not by the type system.
- **Cheapest check.** Debug-assert in the actions.rs primitives (tap, untap, counters, damage,
  attach, move) that the target id is live or the primitive is explicitly LKI-aware, and run the
  every-card sweep with assertions on (one chunk, 40 min); plus engine-audit-1 area 4.

### R6. Replacement and prevention effects (replacement.rs, prevention.rs, draw_rules.rs)
- **Evidence.** 82.4% kill rate (28/34) with no new tests able to kill its survivors; line 1207
  (prevention key loses the player: two players' shields of one kind in one event) unpinned. Bugs
  found by other methods: Equal Treatment's clamp (round-trip table), RevealAndMoveInstead dropping
  the reveal (Darksteel Colossus et al.), Vesuva entering tapped when it copied nothing, Nephalia
  Academy; rulings P040 (Stagger from any source), P043 (shields divided by recipient), P060 (a draw
  another player made instead; a replaced draw's "drew this way"), P067 (which Agent's effect applies
  = 616.1 choice). Properties: the 614.12 `EnteringView::controller_override` survivor is "equivalent
  now" only because of a later refactor (`with_hypothetical_entry`).
- **Why wrong.** Replacement bugs are "wrong but consistent": every property passes, the fuzz's
  ledger reconciles life and counters against the events that *were* emitted, so a wrongly replaced
  event is self-consistent. Only rulings and round trip see them, and both keep finding them.
- **Cheapest check.** engine-audit-1 area 2; a rulings batch filtered to "instead"/"prevent"
  texts; a full mutation run of replacement.rs (361 mutants, about 12 h) is the expensive option.

### R7. Copies and links (copy.rs, ability_grants.rs, layers::acquired_ability)
- **Evidence.** Properties 5 found nine bugs, all links or copy exceptions (docs/PROPERTIES.md "Bugs
  found": Callidus Assassin exceptions, Ixidron timing, Tyrannical Pitlord, Gelatinous Cube, Identity
  Thief, Parallel Thoughts, Smuggler's Buggy, the ante card). Remaining gaps: copies of activated and
  triggered abilities aren't covered; spell-copy resolution is compared only for permanent spells;
  planeswalkers and battles are copied only as tokens; links of continuously granted abilities are
  hashes into 0x8000–0xbfff that can collide. Rulings P022 (unflipped copy of a flipped flip card),
  P029 (copy-target legality registered); layers.rs survivor 2018 (205.3d on a noncreature copy)
  unpinned; register `copy-exception-ability-stacks` (Gigantoplasm twice) was a medium-confidence
  judge call.
- **Why wrong.** The property found 9 bugs on its first run over exactly the paths it covers; the
  uncovered paths (ability copies: Strionic Resonator, Rings of Brighthearth, Lithoform Engine;
  spell copies resolving) have had no equivalent scrutiny.
- **Cheapest check.** Extend `properties::copies` to copy every triggered and activated ability of
  every card once (compare the copy's effects with the original's resolution on a cloned game) and
  to compare a non-permanent spell copy's resolution with the original's. Replace hashed links with a
  per-object counter (S).

### R8. Layers (CR 613) with copy, type and ability changes; the dependency shortcut
- **Evidence.** layers.rs 87.5% with no killable survivor, eval.rs 100%: well tested where sampled.
  But the only reason the Nov 9 engine-audit brief lists layers first is that no rulings-batch fix in
  the 23 visible batches touched layers, and the fuzz found "layer dependency analysis was cubic" and
  fixed it with a shortcut ("the earliest effect applies at once when it depends on nothing": a
  performance special case inside 613.8 ordering). The changeling-CDA decision changed every
  creature-type reader (structure-tests report section 4). No property checks that characteristics are
  independent of the order statics are collected when no dependency exists.
- **Why wrong vs untested.** This is "no evidence because well tested" on the sampled paths, and
  "no evidence because nobody looked" on dependency loops, 613.7 timestamps after control changes and
  copy effects that change types. Lower than R1–R7 on evidence; the register has a *deviation* here
  (`room-door-unlock-timestamp`).
- **Cheapest check.** A property: recompute characteristics on a copy with the active-statics list
  shuffled before dependency analysis; results must agree (CR 613.8 says order matters only through
  dependencies). engine-audit-1 area 1.

### R9. Multiplayer variants the fuzz never plays (Grand Melee, Emperor, Two-Headed Giant, range of influence)
- **Evidence.** docs/EVERY_CARD.md game kinds: standard, commander, planechase, archenemy, vanguard,
  conspiracy, attractions, dungeon: no 2HG, Emperor, Grand Melee or range of influence. Properties 1–4
  use 2–4 player free-for-all only. Seat symmetry found 7 bugs, all in multiplayer edge paths
  (players leaving together, subgames, APNAP defaults). G26 was rewritten (range.rs) and R2 added seats
  this round; the rulings that exercise them are few (Arbiter of Knollridge set aside as
  inconsistent; Mystic Barrier, Pramikon, Inniaz "leftmost player on each team").
- **Why wrong.** Rewritten code, no randomized play, rulings sparse: untested rather than shown
  wrong, but the per-probe hit rate in multiplayer has been high.
- **Cheapest check.** Add `--kind 2hg|emperor|grand-melee` to the properties fuzz (seat symmetry and
  the invariant checks already exist) for a few hundred games each.

### R10. Hand-written card abilities (152 cards) and the equivalence list
- **Evidence.** docs/ROUNDTRIP.md: 152 cards "with hand-written abilities, not checked". manual-check
  guarantees each has a test, but the test encodes its author's reading (the judge audit's limit:
  "the same kind of agent"). compare.rs `EQUIVALENCES` has about 150 rewrite rules plus named
  approximations ("cycle or discard" compiled as discard, justified by 702.29d); each is a place a
  real difference could be normalized away.
- **Why wrong.** No external check at all on the 152; a wrong manual definition stays verified.
- **Cheapest check.** A judge-audit pass over the 152 manual tests (about 1 hour at the measured
  18 min / 60 tests), and a one-session read of the EQUIVALENCES table against the CR.

### R11. Structure coverage: "exercised" is not "asserted", and 15,000 one-to-two-card structures
- **Evidence.** docs/STRUCTURE_COVERAGE.md: 46.7% of structures exercised; the most-shared
  unexercised structure has 4 cards; activated abilities 33.3%. "Exercised" means the code path ran
  in some test, including random pools of property and API tests (Veil of Assimilation was verified
  by a random pool and demoted when the pool changed, verified-refresh-bugs.md). One-shot skip effects
  are never logged (structure-tests report, open gap 4), so a structure can count as exercised with
  no test having observed the step skipped. The structure-tests stream's measured yield was 1 engine
  bug (changeling tokens lacking the CDA) in 600 tests (about 0.2%), far below rulings (about 1–2%).
- **Why untested rather than wrong.** Low yield suggests shared structures are right; but the 11,522
  cards blocked only by structure are mostly 1–2-card structures whose text is, like R1's singletons,
  unusual. Expect 20–40 bugs among them at the structure-test yield, at a cost of about 15,000 tests.
- **Cheapest check.** Not more structure tests. Instead a generic exhaustive property in the style
  of copies-dynamic: for every card, activate each activated ability once with legal choices and
  check (i) the cost was paid exactly (ledger), (ii) at least one event of the effect's kind was
  emitted, (iii) the ability's "if you do"/result flags are consistent. That catches the "resolves
  but does nothing" class that structure logging counts as exercised.

### R12. Loop detection and game-ending rules (game_end.rs)
- **Evidence.** verified-refresh-bugs.md: `check_mandatory_loop` was broken after any forced choice
  (Hostage Taker chains never drew); the fix then over-drew loops a player could stop (Soltari Foot
  Soldier case) and was narrowed again in review. Charismatic Conqueror runs to the action limit
  (deviation `growing-loops-run-to-the-limit`). Draw rates measured: 0 of 100, 1 of 300.
- **Why wrong.** Two bugs in one week in one function; the CR's 104.4b/732 criteria are heuristic
  here by necessity.
- **Cheapest check.** Low card impact; keep the r104 tests, add the draw-rate line to the properties
  run output, and register any further heuristic change.

### R13. Hidden information in the agent protocol
- **Evidence.** Property 6 found 4 bugs (command-zone deck ids, conspiracies, request ids, Petty
  Larceny); P068 fixed face-down exiled cards being offered to players who may not look. Remaining
  gap: "cards a decision shows from a library are taken to be legitimately shown; a decision that
  wrongly showed another player's library card wouldn't be caught".
- **Cheapest check.** In `hidden::check_game`, compare the set of library cards a decision shows with
  the cards the resolving effect's text entitles the player to see (searches: 701.23; looks: 701.20);
  one session.

### R14. Judgment calls decided without the user, and behavioural notes not in the register
- **Evidence.** docs/INTERPRETATIONS.md: 0 provisional entries, 9 deviations. Workers exempt and
  decide on their own under policy (Stone Calendar "exempt as outdated on our own", ruling-conflicts-4).
  The user reversed one engine decision when asked (user-questions-nov7 Part B: "you and that player
  each draw" is one combined instruction, the opposite of what rulings-P060 implemented and tested).
  Inquisitor Eisenhorn's 121.8 behaviour (roundtrip-tail-5) is a knowing deviation with no register
  entry. The judge audit's cr-04 and rulings-18 medium-confidence calls were to be registered.
- **Why wrong.** One of two user reviews reversed a tested behaviour: the error rate of worker
  judgment calls is not small, and 128 of them are "decided".
- **Cheapest check.** The user reads the 9 deviations and the 10 most consequential interpretations
  (finish-line-plan item 4 asks for this list); grep every roundtrip-* "Remaining" for "unless noted"
  cases and register them.

### R15. Rulings coverage itself: 95% of ruling texts untested
- **Evidence.** 1,558 of 34,000 unique ruling texts cited; worker.md measures "about one real engine
  bug per 100 rulings, almost always in interactions". The 23 visible batches' commit subjects show
  about 45 engine-behaviour fixes (section 3), i.e. the rate may be nearer 2 per 100 when parser-level
  fixes are counted.
- **Why wrong.** At 1–2 per 100, the 32,000 uncited texts imply 300–600 engine bugs findable this way,
  concentrated in the interaction areas above. This is the only method with a measured, repeatable
  yield on *semantics*, which is why it ranks the areas above.
- **Cheapest check.** It is the check; prioritize batches by the areas R2–R7 (rulings_batches.py
  groups similar rulings).

## 3. Bugs the rulings batches found, by area (from commit subjects; only 23 batches are in the
visible history, which is 196 commits)

No per-batch report files exist (docs/agents/reports/ has three files), so this is read from
`git log` subjects of rulings-P008…P087 plus the p005 brief; counts are approximate (about 45
engine-behaviour fixes, excluding citation and test-sharpening commits):

| Area | About | Examples |
|---|---:|---|
| Costs, X, casting permissions, legal-action listing | 12 | P005 ×3, P033 ×2, P068 free casts, P069 Primeval Spawn/Jodah, P070 defined X, P071 riders, P043 morph options |
| LKI / new objects / departed objects | 8 | P042, P047, P008, P022 creator left, P069 Gonti and Reanimate, P077 exiled card's power |
| Each-player / per-player referents and simultaneity | 6 | P067 ×2, P070 Kefka, P071 Breeches, P061 Communal Brewing, P060 named twice |
| Triggers (timing, reflexive, delayed, batching) | 6 | P050 ×2, P059 Phelia, P069 Codie, P071 603.2c, P077 Teysa/Drivnod |
| Replacement / prevention / draw replacement | 5 | P040 Stagger, P043 shields per recipient, P060 ×2, P067 616.1 choice |
| Targets and illegal targets | 5 | P047 target_legality, P061 Haktos, P071 Undercity Uprising, conflicts-4 "different players" |
| Copies and faces | 4 | P022 flip copy, P029, P068 Kaya's emblem per face, P069 faces kept |
| Combat requirements and costs | 2 | P048 Iron Golem, P059 forced blocks |
| Hidden information | 1 | P068 face-down exile |
| Layers / characteristics | 0 | none in the visible batches |

Areas with zero hits (layers, SBAs, zones proper, mana abilities) are either well tested (eval.rs
100%, sba.rs survivors equivalent, CR examples 100%) or not where the batches went; the batch topics
visible (draws, exile permissions, forced attacks, D&D, mechanic hate) did not aim at layers.

## 4. Which verification method finds which kind of bug

| Method | Kinds of bug it has actually found (count) | Blind to | Measured yield |
|---|---|---|---|
| Rulings batches (ruling → TestGame) | Interaction semantics: cost/X arithmetic, LKI, per-player referents, replacement choice, trigger timing, illegal targets (~45 in 23 batches) | Anything no one wrote a ruling about; a worker's misreading encoded in the test (one reversed by the user Nov 7) | ~1–2 engine bugs / 100 rulings |
| Oracle round trip (AST → text, compare) | Parser misreads: referents ("it", "that player"), dropped clauses (reveal, "other", "unless"), ownership ("your graveyard"), scope of "this way", zone of a trigger (69) | Engine semantics once the AST is right; the 152 manual cards; differences absorbed by EQUIVALENCES | ~2.5–3.5% of triaged mismatching cards |
| Structure tests (one real card per shared structure) | Compiler omissions on shared constructs (changeling token CDA), coverage-logging gaps (1 engine bug, 3 logging) | Single-card structures; outcomes nobody asserted ("exercised" ≠ asserted) | ~0.2% per structure |
| Mutation testing (1/10 sample, 15 files) | Missing tests on core paths: payment backtracking (Compleated), answer validation panics, requirement-search fallbacks, timing windows; 1 engine bug (simultaneous search) | 90% of mutants never run; semantics that look equivalent "for today's cards" | 1 engine bug + ~30 real missing tests / 551 mutants |
| Metamorphic properties 1–4 (seat, snapshot, rollback, determinism/replay) | Seat-order leaks: APNAP defaults, players leaving together, subgames (7); shared-state/cloning (0) | Any rule applied wrongly but consistently | 7 bugs in first 2,300 games, 0 since |
| Exhaustive properties 5–8 (copies, hidden, payment, damage) | Links and copy exceptions (9), pain-land/Phyrexian ordering (1), trample lethal sharing (1), hidden ids (4) | Outside their catalog: ability copies, cost changes, prevention in combat, 3+ players | 15 bugs on first runs, 0 since |
| Every-card fuzz with invariant checks | Stale state (characteristics not recomputed, stale ids), tapping departed objects, unending loops, performance cliffs, setup bugs (9) | Wrong-but-consistent behaviour: wrong amounts, targets, timing; anything the ledger reconciles | 9 over ~5 sweeps |
| Blind judge audit | Wrong test assertions (0 of 225) | Shared misreadings; rulings tests hand the judge the answer | bounds test error ≤5% |
| Conflict investigations / user questions | Wrong engine calls on contested rulings (Run Away Together compare fix; "named twice" reversal) | Everything not contested | ~1 per case set |
| Gap probes (compiler probe + throwaway engine test) | Features that compile but never work (Leonin Bola family), per-frame variable loss, control-change ordering | Needs a hypothesis | 3 of 7 residual gaps were found this way |

## 5. Summary judgment

- The engine-gap inventory is essentially closed as a *feature* list (every G and R construct
  exists in code); the two G32 bits and Inquisitor Eisenhorn are the only documented open rules gaps,
  all S.
- The methods with zero recent findings (properties 1–8, fuzz) are consistency and crash checks; their
  clean runs say little about semantics. The two methods that find semantic bugs (rulings, round trip)
  keep finding them at a steady rate, so the honest estimate is: about 75–100 parser bugs in the
  untriaged round-trip singletons, several hundred interaction bugs reachable through the 95% of
  rulings not yet tested, concentrated in costs/X/permissions, each-player simultaneity, LKI,
  triggers and replacement, and 5–10 core-file bugs implied by the unsampled 90% of mutants
  (1 bug per 551 sampled).
- "Well tested, no evidence of bugs": eval.rs, sba.rs, stack.rs, turn.rs, casting.rs's sampled tenth,
  the CR worked examples, seat/APNAP ordering, copy values of permanents, 2-player combat damage.
- "Untested, no evidence either way": each-player simultaneity's unkilled survivors, ability copies,
  cost changes in payment, 2HG/Emperor/Grand Melee play, the 152 manual cards, 15,000 single-card
  structures, the EQUIVALENCES list.
