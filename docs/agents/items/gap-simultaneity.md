# gap-simultaneity: Simultaneous multi-player and multi-object actions; shuffling empty sets

GOAL: 'Each player ...' bodies run one player at a time (choices interleaved with actions, separate event batches), ForEach over objects creates one batch per object, and Timetwister-style 'shuffle into' doesn't shuffle when nothing moves.

GAPS:

[G10] "Each player ..." effects run one player at a time instead of choices-then-simultaneous
- Rules: CR 101.4 (choices in APNAP order, then actions simultaneously), 608.2e, 608.2f.
- Engine now: `Effect::ForEachPlayer` (resolve.rs:135-143) executes the whole body for one player before the
  next, so player 2 chooses after player 1's action has already happened, the events of each player land in
  different batches, and a card put onto the battlefield by one player can be copied by another's Clone.
  Only `Effect::Sacrifice` and `Effect::Discard` with plural players use `apnap_round` (resolve.rs:237-260,
  1121-1140) and act simultaneously.
- Cards: Show and Tell, Tempting Wurm, Hunted Wumpus, Kynaios and Tiro ("Each player may put ... from their
  hand onto the battlefield", UNS for this reason); Exhume (supported, moves cards one player at a time,
  r110 pattern); tempting offers (Tempt with Discovery etc.: accepted opponents' effects happen one by one,
  so "one or more" triggers fire per player and Iwamori-style choices see earlier results); "each player
  shuffles ... then draws that many" (done per player).
- Rulings: Show and Tell "After all choices are made, the cards are put onto the battlefield simultaneously"
  and "those choices are made after all players choose their card" (UNCOVERED); tempting offer "the effect
  happens simultaneously for each one who accepted the offer" (cited, but only the order is tested).
- Approach: a two-phase executor for ForEachPlayer bodies whose actions are simultaneous-capable (moves,
  token creation, draws, life changes): collect each player's choices in APNAP order (`apnap_round`), then
  perform one combined action (one `move_objects` call / one event batch). Keep per-player sequencing for
  bodies that the rules process separately (608.2f).
- Size: M-L

[G25] "Shuffle ... into their library" doesn't shuffle when there's nothing to shuffle in
- Rules: CR 701.24c, 701.24d (the library is shuffled even if the set of objects is empty or they aren't
  where they're expected), 701.24e/f (shuffle triggers).
- Engine now: `Effect::ShuffleInto` (resolve.rs:1346-1367) shuffles only the libraries of owners of the
  objects actually moved; with an empty hand and graveyard nothing is shuffled. `ShuffleIntoLibrary`
  (resolve.rs:1368-) already follows 701.24c/d. The Timetwister family compiles to `ShuffleInto`
  (oracle/patterns/r400_zones.rs:143).
- Cards: Timetwister, Time Reversal, Echo of Eons, Commit-style "shuffle your hand and graveyard".
- Approach: give `ShuffleInto` the player(s) whose library is shuffled (or compile these to
  `ShuffleIntoLibrary`) and always shuffle.
- Size: S

ESTIMATED SIZE: G10 M-L, G25 S.

OVERLAPS: Interacts with gap-trigger-timing (event batches).

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
