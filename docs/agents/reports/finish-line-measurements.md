# Done-criteria measurements for finish-line-plan: measured on commit ad23a7a (the branch head) on Nov 9, 2026 by the planner's measuring subagent; every tool report was re-run and compared with the committed docs. docs/agents/finish-line-plan.md §1.2 points here.

# mtg-sim: done-criteria measurements, 2026-11-09 (commit ad23a7a, branch claude/mtg-rules-engine-lkr27h)

All commands run from /home/user/mtg-sim with `CARGO_BUILD_JOBS=2`, one cargo process at a time. Outputs are in
the measurer's scratch directory (not kept) (ROUNDTRIP.md, CARD_COVERAGE.md,
roundtrip_clusters30.txt, rulings_summary.txt, rulings_batches_list.txt, one *.txt per command, cargo-test.log).
No tracked file was modified (`git status --short` empty after every step; `roundtrip --write` rewrote docs/roundtrip-passing.txt byte-identical).
The local clone is shallow (196 commits, all dated 2026-11-05..09); whole-history numbers come from the GitHub API.

## 1. Tool reports (cargo build --release -p mtg-tools: 7m57s wall, cold)

| Criterion | Committed doc value (doc's last commit date) | Measured today | Command |
|---|---|---|---|
| CR rules cited by tests | 3036 (95.9%) (docs/CR_COVERAGE.md, 2026-11-05) | **3038** | `cargo run --release -q -p mtg-tools -- cr-coverage` |
| CR rules covered via subrules | 78 (2.5%) | **76** | same |
| CR rules exempt (with reason) | 52 (1.6%) | **52** | same |
| CR rules uncovered / total | 0 / 3166 | **0 / 3166 (100.0% covered)** | same |
| CR worked examples covered / exempt / open | 277 / 0 / 0 of 277 | **277 / 0 / 0 of 277** | same |
| Round trip: fully supported cards checked | 32076 (docs/ROUNDTRIP.md, 2026-11-05) | **32076** | `cargo run --release -q -p mtg-tools -- roundtrip --write <scratch>/ROUNDTRIP.md` (1m33s) |
| Round trip verified | 28460 (88.7%) | **28460 (88.7%)** | same |
| Round trip mismatches | 3464 | **3464** | same |
| Hand-written, not checked | 152 | **152** | same |
| Round-trip mismatch clusters (top 30) | ROUNDTRIP.md "Mismatch clusters" lists 11: 10, 5, 5, six of 3, two of 2 | **identical head: largest cluster 10 cards (gap: Effect), then 5 (gap: Filter), 5 (gap: replacement Discriminant), six clusters of 3, all remaining clusters are pairs** -- the 3464 mismatches are a long tail; the 30 largest clusters cover only ~85 cards | `cargo run --release -q -p mtg-tools -- roundtrip --clusters 30` (saved to roundtrip_clusters30.txt) |
| Verified cards | 18113 of 32111 (56.4%) (docs/CARD_COVERAGE.md, 2026-11-05) | **18113 / 32111 (56.4%)** | `cargo run --release -q -p mtg-tools -- card-coverage --write <scratch>/CARD_COVERAGE.md` (53s) |
| Round-trip tier (exactly / at least) | 10457 (32.6%) / 28570 (89.0%) | **10457 / 28570 (89.0%)** | same |
| Compiled tier (exactly / at least) | 3506 (10.9%) / 32076 (99.9%) | **3506 / 32076 (99.9%)** | same |
| Unsupported | 35 (0.1%) | **35** | same |
| Failed checks (compiled cards) | round-trip 3506, manual-check 0, structure 11522, fuzz-swept 0, fuzz-cast 78, fuzz-clean 1 | **round-trip 3506, manual-check 0, structure 11522, fuzz-swept 0, fuzz-cast 78, fuzz-clean 1** (unchanged) | same |
| Fully supported / abilities | 32076 (compiled 31923 + hand-written 153); abilities 60150/60188 (99.9%) | **32076 (31923 + 153); 60150/60188** | same |
| Sweep results listed (fuzz staleness) | 32656 cards, target/every-card.tsv written 2026-11-05; partial 26 cards 2026-11-05; partial-review 11 cards 2026-11-05 | **same three sweeps, all dated 2026-11-05 (4 days / ~560 commits old)** | same, "Sweep results" section |
| rulings-coverage --check | docs/RULINGS_COVERAGE.md (2026-11-05) is stale: it says 1558 texts covered (4.58%) | **PASS (exit 0): 26528 (card, ruling) pairs cited; 25777 of 34000 unique texts (75.81%); 166 exempt (0.49%); 0 unmatched citations/exemptions** | `cargo run --release -q -p mtg-tools -- rulings-coverage --check` |
| interpretations --check | docs/INTERPRETATIONS.md (2026-11-09): 128 entries | **PASS: 128 entries (0 provisional), 0 problems** | `cargo run --release -q -p mtg-tools -- interpretations --check` |
| manual-check | CARD_COVERAGE: manual-check failures 0 | **PASS: 156 hand-written abilities, 0 problems** | `cargo run --release -q -p mtg-tools -- manual-check` |
| Interpretation register decided vs provisional | 128 entries: interpretation 85, deviation 9, policy 34; provisional 0 (docs/INTERPRETATIONS.md) | **128 decided, 0 provisional** (tool prints "128 entries (0 provisional), 0 problems"; the per-kind split is from the doc) | `cargo run --release -q -p mtg-tools -- interpretations`; docs/INTERPRETATIONS.md |
| Rulings batches summary | (no doc; RULINGS_COVERAGE.md stale) | **33955 ruling texts in 227 batches: CITED 25777 (75.9%), EXEMPT 189, OPEN 6125, DEFERRED 1864 (digital-only, sticker sheets, not legal anywhere, dexterity); out of scope 662 (Contraptions, host/augment)** | `python3 scripts/rulings_batches.py summary` |
| Rulings batches list | -- | **227 batches of 150 texts; 154 batches have OPEN > 0 (73 fully cited); 55 batches have >= 30 open; the biggest: P144 131, P081 126, P154 126, P139 125, P157 125, P086 124, P155 124, P101 123, P182 123, P138 121, P170 121, P084 120, P153 120, P168 120, P191 118 ... (full list saved)** | `python3 scripts/rulings_batches.py list` > rulings_batches_list.txt |

## 2. Tree counts (no cargo)

| Criterion | Measured today | Command |
|---|---|---|
| Test functions, mtg-engine tests by dir | cr 2432, keywords 1676, actions 258, rulings 17376, cards 6641, properties 17, smoke.rs 10; engine unit tests in src 157 => **mtg-engine 28567** | `grep -rho '#\[test\]' crates/mtg-engine/tests/<dir> \| wc -l` per dir |
| Test functions, other crates | mtg-api 50 (tests+src), mtg-data 21, mtg-sim + mtg-tools 21 => **workspace total 28659** `#[test]` attributes | `grep -rho '#\[test\]' crates \| wc -l` |
| Hand-written cards | **153 distinct card names** (154 `card:` entries: Booby Trap, Ertai's Meddling, Swirl the Mists, Trinisphere have two entries each); 153 card files + mod.rs in src/cards; manual-check counts 156 abilities | `rg -o 'card: *"[^"]+"' crates/mtg-engine/src/cards \| sort -u \| wc -l` |
| Citations | **cr! 28155, ruling! 27008, interp! 340, cr_example! 290** | `grep -rhoF 'cr!(' crates \| wc -l` etc. |
| Engine source lines | **412,765 lines in 1,553 .rs files** under crates/mtg-engine/src | `find crates/mtg-engine/src -name '*.rs' \| xargs cat \| wc -l` |
| The 15 core files of docs/MUTATION.md | **33,113 lines** (casting 4852, resolve 4275, combat 3835, replacement 2639, triggers 2634, layers 2424, actions 2399, eval 2062, stack 2025, simultaneous 1666, mana 1244, turn 1239, permissions 1000, sba 600, trigger_timing 219) | `wc -l` in crates/mtg-engine/src |
| Files in registries | **src/kw 327, src/kwa 32, src/oracle/patterns 778, src/cards 154 (153 cards + mod.rs)** | `ls crates/mtg-engine/src/<dir>/*.rs \| wc -l` |

## 3. Most recent recorded results in docs (not re-run)

| Criterion | Committed doc value and date | Notes |
|---|---|---|
| (a) Every-card fuzz sweep | docs/CARD_COVERAGE.md "Sweep results": **32656 cards, target/every-card.tsv written 2026-11-05** (+ partial sweeps of 26 and 11 cards, 2026-11-05); docs/verified-cards.tsv header: results of 32065 cards carried over + 11 re-swept, 16 structure logs. Failed fuzz checks today: fuzz-cast 78, fuzz-clean 1. docs/EVERY_CARD.md's narrative is older: "Latest full sweep" at commit fb94ed0, pool 29,534 cards, 59,068 games, 196 min, **0 panics/hangs**, 2 rules-violation games (checker's mistake, fixed), 3 slow games, 29,211/29,308 cast (99.7%), abilities used 25,470/26,890 (94.7%), fully exercised 28,141 (95.3%); "Sweep for the card tiers (Oct 21)": pool 30,222, 60,444 games, 4 failed games (2 slow Charismatic Conqueror, 1 slow Shahrazad, 1 Maelstrom Nexus zone violation = engine bug, since fixed) | EVERY_CARD.md has no write-up of the 2026-11-05 sweep of 32,656 cards the tier file uses; the fuzz-clean failure count (1) suggests that sweep was clean except one game |
| (b) Property runs | docs/PROPERTIES.md "Results" (no date in the section; the doc mentions Oct 22 and Oct 28; last commit 2026-11-05): `--seed 10 --games 2000`: **seat 2000/2000 held, 0 violations** (4,561,047 decisions, 2,648 s CPU); **snapshot 2000/2000, 0** (3,585 s); **rollback 2000/2000, 0** (120,750 reversed actions, 2,469 s); **determinism 2000/2000, 0** (4,126 s); **replay 1,990 held + 10 slow, 0 differences** (9 of the 10 re-held; game 1178 too slow) => 1,999/2,000; +100 games each at `--seed 20` held. Properties 5-8: copies 31,181 cards / 173,906 copies, 0; copies-dynamic 24,043 permanents / 76,235 steps, 0; hidden 400 games (seed 2) and 2,000 games (seed 3), 0 (795 s); payment 1,000,000 boards (seed 2) + 200,000 (seed 1), 0; damage 200,000 combats (seed 2), 2,323,462 assignments, 0. Re-check after the Oct 22 merge (pool 31,310): copies 174,616; copies-dynamic 24,139/76,513; payment 200,000 (seed 4); damage 20,000 (seed 4); hidden 200 games (seed 4): all 0 violations | Runs predate ~2.5 weeks (~2,000 commits) |
| (c) Random-deck fuzz (`--random-decks`) | **No doc records a result.** The flag appears only as an instruction in docs/agents/items/*.md ("Run `mtg-sim -- --random-decks --games 300` before finishing") and as a usage example in docs/AGENT_PROTOCOL.md | -- |
| (d) Mutation testing | docs/MUTATION.md "Status (2026-10-21)": 15 core files, 5,281 mutants, 551 sampled (1/10 round-robin; 520 viable): **423 killed (82.1%) before the new tests, 459/520 (88.3%) after**; 61 survivors triaged (equivalent, or unreachable by any card in the pool); 1 engine bug found and fixed (simultaneous.rs). Per file after new tests: eval 100%, casting 96.4%, turn 95.7%, stack 94.4%, combat 93.8%, trigger_timing 93.8%, sba 87.5%, layers 87.5%, resolve 86.7%, actions 85.7%, replacement 82.4%, mana 81.0%, permissions 78.6%, triggers 75.0%, **simultaneous 55.6%**. Runs 2026-10-14..21 | Core files have grown since (33,113 lines today vs 5,281 mutants then) |
| (e) Judge audit | docs/agents/reports/judge-audit-1.md (Oct 14 review; sample seed 20261014 at commit 9b0af15a): **60 tests sampled** (14 cr, 18 rulings, 10 keywords, 12 cards, 6 actions), **58 judged** (2 set aside: no game played), **225 of 225 questions agreed** (223 high, 2 medium confidence); 0 wrong tests => wrong-outcome rate below ~5% (one-sided 95%); 118 agent runs, ~18 min; the doc says ~300 tests would be needed to bound at 1% | -- |

## 4. Pace

| Criterion | Measured today | Command |
|---|---|---|
| Commits (local, shallow clone) | 196 (2026-11-05: 100, 11-06: 37, 11-07: 1, 11-08: 56, 11-09: 2); `.git/shallow` present, local history starts 2026-11-05 12:53 UTC | `git log --oneline \| wc -l`; `git log --since=2026-11-01 --oneline \| wc -l` (also 196) |
| Commits (GitHub API, whole branch) | **5,776 commits**; first commit **2026-09-24T23:00:38Z** ("Add MTG comprehensive rules (effective 2026-09-25)"); repository created 2026-09-24 | `gh api -i "repos/<owner>/mtg-sim/commits?sha=<branch>&per_page=1"` (Link rel=last) |
| Commits in the last 14 days | **1,518 since 2026-10-26T22:00Z => ~108 commits/day**; 870 since 2026-11-01 (9 days => ~97/day); 46 days of history => ~126/day overall | same with `&since=` |

## 5. cargo test --workspace

| Criterion | Measured today | Command |
|---|---|---|
| Workspace test suite | **PASS: 28,659 passed, 0 failed, 6 ignored** across 27 test binaries (exit 0). Wall time **14m04s** (22:33:04 -> 22:47:08 UTC) including a cold debug build; test execution itself ~6.5 min (rulings 114.9 s / 17376 tests, cards 66.9 s / 6641, cr 45.6 s / 2432, properties 41.9 s / 17, replay 37.2 s, smoke 30.7 s, keywords 8.7 s / 1676, actions 1.9 s / 258, engine unit 1.2 s / 157; mtg-api 50, mtg-data 20, mtg-sim 15, mtg-tools 1). Well under the 25-minute cap | `timeout 1500 cargo test --workspace` (log: cargo-test.log) |

Note: the 28,659 passed tests equal the 28,659 `#[test]` attributes counted in the tree (section 2); the 6 ignored are in mtg-data (1) and the mtg-sim/mtg-tools doc-tests (5).

The test build produced no compiler warnings (the only "warning" matches in the log are test names such as `the_ravens_warning_...`). Final `git status --short` is empty: no tracked file was changed.
