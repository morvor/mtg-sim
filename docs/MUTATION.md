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
