# reversal-library-exceptions: keep CR 733.1's library actions when an illegal action is reversed

WHY: on Oct 21 the user decided to follow CR 733.1 exactly (register entry `illegal-action-reversal-restores-libraries`, option 1).

- **What the rules say.** 733.1 ends: "Players may not reverse actions that moved cards to a library, moved cards from a library to any zone other than the stack, caused a library to be shuffled, or caused cards from a library to be revealed."
- **What the engine does.** It reverses an illegal action by restoring a copy of the whole game taken before the attempt (`Game::roll_back`). That also undoes those library actions.
- **Which cards reach it.** The scan test `tests/cr/r733_001_reversal_reachability.rs` found 16 fully supported cards (`KNOWN_REACHABLE`). Each has a replacement effect that turns an ordinary payment into a library action:
  - Darksteel Colossus or Progenitus sacrificed for mana: the card is shuffled into its owner's library instead;
  - Ashiok, Wicked Manipulator: paying life exiles cards from the top of your library instead;
  - Lich: gaining life draws cards instead;
  - Angel of Suffering with Shivan Reef's damage: the damage mills instead;
  - and the others the test lists.

  For example, cast Fling with Ashnod's Altar, Darksteel Colossus and a Mountain. You sacrifice the Colossus to the Altar for mana, which shuffles it into its owner's library. No creature is left for Fling's sacrifice, so the cast is reversed, but by the rules the Colossus stays in the library.

1. **Read first:** the full text of CR 733.1–733.2, 601.2g–i and 605.1a–b; the rulings of the `KNOWN_REACHABLE` cards; the register entry; `Game::roll_back` and its callers (casting.rs); and the rollback property in docs/PROPERTIES.md and tests/properties/.
2. **Design.** Reverse an attempt by restoring the copy as today, then re-applying, in order, the library actions that happened during the attempt:
   - moves to a library;
   - moves from a library to any zone other than the stack;
   - shuffles, with the random generator's state at that point;
   - reveals of library cards: the revealed information stays known to the players who saw it, in the mtg-api observations too.

   Work out and record (in the register entry, or new entries where a reading is a judgment call):
   - **What else stays with a kept move.** A sacrificed Colossus that went to the library has left the battlefield, and the card has a new object (CR 400.7).
   - **The mana.** What happens to the mana of a mana ability whose payment became a library action? 733.1 lets the player reverse the mana ability, but not the library action.
   - **Triggers.** 733.1 says no abilities trigger from an undone action. Do the kept library actions trigger abilities, such as "whenever a card is put into your library" or draw triggers?
   - **Life and damage.** Life totals, damage and the other parts of a replaced event are undone; only the library part stays.
3. **Tests.** Write one test per kind of kept action, each with a real card from `KNOWN_REACHABLE`: a move to a library (Darksteel Colossus), an exile from a library (Ashiok), a draw (Lich), a mill (Angel of Suffering with Shivan Reef), and a shuffle and a reveal where a supported card does one. Each test asserts what stays and that everything else is restored. Cite cr!("733.1") and the cards' rulings.
   - Flip the tests that pin today's behavior: `a_reversed_action_also_reverses_a_draw_made_while_attempting_it` in tests/cr/r733_001_library_exceptions.rs, and the `a_reversed_cast_reverses_...` tests in the scan file.
   - Keep the scan test as the guard that a new way to reach a library action is seen. Change its expectations so the reachable cases are handled rather than listed as deviations.
4. **Properties.** The rollback property (property-tests-1) checks that a reversal restores the game exactly. Change it to "exactly, except the library actions 733.1 keeps", and run `mtg-sim --properties` on a few hundred games.
5. **Register.** Rewrite `illegal-action-reversal-restores-libraries` as the decided rule, no longer a deviation. Then regenerate docs/INTERPRETATIONS.md (`mtg-tools interpretations --write`) and update docs/PROPERTIES.md.

DONE when the engine keeps exactly the library actions 733.1 names, the tests above pass, the register says so, and `cargo test --workspace` passes. Report what stays and how each judgment call was decided.
