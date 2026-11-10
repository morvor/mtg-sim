# mutation-2: full mutation runs of the weakest core files, fresh samples of the rest

WHY: docs/agents/finish-line-plan.md, criterion C5 and risks 3, 4, 6. docs/MUTATION.md sampled
one mutant in ten of the 15 core files (551 of 5,281) in October: 88.3% killed after new tests,
but simultaneous.rs 55.6%, triggers.rs 75.0%, permissions.rs 78.6%, mana.rs 81.0%,
replacement.rs 82.4%. Since then simultaneous.rs, permissions.rs, trigger_timing.rs,
casting.rs's cost code and range.rs were rewritten or extended by the gap items, and no
mutation run has looked at them. The one engine bug mutation testing found came from a
survivor in simultaneous.rs. The 15 files have grown to 33,113 lines since the October sample
(5,281 mutants then), so the mutant counts below are October's; take the current ones from the
run.

Machine rules (CLAUDE.md, docs/MUTATION.md): `export CARGO_BUILD_JOBS=2`; one file at a time
under `scripts/mutants.sh`; nothing else heavy on the machine while a run is going (tell the
coordinator which hours the machine is taken); land the report after every file so an
interruption loses at most one file.

## Runs

1. **Full runs** (every mutant, not a shard; edit the `--shard` argument or add a `FULL=1` switch
   to `scripts/mutants.sh`): `simultaneous.rs` (171 mutants, about 6 h), `trigger_timing.rs`
   (17) with `triggers.rs` (474, about 16 h: run it overnight in two halves),
   `permissions.rs` (169), `replacement.rs` (361, about 12 h). Order: simultaneous, permissions,
   trigger_timing, replacement, triggers.
2. **Fresh 1-in-10 samples** of the other eleven core files on the current code (sba, turn,
   mana, eval, stack, resolve, actions, layers, casting, combat, and `multiplayer/range.rs` as a
   new entry), with a different shard than October's (`--shard 1/10`) so the sample is new.
3. For every survivor: classify it as (a) equivalent (say why, with the rule or the code path
   that makes the change unobservable), (b) a missing test, or (c) an engine bug. Kill every (b)
   with a test using real cards and `cr!`/`ruling!` citations; fix every (c) generally with a
   regression test. Re-run the file's survivors (`RERUN=1`) and record before/after.

## Report (docs/MUTATION.md, new section "Second round, Nov 2026")

Per file: mutants, caught, missed, timeout, unviable, kill rate before and after; every
survivor with its classification; the engine bugs found with their rules. Then the overall
rate. Target: every file at or above 90% with every remaining survivor classified (a).

DONE when the five full runs and the eleven fresh samples are in docs/MUTATION.md with
their survivors classified and the (b)/(c) ones killed, `cargo test --workspace` passes, and the
item's row (mutants run, missing tests added, engine bugs) is in docs/agents/yield.tsv.
