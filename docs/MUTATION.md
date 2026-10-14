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
