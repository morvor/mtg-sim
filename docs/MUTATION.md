# Mutation testing of the core engine

Mutation testing changes engine code on purpose (flip a comparison, drop a statement,
return a default value) and checks that some test fails. A mutant that survives marks
behavior that no test pins down. This report covers the core rules files with
[cargo-mutants](https://mutants.rs/) against the `mtg-engine` test binaries
(the unit tests plus `actions`, `cards`, `cr`, `keywords`, `rulings`, `smoke`).

## How to repeat the measurement

```sh
cargo install cargo-mutants --locked          # v27.1.0 was used
export CARGO_BUILD_JOBS=2
scripts/mutants.sh sba trigger_timing ...     # one file at a time, results in target/mutants/<stem>/
RERUN=1 scripts/mutants.sh sba                # re-test only that file's survivors (target/mutants/sba-rerun/)
```

`scripts/mutants.sh` runs, for each file stem, under the global build lock
(`flock /tmp/cargo-heavy.lock`):

```sh
cargo mutants --in-place -d target/mutsrc -f crates/mtg-engine/src/<stem>.rs \
  --timeout 600 --build-timeout 900 --cap-lints true --test-package mtg-engine \
  --shard 0/10 --sharding round-robin -o target/mutants/<stem>
```

(`TIMEOUT=<seconds>` overrides the 600 s test timeout. It was 300 s until November 2026,
when the unmutated `mtg-engine` suite, ~105 s in October, had grown to ~290 s: cards 66 s,
cr 49 s, properties 42 s, rulings 109 s, plus the unit, actions, keywords and smoke binaries.
A baseline that doesn't fit the timeout means no mutant is tested at all.)

- `target/mutsrc` is a copy of the committed tree (`git archive HEAD`, so commit the tests
  first; uncommitted changes aren't measured) refreshed before each file, so one
  incremental build serves every mutant and an interrupted run never leaves a mutation in
  the real tree. `--in-place` there also means one mutant at a time.
- A whole run over the 15 files would be about 5,200 mutants at roughly two minutes each
  (an incremental build of the engine and its seven test binaries takes ~50 s, the full
  `mtg-engine` suite ~105 s), i.e. a week. So each file is **sampled**: round-robin shard
  `0/10` takes every tenth mutant in source order, which spreads the sample evenly across
  the file. Files with fewer than 60 mutants run in full.
- `--timeout 600` (300 before November 2026) bounds a mutant's test run (the unmutated
  suite took ~105 s in October and ~290 s in November), so a mutant that makes the engine
  loop counts as `timeout`, which is a detection.
- The re-run after adding tests (`RERUN=1`) finds each survivor again in the file's
  current mutant list (same function and mutation whose diff, the mutated line with the
  lines around it as saved in the earlier run's `mutants.out/diff/`, matches best; then
  the same column and the nearest line, since merges move lines) and tests exactly those.
  Matching by line distance alone picked a neighbouring identical
  mutation once code had moved by more than the distance between them. cargo-mutants
  27.1 applies neither `-F` nor `-E` to "delete field" mutants (it selected every one of
  them, 163 for triggers.rs), so the re-run applies each survivor's diff to
  `target/mutsrc` itself and runs the same build and `cargo test -p mtg-engine` (the same
  timeout), recording caught/missed/timeout/unviable in `target/mutants/<stem>-rerun/`.
  `ONLY=<regex>` re-runs just the matching survivors, keeping the others' results.

Kill rate = (caught + timeout) / (caught + missed + timeout); unviable mutants (that do
not compile) are excluded.

## Triage of survivors

Each surviving mutant is classified as

- **(a) equivalent**: the change cannot alter behavior (the reason is given);
- **(b) missing test**: behavior the CR or a card needs that no test checked; a test that
  kills it was added (named in the entry);
- **(c) engine bug**: fixed with a regression test.

## Results

| File | Mutants (all) | Sampled | Caught | Missed | Timeout | Unviable | Kill rate | After new tests |
|---|---|---|---|---|---|---|---|---|
| sba.rs | 154 | 16 (1/10) | 14 | 2 | 0 | 0 | 87.5% | 87.5% (both survivors equivalent) |
| trigger_timing.rs | 17 | 17 (all) | 14 | 2 | 0 | 1 | 87.5% | 93.8% (15/16) |
| permissions.rs | 169 | 17 (1/10) | 10 | 4 | 0 | 3 | 71.4% | 78.6% (11/14) |
| simultaneous.rs | 171 | 18 (1/10) | 6 | 12 | 0 | 0 | 33.3% | 55.6% (10/18) |
| turn.rs | 234 | 24 (1/10) | 21 | 2 | 0 | 1 | 91.3% | 95.7% (22/23) |
| mana.rs | 228 | 23 (1/10) | 15 | 6 | 0 | 2 | 71.4% | 81.0% (17/21) |
| eval.rs | 268 | 27 (1/10) | 25 | 0 | 0 | 2 | 100% | 100% |
| stack.rs | 361 | 37 (1/10) | 33 | 3 | 0 | 1 | 91.7% | 94.4% (34/36) |
| replacement.rs | 361 | 37 (1/10) | 28 | 6 | 0 | 3 | 82.4% | 82.4% (28/34; its two tests' mutants are equivalent now) |
| triggers.rs | 474 | 48 (1/10) | 32 | 16 | 0 | 0 | 66.7% | 75.0% (36/48) |
| triggers.rs (2026-11-11, after the triggers audit; 17 of 55 sampled reached when written, see below) | 541 | 55 (1/10) | 14 | 3 | 0 | 0 | 82.4% | 82.4% (all three survivors equivalent) |
| resolve.rs | 314 | 32 (1/10) | 22 | 8 | 0 | 2 | 73.3% | 86.7% (26/30) |
| actions.rs | 390 | 39 (1/10) | 27 | 5 | 3 | 4 | 85.7% | 85.7% (no new tests: no killable survivor found) |
| layers.rs | 423 | 43 (1/10) | 34 | 5 | 1 | 3 | 87.5% | 87.5% (no killable survivor) |
| layers.rs (2026-11-10, after the layers audit; all 53 tested, see below) | 524 | 53 (1/10) | 45 | 6 | 2 | 0 | 88.7% | 88.7% (no killable survivor) |
| casting.rs | 841 | 85 (1/10) | 74 | 9 | 0 | 2 | 89.2% | 96.4% (80/83) |
| casting.rs (2026-11-10, after the casting-costs audit; all 105 tested, see below) | 1049 | 105 (1/10) | 87 | 16 | 1 | 1 | 84.6% | 89.4% (93/104): the 5 killable survivors and the timed-out mutant caught in the re-run, 11 equivalent |
| combat.rs | 876 | 88 (1/10) | 68 | 13 | 0 | 7 | 84.0% | 93.8% (76/81) |
| zones.rs (2026-11-12, during the zones-lki audit; all 29 sampled tested, see below) | 288 | 29 (1/10) | 19 | 6 | 1 | 3 | 76.9% | 84.6% (22/26): the 2 killable survivors killed by review tests, each checked with the mutant applied by hand (no `RERUN` yet); 4 equivalent |
| **all 15** | 5,281 | 551 | 423 | 93 | 4 | 31 | **82.1%** | **88.3%** (459/520) |

"After new tests" re-tests the first run's survivors (`RERUN=1`, 2026-10-21, on the tree
with all the new tests): each survivor now caught counts as caught. Some were killed by
tests other work added in the meantime (noted in their entries); five whose tests were
written for them turned out to be equivalent in today's code and are re-triaged below.

### sba.rs

Run 2026-10-14, 16 of 154 mutants (round-robin 0/10), 23 minutes.

- `sba.rs:293:34: replace || with && in Game::check_sbas` (704.5p, `!(is_aura || is_equipment
  || is_fortification)` becomes `!(is_aura && is_equipment || is_fortification)`): **(a)
  equivalent**. The two differ only for a permanent with the Aura subtype that is not an
  enchantment, not a creature and not a battle (Auras that are enchantments take the branch
  above). The rules do not allow that object: Aura is an enchantment subtype, an object can't
  gain it without being an enchantment (CR 205.3d), and an object that stops being an
  enchantment loses it (CR 205.1a); `layers.rs` enforces both.
- `sba.rs:369:26: replace > with >= in Game::check_sbas` (704.5z roles, `ids.len() > 1`):
  **(a) equivalent**. With exactly one Role the mutant sorts the one-element list, pops it
  and puts nothing into the graveyard, the same result as skipping it.

### trigger_timing.rs

Run 2026-10-14, all 17 mutants, 27 minutes.

- `trigger_timing.rs:55:9: replace <impl Clone for TriggerTiming>::clone -> Self with
  Default::default()`: **(b) missing test**. A copy of the game must keep the count of
  actions in progress (a snapshot taken inside an atomic effect, such as a cast attempted
  while an effect resolves, is restored by `roll_back` when the cast is illegal), while the
  "triggers are being detected" flag is the original's own. No test reaches a rollback inside
  an atomic action followed by a trigger check in the same action, so the contract is pinned
  by the unit test `trigger_timing::tests::a_copy_keeps_the_actions_in_progress_but_not_the_check_in_progress`.
- `trigger_timing.rs:95:13: replace > with >= in creates_tokens_together`: **(a)
  equivalent**. It makes a one-instruction `Seq` of a token creation count as one atomic
  action instead of a sequence of one action. Either way the events before it are checked at
  the boundary before the `Seq`, the creation runs as one action, and its events are checked
  at the next boundary.

### permissions.rs

Run 2026-10-14, 17 of 169 mutants (round-robin 0/10), 23 minutes.

- `permissions.rs:69:9: replace <impl PartialEq for CastPermission>::eq -> bool with true`:
  **(a) equivalent**. Nothing compares two `CastPermission`s: `CastOption`, the only type
  holding one, has no `PartialEq`, and no code uses `==` on permissions.
- `permissions.rs:425:38: replace match guard matches!(**s, Sel::This) with true in
  spell_relative_cost`: **(a) equivalent with today's cards**. The guard rewrites "life
  equal to its mana value" in a permission's cost to the spell's mana value; the mutant also
  rewrites the mana value of any other object. Every permission cost in the card pool that
  mentions a mana value means the spell's own ("its mana value", Bolas's Citadel;
  "the spell's mana value", Bismuth Mindrender, which
  `tests/rulings/r_s22_bismuth.rs` covers), so no `ManaValueOf` with another selector
  reaches it.
- `permissions.rs:497:39: replace - with / in choose_cast_permission` (`i.min(ways.len() -
  1)` becomes `i.min(ways.len())`): **(a) equivalent**. The clamp only matters for an index
  out of range, and both `choose` and `Game::ask_option` already map invalid answers to a
  valid default.
- `permissions.rs:541:46: replace match guard i < perms.len() with true in choose`: **(b)
  missing test**. An agent's out-of-range answer to "which permission" was accepted (the
  cast then fell to the last permission; playing a land that way would index out of
  bounds). Killed by `tests/cr/r601_003_permission_choice_answers.rs`
  (Muldrotha's artifact and creature permissions for Ornithopter, answer `Index(7)`: the
  default, the artifact permission, is used).

### simultaneous.rs

Run 2026-10-14, 18 of 171 mutants (round-robin 0/10), 40 minutes. The weakest file so far:
the "each player" machinery is exercised by many card tests, but few of them distinguish
"performed for all players at once" from "performed for each player in turn", or one
player's results from another's.

- `simultaneous.rs:100:31: replace || with && in run` (stop performing the body's
  instructions once the game is over): **(a) equivalent**, found by the re-run (the test
  written for it, `tests/cr/r104_001_game_ends_during_each_player.rs`: Laboratory Maniac
  wins the game while each player draws for Burning Inquiry and nobody discards
  afterwards, CR 104.1, passes with the mutant). `Game::exec` performs nothing once the
  game has ended or is restarting, so the instructions after the one that ended it do
  nothing whether or not the loop stops; the test stays as the CR 104.1 check. The
  planned instructions (`perform_plans`: sacrifices, discards, moves chosen by every
  player and performed together) don't go through `Game::exec`, but a second test added in
  review (Lore Broker, "each player draws a card, then discards a card": a chosen discard
  after the winning draw) also passes with the mutant, so the claim holds for the cards
  tried.
  **Overturned in the second round** (`149:31` in the Nov 2026 full run, see "Second round,
  Nov 2026"): **(b)**. Planned instructions don't go through `Game::exec`, and a planned
  instruction that asks nothing (a sacrifice of "all creatures they control") is performed
  after the game has ended; the Lore Broker discard was skipped only because the draw that
  won had already ended the game before anyone was asked. Killed by
  `a_win_while_each_player_draws_ends_the_game_before_anyone_sacrifices` in the same file.
- `simultaneous.rs:224:20: replace == with != in updates_number::reads`: **(b) missing
  test, not killed**. The mutant makes every instruction that stores a number count as one
  whose number is shared by all players. The cards whose per-player "that many" goes
  through this code record it per player anyway (`RECORD_THAT_MANY`,
  `kw/hand_graveyard_actions.rs`), and the other per-player numbers seen in the card pool
  (the outer X of "each player ... X") hold the same value for every player, so no card
  test found shows the difference. The re-run (2026-10-21) found it **killed** by tests
  added since by other work.
- `simultaneous.rs:249:17: replace && with || in records_decision`: **(b) not killed**.
  It turns "if ..., [action]" inside an "each player" body into a choice attached to the
  next instruction; the actions still happen in the same order for each player, and only
  the trigger checks between the two instructions move (to after the second one). No
  observable case was found with today's cards.
  **Overturned in the second round** (`314:17` in the Nov 2026 full run): **(b)**. A
  trailing "if" goes into the previous instruction's `post`, which interleaves the players
  (Kwain, Itinerant Meddler: draw P0, gain P0, draw P1, ... instead of all the draws
  first), and a single-"if" body is performed in turn instead of planned (Borderland
  Explorer's searches stop being simultaneous). Killed by
  `kwain_every_player_draws_before_any_player_gains_life`
  (`tests/cards/followups_each_player.rs`) and
  `borderland_explorer_players_discard_then_search_together`
  (`tests/rulings/r_s33_each_player_searches.rs`).
- `simultaneous.rs:262:9: delete match arm Effect::Choose{..} in makes_choice`: **(b)
  missing test**. A "choose a creature type" became its own instruction, so all players
  chose before anyone acted and every player's action read the last choice. Killed by
  `tests/cr/r101_004_each_player_chooses_then_acts.rs` (Grave Sifter: each player returns
  cards of the type *they* chose).
- `simultaneous.rs:283:9: delete match arm Sel::Union(v) in choosers`: **(b) not killed**:
  needs an each-player move of "this and that" where another player chooses one of the
  parts; no such card was found.
  Second round (`349:9`): killed by
  `highcliff_felidar_no_opponent_is_recorded_as_choosing` (`r_p138_apnap_choices.rs`): the
  Union's chooser is the resolving player, whose choice for each opponent was recorded as
  the opponent's.
- `simultaneous.rs:552:5: replace plannable -> bool with true`, `557:41: replace && with
  || in plannable` (an "if" counts as plannable when only one branch is): **(b) not
  killed**. They plan actions whose choices another player makes (or "if/otherwise" with an
  unplannable branch) together instead of in turn. With the cards tested, the planned and
  the in-turn performance produce the same results; a test needs a card where one player's
  part depends on what an earlier player's part already did, chosen by a third player.
  The re-run (2026-10-21) found `plannable -> true` **killed** by tests added since by
  other work; the `&&`/`||` one still survives.
  Second round (`700:41`): **(c)**, HEAD was the one breaking CR 101.4 (an "if" after a
  choice was performed in turn, so the conditions weren't settled first and the sacrifices
  weren't simultaneous); fixed by making every "if" plannable, see "Second round".
- `simultaneous.rs:664:46: replace match guard plannable(g, e, ctx, p) with false in plan`
  (sacrificing chosen objects, "sacrifices the rest", in turn instead of together):
  **(b) not killed**. Simultaneous sacrifice is observable with a "whenever another
  creature dies" permanent sacrificed in the same event as another player's creature (it
  sees that death only if they leave together). The cards tried compile to their own
  effects instead of this path (Cataclysm is `KeepAndSacrificeRest`, Balance is
  unsupported); the behavior itself is now pinned for Cataclysm by
  `tests/cr/r608_002f_each_player_sacrifices_together.rs` (Blood Artist sees the death of
  the opponent's creature sacrificed with it). The re-run (2026-10-21) found it
  **killed** by tests added since by other work.
- `simultaneous.rs:628:38: replace == with != in plan` (link the moved permanents to the
  effect's source when they enter the battlefield, CR 607.1d): **(b) not killed**: no card
  puts a permanent onto the battlefield with an "each player" move whose abilities refer to
  the card linked to the object that put it there.
  Second round (`948:38`): killed by the hand-built
  `creatures_each_player_puts_onto_the_battlefield_are_put_there_with_the_source`
  (`tests/cr/r608_002f_planned_actions.rs`, CR 607.2c).
- `simultaneous.rs:766:17: replace && with || in plan` (a search that shuffles is performed
  in turn instead of planned): **(a) equivalent for the cards that reach it**. The
  "each player may search ... put onto the battlefield, then shuffle" cards (Veteran
  Explorer, Old-Growth Dryads) find basic lands, which make no choices as they enter, so the
  lands entering one player at a time inside one batch looks the same.
  Investigating it found a real bug in the other multi-player search path, see below.
  **Overturned in the second round** (`1090:17`, with `1090:28`, `1091:17` and `1086:14`
  → `false`): **(b)**. Performed in turn, each opponent searches, puts the card onto the
  battlefield and shuffles before the next one searches: the shuffles interleave (Old-Growth
  Dryads, `old_growth_dryads_opponents_search_then_shuffle_in_turn_order`,
  `r_s33_each_player_searches.rs`) and a Clone one opponent finds can copy a creature another
  found (Boldwyr Heavyweights,
  `boldwyr_heavyweights_a_clone_one_opponent_finds_cant_copy_what_another_finds`,
  `r701_023i_each_player_searches_then_cards_move.rs`); "for the cards that reach it" isn't
  (a) anyway.
- `simultaneous.rs:1061:21: delete match arm Event::Damage{source, target, ..} in record`,
  `1081:47: delete ! in record`: **(b) not killed**. They change a player's "damage dealt
  this way" total and "creatures dealt damage this way" when several players' parts deal
  damage from the same source; no card in the pool reads those results after an
  "each player" damage instruction.
  Second round (`1701:21`, `1725:43`): Spiteful Repossession now compiles and reads the
  total (`spiteful_repossession_counts_each_opponents_damage_once`,
  `tests/cards/referents_this_way.rs`); the players dealt damage this way are pinned by the
  hand-built
  `a_player_dealt_damage_by_two_sources_at_once_is_dealt_damage_this_way_once`
  (`tests/cr/r608_002f_individual_processes.rs`).

**(c) engine bug found**: `Effect::Search` with several searchers ("Each player searches
their library for ... puts them onto the battlefield", Incoming!) let all players choose
first (CR 701.23i) but then moved each player's cards separately, so a Clone found by P1
entered after P0's creature and could copy it (CR 614.12; Clone ruling "can't become a copy
of a creature entering at the same time"). `resolve.rs` now collects every searcher's moves
and makes one simultaneous zone change. Regression test:
`tests/cr/r701_023i_each_player_searches_then_cards_move.rs`.

### turn.rs

Run 2026-10-14, 24 of 234 mutants (round-robin 0/10), 31 minutes.

- `turn.rs:545:48: replace += with *= in Game::take_action` (count an agent's illegal
  attempts in a row): **(b) missing test**. The count never grew, so an agent that keeps
  proposing illegal actions kept priority forever and the game stalled. Killed by
  `tests/cr/r733_002_repeated_illegal_actions.rs` (the third illegal attempt in a row
  passes priority).
- `turn.rs:725:17: replace || with && in Game::current_phase_end` (`|| s.is_main() ||
  matches!(s, Untap | BeginningOfCombat)` becomes `|| s.is_main() && matches!(..)`, so
  only a change of phase or a step earlier than the one before it ends the current
  phase): **(a) equivalent**, found by the re-run (the test written for it,
  `tests/cr/r500_008_combat_after_the_postcombat_main_phase.rs`, Relentless Assault in the
  postcombat main phase, passes with the mutant and stays as the CR 500.8 check). The
  first step of a phase (a main phase, untap, beginning of combat) that follows a step of
  the same phase is either earlier in the turn's order than that step (an additional combat
  phase after this one: beginning of combat after end of combat), which `*s < prev`
  catches, or the same step again, which would need an additional main phase directly
  after a main phase, or an additional beginning phase directly after an untap step. No
  effect adds those: additional main phases come after an additional combat phase
  ("followed by an additional main phase"), and additional beginning phases come after a
  main phase (Sphinx of the Second Sun).

### mana.rs

Run 2026-10-14, 23 of 228 mutants (round-robin 0/10), 28 minutes.

- `mana.rs:98:13: delete match arm ManaSymbol::Half(_) in ManaSymbol::mana_value` and
  `mana.rs:147:13: delete match arm ["½"] in ManaSymbol::parse`: **(a) out of scope**.
  Half-mana symbols ({½}, {HW}) appear only on Un-set cards that aren't legal in any
  format, which `docs/DEFERRED.md` excludes; the Comprehensive Rules have no rules for
  them.
- `mana.rs:214:30: replace + with - in ManaCost::parse` (`rest = &rest[end + 1..]` becomes
  `end - 1`): **(a) equivalent for in-scope cards**. The next search for `{` skips the
  leftover last character of the symbol just read, so ASCII costs parse the same; only a
  multi-byte symbol ({∞}, {½}, Un-set cards again) would slice inside a character.
- `mana.rs:507:21: replace && with || in ManaRestriction::allows` ("Spend this mana only to
  cast an instant or sorcery spell" allowed any spell): **(b) missing test**. Killed by
  `tests/cr/r106_006_instant_or_sorcery_mana.rs` (Vodalian Arcanist's mana pays for
  Titanic Growth, not for Grizzly Bears).
- `mana.rs:586:9: replace <impl PartialEq for ManaRider>::eq -> bool with false`: **(a)
  equivalent**. Nothing compares mana units that carry a rider (the pool removes spent
  mana by index), so the equality of riders is never consulted.
- `mana.rs:937:24: replace -= with += in PoolSolver<'_>::solve` (undo the count of
  Phyrexian symbols paid with life when a way of paying that tried life for one is
  abandoned): **(b) missing test**. The first test written,
  `tests/cr/r107_004f_phyrexian_payment_backtracking.rs` ({B/G}{G/P}{G/P} from {G}{B}
  with 3 life), pins the life undone on the line before and passed with the mutant (found
  by the re-run). The count is what Compleated reads (CR 702.150a): with the mutant the
  abandoned way's symbol stayed counted, and the planeswalker entered with two loyalty
  counters fewer for each. Killed by the second test in that file (the same cost on a
  compleated planeswalker with five loyalty enters with three).

### eval.rs

Run 2026-10-14, 27 of 268 mutants (round-robin 0/10), 39 minutes: every viable mutant was
caught.

### stack.rs

Run 2026-10-14, 37 of 361 mutants (round-robin 0/10), 55 minutes.

- `stack.rs:599:62: replace && with || in Game::is_legal_target` (for "target spell",
  `ob.zone == Battlefield && filter_allows_permanent(f)` becomes `||`): **(a)
  equivalent**, found by the re-run. The test written for it,
  `tests/cr/r115_001_target_spell_is_on_the_stack.rs` (Counterspell can't target Grizzly
  Bears on the battlefield; Unsubstantiate, "target spell or creature", can't target a
  creature card in a graveyard), passes with the mutant: the compiled spell filters include
  `Filter::Spell` for their spell part, and the candidates for "target spell" are the
  objects on the stack plus, only when the filter allows permanents, the permanents, for
  which both conditions agree. An answer naming anything else isn't a candidate, and a
  target that changes zones is a new object (CR 400.7).
- `stack.rs:740:13: replace || with && in Game::player_untargetable`: **(a) equivalent
  (dead path)**. It changes how `Restriction::PlayerCantBeTargeted` from static abilities
  and from resolved effects combine, but no compiled card or engine code creates that
  restriction (players get hexproof and shroud as `PlayerModification`s, which are checked
  before it).
- `stack.rs:1745:9: delete match arm Filter::And(v) | Filter::Or(v) in
  filter_mentions_attached` (CR 603.6e: an Aura's "when enchanted creature dies" trigger
  finds the Aura card in the graveyard): **(b) missing test**. The existing test used a
  bare "enchanted permanent" filter. Killed by
  `tests/cr/r603_006e_aura_finds_itself_enchanted_creature_filter.rs` (the filter "a
  creature that is the enchanted permanent").

### replacement.rs

Run 2026-10-14, 37 of 361 mutants (round-robin 0/10), 54 minutes.

- `replacement.rs:233:18: replace > with >= in Game::replace_rec` (recursion guard at
  depth 32): **(a) equivalent**. Each replacement effect applies to an event at most once
  (CR 614.5), so no real chain gets near 32 levels; the guard only stops runaway loops.
- `replacement.rs:1054:20: replace > with >= in Game::repl_event_matches` ("If you would
  put one or more counters on a permanent or player" also matching zero counters):
  **(a) equivalent**. The only effects compiled for that event double or halve the
  number, and twice or half of 0 is 0; adding no counters does nothing.
- `replacement.rs:1207:74: replace ^ with & in Game::apply_replacement` (the key that
  identifies a player-level replacement effect loses the player): **(b) not killed**. The
  key only names a prevention shield for "damage prevented this way"; it would matter for
  two players' shields of the same kind preventing damage in the same event and a card
  reading the amount, which no card in the pool combines.
- `replacement.rs:1600:28: replace match guard ... with true in Game::apply_replacement`
  ("it deals N damage instead" handled as "amount minus (amount minus N)"): **(a)
  equivalent for the compiled forms**. The other `Subtract(Diff(..))` the compiler makes
  is "deals half that damage" (amount minus (amount minus half)), and both branches give
  half.
- `replacement.rs:1786:9: replace <impl View for EnteringView>::controller_override ->
  Option<PlayerId> with None`: **(a) equivalent now**, found by the re-run. When it was
  found, "Creatures your opponents control enter tapped" was checked against the card's
  owner instead of the player it enters under (CR 614.12), and
  `tests/cr/r614_012_enters_under_another_players_control.rs` (Reanimate on a creature
  card in the opponent's graveyard, Authority of the Consuls) was written for it. Since
  then the effects that modify how a permanent enters are gathered inside
  `with_hypothetical_entry`, with the object on the battlefield under the player it
  enters under, so the test passes with the mutant. The override still covers the other
  replacement effects checked against the entering object in its old zone, but no card
  has one whose filter names the entering permanent's controller ("If a creature an
  opponent controls would enter ..." doesn't exist).
- `replacement.rs:1875:5: replace event_amount -> Option<u32> with None`: **(a)
  equivalent now**, found by the re-run. When it was found, damage a replacement reduced
  to 0 stayed an event that a later "deals that much damage plus 2" turned back into
  damage (CR 120.8), and `tests/cr/r120_008_zero_damage_isnt_dealt.rs` (Benevolent Unicorn
  and Torbran on Spark Jolt, in both orders) was written for it. Since then
  `repl_event_matches` itself refuses a damage event of 0 for every damage replacement, so
  the test passes with the mutant (and stays as the CR 120.8 check); the other amounts this
  drops (0 life, 0 counters, 0 tokens) do nothing when performed.

### triggers.rs

Run 2026-10-14, 48 of 474 mutants (round-robin 0/10), 2 hours. Most survivors delete one
field of the `EventInfo` a trigger records (what "it", "that player", "that source" and
so on refer to); the field is only read by abilities whose wording refers to it.

- `triggers.rs:17:5: replace keyword_has_trigger -> bool with true` and
  `triggers.rs:530:9: replace Game::check_triggers with ()`: **(a) equivalent (dead
  code)**. Neither public function has a caller.
- `triggers.rs:269:73: replace != with == in Game::check_batch_triggers` (which sources
  in a batch's look-back snapshot have left the battlefield): **(b) not killed**. An object
  that left has a new id (CR 400.7), so its old id is never live and the `!is_live` half
  decides; the mutant differs only for a source still on the battlefield that lost the
  triggered ability between the event and the check, which no test sets up. 2026-11-11:
  that case was an engine bug (CR 603.10a: the abilities that existed immediately before
  the event look back, also on a permanent that stays but lost the ability because of the
  event), found by the `engine-audit-1/triggers` audit. The code is gone:
  `check_batch_triggers` adds every snapshot source that doesn't have the ability now,
  pinned by `a_one_or_more_ability_lost_because_of_the_event_still_triggers_on_it`
  (`tests/cr/r603_audit_simultaneous.rs`: an artifact with a "one or more creatures you
  control die" ability only while you control a Cleric, and a Wrath).
- `triggers.rs:602:29: replace && with || in Game::detect_triggers` (abilities of a
  permanent that phases out, and of what's attached to it, that look back for the phasing
  event): **(a) equivalent**. The extra abilities the mutant adds don't trigger on a
  phase-out event (the phase-out triggers were added anyway), and phased-out objects aren't
  in the source list, so nothing is added twice.
- `triggers.rs:688:40: replace += with *= in Game::detect_triggers` (the order number of a
  detected trigger): **(a) equivalent in practice**. Pending triggers are kept in detection
  order and the sort by order number is stable, so equal numbers keep the same order.
- `triggers.rs:731:43: replace == with != in Game::detect_triggers` ("at the beginning of
  the next end step" created during that step doesn't fire in it, except in a cleanup step
  that's followed by another, CR 514.3a): **(b) not killed**. The check only matters for a
  delayed trigger created between a step's beginning and the trigger check for it; no
  card creates one there. The re-run (2026-10-21) found it **killed** by tests added
  since by other work.
- `triggers.rs:1088:21` (Attacks: `objects`, all the attackers), `1272:25`
  (IsDealtDamage: `other`, the source), `1528:25` (SpellCopied: `object`; `spell` holds the
  same copy), `1606:25` (TokenCreated: `player`), `1650:25` (TappedForManaOfType:
  `object`), `1758:25` (Mills: `objects`), `1843:25` (chapter abilities: `lki`): **(b) not
  killed**: no card was found whose ability reads that field for that trigger (the "they" of
  "whenever one or more creatures you control attack" comes from each attacker's
  `object`, which Angelic Guardian's test checks).
- `triggers.rs:1204:29: delete field object` (BlocksOrBecomesBlocked, the attacking
  creature that became blocked): **(b) missing test**. The first tests,
  `tests/cr/r509_003_blocks_or_becomes_blocked_it.rs` (Raging Gorilla gets +2/-2 when it
  blocks and when it becomes blocked), passed with the mutant (found by the re-run): its
  "it" is the source itself. Killed by the Gift of the Woods tests added to that file
  ("whenever enchanted creature blocks or becomes blocked, it gets +0/+3": "it" is the
  creature the trigger recorded).
- `triggers.rs:1934:25: delete match arm TriggerCond::CastSpell{..} | ... in
  player_event`: **(b) missing test**. "Their first spell each turn" stopped being per
  player: a second opponent's first spell no longer triggered once someone else had cast
  one. Killed by `tests/cr/r603_002_first_spell_each_turn_per_player.rs` (Esper Sentinel in
  a three-player game).
- `triggers.rs:2051:50: replace && with || in Game::trigger_matches_ctx` ("is dealt excess
  damage" stopped triggering on combat damage): **(b) missing test**. The one card with an
  excess-damage trigger that isn't "noncombat" only, Aegar, the Freezing Flame, isn't
  supported yet, so the test uses a custom permanent. Killed by
  `tests/cr/r120_010_excess_combat_damage_trigger.rs` (a 3/3 blocked by a 1/1 deals
  excess combat damage).

### triggers.rs, second run (2026-11-11)

Run after the `engine-audit-1/triggers` tests (the implementer's 74 tests in
`tests/cr/r603_audit_*.rs`, with that audit's three engine fixes) against the file at the
implementer's last commit `7ee1be9`: 55 of 541 mutants (round-robin 0/10). Line numbers are
`7ee1be9`'s; the review's later edits to the file (the look-back snapshot of the "triggers an
additional time" effects in `check_batch_triggers`, the cause object in the
`AbilityTriggered` arm of `trigger_matches_ctx`) aren't in the measured copy. The run was
still going when the audit landed (about five minutes per mutant, more beside the review's
own builds): 17 of the 55 were reached when this was written, 14 caught, 3 missed, 0
timeouts, 0 unviable, kill rate **82.4%** (14/17); all three survivors are equivalent. The
first run's `269:73` survivor is gone with the code it mutated (see "triggers.rs" above).

- `triggers.rs:17:5: replace keyword_has_trigger -> bool with true`: **(a) equivalent (dead
  code)**, as in the first run: the function still has no caller.
- `triggers.rs:561:49: replace > with >= in Game::check_batch_triggers` ("Do this only once
  each turn", CR 603.2h: whether the action was taken this turn): **(a) equivalent**. The
  `DONE_ONCE` count is only ever inserted as it's incremented (`or_insert(0) += 1` in
  `resolve_ability`, `stack.rs`), and `triggers_this_turn` is cleared as each turn begins
  (`turn.rs`), so a present count is at least 1 and `> 0` and `>= 0` agree.
- `triggers.rs:877:40: replace += with *= in Game::detect_triggers` (the order number of a
  detected trigger): **(a) equivalent in practice**, as `688:40` in the first run. The
  counter only grows (the other detection paths still increment it), so the pending
  triggers' numbers stay non-decreasing in detection order, and the stable sort in
  `put_triggers_on_stack`, the number's only reader, keeps that order.
- One sampled mutant not yet reached has a test written for it in the review:
  `triggers.rs:2169:33: replace && with || in Game::trigger_matches_ctx` (`LoseControl` on a
  control change: the change is from the ability's controller *and* of the object it names).
  `a_lose_control_of_itself_ability_ignores_the_permanents_taken_with_it`
  (`tests/cr/r603_audit_lookback.rs`: Insurrection takes Khârn the Betrayer with two other
  permanents; one trigger, for Khârn) fails with it applied to the worktree and passes
  without it.
- Remaining (not reached when this was written, to be tested one at a time as the layers.rs
  second run's last 16 were, or with a new `scripts/mutants.sh triggers` run): the 38
  sampled mutants from `936:25` on, in source order: `936:25` and `978:24`
  (`detect_triggers`); `1056:39` (`fire_once_delayed`); `1094:9` (`additional_triggers`);
  `1135:9` (`trigger_matches`); the 29 in `trigger_matches_ctx` (`1172:25` to `2426:50`, with
  `player_event`'s `2309:25`);
  `2528:39` (`put_triggers_on_stack`); `2612:45` (`put_trigger_on_this_stack`); `2681:38`
  (`resolve_trigger_immediately`); and `2728:9` (`on_sacrifice`).

### resolve.rs

Run 2026-10-14, 32 of 314 mutants (round-robin 0/10), 52 minutes.

- `resolve.rs:658:31: replace += with *= in Game::exec_effect` ("that many" after putting
  one object's counters on another stayed 0), `1317:37: delete ! in Game::exec_effect`
  (turning a permanent face down reported "didn't happen"), `1610:43: replace && with ||
  in Game::exec_effect` (scrying 0 reported "happened", CR 701.22b): **(b) missing
  tests**. No card in the pool reads these results yet, but they're what "if you do" and
  "that many" read (CR 608.2c). Killed by
  `tests/actions/a701_022b_instruction_results.rs`.
- `resolve.rs:2364:26: replace > with >= in Game::effect_on_entering_object` ("enters with
  N counters" with N = 0 records an entry of 0 counters): **(a) equivalent**. Putting 0
  counters on the entering permanent does nothing.
- `resolve.rs:2847:9: replace Game::lock_replacement_objects -> Option<Vec<ObjectId>> with
  None` and `3274:9: delete match arm ReplacementEvent::Where{event, ..} in
  unlock_replacement_def`: **(b) not killed**. They concern a replacement effect created
  by a resolving effect with an extra condition (`Where`) whose affected objects are
  locked in as it's created (CR 611.2c); no card found produces that combination.
- `resolve.rs:3230:35: replace < with <= in Game::choose_mana_color`: **(b) missing
  test**. An answer one past the last offered mana type indexed out of bounds (a panic)
  instead of getting the default. Killed by
  `tests/cr/r106_004_mana_type_choice_answers.rs` (Selesnya Guildgate, answer `Index(2)`).
- `resolve.rs:3407:20: replace match guard c != ctx.controller with true in
  delayed_controller`: **(a) equivalent**. When the resolving spell's controller is the
  same player, the mutant names that player as both the controller and the performer of
  the delayed trigger, which is the same as naming no separate performer.

### actions.rs

Run 2026-10-14, 39 of 390 mutants (round-robin 0/10), 58 minutes. Three mutants
(`87:16` in `move_objects`, `292:39` in `move_forbidden`, `339:13` in `can_move`) made the
suite hang and count as detected (timeout).

- `actions.rs:498:42: replace && with || in Game::perform_move_now` (remember a commander
  put into a graveyard or exile for CR 704.6d): **(a) equivalent**. The state-based action
  checks again that the object is a commander in a graveyard or exile before offering the
  command zone, so the extra objects the mutant remembers are ignored.
- `actions.rs:822:38: replace && with || in Game::perform_move_now` (record a permanent
  that entered the battlefield): **(a) equivalent**. By this point an object moved to the
  battlefield is on it (moves that can't happen were dropped before), and an object moved
  elsewhere isn't, so both conditions agree.
- `actions.rs:1477:35: replace += with *= in Game::perform_add_counters`
  (`history.counters_put`): **(a) equivalent (dead state)**. Nothing reads that total.
- `actions.rs:1561:20: delete ! in Game::gain_life` (when the player can't gain life, which
  replaced events still happen): **(a) equivalent**. An "instead" self-replacement runs its
  instruction while it's applied and returns no event, and a life-gain event that remains
  is checked against "can't gain life" again when performed; CR 614.17c is pinned by
  `tests/cr/r609_616_interaction.rs`, which passes with the mutant.
- `actions.rs:1973:31: replace match guard spec.card.is_some() with true in
  Game::token_entry` (a token copy of a double-faced permanent with no card of its own):
  **(b) not killed**. Every path found that copies a double-faced permanent into a token
  gives the token its card, so the guard's false branch isn't reached by the tests.

### layers.rs

Run 2026-10-14, 43 of 423 mutants (round-robin 0/10), 72 minutes. One mutant (`741:17` in
`layer_candidates`) made the suite hang (timeout, detected).

- `layers.rs:1092:28: delete ! in Game::apply_effect_in_layer` (remember the objects a
  filter-defined resolved effect started applying to, CR 613.6): **(a) equivalent (dead
  path)**. `Affected::Filter` is never constructed: resolved effects lock in their objects
  (`Affected::Objects`, CR 611.2c), and the CR 613.6 examples for those are covered in
  `tests/cr/r609_613_layers.rs`.
- `layers.rs:1489:49: replace + with * in Game::compute_player_effects` and
  `layers.rs:1550:56: replace | with & in key_order`: **(a) equivalent**. Both change only
  the tie-break between effects with the same timestamp; the effects in question (several
  player effects created at once, several static abilities of one object) keep their
  relative order because the sort is stable and they're collected in that order.
- `layers.rs:1519:28: replace match guard crate::choices::filter_mentions_choice(&f) with
  true in Game::compute_player_effects`: **(a) equivalent in practice**. Binding the
  source's choices into a protection filter that mentions no choice leaves it unchanged;
  it would differ only for a player protection effect with no source, which nothing
  creates.
- `layers.rs:2018:12: delete ! in drop_ungainable_subtypes` (CR 205.3d: a copy
  exception's "every creature type" on a noncreature copy is dropped): **(b) not killed**.
  No card found gives a copy of a noncreature permanent all creature types as a copy
  exception.

### layers.rs, second run (2026-11-10)

Run after the `engine-audit-1/layers` tests (65 new tests in `tests/cr/r613_audit_*.rs`)
against the current file: 53 of 524 mutants (round-robin 0/10). The run took two passes:
cargo-mutants' 300 s test timeout no longer fit the unmutated suite (~290 s), so
`scripts/mutants.sh` now takes `TIMEOUT` (default 600 s); the first 37 mutants ran in one
2-hour `cargo mutants` invocation, the remaining 16 one at a time the way the RERUN path
does (apply the diff in `target/mutsrc`, build, run the suite), the loop finishing during the
review. Of the 53 mutants tested: 45 caught, 6 missed, 2 timeouts, 0 unviable, kill rate
**88.7%**; every survivor is equivalent, so no re-run was needed. Every sampled mutant
was reached.

- `layers.rs:408:25: delete field rules_text from struct Characteristics expression in
  Game::compute_characteristics` (the characteristics of a card exiled face down):
  **(a) equivalent**. `Characteristics` derives `Default`, and the default `Arc<str>` is the
  empty string the field was set to.
- `layers.rs:766:48: replace || with && in Game::expire_dependent_effects` (removing a
  `Duration::UntilHostLeaves` effect whose objects have all left the battlefield): **(a)
  equivalent (dead path)**. `Duration::UntilHostLeaves` is defined and rendered but never
  constructed, so the branch never runs.
- `layers.rs:1256:28: delete ! in Game::apply_effect_in_layer` (recording the objects a
  resolved effect started applying to, CR 613.6): **(a) equivalent (dead path)**, the same
  site as `1092:28` in the first run. A resolved effect's `Affected::Objects` set is already
  locked (CR 611.2c), so recording it on trial applications instead of real ones changes
  nothing; it would matter only for `Affected::Filter`, which is never constructed.
- `layers.rs:1696:49: replace + with * in Game::compute_player_effects`: **(a) equivalent**,
  the same site as `1489:49` in the first run. It changes only the tie-break key among
  effects with identical timestamps; the sort is stable, the key stays monotonic in the
  effect's index, and a resolved player effect never shares a timestamp with a static one
  (each comes from its own `new_timestamp()` call).
- Timeouts (detections): `layers.rs:896:17: replace && with || in Game::layer_candidates`
  (layer-1 effects become candidates in every layer on every pass) and `layers.rs:931:40:
  replace || with && in Game::layer_candidates` (an applied static ability is gathered
  again every pass); both make `apply_layer` run to its iteration cap.
- The 16 sampled mutants the run didn't reach, in `filter_is_self`, `key_order`,
  `mod_values`, `parse_pt_counter`, `keyword_counter` (the "flying" and "menace" arms),
  `apply_mod` (four), `acquired_ability`, `copied_link`, `drop_ungainable_subtypes`,
  `subtype_still_valid`, `intrinsic_mana_ability` (the "Forest" arm) and `qualities_among`,
  were tested afterwards one at a time (results in `target/mutants/layers-rest/`); the loop
  finished during the review. All 16 were reached: 14 were caught (`mod_values` `==` with
  `!=`, `parse_pt_counter` `Some((1, 1))`, the "flying" and "menace" `keyword_counter`
  arms, the four `apply_mod` mutants `1899:5`, `1985:20`, `2099:25` and `2179:20`,
  `acquired_ability` `2218:55`, `copied_link` `2273:12`, `drop_ungainable_subtypes`
  `2282:5`, `subtype_still_valid` `2299:52`, the "Forest" arm of `intrinsic_mana_ability`
  `2354:9` and the `qualities_among` guard `2393:37`); the two below survived.
- `layers.rs:1749:5: replace filter_is_self -> bool with true` (`static_affected` then
  includes an ability's own source in its effect even when the source isn't on the
  battlefield): **(a) equivalent in practice**. It can only reach a static ability that
  functions outside the battlefield with a filter that isn't zone-bound, such as Wonder's
  "creatures you control have flying" while it's in a graveyard, where the card would give
  itself flying in the graveyard; nothing reads abilities of a graveyard card.
- `layers.rs:1759:50: replace << with >> in key_order`: **(a) equivalent**, like `1550:56`
  in the first run. The key only breaks ties between static effects with identical
  timestamps: abilities of one object still sort by uid, and two objects never share a
  timestamp (each gets its own `new_timestamp()`; the sort is stable).

### casting.rs, second run (2026-11-10)

Run after the `engine-audit-1/casting-costs` tests (64 new tests in `tests/cr/r601_audit_*.rs`
and `tests/cr/r701_018b_face_down_card_played_as_what_it_is.rs`, with that audit's engine fixes)
against the current file: 105 of 1049 mutants (round-robin 0/10), six hours (65–115 s build
+ 320–550 s test per mutant; the first hours overlapped the review's own builds). All 105 were
reached: 87 caught, 16 missed, 1 timeout, 1 unviable, kill rate **84.6%** (88/104). Five of
the sixteen survivors are killable and have tests, each checked to fail with its mutant applied
to the worktree and to pass without it; the re-run (`RERUN=1 scripts/mutants.sh casting`)
over all sixteen (and the timed-out mutant, which it caught outright) caught the five and
missed the eleven again, kill rate after it **89.4%** (93/104). The re-run matched each
survivor to its line in the current file (e.g. `420:41` had become `495:41`); the numbers
below are the first run's. The other eleven are equivalent or dead paths.

- `casting.rs:50:9: replace CastOption::normal -> CastOption with Default::default()`:
  unviable (`CastOption` has no `Default`).
- `casting.rs:194:9: delete match arm 1 in choose_face_to_cast` (one castable face): **(a)
  equivalent**. `Game::ask_option` returns 0 without asking for one option or none, so the
  `_` arm yields `faces[0]` as the deleted arm did. Re-run: missed again (equivalent).
- `casting.rs:420:41: replace != with == in free_cast_options` (the "(prototype)" suffix on
  the name of a tagged way to cast a card without paying its mana cost): **(b) missing
  test**. The suffix moved onto `CastMethod::Free` options, none of which is tagged, and
  off the prototyped one, so cascade and discover offered "Goring Warplow" twice. Killed by
  `tests/cr/r601_audit_alternative_additional.rs`
  (`an_effect_casting_a_card_free_offers_each_way_to_cast_it_by_name`: Maelstrom Wanderer
  cascading into Goring Warplow offers "Cast Goring Warplow (mana value 6)" and "Cast Goring
  Warplow (prototype) (mana value 2)"). Re-run: caught.
- `casting.rs:686:17: replace && with || in Game::card_has_land_face`: **(b) missing test**.
  `o.face_down || (zone != Battlefield && front is a land)` made any face-down card outside
  the battlefield a land card to play: a creature card exiled face down with a play
  permission (Praetor's Grasp) was offered as a land play and `play_land` accepted it. Killed
  by `tests/cr/r701_018b_face_down_card_played_as_what_it_is.rs` (Grizzly Bears exiled face
  down is cast, never played as a land; an Island is played as a land face up, CR 701.18b).
  Re-run: caught.
- `casting.rs:958:41: replace && with || in Game::face_characteristics`: **(a) equivalent in
  practice**. For a face-up card outside the battlefield the mutant returns the printed front
  characteristics instead of the computed ones. They differ only under effects that change
  cards' characteristics outside the battlefield (Painter's Servant, Conspiracy, Maskwood
  Nexus, Mycosynth Lattice), all of which also mention spells, so
  `spell_preview::spell_characteristics` recomputes the characteristics on the stack before
  any prohibition or permission is judged; the other callers filter lands
  (`castable_faces`, `card_has_land_face`), name the face (`choose_face_to_cast`) or check
  keyword prohibitions printed on the card. Re-run: missed again (equivalent).
- `casting.rs:1342:32: replace || with && in Game::play_land_during_resolution`: **(a)
  equivalent (dead path)**. The guard `!is_live(card) || !card_has_land_face(card)` is
  reached only from `Effect::PlayCard`, for the live objects a selection resolved whose front
  face is a land, so neither disjunct is ever true there. Re-run: missed again (equivalent).
- `casting.rs:1703:42: replace match guard n >= 0 with true in Game::cast_inner` (how many
  times a repeatable optional additional cost is paid, multikicker and replicate): **(b)
  missing test**, and an engine bug behind it. A negative answer became `u32::MAX` times, and
  on the unmutated engine an absurd count (a billion) built the total cost up one copy at a
  time without a bound: unbounded memory through the agent protocol. `cast_inner` now caps
  the count at the mana available plus the permanents on the battlefield plus 100 (any larger
  count is unpayable, so the casting is reversed as before), and the test
  `a_repeatable_additional_cost_is_paid_a_whole_number_of_times`
  (`tests/cr/r601_audit_alternative_additional.rs`: Everflowing Chalice kicked twice, then
  answered -1, 3 and 1_000_000_000) kills the mutant in a second instead of running out of
  memory. Re-run: caught.
- `casting.rs:2168:31: replace += with *= in Game::cast_inner` (`cast.life_paid` of a spell):
  **(a) equivalent (dead field for spells)**. A spell's `life_paid` is recorded here and in
  `mana_abilities::pay_mana` but read nowhere: the only readers (`kw/trigger_modifiers.rs`,
  "if life was paid to activate it", "the amount of life paid") read an activated ability's,
  recorded by `activate_inner`. Re-run: missed again (equivalent).
- `casting.rs:2689:30: replace && with || in Game::can_activate` (the search for a value of X
  the cost can be paid at): **(a) equivalent in practice**. With no X in the cost the search
  tries the same cost at every x and fails as before. It differs only for an ability whose
  condition needs X ≥ 1 (`least_x` set) and whose non-mana X cost is payable at some X but
  not at the least one, where the mutant would (rightly) find the other X; the one printed
  card with both, Ruthless Technomancer ("Sacrifice X artifacts", "X can't be 0"), can pay at
  X = 1 whenever it can pay at any X. Re-run: missed again (equivalent).
- `casting.rs:3549:13: delete field source from struct SpendContext expression in
  Game::pay_cost` (a cost paid while a spell or ability resolves, an attack tax, a face-up
  cost): **(a) equivalent**. Every reader of `SpendContext::source` is gated on `is_spell` or
  `is_ability` (`ManaRestriction::allows_in`'s CastSpell, ActivateAbilityOf and
  ArtifactSpellOrAbility arms, `cast_history::record_payment`, the cast-record paths of
  `mana_abilities::pay_mana`), both false for `pay_cost`. Re-run: missed again (equivalent).
- `casting.rs:3770:27: replace >= with < in Game::cost_part_payable` ("Remove X counters from
  an artifact or creature you control", `CostPart::RemoveCountersFromOne`; Moxite Refinery is
  the one printed card compiled to it, the n = 1 form being another part): **(b) missing
  test**. The production check is right (a permanent must carry at least X counters); the
  mutant inverts it, which shows in the X a player leaves to the engine, the least value the
  cost can be paid at (`default-numbers`): 0 with the check, 1 with the mutant. Killed by
  `tests/cr/r601_audit_x.rs`
  (`an_unannounced_x_of_an_activation_cost_paid_with_counters_is_the_least_payable_value`:
  Moxite Refinery with a Grizzly Bears carrying two +1/+1 counters, the default X removes
  nothing and puts no charge counter on Bonesplitter; X = 2 announced removes both and puts
  two). Re-run: caught.
- `casting.rs:3810:29: replace && with || in Game::cost_part_payable` ("Put a card from your
  hand on top of your library: ...", `CostPart::PutFromHandOnLibrary`): **(a) equivalent in
  practice**. The three printed cards with that cost (Penance, Leashling, Hidden Retreat) take
  any card, so `usable_for_cost` (the filter) is true for every card in hand, and the
  ability's source is a permanent, never a card in the hand; both sides of the `||` are true
  for every card either way. Re-run: missed again (equivalent).
- `casting.rs:4278:81: replace < with > in Game::pay_cost_part` (`CostPart::RemoveCounters`:
  "not enough counters" after removing them): **(a) equivalent (dead path in practice)**.
  `pay_total_cost_ordered` checks every part's payability (`cost_part_payable`: the source
  carries at least X counters) right before paying any part, and nothing removes counters
  from the source in between (a mana ability removing counters from the same permanent during
  the payment would; no printed card has one beside such a cost). The test written for it,
  `an_x_greater_than_the_counters_the_source_carries_cant_be_paid` (`tests/cr/r601_audit_x.rs`:
  Calciform Pools with three storage counters and X = 5, an illegal activation reversed with
  nothing added), is satisfied by that earlier check and kept as a test of CR 601.2h and
  118.3. Re-run: missed again (equivalent).
- `casting.rs:4329:53: replace - with / in Game::pay_cost_part` (`kinds[i.min(kinds.len() -
  1)]`, the kind of counter to remove): **(a) equivalent**. `ask_option` returns an index
  below the number of options, so the `.min` guard never applies either way. Re-run: missed again (equivalent).
- `casting.rs:4637:21: delete field is_ability from struct SpendContext expression in
  Game::pay_cost_part` (the `Repeated` arm): **(a) equivalent (dead path)**.
  `kw::cumulative_upkeep::expand_repeated` flattens every `Repeated` part of a cost before
  `pay_cost_part` sees it (the first step of `pay_total_cost_ordered`, and of `can_pay_cost`),
  so the arm is never reached. Its hardcoded `is_ability: true` would be wrong for a
  cumulative upkeep cost, which is paid as a triggered ability resolves, not an activation
  cost; the flattened payment uses the outer context (`is_ability` false), so mana that may
  be spent only to activate abilities (Omen Hawker) can't pay cumulative upkeep, as the rules
  say. Re-run: missed again (equivalent).
- `casting.rs:4671:21: delete field purpose from struct SpendContext expression in
  Game::pay_cost_part` (the arm paying a choice of costs): **(a) equivalent in practice**.
  `purpose` matters only to mana restricted to a purpose (Adarkar Unicorn's "only to pay
  cumulative upkeep costs", Qarsi Deceiver's face-up costs), and no cumulative upkeep or morph
  cost offers a choice of costs. Re-run: missed again (equivalent).
- `casting.rs:4735:41: replace match guard is_x(v) with false in Game::x_bound_of_parts` (the
  "pay X life" bound becomes the generic 20): **(b) missing test**. Killed by the 30-life
  Toxic Deluge scenario of
  `an_x_paid_as_life_is_bounded_by_the_life_total_not_by_the_mana_available`
  (`tests/cr/r601_audit_x.rs`: with 30 life, X up to 30 is offered). Re-run: caught.
- Timeout (a detection): `casting.rs:3725:45: replace && with || in Game::cost_part_payable`
  (the tapped permanents an "untap" cost could untap excluded every permanent whenever the
  cost untaps its own source): the suite ran past the 600 s timeout in the first run and
  failed outright in the re-run (caught).

### casting.rs

Run 2026-10-14/15, 85 of 841 mutants (round-robin 0/10), about 3 hours. Tests for the
(b) survivors were written on 2026-10-21; each was checked to fail with its mutant
applied to the worktree and to pass without it.

- `casting.rs:511:18: replace && with || in Game::permission_allows_with` ("a commander
  in the command zone may be cast" becomes "any nonland card the player may be permitted
  to cast passes the permission check"): **(a) equivalent**, not an engine bug. Every
  caller (casting from outside the hand in `permitted_cast_options`, and the keywords'
  own cast options: evoke, dash, morph, disguise, prototype, ...) only reaches the check
  for a card in `permitted_cards`, and for those the check is redundant: when an
  effect's permission may cover the card, `cast_options` rebuilds the ways to cast it from
  the card's own permissions plus `permissions::attach`, which keeps a way only if
  `permissions::allowing` finds a permission for the characteristics it would have
  (CR 601.3e). Without such a permission the only cards outside the hand in
  `permitted_cards` are castable commanders and prepared copies, which pass the check
  either way. Casting a graveyard or library card with no permission is refused before
  that (it isn't in `permitted_cards`), so the mutant can't let one through.
- `casting.rs:884:61: replace && with || in Game::can_begin_cast` ("X can't be 0" with
  an alternative cost that has X in its other parts only): **(b) missing test**. The
  mutant made such a spell uncastable for that cost. Light Up the Night (the one real card
  with that shape) isn't supported yet, so the test compiles the same shape from oracle
  text: `tests/cr/r107_003a_alternative_cost_with_x_in_another_part.rs` ("Flashback—{R},
  Pay X life." is offered as a legal action and cast with X = 2).
- `casting.rs:957:21: replace && with || in Game::flash_permitted` (a "next spell" effect
  that doesn't give flash still gave it when its filter matched): **(b) missing test**.
  Killed by `tests/cr/r601_003_next_spell_effect_without_flash.rs` (after Insist, Grizzly
  Bears can't be cast while Lightning Bolt is on the stack).
- `casting.rs:2142:21: replace + with - in Game::can_activate` (the bound for the search
  for a value of X that satisfies an activated ability's condition subtracts the source's
  loyalty counters instead of adding them): **(b) missing test**. For a planeswalker's {X}
  ability with "X can't be 0" and less mana than loyalty the subtraction underflows (a
  panic). Killed by `tests/cr/r602_002b_x_cant_be_zero_ability_of_a_planeswalker.rs` (a
  planeswalker compiled from oracle text, three loyalty, one Island).
- `casting.rs:2309:24: replace * with + in Game::ability_total_cost_and_x` (the generic
  mana {X} stands for): **(b) missing test**. "Spend only black mana on X" then covered
  part of a cost increase too. Killed by
  `tests/cr/r107_003_x_spend_restriction_with_cost_increase.rs` (Crypt Rats with X = 1
  under Suppression Field, paid with {B}{R}{R}).
- `casting.rs:2713:51: replace && with || in Game::any_color_mana`: **(a) equivalent
  (dead path)**. The arm is for `SpendAsAnyColor` applying to "this spell"; no compiled
  card produces it (the patterns in `spend_mana_as_though.rs` and
  `r613_abilities_of_objects.rs` make only `Spells` and `Abilities`; the cards that say "to
  cast this spell" are digital-only conjured cards, out of scope).
- `casting.rs:2960:43: delete - in Game::cost_part_payable` (a "−N" loyalty cost counted as
  payable with less than N loyalty): **(b) missing test**. The cost still couldn't be
  paid, but the ability was offered as a legal action. Killed by
  `tests/cr/r606_006_negative_loyalty_cost_isnt_offered.rs` (Liliana of the Veil with
  three loyalty: +1 and −2 offered, −6 not).
- `casting.rs:3663:41: replace && with || in Game::pay_cost_part` (the cards offered to
  pay a reveal-from-hand cost included cards without the cost's quality): **(b) missing
  test**. Killed by `tests/cr/r118_003_reveal_cost_cards_must_match.rs` (Martyr of
  Spores with X = 1: P0's answer naming Shock doesn't pay "reveal X green cards"; Llanowar
  Elves is revealed).
- `casting.rs:3903:9: delete match arm CostPart::Loyalty(_) in cost_part_has_x`: **(a)
  equivalent**. The arm returns `false`, as the catch-all arm does.

### combat.rs

Run 2026-10-21, 88 of 876 mutants (round-robin 0/10), 3 hours (the unmutated suite now
takes ~175 s). Tests for the (b) survivors were checked to fail with their mutant applied
to the worktree and to pass without it.

- `combat.rs:136:9: replace CombatState::attacked_alone -> bool with false`: **(b)
  missing test** (first triaged as dead code). No engine code calls it (exalted and
  "attacks alone" triggers use `TriggerCond::AttacksAlone` on the declaration event), but
  it's public API that `tests/cr/r506_combat_phase.rs` (`attacks_alone_and_attacking_alone`)
  called only to check that it's false, which the mutant satisfies. Killed by the
  assertions added there in review (Rogue Kavu declared alone attacked alone, the Bears
  didn't).
- `combat.rs:314:34: replace match guard self.obj(o).is(CardType::Battle) with false in
  Game::can_attack_target`: **(b) missing test**. An attack on a battle wasn't treated as
  an attack on its protector, so "~ can't attack unless defending player controls an
  Island" didn't stop it. Killed by
  `tests/cr/r508_005_defending_player_condition_attacking_a_battle.rs` (Sea Monster and
  P0's Invasion of Segovia protected by P1).
- `combat.rs:645:12: delete ! in attacking_players`: **(a) equivalent**. The combat state
  made at the beginning of combat always lists the attacking players (the active player,
  plus teammates with shared team turns), so the fallback for an empty list, which the
  mutant returns instead, is only reached by a combat state created by declaring attackers
  without a beginning of combat step, which the turn structure never does.
- `combat.rs:1073:14: replace > with == in best_attack` and `combat.rs:1808:14: replace >
  with == in best_blocks` (when there are too many possible declarations to search, more
  than 100,000, the greedy declaration that obeys the requirements is never kept): **(b)
  missing tests**. Declaring no attackers (or blockers) was then accepted against
  seventeen "attacks/blocks each combat if able" requirements. Killed by
  `tests/cr/r508_001d_509_001c_requirements_with_many_creatures.rs` (Grand Melee with
  seventeen Grizzly Bears attacking, and blocking).
- `combat.rs:1370:9: replace || with && in make_attacking` (a creature controlled by a
  player who isn't attacking could be made an attacking creature, CR 506.3b): **(b) not
  killed**. The only caller is Tahngarth, First Mate, which first gives control of it to
  the attacking player; no card makes a creature of a non-attacking player attacking.
- `combat.rs:1764:59: replace + with * in best_blocks` (the size of the search for the
  best block declaration counted without the "doesn't block" choice): **(a) equivalent**.
  It only makes the exhaustive search run where the greedy one would have; the exhaustive
  search finds a declaration obeying at least as many requirements, and both are only used
  to judge the declaration a player made.
- `combat.rs:2429:31: replace + with * in TrampleShape::valid` (for trample over
  planeswalkers, damage to the planeswalker's controller checked against the damage to the
  planeswalker instead): **(b) missing test**. An assignment of nothing to the planeswalker
  and the rest to its controller was accepted. Killed by
  `tests/keywords/k702_019c_trample_over_planeswalkers_skipping_the_planeswalker.rs`
  (Thrasta attacking Jace Beleren, blocked by Hill Giant, assignment 3/0/4).
- `combat.rs:2471:17: replace && with || in divide_freely` (an answer dividing a
  creature's damage among banding blockers was accepted with the right total but too few
  amounts): **(b) missing test**. Killed by
  `tests/keywords/k702_022j_banding_assignment_answers.rs` (Hill Giant blocked by
  Benalish Hero and Grizzly Bears; P1's answer `[3]` gets the default, both die).
- `combat.rs:2863:9: delete match arm 1 in block_attacker_of_choice`: **(a) equivalent**.
  With one attacking creature to choose from, the player is asked to choose it, and any
  answer other than that creature gets it anyway; only the pointless question differs.
- `combat.rs:3054:44: replace || with && in combat_timing_ok` ("after combat" was never
  reached): **(b) missing test**. Killed by `tests/cr/r506_008d_cast_only_after_combat.rs`
  (Glyph of Reincarnation can't be cast in the precombat main phase and can in the
  postcombat main phase).
- `combat.rs:3188:49: replace == with != in combat_trigger_matches` (the attackers
  recorded by "whenever [a player or permanent] is attacked" are those attacking something
  else): **(b) not killed**. No compiled card reads those attackers: the Curses of
  Commander 2017 find "each opponent attacking that player" from the combat itself
  (`kw/attacking_that_player.rs`), and "whenever one or more creatures attack you" cards
  compile to other triggers.
- `combat.rs:3245:29: replace && with || in combat_trigger_matches` ("whenever [a player]
  attacks [another player]" triggered when either player matched): **(b) missing test**.
  Killed by `tests/cr/r508_003e_player_attacks_you_trigger.rs` (Jolene, the Plunder Queen:
  only an attack on one of its controller's opponents makes a Treasure; the file also
  covers Emberwilde Captain, which compiles to a different trigger).

### zones.rs (2026-11-12)

First run, during the `engine-audit-1/zones-lki` audit, against the implementer's first
commit `d5a6c7b` (its tests then: the six phasing fixes' tests; `zones.rs` itself hasn't
changed since, so the line numbers are today's): 29 of 288 mutants (round-robin 0/10), 2
hours. 19 caught, 6 missed, 1 timeout, 3 unviable, kill rate **76.9%** (20/26). Every
sampled mutant was reached. Two survivors are killable and were killed by tests written in
the review, each checked by applying the mutant to the worktree by hand (the `cr` binary
then fails in that test alone; no `RERUN` yet): **84.6%** (22/26) with them. The other four
are equivalent.

- `zones.rs:164:26: replace || with && in own_move_replacements` (a moving object's own
  zone-change replacement abilities aren't looked at for a move to a hidden zone, nor for a
  face-down move): **(a) equivalent with today's cards**. The mutant also collects them for
  a face-up move into a library or hand, and for a face-down move of an object that isn't
  face down yet (a card manifested from a library). The function only returns
  `ZoneChange` replacements about the object itself: "If ~ would be put into a graveyard
  from anywhere", "If ~ would die" and a granted "If this permanent would leave the
  battlefield, exile it instead". None matches a move into a hand or library from a hidden
  zone, a permanent's are already collected with the battlefield's static abilities (and
  deduplicated), and no face-down move goes to a graveyard. The "If ~ would enter, ...
  instead" ones (Mox Diamond, Lotus Vale) are `EntersBattlefield` replacements, which the
  hypothetical entry hides for a face-down permanent. The review's
  `a_manifested_card_enters_without_its_own_enters_replacement`
  (`tests/cr/r708_003_audit_face_down_entry.rs`: Soul Summons manifests Mox Diamond, which
  doesn't ask for a land or go to the graveyard; cast face up, it goes to the graveyard)
  passes with the mutant; it pins that hiding instead (it fails, with
  `r708_face_down::permanents_put_onto_the_battlefield_face_down_have_no_enters_abilities`,
  when `with_hypothetical_entry` doesn't turn the card face down).
- `zones.rs:335:36: replace && with || in record_cost_moved` (the cards an action in the
  cost exiled, `COST_EXILED`): **(a) equivalent with today's cards**. The mutant adds every
  other live object the cost used (a creature it tapped, a card it revealed) and dead
  objects last seen in exile. Only an ability whose cost has an exile part reads
  `COST_EXILED` ("the exiled card"), and no such cost in the pool also taps, reveals or
  uses a permanent or card that stays where it is, or moves a card out of exile.
- `zones.rs:922:76: replace == with != in update_revealed_tops` (a top card revealed again
  became a new object, CR 401.6: the entries of players who may look at that library's top
  card follow it): **(a) equivalent in practice**. `looked_top_by` is rebuilt from the
  effects at the start of every call, and the function runs on every recompute; the card
  that became a new object is revealed, so everyone sees it anyway. Until the next
  recompute the mutant leaves a stale entry for the old object (dead, so never visible) and
  points a viewer's entries for other players' top cards at the revealed one.
- `zones.rs:1073:10: replace < with > in order_simultaneous_stack_objects` and
  `zones.rs:1092:28: replace - with / in order_simultaneous_stack_objects` (CR 405.3, the
  copies of "each other player copies that spell"): **(b) missing test**. The first returns
  early whenever anything is below the new objects (always: the copied spell is), so a player
  with more than one copy doesn't order them; the second truncates the stack to `n / k`
  objects before putting the copies back, which loses objects below once there are enough of
  them (here, with the resolving trigger still on the stack, five below two copies).
  Killed by `a_player_orders_their_simultaneous_copies_above_the_rest_of_the_stack`
  (`tests/cr/r405_003_audit_simultaneous_copies.rs`: three Prodigal Pyromancer pings below
  Lightning Bolt; Hive Mind and Twinning Staff give P0 two copies, which P0 orders second
  copy lowest): with the first the copies keep their creation order, with the second the
  stack has 5 objects instead of 6 (the Bolt is gone).
- `zones.rs:1169:40: replace && with || in face_down_exiled` (pruning the piles of
  face-down exiled cards, CR 406.4): **(a) equivalent**. The mutant keeps the entries of
  objects that are no longer face down in exile. Pile numbers come from a counter, and a
  pile is looked up only for a face-down exiled card that can't be looked at, whose own
  entry both versions keep; the extra entries (dead objects, a card turned face up in exile)
  are never looked up.
- The timeout, `zones.rs:99:5: replace contradictory_moves -> Vec<MoveEv> with vec![]`,
  drops every batch of zone moves (the function filters all of them, not only those a
  state-based action contradicts), so a state-based action that should move an object
  repeats forever: a detection.

## Status (2026-10-21)

Done. All 15 core files are sampled (551 of 5,281 mutants, 520 viable): 427 killed
before the new tests (**82.1%**), 459 after (**88.3%**), counting the first run's
survivors that the re-run found caught (and `combat.rs:136`, killed by a test added in
review and checked by hand). The 61 survivors left are each triaged above:
equivalent (a), or behavior no card in the pool reaches yet (b, "not killed", with what a
test would need). One engine bug was found and fixed (multi-player search moves,
simultaneous.rs). Re-running a file later: `scripts/mutants.sh <stem>` for a new sample,
`RERUN=1 scripts/mutants.sh <stem>` for the survivors.

2026-11-10: `layers.rs` was re-sampled after the `engine-audit-1/layers` tests (66 tests after
the review, see "layers.rs, second run" above): 53 of 53 sampled mutants tested, 45 killed and 2
timeouts (**88.7%**), 6 equivalent survivors.

2026-11-11: `triggers.rs` was re-sampled after the `engine-audit-1/triggers` tests (see
"triggers.rs, second run" above): 17 of 55 sampled mutants reached when the audit landed, 14
killed (**82.4%**), 3 equivalent survivors; the other 38 remain to be tested.

2026-11-12: `zones.rs` was sampled for the first time, during the `engine-audit-1/zones-lki`
audit (see "zones.rs" above): 29 of 29 sampled mutants tested, 19 killed and 1 timeout
(**76.9%**), 6 survivors: 2 killed by review tests (**84.6%**), 4 equivalent.

## Second round, Nov 2026

The `mutation-2` item (`docs/agents/items/mutation-2.md`): full runs (every mutant, `FULL=1`
in `scripts/mutants.sh`) of the weakest core files, then fresh 1-in-10 samples of the rest.
Kill rate as above: (caught + timeout) / (caught + missed + timeout).

| File | Mutants | Caught | Missed | Timeout | Unviable | Kill rate | After new tests |
|---|---|---|---|---|---|---|---|
| simultaneous.rs (full run) | 279 | 191 | 75 | 0 | 13 | 71.8% (191/266) | 98.1% (261/266): the 5 survivors not killed are equivalent (one, `1117:14`, in code the fixes removed) |
| permissions.rs (full run) | 238 | 153 | 74 | 0 | 11 | 67.4% (153/227) | 81.9% (186/227): the 41 survivors not killed are equivalent (three, `514:17`–`516:17`, only since the `attach` fix) |

### simultaneous.rs (full run)

A full run (`FULL=1 scripts/mutants.sh simultaneous`) of the file at `e08e631`: all 279
mutants. The machine restarted after 87 of them, and the run was resumed with `FULL=1
RESUME=1` (which skips the mutants already tested and merges the results). About five
minutes per mutant (an incremental build, then the ~290 s `mtg-engine` suite), about 16 hours
in all. 191 caught, 75 missed, 0 timeouts, 13 unviable: kill rate **71.8%** (191/266).

The 75 survivors were classified from the code in six batches, which proposed tests T1–T37
and found ten engine problems (C1–C10, and the `any_player_may_have_damage` bug) and seven
latent ones (L1–L7). Every test was checked by hand to fail with its mutant applied to the
source (before the engine fixes) and to pass without it. Then the engine was fixed (commit
`cad08da`, with the tests) and the 75 survivors were re-run (`RERUN=1 scripts/mutants.sh
simultaneous`, about 3.5 hours): the 75 names matched 70 distinct mutants of the file at
`cad08da` (see "Gone with the fixes" for the five that two survivors shared), 64 caught and
6 missed. Two of the six were killable after all: the fixes' "search the same library
again" check had made the T24 search row blind to them (`764:17`, the re-run's `773:17`,
and `1086:14` → `true`, the re-run's `1099:14`); a test added for them
(`each_player_searches_once_for_each_opponent`, below) and a second re-run
(`RERUN=1 ONLY=...`, commit `e4adbed`) caught both. Final: **66 caught, 4 missed** (all
four equivalent). Of the seven survivors whose code the fixes removed, six were engine
problems the fixes replaced (counted as killed) and `1117:14` was equivalent in the run, so
the kill rate after the new tests is **98.1%** (261/266: 191 + 64 + 6), with 5 equivalent
survivors.

Line numbers below are the run's (`e08e631`); the fixes moved the code by 4 to 80 lines.

**Survivors killed by new tests (b)** (test file in brackets; the real-card ones first):

- `149:31` `||`→`&&` in `run` (keep performing the body's instructions after the game ended):
  a planned instruction is performed after a win. T3, `a_win_while_each_player_draws_ends_the_game_before_anyone_sacrifices` (`tests/cr/r104_001_game_ends_during_each_player.rs`). Overturns October's (a) for `100:31`.
- `314:17`, `315:17` `&&`→`||` in `records_decision` (every "if" a choice): Kwain's draws and life gains interleave, Borderland Explorer's searches stop being simultaneous. `kwain_every_player_draws_before_any_player_gains_life` (`tests/cards/followups_each_player.rs`), `borderland_explorer_players_discard_then_search_together` (`tests/rulings/r_s33_each_player_searches.rs`, which also cites Borderland Explorer's ruling), `a_decision_made_with_an_action_is_an_instruction_everyone_performs_first`. Overturns October's `249:17`.
- `346:9`, `349:9` (delete `Sel::Choose`/`Sel::Union` in `choosers`), `356:5` (`chosen_by` → `true`), `1441:24` (`&&`→`||` in `record_choice`): Highcliff Felidar's choices for each opponent recorded as the opponents' (CR 101.4b). `highcliff_felidar_no_opponent_is_recorded_as_choosing` (`tests/rulings/r_p138_apnap_choices.rs`); `345:5` (`choosers` → `()`), `346:9` and `356:5` also by the three tests of `tests/cr/r608_002f_one_player_chooses_for_each.rs`.
- `762:9` (delete the `Search` arm of `plannable`), `763:56`, `1086:14` (guard → `false`), `1090:17`, `1090:28`, `1091:17` (the "shuffle and put on top" test): opponents' searches performed in turn. `old_growth_dryads_opponents_search_then_shuffle_in_turn_order`, `borderland_explorer_players_discard_then_search_together` (`r_s33`), `boldwyr_heavyweights_a_clone_one_opponent_finds_cant_copy_what_another_finds` (`tests/cr/r701_023i_each_player_searches_then_cards_move.rs`, Clone's ruling). Overturns October's (a) for `766:17`.
- `803:55` `&&`→`||` in `choice_cost_candidates` (a creature that can't be sacrificed offered to pay "unless that player sacrifices a creature"): `acererak_the_equipped_creature_cant_pay_for_the_zombie` (`r_p138_apnap_choices.rs`, Assault Suit's ruling).
- `1037:49` `&&`→`||` (a planned discard ignores its filter): `flux_each_player_discards_the_cards_they_chose` (`tests/rulings/gap_simultaneity.rs`, Flux's ruling).
- `1175:55` (other players' life payments count against yours): `killing_wave_another_players_payment_doesnt_count_against_yours` (`tests/rulings/r_p137_resolution.rs`, Killing Wave's ruling).
- `1414:17`, `1414:63` (`>=`), `1423:5` (→ `1`), `1426:14` (guard → `false`): one action with what records it counted as a process, each player's part its own event. `browbeat_damage_to_every_player_who_takes_it_is_one_event` (`tests/cards/grant_targets.rs`: Master of Barbs triggers once), and `an_action_then_a_permission_and_what_records_it_is_one_action` (hand-built, `tests/cr/r608_002f_individual_processes.rs`), which also kills `1424:9` and `1425:9` (delete the `Seq` / `GrantPlayPermission` arms of `process_steps`).
- `1514:16` delete `!` (Opposition Agent never applies to a planned search): `an_opponents_cards_found_in_an_each_player_search_are_exiled` (`tests/cards/m_opposition_agent.rs`, Natural Balance).
- `1701:21`, `1702:64` (each part's "damage dealt this way" includes the others'): `spiteful_repossession_counts_each_opponents_damage_once` (`tests/cards/referents_this_way.rs`).
- `148:22` `-=`→`+=`, `/=` (`batch_hold` never released): `valgavoths_onslaught_manifests_one_at_a_time_after_an_each_player_spell` (`tests/rulings/r_p218_manifest.rs`, after Exhume).
- Hand-built bodies (no card in the pool reaches them), in `tests/cr/`:
  `r608_002e_each_player_instruction_parts.rs`: `139:14` (`==`, `<`: no trigger check between instructions; Archfiend of Ifnir sees the discards before it's sacrificed), `168:27` ("create a Treasure and a Clue" split; Ingenious Artillerist triggers once), `266:13`, `271:13` (a running count over coin flips / "may" not shared), `520:38`, `611:13` ("the life lost this way");
  `r101_004_each_player_simultaneous.rs`: `180:31` (`true`, `false`), `310:9`, `313:37`, `314:49`, `315:20`, `315:43`, `317:9`, `319:57`, `319:17`, `330:9` (how choices, records and recorded decisions are split into instructions; CR 101.4, 101.4c, 608.2e);
  `r608_002f_one_player_chooses_for_each.rs`: `928:38`, `953:14`, `984:46`, `1002:53`, `1026:14`, `1086:14` (`true`), `1052:54`, `1204:50`, `1204:74`, `764:17`, `345:5` (after the fixes, `764:17` and `1086:14` → `true` only by `each_player_searches_once_for_each_opponent`: "for each opponent, each player searches ...") ("for each opponent, you [act on an object you choose]" planned: the same object chosen twice, choices recorded as the opponents', damage counted twice; register `choices-for-each-player-made-in-turn`, provisional); `r701_023h_one_search_for_each_object.rs` also kills `764:17`;
  `r608_002f_planned_actions.rs`: `994:65` ("sacrifice ~" by a player who doesn't control it), `1100:17` ("if they do" after a planned search), `948:38` (CR 607.2c link);
  `r608_002f_individual_processes.rs`: `1374:23` (a step before every part after a process: the other players' single actions split; register `mixed-each-player-instruction`, provisional), `1725:36` (`==`, `<`), `1725:40`, `1725:43` ("players dealt damage this way");
  `r608_each_player_control.rs`: `each_chooser_knows_the_earlier_players_choices_of_what_to_gain_control_of` (`345:5`'s control record, CR 101.4b).

**Engine bugs (c)**, each fixed generally, with a regression test that fails before the fix:

- `any_player_may_have_damage` (Browbeat, Book Burning, Breaking Point, Vexing Devil; CR
  101.4, Vexing Devil's ruling "If any of them do, you sacrifice Vexing Devil."): the note
  that a player took the damage was each player's own, so an earlier player's choice was lost
  when the last one declined (Browbeat's target still drew; Vexing Devil wasn't sacrificed).
  The note now reads the flag it sets, so it is shared. `any_player_may_have_damage_an_earlier_players_choice_counts` (`tests/cards/grant_targets.rs`).
- C10 (the same cards; CR 101.4, 120.3f, 608.2f): each accepting player's damage was dealt
  in its own `deal_damage_batch`, so a source with lifelink gained life once per player.
  `plan` now plans a part that is one action and what records it as that action (`Plan::Then`),
  so the damage is dealt at once. `vexing_devil_with_lifelink_dealing_damage_to_two_opponents_is_one_life_gain` (Whip of Erebos, Ajani's Pridemate).
- C1 (`729:62`, `1233:14`; Parallel Evolution; CR 608.2f, 614.12, Squad Captain's ruling):
  token copies of objects nobody chooses were created one player at a time. Every token copy
  is now planned and created together; a copy choice is recorded under its real chooser.
  `parallel_evolution_creates_all_its_copies_at_the_same_time` (`tests/rulings/r_p200_creatures.rs`).
- C5 (outside this file, `resolve.rs`; Clone Legion, Doppelgang, Hate Mirage; Clone Legion's
  ruling "The token creatures all enter the battlefield at the same time."): each source's
  token copies were created in a separate event. `clone_legion_creates_all_its_copies_at_the_same_time`.
- C2, C3, C6 (`705:33` and `919:47`, `700:41`, `882:45`; no compiled card; CR 101.4,
  608.2e): "choose one", an "if" and "[that player] may" were settled player by player
  (after the earlier players had acted) when the action after them couldn't be planned or a
  choice came before them. They are now always plannable (their actions still fall back to
  being performed in turn). `each_player_chooses_one_before_anyone_creates_a_token`,
  `conditions_are_settled_for_every_player_before_anyone_sacrifices`,
  `each_controllers_you_may_is_asked_before_any_of_them_acts` (`r101_004_each_player_simultaneous.rs`).
- C4 (`764:17`; Krenko's Buzzcrusher, From the Ashes, Wave of Vitriol; CR 701.23i, 608.2f):
  the controllers of the destroyed lands searched, put the land onto the battlefield and
  shuffled one after another. Searches of object frames are now planned when they have one
  searcher, and a planned search reads "their library" (and looks for the cards) as the
  searcher, as `Game::exec` does. A player who searches the same library again in the
  instruction (two of their lands destroyed) continues the same search, one search for all
  those cards (CR 701.23h): a card found for one land isn't offered for the next, the cards
  found move together and the library is shuffled once (the review's version; see
  "Review"). `krenkos_buzzcrusher_the_controllers_search_then_shuffle_in_turn_order`,
  `a_player_searching_once_for_each_of_two_objects_finds_two_cards`,
  `searching_for_each_of_two_objects_then_shuffling_is_one_search_and_one_shuffle`.
- C8 (Identity Thief, Ugin's Labyrinth, Spell Queller's family; CR 607.2a, 400.7): "return the
  exiled card" read every object linked to the source in any zone, including the creature an
  earlier end step returned (linked as put onto the battlefield with it). It reads only cards
  still in exile now. `identity_thief_returns_only_the_card_it_exiled` (`tests/rulings/r_p024_misc.rs`).
- C9 (Old-Growth Dryads, Boldwyr Heavyweights, Tempt with Discovery, Hired Giant; Stranglehold's
  ruling "your opponents can’t choose to search, so they won’t shuffle"): an opponent who
  can't search was offered the search and then shuffled. The opt-in of "each opponent may
  search ..." is now offered only to a player who can search.
  `old_growth_dryads_an_opponent_who_cant_search_isnt_offered_it_and_doesnt_shuffle`.
- L5 (latent, but cheap; CR 614.6): a planned move onto the battlefield that a replacement sent
  elsewhere (Containment Priest) still counted as done ("if they do", "it").
  `a_card_put_into_exile_instead_of_onto_the_battlefield_wasnt_returned`.

**Not engine bugs, or not fixed:**

- C7 (a design question): instructions whose choices someone other than the frame's player
  makes are performed for each frame in turn. Kept, register
  `choices-for-each-player-made-in-turn` (provisional).
- L1, L2 (`Frames::finish`): a part's "that many" is detected by a change of value, so a part
  setting it to the value it had drops out of the total, and with no part setting it the total
  is the stale number from before. No compiled card has a number-setting instruction before a
  conditional each-player life change. Not fixed (needs a "number was set" flag in `Ctx`).
- L3: two parts planning the same (source, recipient) damage each count both events (no card).
- L4: the planned `Exile` skips some checks `Game::exec`'s `Exile` makes (the "can't cause you
  to exile" restriction, `found_after_move`, the face-down `looked_then_exiled`,
  `note_source_moved`, `set_dug`); no card reaches it.
- L6: in an instruction mixing processes and single actions, the first process's events share
  the single actions' batch (no card).
- L7: "dealt damage this way" isn't combined across the players after an each-player damage
  instruction, and a part's "excess damage" counts every part's (no card reads them).
- The other `Sel::Linked` readers without an exile filter (`r700_exiled_piles`,
  `gains_suspend(Sel::Linked)`) were left as they are (C8's note).

**Equivalent (a)**, still missed by the re-run:

- `139:14` `>`→`>=` in `run`: the extra boundary before the first instruction sees nothing new
  (`Game::exec` has just flushed; the state triggers were checked).
- `973:25` (delete `link` from the planned exile's `EtbInfo`): `move_objects` reads it as
  `unwrap_or(current_link)`, and `current_link == ctx.link` there.
- `1533:19` (guard `!found.is_empty()` → `true`): an empty search adds no move and its
  `enter`/`leave` pair writes back what it read.
- `1725:36` `>`→`>=`: no `Event::Damage` has amount 0 (CR 120.8).

**Gone with the fixes**: `700:41`, `705:33`, `729:62` (the `plannable` arms are now `true`),
`882:45`, `919:47`, `1117:14` (equivalent in the run, so counted as (a): `plannable(GainControl)`
was always true), `1233:14` (the guards are gone). The re-run script matches each survivor to
the most similar mutant of the file at `cad08da`; for these seven it found only unrelated ones (diff similarity
0.0–0.44: `700:41`, `705:33` → `755:17`, `729:62` → `752:59`, `882:45` → `1015:53`, `919:47` →
`941:38`, `1117:14` → `966:14`, `1233:14` → `1099:14`), so their re-run lines are those
mutants', not theirs. The new code (`Plan::Then`, the continued search, the `process_steps`
"if" arm, the shuffles of `perform_plans`) hasn't had its own mutants run; a later full run
of the file would.

**Review.** Every engine fix's regression test was checked again to fail with its fix
reverted (`any_player_may_have_damage`, C1, C2/C3/C6, C4, C5, C8, C9, C10, L5), and
`browbeat_damage_to_every_player_who_takes_it_is_one_event` turned out to pin the process
classification (`1414:17` and the rest), not C10, which `vexing_devil_with_lifelink_...`
pins. Changes:

- The merge with upstream's `per_object_order` (the order of one player's objects chosen
  as the instruction resolves, CR 608.2f): `choose_next_object` rotates the frames, plans and
  outcomes of a group, and now each part's `Plan::Then` records with them, so they stay with
  their part (`what_records_a_planned_part_stays_with_it_when_the_objects_are_reordered`,
  `tests/cr/r608_002f_individual_processes.rs`, hand-built: without the rotation a land's
  part ran the Bears' record).
- C4's repeated search: the implementer had a player's second search of the same library
  wait until the first one's cards had moved; with "then shuffle" after each search that's a
  new search after the shuffle, so two searches and two shuffles, against CR 701.23h ("The
  player searches that library only once"). It now continues the same search
  (`library::search_excluding`: the cards found for the earlier parts can't be found again),
  all the found cards move together and the library is shuffled once
  (`searching_for_each_of_two_objects_then_shuffling_is_one_search_and_one_shuffle`, which
  fails with the implementer's version: two searches, two shuffles).
- The register entry `choices-for-each-player-made-in-turn` names its exceptions ("choose
  one", conditions and "[that player]", token copies, searches), quotes 701.23h, and names
  the compiled cards its search part affects (From the Ashes, Wave of Vitriol, Krenko's
  Buzzcrusher); the item had "no compiled card".
- Tests: Old-Growth Dryads with Stranglehold cited 701.23a and 118.12b, but the card has
  no "if they do" clause: now 101.2 and 608.2d (a player can't choose to search when they
  can't); the Krenko's Buzzcrusher test quoted its lands as targets (they aren't).
- `1117:14` (equivalent in the run, code removed) counted as equivalent, not killed: 98.1%.

### permissions.rs (full run)

A full run (`FULL=1 scripts/mutants.sh permissions`) of the file at `43fcf29`: all 238
mutants. The run was stopped by the 2-hour session limit several times and resumed each time
with `FULL=1 RESUME=1 REV=43fcf29` (this worktree's own `RESUME`, through `--iterate`, `9c8dd49`–`cb69cb6`; the merge took the
simultaneous.rs run's version of the script instead, which does the same). About
3.5 minutes per mutant. 153 caught, 74 missed, 0 timeouts, 11 unviable: kill rate **67.4%**
(153/227).

The 74 survivors were classified from the code in eight batches, written while the run went
on: 35 killable (b), 39 equivalent (a), and three engine bugs (c), found triaging `196:32`,
`514:17` and `969:45`. The tests were committed with their batches (`f67b79f`–`ccf8cb7`,
unbuilt until the run ended); then the engine was fixed (`c83fe93`: fixes 1 and 2 and the
register entry `own-permission-first`; `09bc166`: fix 3). The 74 survivors were re-run at
the fixed commit (`RERUN=1 scripts/mutants.sh permissions`; the fixes moved the code by up
to 15 lines): 32 of the 35 killable ones caught; the other three, `514:17`, `515:17`,
`516:17`, missed, because fix 2 made them equivalent (below); the 39 equivalent ones
missed. One of those, `347:13`, was killable after all (a land gets a free copy of its
permission, and playing it asks which permission; below): a test was added (`50c47b1`) and a
second re-run (`RERUN=1 ONLY=...`) caught it. Final: **33 caught, 41 missed** (all
equivalent), kill rate after the new tests **81.9%** (186/227: 153 + 33).

Line numbers below are the run's (`43fcf29`). New tests are in `tests/cr/` unless named
otherwise.

**Survivors killed by new tests (b)**:

- `played_this_way` (`74:28`, `75:28`, `75:50`, `&&`→`||`): a card played with any other
  permission counts as played "this way" (a delayed ability's way is never 0; every static
  permission's use is way 0, so `None` matched any source).
  `fires_of_mount_doom_ignores_a_land_played_with_another_permission` (a Mountain played with
  Crucible of Worlds deals no damage; Fires of Mount Doom's ruling "but don't play the exiled
  card this turn using that casting permission, ...") and
  `galea_ignores_an_equipment_cast_with_another_permission` (Bonesplitter cast with
  Muldrotha's permission doesn't trigger Galea), both in
  `r601_003_played_this_way_other_permissions.rs`.
- `in_permission_zone` (`180:5` → `true`, `184:30`/`184:44`, `196:32`, `197:32`, `201:27`,
  `202:34`) and `allowing`'s `279:53` (`||`→`&&`): a static permission applied to a card
  another permission lists outside its zone ("top of your library" as any library's top,
  "your graveyard" as any graveyard, "top of your graveyard" as the whole graveyard, "your
  hand" as any hand), or another player's permission read as yours (CR 109.5). One test
  per pair of permissions in `r601_003_permission_zones.rs`:
  `bosium_strip_casts_only_the_top_card_while_crucible_of_worlds_lists_the_graveyard`,
  `crucible_of_worlds_doesnt_play_a_land_from_another_players_graveyard` (Coram, the
  Undertaker lists every graveyard), `omniscience_doesnt_cast_spells_from_another_players_hand`
  (three players, Sen Triplets), `future_sight_doesnt_cast_the_top_card_of_another_players_library`
  (three players, Xanathar, Guild Kingpin), and for `279:53`'s second half
  `another_players_crucible_of_worlds_doesnt_play_a_land_from_your_graveyard` (Lurrus of the
  Dream-Den's ruling "Lurrus doesn't let you play lands from your graveyard.").
- `effect_permitted_cards` (`407:34`/`407:48`, `419:36`, `424:36`, `429:31`; the grants'
  loop `455:28`, `455:53`): the list of cards a player may play outside the hand included
  cards no permission of theirs covers, and mutate's way (which asks only that list) let them
  be cast for their mutate cost with no permission (CR 601.3). Four tests in
  `r601_003_permission_effects.rs`:
  `future_sight_doesnt_offer_mutate_for_the_top_card_of_another_players_library` (Future
  Sight's ruling "If the card has alternative or additional costs, you may pay those."),
  `bosium_strip_doesnt_offer_mutate_for_a_card_under_the_top_of_your_graveyard` (Muldrotha's
  ruling "You must pay the costs to cast a spell this way. If it has an alternative cost, you
  may cast it for that cost instead."), `omniscience_doesnt_offer_mutate_for_a_card_in_another_players_hand`,
  and `another_players_permission_for_a_card_doesnt_offer_you_mutate_for_it` (P1 activates
  Fires of Mount Doom: the Sea-Dasher Octopus it exiles is P1's to play, not P0's).
- `attach` (`510:46` `>`→`==`, `>=`): every permission "changes the way", so a card in the
  hand with Omniscience out gets a second "for its mana cost" way with Omniscience's
  permission, and casting it asks which permission. `a_card_cast_from_hand_for_its_mana_cost_with_omniscience_out_asks_no_permission`
  (`r601_003_permission_choice.rs`, `interp!("own-permission-first")`).
- `spell_relative_cost` (`583:38` guard → `false`, `598:21` delete the `KeywordAction` arm):
  "its mana value" left unresolved, so the legal-action check read it for the card where it
  is. `hama_waterbends_the_mana_value_of_the_half_cast_from_a_split_card` (Fire // Ice in
  exile: waterbend {2}, Fire's, not {4}; CR 709.4b; Fire // Ice's ruling "A split card's
  characteristics are a combination of its two halves while it is not on the stack" and Hama,
  the Bloodbender's on tapping creatures to waterbend), `r601_003_permission_choice.rs`.
- `choose_land_permission` (`620:42` `||`→`&&`): a land with its own mayhem permission
  (Oscorp Industries) was played with Muldrotha's once-each-turn land.
  `a_land_played_with_its_own_mayhem_ability_leaves_muldrothas_land_unused`
  (`r601_003_permission_choice.rs`, `interp!("own-permission-first")`; Muldrotha's ruling "If
  multiple effects allow you to play a card from your graveyard, you must announce which
  permission you're using ...").
- `choose` (`715:48` `<`→`<=`): an answer one past the last permission named was taken (the
  last way, not the default; for a land it would index out of bounds).
  `an_answer_one_past_the_last_permission_uses_the_default_permission`
  (`r601_003_permission_choice_answers.rs`, beside the `Index(7)` test; Muldrotha's ruling
  "you may cast an artifact creature spell as your artifact spell and cast another artifact
  creature spell as your creature spell").
- `label` (`984:5` → `String::new()`, `"xyzzy".into()`; `995:12` delete `!`): the
  permissions asked by name all read the same, so an agent can't announce the one it means
  (the same defect as `casting.rs:420:41`'s option names, also (b)).
  `the_permissions_asked_by_name_are_named_by_source_and_use` (the options "Muldrotha, the
  Gravetide's permission (as your artifact spell)" / "(as your creature spell)"; the "you must
  announce" ruling), `r601_003_permission_choice_answers.rs`.
- `record_use` (`811:72` `==`→`!=`): the count left of a limited group was written onto every
  other permission, so all of March of Reckless Joy's cards could be played.
  `march_of_reckless_joy_plays_only_two_of_the_exiled_cards` (`r601_003_permission_records.rs`;
  its ruling "You choose which cards that you are playing from exile as you play them, ...").
- `note_way` (`917:5` → `()`, `917:15` `>=`→`<`): the permissions kept way 0, so with two
  activations a card cast with the first's permission triggered both delayed abilities.
  `each_fires_of_mount_doom_activation_deals_damage_only_for_its_own_card`
  (`r601_003_permission_records.rs`, 18/18 not 16/16; Fires's "this casting permission"
  ruling).
- `given` (`969:45`, `975:64`, `&&`→`||`): "until you exile another card with ~" permissions
  ended by any permission with terms from the same object, or by any object's new one.
  `myr_welders_other_exile_ability_doesnt_end_its_unstable_amulet_permission` (Myr Welder
  with Unstable Amulet and Oracle's Vault imprinted; CR 607.1, 607.1c, 607.5; Myr Welder's
  ruling "treat Myr Welder's version of that ability as though it referenced Myr Welder by
  name instead") and `another_unstable_amulet_exiling_a_card_doesnt_end_this_ones_permission`
  (CR 700.7), both in the new `r601_003_permission_given.rs`. The Myr Welder kill rested on
  Oracle's Vault's free-play text compiling to `WithPlayTerms`; the re-run confirms it.
- `347:13` (`free_option`, `land || last.free || ...` → `land && last.free || ...`, classified
  (a) at first, killed after the re-run): a land the permission allows gets the "without
  paying their mana costs" way too, the same permission twice, and playing it asks "Choose
  the permission you're using". `a_land_ignite_the_future_exiled_after_flashback_is_played_without_a_permission_choice`
  (`r601_003_permission_choice.rs`: an Island Ignite the Future exiled after flashback is
  played with no question; CR 305.1, 601.3).

**Engine bugs (c)**, each fixed generally, with a regression test that fails before the fix:

- Shaman's Trance (CR 609.4, 609.4a; found triaging `196:32`): "You may play lands and cast
  spells from other players' graveyards this turn as though those cards were in your
  graveyard" extended only the keywords' graveyard permissions (flashback, retrace, ...), not
  effects' "from your graveyard" ones (Crucible of Worlds, Muldrotha, Yawgmoth's Will,
  Lurrus). `in_permission_zone`'s graveyard arm now asks `as_though::in_graveyard_for`, and
  `effect_permitted_cards` adds `as_though::other_graveyard_cards`; the `any_owner` and
  "top of your graveyard" arms stay. `shamans_trance_lets_crucible_of_worlds_play_a_land_from_another_players_graveyard`
  (`r601_003_permission_zones.rs`). Commit `c83fe93`.
- One permission's cost per way (CR 702.81a, 601.2f; found triaging `514:17`–`516:17`):
  `attach` combined a card's own way whose additional cost comes with its own permission
  (retrace, jump-start, Rona, Sheoldred's Faithful) with another permission's terms: Risen
  Executioner with retrace (Six) had a third way, a land discarded *and* {1} more for each
  other creature card. The player uses one permission (the Muldrotha ruling), and the total
  cost is that way's. `attach` now skips, outside the hand, own ways with an additional cost
  of their own, as it already did for flashback's alternative cost (in the hand an own
  additional cost belongs to a timing permission, which an effect's permission may combine
  with). `risen_executioner_cast_with_retrace_doesnt_cost_more_for_other_creature_cards`
  (`r601_003_permission_ways.rs`, the Muldrotha ruling). Commit `c83fe93`. Since the fix
  `514:17`, `515:17` and `516:17` are equivalent with today's cards (no own way without a
  cost of its own meets those terms), so they count as (a), not killed.
- Unstable Amulet's "until you exile another card with this artifact" (CR 109.5, 611.2a;
  found triaging `969:45`, `975:64`): `given` ended a source's earlier such permissions
  whichever player the new one was for, so P1 exiling a card with the Amulet they took (Steal
  Artifact) ended P0's (likewise Superior Foes of Spider-Man under Act of Treason, Furious
  Rise under Confiscate). `newer` now pairs the source with the player the permission is for.
  `another_player_exiling_a_card_with_your_unstable_amulet_doesnt_end_your_permission`
  (`r601_003_permission_given.rs`). Commit `09bc166`.

**Equivalent survivors (a)**, missed by the re-run:

- `102:9` (`true`, `false`), `102:19` (`CastPermission::eq`): dead code; nothing compares
  permissions (`CastOption` has no `PartialEq`, `same_way` leaves the permission out).
- `192:24` (`in_permission_zone`'s library branch without `top_only`): every library
  permission is for the top card.
- `202:9` (delete the `Hand` arm): differs only for a card in the player's own hand, where
  nothing reads the answer (the hand needs no effect's permission; hand permissions change no
  way). Likewise `435:13` in `effect_permitted_cards`: every reader takes the hand's cards
  without the list.
- `203:9`, `203:37`, `204:9`, `204:37` (`Command`/`Outside` arms), `436:13`, `439:13`: no play
  permission is built for the command zone or outside the game (commanders and Wish go other
  ways; the cards that would need one are legal in no format).
- `296:30` (delete `!` in a static permission's condition): dead path; no collected play
  permission has `PlayTerms::condition` set (a static ability's condition is checked by
  `collect_statics`, and no card's timed class permission has one).
- `347:26` (`free_option`'s guard, `||`→`&&`): the free rider exists only on Nahiri, Forged in
  Fury and Ignite the Future, whose grants have no free flag and no alternative cost, so the
  guard's last two terms are false and only `land` matters, as in the original.
- `371:30` (every custom quality dropped from the qualities rechecked once proposed): the
  custom qualities play permissions hold are provenance or graveyard history, which casting
  doesn't change; the recheck repeats the first check.
- `380:5`, `385:34`, `391:21` (`may_be_permitted` → `true`, or wider): a fast path; where the
  original says false, `attach` returns the card's own ways unchanged.
- `510:46` `>`→`<`, `511:17`, `512:17`, `513:17` (`attach`'s way-changing terms): Urianger
  Augurelt's reduction, the Adventure face, Abstruse Appropriation's and Draugr Necromancer's
  spend terms never meet a card's own way. `514:17`, `515:17`, `516:17`: equivalent since fix 2
  (above).
- `583:38` (guard → `true`): every relative amount reaching `spell_relative_cost` is
  `ManaValueOf(Sel::This)`.
- `671:39`, `697:47` (`-`→`+`, `/`; the clamps): the index is always below the length.
- `736:68` (`matches_for_some_x`, `&&`→`||`): trying X on a cost without {X} only merges its
  generic symbols, which no filter can tell; no land has an {X}.
- `906:13` (`lock_permission_players`, delete the `Not` arm): no class permission has an
  `OwnedByPlayer` under a `Not`.
- `921:20` (`true`), `921:23` `>=`: `THIS_WAY` is never stored as 0. `921:20` (`false`),
  `921:23` `==`, `<`: a fresh way for each permission instruction differs only for an effect
  with two such instructions and a "this way" ability, which no card has.
- `957:44`: the group id taken for nobody shifts every later effect id by one, which keeps
  every comparison. `960:36` (`+`→`*`, `-`): a group need only be nonzero and its own
  (`new_effect_id()` is at least 2 there). `962:41`: no limited `WithPlayTerms` wraps another
  or more than one grant.

**Not fixed / notes:**

- Myr Welder that has imprinted two Unstable Amulets (or Mairsil, the Pretender caging them):
  the engine identifies the "until another" ability by its source object, so each Amulet's
  ability ends the other's permissions, though CR 607.1c ("An ability printed on an object
  that fulfills both criteria described in rule 607.1 is linked to itself.") and 607.5 keep
  each linked only to itself. Telling them apart needs the ability's identity on the grant;
  no other card has two such abilities.
- Register entry `own-permission-first` (`docs/interpretations/decisions-policies.md`,
  policy, provisional): a card's own permission is the one used when an effect's permission
  would allow the same way, without asking; the player is asked only among effects'
  permissions. Rules: 601.3 "A player can begin to cast a spell only if a rule or effect allows
  that player to cast it and no rule or effect prohibits that player from casting it.";
  702.187c "“Mayhem” without a cost means “You may play this card from your graveyard if you
  discarded it this turn.”"; Muldrotha, the Gravetide's ruling (2020-11-10): "If multiple
  effects allow you to play a card from your graveyard, you must announce which permission
  you're using as you begin to play the card." The card's own permission can't be used up,
  so announcing it leaves the player every option. Cited by the Omniscience and Oscorp
  Industries tests.
- Latent, no card: a static permission's use is identified only by its source (way 0), which
  would confuse two static permissions of one source; `choose_cast_permission`'s default for a
  group mixing the card's own way with effects' is option 0 (the own way), not the first
  payable one as its doc says (after fix 2 no such group arises with today's cards).
- The new code (the `as_though` arms, `attach`'s extra-cost skip, `given`'s per-player
  `newer`) hasn't had its own mutants run.
