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
  --timeout 300 --build-timeout 900 --cap-lints true --test-package mtg-engine \
  --shard 0/10 --sharding round-robin -o target/mutants/<stem>
```

- `target/mutsrc` is a copy of the worktree (without `target/` and `.git`) refreshed before
  each file, so one incremental build serves every mutant and an interrupted run never
  leaves a mutation in the real tree. `--in-place` there also means one mutant at a time.
- A whole run over the 15 files would be about 5,200 mutants at roughly two minutes each
  (an incremental build of the engine and its seven test binaries takes ~50 s, the full
  `mtg-engine` suite ~105 s), i.e. a week. So each file is **sampled**: round-robin shard
  `0/10` takes every tenth mutant in source order, which spreads the sample evenly across
  the file. Files with fewer than 60 mutants run in full.
- `--timeout 300` bounds a mutant's test run (the unmutated suite takes ~105 s), so a
  mutant that makes the engine loop counts as `timeout`, which is a detection.
- The re-run after adding tests (`RERUN=1`) finds each survivor again in the file's
  current mutant list (same function and mutation whose diff, the mutated line with the
  lines around it as saved in the earlier run's `mutants.out/diff/`, matches best; then
  the same column and the nearest line, since merges move lines) and tests exactly those
  (`-F '^<name>$'`). Matching by line distance alone picked a neighbouring identical
  mutation once code had moved by more than the distance between them.

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
| trigger_timing.rs | 17 | 17 (all) | 14 | 2 | 0 | 1 | 87.5% | TBD |
| permissions.rs | 169 | 17 (1/10) | 10 | 4 | 0 | 3 | 71.4% | TBD |
| simultaneous.rs | 171 | 18 (1/10) | 6 | 12 | 0 | 0 | 33.3% | TBD |
| turn.rs | 234 | 24 (1/10) | 21 | 2 | 0 | 1 | 91.3% | TBD |
| mana.rs | 228 | 23 (1/10) | 15 | 6 | 0 | 2 | 71.4% | TBD |
| eval.rs | 268 | 27 (1/10) | 25 | 0 | 0 | 2 | 100% | 100% |
| stack.rs | 361 | 37 (1/10) | 33 | 3 | 0 | 1 | 91.7% | TBD |
| replacement.rs | 361 | 37 (1/10) | 28 | 6 | 0 | 3 | 82.4% | TBD |
| triggers.rs | 474 | 48 (1/10) | 32 | 16 | 0 | 0 | 66.7% | TBD |
| resolve.rs | 314 | 32 (1/10) | 22 | 8 | 0 | 2 | 73.3% | TBD |
| actions.rs | 390 | 39 (1/10) | 27 | 5 | 3 | 4 | 85.7% | TBD |
| layers.rs | 423 | 43 (1/10) | 34 | 5 | 1 | 3 | 87.5% | 87.5% (no killable survivor) |
| casting.rs | 841 | 85 (1/10) | 74 | 9 | 0 | 2 | 89.2% | TBD |
| combat.rs | 876 | 88 (1/10) | 68 | 13 | 0 | 7 | 84.0% | TBD |

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
  nothing whether or not the loop stops; the test stays as the CR 104.1 check.
- `simultaneous.rs:224:20: replace == with != in updates_number::reads`: **(b) missing
  test, not killed**. The mutant makes every instruction that stores a number count as one
  whose number is shared by all players. The cards whose per-player "that many" goes
  through this code record it per player anyway (`RECORD_THAT_MANY`,
  `kw/hand_graveyard_actions.rs`), and the other per-player numbers seen in the card pool
  (the outer X of "each player ... X") hold the same value for every player, so no card
  test found shows the difference.
- `simultaneous.rs:249:17: replace && with || in records_decision`: **(b) not killed**.
  It turns "if ..., [action]" inside an "each player" body into a choice attached to the
  next instruction; the actions still happen in the same order for each player, and only
  the trigger checks between the two instructions move (to after the second one). No
  observable case was found with today's cards.
- `simultaneous.rs:262:9: delete match arm Effect::Choose{..} in makes_choice`: **(b)
  missing test**. A "choose a creature type" became its own instruction, so all players
  chose before anyone acted and every player's action read the last choice. Killed by
  `tests/cr/r101_004_each_player_chooses_then_acts.rs` (Grave Sifter: each player returns
  cards of the type *they* chose).
- `simultaneous.rs:283:9: delete match arm Sel::Union(v) in choosers`: **(b) not killed**:
  needs an each-player move of "this and that" where another player chooses one of the
  parts; no such card was found.
- `simultaneous.rs:552:5: replace plannable -> bool with true`, `557:41: replace && with
  || in plannable` (an "if" counts as plannable when only one branch is): **(b) not
  killed**. They plan actions whose choices another player makes (or "if/otherwise" with an
  unplannable branch) together instead of in turn. With the cards tested, the planned and
  the in-turn performance produce the same results; a test needs a card where one player's
  part depends on what an earlier player's part already did, chosen by a third player.
- `simultaneous.rs:664:46: replace match guard plannable(g, e, ctx, p) with false in plan`
  (sacrificing chosen objects, "sacrifices the rest", in turn instead of together):
  **(b) not killed**. Simultaneous sacrifice is observable with a "whenever another
  creature dies" permanent sacrificed in the same event as another player's creature (it
  sees that death only if they leave together). The cards tried compile to their own
  effects instead of this path (Cataclysm is `KeepAndSacrificeRest`, Balance is
  unsupported); the behavior itself is now pinned for Cataclysm by
  `tests/cr/r608_002f_each_player_sacrifices_together.rs` (Blood Artist sees the death of
  the opponent's creature sacrificed with it).
- `simultaneous.rs:628:38: replace == with != in plan` (link the moved permanents to the
  effect's source when they enter the battlefield, CR 607.1d): **(b) not killed**: no card
  puts a permanent onto the battlefield with an "each player" move whose abilities refer to
  the card linked to the object that put it there.
- `simultaneous.rs:766:17: replace && with || in plan` (a search that shuffles is performed
  in turn instead of planned): **(a) equivalent for the cards that reach it**. The
  "each player may search ... put onto the battlefield, then shuffle" cards (Veteran
  Explorer, Old-Growth Dryads) find basic lands, which make no choices as they enter, so the
  lands entering one player at a time inside one batch looks the same.
  Investigating it found a real bug in the other multi-player search path, see below.
- `simultaneous.rs:1061:21: delete match arm Event::Damage{source, target, ..} in record`,
  `1081:47: delete ! in record`: **(b) not killed**. They change a player's "damage dealt
  this way" total and "creatures dealt damage this way" when several players' parts deal
  damage from the same source; no card in the pool reads those results after an
  "each player" damage instruction.

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
- `mana.rs:937:24: replace -= with += in PoolSolver<'_>::solve` (undo the life a Phyrexian
  symbol tentatively paid when the rest of the cost can't be paid that way): **(b) missing
  test**. With the mutant the abandoned life stayed counted, so a later, valid way of
  paying exceeded the life total and the spell couldn't be cast. Killed by
  `tests/cr/r107_004f_phyrexian_payment_backtracking.rs` ({B/G}{G/P}{G/P} from {G}{B}
  with 3 life).

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
  triggered ability between the event and the check, which no test sets up.
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
  card creates one there.
- `triggers.rs:1088:21` (Attacks: `objects`, all the attackers), `1272:25`
  (IsDealtDamage: `other`, the source), `1528:25` (SpellCopied: `object`; `spell` holds the
  same copy), `1606:25` (TokenCreated: `player`), `1650:25` (TappedForManaOfType:
  `object`), `1758:25` (Mills: `objects`), `1843:25` (chapter abilities: `lki`): **(b) not
  killed**: no card was found whose ability reads that field for that trigger (the "they" of
  "whenever one or more creatures you control attack" comes from each attacker's
  `object`, which Angelic Guardian's test checks).
- `triggers.rs:1204:29: delete field object` (BlocksOrBecomesBlocked, the attacking
  creature that became blocked): **(b) missing test**. Killed by
  `tests/cr/r509_003_blocks_or_becomes_blocked_it.rs` (Raging Gorilla gets +2/-2 when it
  blocks and when it becomes blocked).
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

- `combat.rs:136:9: replace CombatState::attacked_alone -> bool with false`: **(a)
  equivalent (dead code)**. The function has no caller (exalted and "attacks alone"
  triggers use `TriggerCond::AttacksAlone` on the declaration event).
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

## Status (2026-10-21)

All 15 files have been sampled: 520 viable mutants (plus 31 unviable), 423 caught, 4
timeouts, 93 missed: a kill rate of **82.1%** before the new tests. The casting.rs and
combat.rs survivors are triaged and have tests. Remaining:

1. Re-run the survivors of every file with new tests (`RERUN=1 scripts/mutants.sh
   trigger_timing permissions simultaneous turn mana stack replacement triggers resolve
   casting combat`, about 6 hours at ~4 minutes a mutant) and fill in the "After new tests"
   column and the overall kill rate after. Each new test was written against its mutant's
   code and checked to pass on the unmutated engine (the casting.rs and combat.rs ones
   also to fail with their mutant applied); the re-run is what confirms the kill.
