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

## Progress

- `simultaneous.rs` (landed Nov 19, `mutation-2/simultaneous`): full run of all 279 mutants,
  71.8% → 98.1% with 5 equivalent survivors; 8 engine bugs and 3 compiler fixes; two
  provisional register entries (`choices-for-each-player-made-in-turn`,
  `mixed-each-player-instruction`). Report: docs/MUTATION.md, "Second round, Nov 2026".
  `scripts/mutants.sh` gained `FULL=1` (every mutant) and `RESUME=1` (continue a run a
  machine restart interrupted). Next in the order above: `permissions.rs`.
- `trigger_timing.rs` + `triggers.rs` half 1 (landed Nov 21, `mutation-2/triggers-half-1`;
  runs at `45bb684`, tests and fixes in `6b08efb`, review in `9fce77f` and `852fa0c`):
  trigger_timing.rs full run, 35 mutants, 88.2% → 91.2% (31/34), 3 equivalent survivors;
  triggers.rs half 1 (`FULL=1 SHARD=0/2`, mutants 1–288 of 576), 81.2% → 88.3% (249/282), all
  33 remaining survivors equivalent (`361:24` since C1; `1245:25`, (b) in the triage, was
  equivalent all along: `Game::matches` already range-checks the object as it last existed).
  22 new tests (implementer), 3 (review). Engine bugs (6): C1 a qualifier over alternatives
  dropped the "one or more" one (CR 603.2c); C2 a phased-out Edgar Markov's eminence triggered
  (CR 702.26b); C3 Elesh Norn, Mother of Machines stopped an opponent's delayed triggered
  ability (Thunder of Unity; CR 113.7, 603.7e); C4 under limited range of influence nothing at
  all triggered on a permanent leaving the game with its owner (it was seen by no one, judged
  by its departed owner), and a commander put into the command zone was judged by its owner
  there (CR 801.7a, 903.9b; the commander case tested by the review); L1 a prevention follow-up
  ran inside a held "together" damage event (CR 615.5; `rider-order` updated); L2 (review)
  Hushbringer didn't stop delayed "one or more ... die" abilities (`delayed_batches.rs`).
  Compiler fix (1): C5, records stripped after `join_instructions` split or merged damage
  sentences (The Fall of Kroog, Fear, Fire, Foes!; the whole pool's compiled output compared,
  only those two and Camaraderie, unchanged in behaviour, differ). Provisional register entry
  `delayed-abilities-arent-abilities-of-permanents`, grounded by the review in CR 603.2d (which
  settles Elesh Norn's "triggers an additional time" half) and pinned for both halves
  (`elesh_norn_doesnt_double_your_delayed_triggered_ability`, review). Review test errors: one
  ruling citation a test didn't exercise; register-entry errors: two (603.2d not quoted, an
  alternative 603.2d rules out). Latent, not fixed: L1, L3, L4 of triggers.rs (L5 since covered
  upstream by `that-opponent-fixed-as-it-triggers`), C1's `FirstTimeEachTurn` form, C5's
  referenced-records case, and dead code to delete (`keyword_has_trigger`, `check_triggers`,
  `looks_back`'s `Event::Sacrificed` arm, the no-op filter at `319:36`). Report:
  docs/MUTATION.md, "trigger_timing.rs (full run)" and "triggers.rs, half 1".
  `scripts/mutants.sh` gained `SHARD=k/n` (a contiguous slice of the file's mutants in
  `cargo mutants --list` order, saved in `target/mutants/<stem>.shard`; `RESUME=1` with it
  tests the slice's untested mutants itself, since cargo-mutants shards after
  `--iterate`/`-E`).
  - **Half 2**, 288 mutants (289–576 at `45bb684`). At `8199d6d` (the merge this report landed
    with) the file has 586 mutants and upstream renamed `trigger_matches_ctx`'s body
    `Game::trigger_event_infos`: half 1's last is the 298th (`triggers.rs:1554:33: delete field
    amount from struct EventInfo expression in Game::trigger_event_infos`, in the
    `(PlayerAttacked, AttackersDeclared)` arm), and half 2 is the 299th to the 586th, from
    `triggers.rs:1555:33: delete field objects from struct EventInfo expression in
    Game::trigger_event_infos` to `triggers.rs:2976:9: delete match arm TriggerCond::AnyOf(ts)
    in on_sacrifice`. A plain `SHARD=1/2` (the 294th on) would start five early, so make the
    slice by hand and run it with `RESUME=1` (docs/MUTATION.md, "triggers.rs, half 1", gives the
    commands, which find the slice from that arm at whatever commit is measured):
    `target/mutants/triggers.shard` = the list from the mutant after half 1's last, then
    `FULL=1 SHARD=1/2 RESUME=1 scripts/mutants.sh triggers`. Ten mutants of half 1's region
    added since its run (2 by C1, 8 upstream) have never been tested; the same section says how
    to append them to the slice.
  - **Operational lesson.** The cloud container is reclaimed when the session is idle with no
    harness-tracked background task; that killed this run three times. Keep the session
    active while a run goes: foreground waits of under 10 minutes, or a background
    `sleep 7000` keepalive re-armed when it expires. After an interruption resume with
    `RESUME=1` (with the same `SHARD` for a slice). Budget about 5 minutes for a mutant the
    suite misses (a ~55 s incremental build, then the ~255 s `mtg-engine` suite), less for a
    caught one, 10 for a timeout.
- `replacement.rs` (Nov 22, branch `wip/mutation-2-replacement`): full run of all 536 mutants at
  `eaa55a4` (31 h), 78.4% → 88.2% (457/518). The 51 killable survivors are caught (56 new tests
  with the fixes' regression tests, one adjusted); the 55 left are equivalent; 6 are gone with
  code the fixes removed (89.2% counting the five engine problems among them as killed). 8
  engine fixes (a replacement chain cut after 33 effects, CR 616.1f; entry effects after a copy
  effect and copies' granted entry abilities, CR 614.12; the CR 616.1 choice for a spell's
  "exile it instead"; a kind of counter with no counters; division parts on redirected damage;
  stack-only grants on the permanent, CR 400.7; a move's cause and its exile link). Four
  provisional register entries (`entry-effects-of-a-leaving-source`,
  `face-down-entry-checked-as-the-face-down-card`,
  `entry-types-added-after-what-the-effect-says-it-is`,
  `deathtouch-lethal-for-excess-by-damage-dealt`) and one decided
  (`leaving-the-stack-exile-among-other-replacements`). Report: docs/MUTATION.md,
  "replacement.rs (full run)". `scripts/mutants.sh` gained `REV=<commit>` (measure a fixed
  commit while tests land on top) and a re-run match that ignores whitespace and reports weak
  matches. Review: every fix's regression tests fail with the fix reverted; a problem in fix 4,
  not counted as an engine bug (the implementer's unlanded code: a default answer applied Dauthi
  Voidwalker's effect before a countered or resolving flashback spell's own exile, giving it a
  void counter; a spell leaving the stack now lists its own effects first); the merge kept
  user-decisions-nov20b's re-check of the entry effects (`Game::still_modifies`) with fix 2's
  snapshot taken again at each step; the implementer's four unregistered readings registered
  (`copy-effect-doesnt-see-an-original-out-of-range`, provisional; `questions-only-for-a-choice`,
  `replacement-question-names-the-card-as-it-is`, `parts-of-one-modified-event-unordered`,
  decided); `2885:9`/`2889:13` re-checked by running one (only the event feed's order changes).
- Next: `triggers.rs` half 2 (`FULL=1 SHARD=1/2`, above).
