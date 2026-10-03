# Worker guide: running work items in parallel

Work on this engine is split into items (a batch of rulings, a feature, an engine gap,
a card-support theme). Several machines work through items at the same time, all
landing on the PR branch `claude/mtg-rules-engine-lkr27h`. This guide is for a Claude
session acting as a **worker**: it gets a queue of items and drives each one through
implementation, an adversarial review, and landing, using subagents.

## Setup (once per machine)

1. In the repo checkout: `git fetch origin claude/mtg-rules-engine-lkr27h` and make sure
   the checkout is on that branch and up to date (`git pull --ff-only`).
2. `cargo --version` must work. If it doesn't, install the stable toolchain
   (`curl -sSf https://sh.rustup.rs | sh -s -- -y`, then `source ~/.cargo/env`).
3. Disk and CPU are limited: run at most the number of items in parallel your queue
   says (default 3). Each worktree's `target/` grows to about 4-5 GB.

## Per item

1. Worktree: `git worktree add -b wip/<slug> ../wt/<slug> origin/claude/mtg-rules-engine-lkr27h`
   (if the branch exists from an interrupted run, add the worktree on it instead and
   tell the implementer to continue). Use the absolute path below as `<worktree>`.
2. Implementer: launch a subagent (Agent tool, general-purpose, in the background) with
   the **implementer brief** below, filled in for the item.
3. Reviewer: when the implementer returns, launch a fresh subagent with the **reviewer
   brief**, including the implementer's full report. It reviews, fixes, and lands.
4. When the reviewer reports, record the result. After a successful landing, remove the
   worktree (`cargo clean` inside it, then `git worktree remove <worktree>`) and start the
   next item. If a subagent dies or stops early, launch a fresh one on the same worktree,
   telling it to continue from `git log`/`git status`/`git diff`.

Keep your own context small: don't read engine code yourself, delegate. Never push
anything yourself except through `scripts/land.sh` (the reviewer runs it); never push
another branch, force-push, or open pull requests.

When the queue is done, report per item: landed commit (or why not), the counts the
reviewer reported, and remaining gaps.

## Implementer brief

> You are implementing one work item of a Rust Magic: The Gathering rules engine (repo
> morvor/mtg-sim; CLAUDE.md describes the architecture and conventions; read it first).
> Work autonomously until the item is done.
>
> WORKTREE: `<worktree>`, branch `wip/<slug>`. Your shell may start elsewhere: `cd` there
> first and use absolute paths under it for every edit. If the branch already has commits
> (an interrupted run), inspect `git log origin/claude/mtg-rules-engine-lkr27h..HEAD`,
> `git status` and `git diff`, keep what is sound, and continue.
>
> ITEM `<slug>`: `<the item's brief, see "Item briefs" below>`
>
> GROUND RULES:
> - Do all edits inside your worktree. Never push and never run scripts/land.sh: a
>   reviewer lands your work after you.
> - Other agents on several machines land into `origin/claude/mtg-rules-engine-lkr27h`
>   continuously. Keep edits to shared core files (ability.rs, game.rs, resolve.rs,
>   eval.rs, actions.rs, turn.rs, casting.rs, oracle/*.rs) small and localized; prefer new
>   modules and registry files. Before a large core edit, run
>   `git fetch -q origin claude/mtg-rules-engine-lkr27h && git merge --no-edit origin/claude/mtg-rules-engine-lkr27h`.
> - Commit early and often (each coherent chunk) so progress survives interruptions.
>   Commit subjects start with `<slug>: `; follow your session's attribution
>   instructions for commit trailers; never put model names in commits.
> - No todo!/unimplemented!/panics in engine code paths; no card-name special cases in
>   the engine; never #[ignore], delete, or weaken existing tests. Keep tests fast.
> - Iterate with targeted builds and tests (`cargo build -p mtg-engine --all-targets`;
>   `cargo test -p mtg-engine --test <cr|keywords|actions|cards|rulings> <filter>`).
>   Before returning: `cargo fmt`; `cargo build --workspace --all-targets` with no new
>   warnings; `cargo test --workspace` passing; everything committed.
>
> REPORT: worktree, branch, commits, summary (engine changes, new files, tests),
> coverage before and after, remaining gaps (each with a reason), and whether tests pass.

## Reviewer brief

> You are the adversarial reviewer and integrator for work item `<slug>` of a Rust Magic:
> The Gathering rules engine (repo morvor/mtg-sim; CLAUDE.md has the conventions). The
> implementer worked in `<worktree>` on branch `wip/<slug>`. Their report:
> `<report>`
>
> `cd <worktree>` first and use absolute paths under it.
> 1. If anything is uncommitted, inspect it and commit it (or fix it first).
> 2. Review `git diff origin/claude/mtg-rules-engine-lkr27h...HEAD` critically:
>    - Tests: for every new or changed test, check that each cr!/ruling! citation matches
>      what the test actually asserts (read the rule: `grep -n "^<rule>" data/comprehensive-rules.txt`;
>      read the ruling) and that the assertions would fail if the engine got it wrong (no
>      tautologies, no asserting only the setup, no testing the harness). When unsure,
>      temporarily break the engine behavior and confirm the test fails. A ruling!
>      citation must sit in a test that sets up that card.
>    - Shared rulings: when cards sharing a ruling word the relevant ability differently,
>      each wording needs a test.
>    - Exemptions (docs/cr-exemptions, docs/rulings-exemptions): only for content with no
>      game behavior (history, presentation, policy); otherwise write the test.
>      `cargo run --release -q -p mtg-tools -- rulings-coverage --check` must pass.
>    - Engine: correct per the CR text; general (no card-name special cases); no
>      todo!/unimplemented!/panics on reachable paths; no #[ignore], deleted or weakened
>      tests; shared-core edits minimal.
>    - Oracle patterns: the parser must not accept text the engine doesn't implement
>      faithfully. Hand-written card definitions must match the Oracle text exactly and
>      have tests.
>    - Gap and feature items: everything reported as done must really work (test it
>      yourself); push back on approximations.
> 3. Fix every problem you find (rewrite weak tests, correct citations, fix engine bugs),
>    and commit (subjects start with `<slug>: `).
> 4. Land: run `scripts/land.sh` from the worktree. It merges the latest origin PR branch,
>    builds, tests, and pushes fast-forward only, retrying if another machine landed
>    first. On CONFLICT: resolve keeping both sides' behavior, `cargo fmt`, `git add`,
>    `git commit --no-edit`, re-run. On build or test failure: find the root cause and fix
>    it (never skip, ignore or delete tests), commit, re-run. Repeat until it prints LANDED.
> 5. After LANDED: `cargo clean` inside the worktree.
>
> REPORT: landed (yes/no), commit, problems found, fixes made, remaining gaps.

## Item briefs

**Rulings batch `rulings-PNNN`** (the batch id is the item):

> Scryfall rulings batch PNNN. Get it with `python3 scripts/rulings_batches.py show PNNN --open`
> from the worktree root: a JSON list of open ruling texts, each with the cards it applies
> to and the keyword or Scryfall Tagger function the batch was grouped by. Re-run it to
> track progress.
> GOAL: every ruling in the batch ends up in one of three states:
> (a) CITED: a test exercises exactly what the ruling says and cites it with
>     `ruling!("Card Name", "distinctive substring copied from the ruling")`. A ruling
>     shared by several cards is covered by a test on any of them: use a supported card
>     with typical wording, and when the cards word the relevant ability differently
>     (qualifiers, durations, targets, structure), test each wording.
> (b) EXEMPT: listed in `docs/rulings-exemptions/rulings-PNNN.tsv` as
>     `Card<TAB>substring<TAB>reason`, only when it has no engine-testable content
>     (release or Oracle-wording history, printing or presentation, tournament policy).
>     Any statement about how the game plays is testable.
> (c) UNCOVERED: every card it applies to is unsupported by the compiler, or the engine
>     lacks the behavior and the fix is not small. List each in remaining_gaps with its
>     reason. When the fix IS small and general (a missing compositional phrase pattern,
>     an engine bug the ruling exposes), make it, with tests, instead. If supporting a
>     card would be hacky, leave it as "unsupported: <text>" (a card-support workstream
>     handles the compiler and hand-written definitions).
> SPEND EFFORT WHERE BUGS ARE (measured: rulings batches find about one real engine bug
> per 100 rulings, almost always in interactions):
> - Do the interaction rulings first and test them thoroughly: timing and triggers,
>   replacement and prevention effects, layers and characteristic changes, copies, control
>   changes, last known information, zone changes and new objects, multiplayer, costs and
>   mana. These are where the engine is most often wrong.
> - Rulings that only restate what the card's own text says (e.g. "~ can target a creature
>   you control", "the token is a 1/1") still get a test, but a cheap one: put them in a
>   compact table-driven test (a list of cases, each with its card, a minimal setup and the
>   asserted outcome, and its own literal ruling! line), not a separate long test each.
> - Keep each test to the setup the ruling needs; no elaborate scenarios beyond it.
> The rulings in a batch are about similar abilities: share helpers and scenarios. Check
> a card with `cargo run --release -q -p mtg-tools -- unsupported --card "Name"` and a
> ruling with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`.
> Tests go in `crates/mtg-engine/tests/rulings/r_pNNN_<topic>.rs`; each uses real cards
> through TestGame, asserts the in-game outcome the ruling describes, and cites with
> cr!(...) the rules it exercises. Rulings that reveal an engine bug are the most
> valuable: fix the bug generally and keep the regression test.
> Report counts: cited, exempt, uncovered, bugs fixed.

**Other items** (features, engine gaps, card-support themes): the brief is in
`docs/agents/items/<slug>.md`.
