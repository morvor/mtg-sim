# property-tests-2: properties over every card and every small case (copies, hidden information, payments, damage assignment)

WHY: as for property-tests-1 (the user chose property tests on Oct 14). This item adds the properties that can be checked exhaustively: for every card, or for every small case. Do it after property-tests-1 has landed; it reuses that item's `tests/properties/` directory, fuzz mode and docs/PROPERTIES.md. Every violation is a bug: fix it (minimal engine edits per CLAUDE.md) with a regression test citing the rules.

PROPERTIES:
1. **A copy has the original's copiable values** (CR 707.2–707.4, 707.8–707.10).
   - **Static check, every card.** For every fully supported permanent card, take an object that becomes a copy of it, and a token created as a copy of it. Each must have the same copiable values as the card:
     - name, mana cost, colors and color indicator;
     - card types, subtypes and supertypes;
     - abilities, compared by their structure fingerprints (`structure.rs`);
     - power/toughness, loyalty and defense.

     Only the differences the rules call for are allowed: token status, and exceptions that the copy effect itself states (707.9). Cover double-faced and split cards per 707.8. For every instant or sorcery, a copy of the spell (707.10) has the original's characteristics and choices.
   - **Dynamic check, sampled.** On a sample of a few hundred cards, play the same scripted sequence (activate each ability, set off each trigger) with the original and with a copy, and compare the outcomes.
2. **Hidden information stays hidden** (CR 400.2, 708; the mtg-api visibility rules).
   - **The property.** What a player observes, meaning the observation plus every legal option offered, must not depend on information hidden from them.
   - **The check.** Compare the player's observation at the same point of two games that differ only in hidden information:
     - another player's hand contents (same size);
     - the order of any library;
     - the identity of a face-down card they can't look at.
   - **Ids.** Object ids and their order must not leak which card is which.
3. **Mana payment** (CR 601.2g–h, 106, 107.4).
   - **The check.** Build small random boards: lands, mana creatures, Treasures, and spending restrictions. Pay small random costs: hybrid, Phyrexian, X, snow and generic. Compare against a brute-force search over all mana abilities:
     - Whenever the search finds a payment, the automatic payer must find one too.
     - Every payment the payer makes must be legal.
   - **Bugs it finds.** A payer that misses a valid payment makes a spell look uncastable.
4. **Combat damage assignment** (CR 510.1c–d; damage assignment order no longer exists).
   - **The check.** Over small random attack and block configurations, compare the set of assignments the engine accepts with the set the rules allow, enumerated by brute force. Cover trample, deathtouch, first and double strike, planeswalkers and battles, and banding.

DONE when the four properties run in tests (the every-card one over the whole pool) and their violations are fixed, and docs/PROPERTIES.md has the results. Report the bugs found, each with its rule and fix.
