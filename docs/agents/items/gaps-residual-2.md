# gaps-residual-2: the last documented rules gaps, and liveness assertions for departed objects

WHY: docs/agents/finish-line-plan.md, criterion C9 and risk 5. The 33-gap inventory
(docs/agents/engine-gaps.md) is closed as a feature list; what remains is small:

1. **G32, two bits never verified**: a graveyard-size count that may include tokens (tokens
   cease to exist in a graveyard before anything counts them, CR 111.7; `eval.rs` near the
   graveyard count), and meld on a permanent that is a copy of a meld card (`merge.rs`,
   `meld_pair(...)`: a copy has the meld ability but 701.37a melds the two *cards*; a token or
   copy can't meld). Read the rules, write the test, fix if wrong.
2. **Hashed ability links can collide** (docs/PROPERTIES.md "Remaining gaps"; CR 607.5): links
   of abilities that continuous effects give (`layers::acquired_ability`,
   `ability_grants::gained_link`) are hashes into 0x8000–0xbfff; two grants to one object could
   hash alike. Replace them with a per-object counter (or a per-game allocation that is stable
   across recomputation), keep determinism and snapshot equivalence (run `--properties
   --property snapshot --games 200` and `determinism`), and remove the gap from the doc.
3. **Liveness of object ids in the primitives** (CR 400.7). Every zone change makes a new
   `ObjectId`; the invariant that primitives never act on a departed object is enforced by
   convention (`game.current(id)`). Two mutation survivors were dismissed with "the old id is
   never live", and eight rulings fixes plus two fuzz bugs were stale-id or LKI mistakes. Add
   `debug_assert!` liveness checks (or an explicit `lki`-aware entry point where an effect
   legitimately reads a departed object's last known information) to the `actions.rs`
   primitives (tap, untap, damage, counters, attach, move, destroy, sacrifice), run the whole
   test suite in debug, then one every-card chunk (`--every-card --from 0 --count 4000`, debug
   build, with the assertions on) and 500 properties-fuzz games. Every assertion that fires is a
   bug: fix it generally with a regression test. Keep the assertions (debug only) so later work
   trips them.
4. **Inquisitor Eisenhorn** (CR 121.8): only if `register-sweep-1` has not already registered
   or fixed it. If the engine's behavior is a deviation, fix it (a card drawn while casting is
   revealed when the cast completes, and "drawn this way" triggers then) rather than register it,
   unless the fix is large; say which.

Routine; effort high for the implementer, max for the reviewer. Land each numbered part as its
own commit so a partial result survives.

DONE when the four parts are landed with tests citing their rules, docs/agents/engine-gaps.md
records them as closed, and `cargo test --workspace` passes in debug with the assertions on.
