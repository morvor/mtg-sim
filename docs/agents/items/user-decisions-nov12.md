# user-decisions-nov12: the user's answers on buyback, Strict Proctor, Headliner Scarlett and returned-card values

Run the usual implementer and reviewer flow (worker.md). The reviewer runs at max. The user answered on Nov 12.

## Part A: buyback and alternative-cost reductions (docs only)

**The user:** "I agree with the recommendations about buyback for now, but could discover that I am wrong about this in the future, so let's make sure it is noted somehow."

Mark both entries decided ("Decided by the user, Nov 12, for now"). In each, add a line: "The user noted they may revisit this reading; see the 'Revisit later' list in docs/agents/finish-line-plan.md §5." The entries:
- `keyword-cost-reduction-on-an-additional-cost-within-it`: keep the reading, buyback reductions bounded by the buyback cost.
- `alternative-keyword-cost-reduction-reduces-additional-costs`: accept the reading, flashback, dash and blitz reductions also come off additional costs paid with them.

## Part B: Strict Proctor's family, except abilities that trigger on leaving a graveyard

**The user:** "For the 'creature entering the battlefield' question, I agree with the situations you've described, except for the desecrated tomb case - in that case, even though the creature leaves the graveyard at the same time it enters the battlefield, those are two separate things, and desecrated tomb should still trigger."

**Reading.**
- A card put onto the battlefield from a graveyard both leaves the graveyard and enters the battlefield. For "[a permanent entering] causes a triggered ability to trigger", only abilities whose matched trigger event is the entering count:
  - enters abilities of any wording: "enters or attacks" on the entering, "enters, if", "enters from a graveyard";
  - not abilities that matched the leaving: Desecrated Tomb's "Whenever one or more creature cards leave your graveyard", and any other leaves-a-zone trigger.
- Strict Proctor therefore doesn't counter Desecrated Tomb's ability, and the Bat is made with no {2} to pay. Aboleth Spawn and Firebender Ascension work the same way.
- Everything else in `ability-triggered-cause-is-the-event` stays as it is: Sun Titan's attack isn't an entering, and reflexive abilities don't count.

**Task.**
1. Implement this in triggers.rs (`TriggerCond::AbilityTriggered`, `tag_triggering_event`). Judge it by which trigger event matched: the zone-change event's destination side (entering the battlefield) against its origin side (leaving a zone).
2. Flip `an_enters_from_a_graveyard_ability_is_caused_by_the_permanent_entering` where it covers Desecrated Tomb. Add a test: Strict Proctor with Desecrated Tomb on a reanimation, where the Bat is made and no {2} is asked. Keep Triarch Praetorian's "enters from a graveyard" counted.
3. Rewrite the entry as decided ("Decided by the user, Nov 12"), with the user's words. Keep the old graveyard part as the rejected alternative.

## Part C: Headliner Scarlett, the "can't look" half set aside

**The user:** "I agree that the headliner scarlett ruling is incorrect, and we should be able to look at the cards."

Follow ruling-conflicts-6's recommendation (docs/agents/items/ruling-conflicts-6.md):
- add the exemption line for the look half in docs/rulings-exemptions/rulings-P144.tsv, naming the decision;
- cite the play half in `headliner_scarlett_card_cant_be_played_after_that_turn` (tests/cards/permission_scopes.rs);
- add a Kaya the Inexorable emblem test: a legendary card Scarlett exiled on an earlier turn can still be cast through the emblem, since its owner may still look at it (CR 406.3, 406.3b);
- update the docs/RULING_DISCREPANCIES.md entry's Recorded line (status stays open for review);
- fix the "CR 406.3b" comment in oracle/patterns/permission_grammar.rs to 406.3.

## Part D: returned-card values (docs and small notes)

**The user:** "For sister Hospitaller, I accept the recommendations and will try to resolve the gaps later."

1. Mark `returned-card-values-by-wording` decided ("Decided by the user, Nov 12"). Accept:
   - the wording split;
   - "its power/toughness" after a graveyard return reads the permanent;
   - a card that never arrives (Containment Priest) loses 0 for Reanimate;
   - the Deicide name search stays a known gap.
2. List the three edge cases as open gaps in the entry, pointing to the "Revisit later" list.

Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.
