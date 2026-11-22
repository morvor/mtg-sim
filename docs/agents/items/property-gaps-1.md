# property-gaps-1: settle the rules gaps the property tests found

WHY: property-tests-1 (docs/PROPERTIES.md, "Remaining gaps") found two places where the engine knowingly differs from the Comprehensive Rules, plus one engine policy. Neither is in the interpretation register yet. The register exists so the user can see and overrule such calls (docs/agents/verification-plan.md), so they belong there, and fixed where a card makes them matter.

1. **CR 733.1's exceptions.** When an action has to be reversed, the engine restores a copy of the whole game. CR 733.1 says some events can't be reversed: moving cards to or from a library, shuffling, and revealing library cards. Read the full text of 733.1–733.2.
   - Find whether any supported card can actually reach this. That means a cost or an activation step that moves, shuffles or reveals library cards during an action that can then turn out illegal. Search the compiled costs, e.g. "exile the top card of your library" or "reveal the top card" as a cost, or a library search during a cost.
   - If a card can reach it, implement the exception: when an attempt is reversed, keep what 733.1 says stays. Test it with that card.
   - Either way, add a register entry, kind deviation if it isn't fixed, with the exact rule text, what the engine does, the cards found, and the decision. Mark it provisional so the user can review it.
2. **Links of granted abilities (CR 607.5).** `ability_grants::snapshot` derives fresh links from the ability-id counter modulo 0x7fff, so a link can rarely equal another link on the same object. Make the links distinct per object, and add a test that would catch a collision.
3. **Ability ids depend on the process.** The ids in agent-protocol requests differ between processes, while answers use option indexes. Add a policy entry for this, unless an existing entry already covers it.

Regenerate docs/INTERPRETATIONS.md and update the "Remaining gaps" section of docs/PROPERTIES.md.

DONE when the 733.1 gap is fixed or registered (with the list of affected cards), granted-ability links can't collide, and the ids policy is registered. Report what you found for 733.1.
