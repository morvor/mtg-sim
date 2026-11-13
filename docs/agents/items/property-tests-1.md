# property-tests-1: metamorphic properties of whole games (seat symmetry, snapshots, rollback, determinism)

WHY: the user chose property tests on Oct 14. Tests that check one situation at a time can share the implementer's misunderstanding. A property that must hold across every game instead catches whole classes of bugs, on every card, without anyone predicting outcomes.

The every-card fuzz already checks invariants inside one game (`crates/mtg-sim/src/checks.rs`: no state-based actions pending at priority, zone consistency, recomputed characteristics, damage marking, pools and stack at step ends, life and counter accounting). This item adds metamorphic properties: relations between two runs that must agree.

SETUP:
- Add a test directory `crates/mtg-engine/tests/properties/`. Wire it into its own binary by adding "properties" to the list in `crates/mtg-engine/build.rs`.
- Add a fuzz mode in mtg-sim (e.g. `--properties`, with `--games`/`--seed` and the usual repro printing) that checks the properties over random games.
- Write `docs/PROPERTIES.md`: each property, its rules basis, how to run it, and the latest results.
- Every violation found is a bug. Fix the engine (minimal edits per CLAUDE.md) and add a regression test citing the rules.

PROPERTIES:
1. **Seat symmetry** (CR 101.4, 102, 103, 500.1). Nothing in the rules depends on a player's seat number, only on turn order and on the players' choices.
   - **The property.** Play two games that differ only in which seat each deck sits in. In both, the same deck goes first, libraries start in the same order, and the agents decide by a seat-independent key. The two games must be identical up to renaming the players and objects.
   - **Comparison.** Compare normalized transcripts, with player and object identities mapped between the two games.
   - **Scope.** Run it over random deck pairs in the fuzz mode, and over a fixed set in tests. Cover 3- and 4-player games too.
   - **Bugs it finds.** Iterating players by seat instead of in APNAP or turn order, ties broken by player id, and state keyed by seat.
   - **Hints:**
     - All engine randomness comes from one `ChaCha8Rng` (`Game.rng` in game.rs), drawn in seat-dependent order. Add a test-only `GameConfig` option that gives each player their own stream, keyed by the player's identity rather than their seat. Don't change the default behavior.
     - Object ids are allocated in creation order, so seat 0's cards come first. The deciding agent therefore can't choose by id or by option index alone: give it a canonical key, such as card name, zone and controller relative to itself. Map ids when comparing.
2. **Snapshot equivalence.** The engine snapshots the whole `Game` (a clone) to reverse actions (`Game::roll_back`).
   - **The property.** At random points of random games, clone the game, continue the original and the copy with identical agents and seeds, and check that the two games stay identical.
   - **Bugs it finds.** State a clone doesn't copy faithfully, or shares with the original: shared mutable state behind Rc/Arc, registries, caches, or per-action trackers such as `TriggerTiming`.
3. **Rollback identity** (CR 733.1). When a player starts an action that can't legally be completed, the entire action is reversed and no abilities trigger.
   - **The property.** Whenever the engine reverses a cast or activation, the game afterwards equals the game before (a canonical serialization or hash of the state). Exclude only what the rules let stay; read 733 and decide what that is.
   - **How to find cases.** Make the fuzz agents attempt illegal and unaffordable actions on purpose.
4. **Determinism and replay.**
   - **The property.** The same seed and the same decisions give the same game. A game played through the external agent protocol (mtg-api `Session` and `ExternalAgent`; mtg-sim `--transcript`) and replayed from its transcript's decisions reproduces the same game.
   - Check what already exists before adding to it.

DONE when the four properties run in tests and in the fuzz mode, a fuzz run of at least 2,000 games per property is clean (or every violation is fixed), and docs/PROPERTIES.md records the results. Report the bugs found, each with its rule and fix.
