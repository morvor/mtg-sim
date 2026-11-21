# register-fixes-1: fix the three provisional deviations the user ruled on

WHY: on Nov 4 the user accepted the coordinator's recommendation to fix three register entries where the engine knowingly departs from a ruling. Make the engine follow the rulings and the CR, without card-name special cases.

1. **`library-arranged-by-owner-not-instructing-player`** (docs/interpretations/zones-objects.md):
   - What is wrong: cards an effect puts into another player's library "in any order" are arranged by their owner. The player the effect instructs should arrange them, as the rulings say.
   - Cards: Misinformation, Drafna's Restoration, Lodestone Bauble, Agonizing Memories.
   - The entry's suggested fix: record "in any order" on `Destination`, and carry the arranging player through `MoveEv` to `order_simultaneous`.
2. **`linked-reveal-trigger-as-reflexive`** (docs/interpretations/dice-coins.md):
   - What is wrong: Priority Boarding's second ability ("Whenever you reveal a card with mana value less than the result this way, ...") is compiled into its first ability as a reflexive trigger.
   - The fix: make it the permanent's own triggered ability, linked to the first (CR 603.2, 607.2). Then it doesn't trigger if Priority Boarding has lost its abilities by the time of the reveal, and it behaves as the rules say for a copy of the first ability.
3. **`nihiloor-taps-one-opponent-at-a-time`** (docs/interpretations/triggers-durations.md):
   - What is wrong: the engine taps one opponent's creature at a time.
   - The ruling: "As Nihiloor's first triggered ability resolves, you choose up to one creature for each opponent, and tap them all at the same time. Then, a separate triggered ability is put on the stack for each creature you tapped this way."
   - The fix: make every choice first, tap the creatures in one event, then create one reflexive ability per tapped creature, each tied to its opponent and its tapped creature.

**For each:**
- Read the entry, the card's rulings and the CR rules it cites.
- Implement the fix generally, so every card with the same construct gets it.
- Add tests that fail on the old behavior and cite the rulings with `ruling!`.
- Then update the register. If no judgment call remains, delete the entry and its `interp!` citations. Otherwise rewrite it as decided ("Decided by the user, Nov 4: ..."), as an interpretation rather than a deviation.
- Regenerate docs/INTERPRETATIONS.md.

**Also in this item: River Song**, the last card of cards5-manual-defs, now that its cards5 items have landed. Draws from the bottom of the library already have a hook (3439e35b); see items/cards5-manual-defs.md and its STATUS.

DONE when the three behaviors follow the rulings, tests pin them, River Song is fully supported and tested, the register is updated, and `cargo test --workspace` passes.
