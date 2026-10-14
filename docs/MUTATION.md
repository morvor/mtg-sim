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
- The re-run after adding tests (`RERUN=1`) selects the survivors by their exact
  cargo-mutants names (`-F '^<name>$'`), so it must run on the same revision of the file.

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
  instructions once the game is over): **(b) missing test**. Killed by
  `tests/cr/r104_001_game_ends_during_each_player.rs` (Laboratory Maniac wins the game
  while each player draws for Burning Inquiry; nobody discards afterwards, CR 104.1).
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
- `turn.rs:725:17: replace || with && in Game::current_phase_end`: **(b) missing test**. A
  step of another phase no longer started a new phase unless it was a main phase, so
  "after this main phase, there is an additional combat phase" cast in the postcombat main
  phase put the combat after the ending phase. Killed by
  `tests/cr/r500_008_combat_after_the_postcombat_main_phase.rs` (Relentless Assault in the
  postcombat main phase).
