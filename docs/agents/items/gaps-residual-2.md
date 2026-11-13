# gaps-residual-2: the last documented rules gaps, and liveness assertions for departed objects

WHY: docs/agents/finish-line-plan.md, criterion C9 and risk 5. The 33-gap inventory
(docs/agents/engine-gaps.md) is closed as a feature list; what remains is small:

1. **G32, two bits never verified**: a graveyard-size count that may include tokens (tokens
   cease to exist in a graveyard before anything counts them, CR 111.7; `eval.rs` near the
   graveyard count), and meld on a permanent that is a copy of a meld card (`merge.rs`,
   `meld_pair(...)`: a copy has the meld ability but 701.42a melds the two *cards*; a token or
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
4. **Inquisitor Eisenhorn** (CR 121.8). A card drawn while a spell is being cast or an ability
   activated is kept face down until the spell becomes cast, and "If an effect allows or
   instructs a player to reveal the card as it's being drawn, it's revealed after the spell
   becomes cast or the ability becomes activated." The engine reveals it then but never fires the
   "Whenever you reveal ... this way" trigger (roundtrip-tail-5, "Remaining"). Fix it: the reveal
   after the cast is the reveal "this way", so the trigger fires then; test it with a draw during
   casting (a mana ability that draws, paid as a cost). Register nothing unless the fix needs a
   reading the CR doesn't give; say which.

Routine; effort high for the implementer, max for the reviewer. Land each numbered part as its
own commit so a partial result survives.

DONE when the four parts are landed with tests citing their rules, docs/agents/engine-gaps.md
records them as closed, `cargo test --workspace` passes in debug with the assertions on, and the
item's row (bugs found by the assertions and the fuzz chunk) is in docs/agents/yield.tsv.
