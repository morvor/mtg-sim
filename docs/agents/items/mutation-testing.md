# mutation-testing: measure whether the test suite catches regressions in core engine logic

WHY: the suite has about 12,800 tests, and most of the effort now goes into tests. Counting tests says nothing about whether they would catch a real regression. Mutation testing does: change the engine code on purpose (flip a comparison, drop a statement, return a default) and check that some test fails. A mutant that survives marks behavior no test pins down.

1. Tool: cargo-mutants (`cargo install cargo-mutants --locked`; it needs crates.io access). If it can't be installed, write a small script that applies the same kinds of mutations to one function at a time and runs the relevant test binary.
2. Machine limits (CLAUDE.md): 4 cores, ~15 GB RAM; running out of memory restarts the machine. Use `CARGO_BUILD_JOBS=2`, one mutant at a time (`-j 1`), a per-mutant timeout, and never run another cargo build in parallel.
3. Scope, since a full run over the whole engine would take days: the core rules files first, i.e. sba.rs, layers.rs, replacement.rs, triggers.rs, trigger_timing.rs, stack.rs, combat.rs, casting.rs, mana.rs, resolve.rs, eval.rs, actions.rs, turn.rs, simultaneous.rs, permissions.rs. Use cargo-mutants' file filters and sharding or sampling to keep each run to a few hours, and restrict the tests to `-p mtg-engine` (the test binaries cr, keywords, actions, rulings, cards and smoke). Record the exact commands.
4. Report: docs/MUTATION.md with caught / missed / timeout / unviable counts per file, the kill rate, and every surviving mutant, each triaged as (a) equivalent (the change can't alter behavior; say why), (b) missing test (behavior the CR or a card needs that no test checks), or (c) a real engine bug found on the way.
5. Kill the meaningful survivors: for each (b), write a test that would fail with the mutant, with real cards via TestGame and cr!/ruling! citations as usual. For each (c), fix the bug with a regression test. Re-run the affected files and report the kill rate before and after.
6. Leave the commands in docs/MUTATION.md so the measurement can be repeated later, for example before the final PR.

DONE when docs/MUTATION.md covers the core files above, the meaningful survivors are killed or listed with reasons, and the kill rates before and after are recorded.
