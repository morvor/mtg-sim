# user-questions-nov7: the user's follow-ups on the two open decisions of Nov 6

## Part A: Fiery Annihilation, control against attachment (an investigation)

Run this at effort max, with an independent investigator subagent and a separate adversarial skeptic subagent, as in ruling-conflicts-1.

**Background.** Register entry `dependent-target-checked-against-an-illegal-target`; ruling-conflicts-4, case 1.

**The user's question (Nov 7):** "I believe the difference is that Fiery Annihilation is the only one that doesn't deal with control. All of the others are just checking who the controller of something is, while Fiery Annihilation deals with whether an equipment is attached to a creature? I'm curious if anything in the CR aligns with that being a real distinction. If not, I agree with the recommendation and we should just make sure to log this."

**Task.** Find whether anything in the current CR (data/comprehensive-rules.txt) makes the kind of relationship matter. The question is how a dependent target is checked against an illegal target that is still in its zone:
- the dependent target is described by its controller, as in Searing Blaze, Yosei and Run Away Together;
- or by being attached to the illegal target, as in Fiery Annihilation.

Look at least at:
- 608.2b, especially its last sentences;
- 115 (targets), 115.9b, 303.4 and 301.5 (attachment), and 701.3 (attach);
- 608.2g and 608.2h (information, last known information).

Also check every other ruling on a dependent target defined by attachment, "enchanted by" or "equipped by" to an illegal target. Use `zcat data/rulings.jsonl.gz` and look for cards with "attached to that creature", "that creature's controller", "Aura attached to it" and similar. Check whether those rulings group by relationship kind.

Quote every source verbatim with its date.

**Outcomes.**
- **No CR-grounded distinction:**
  - make the entry decided ("Decided by the user, Nov 7") as recommended: the engine keeps checking the dependent target against the illegal one;
  - set Fiery Annihilation's ruling aside as inconsistent, with an exemption line naming the decision;
  - update its docs/RULING_DISCREPANCIES.md entry, and add that the user asked whether attachment differs from control, and what the investigation found.
- **A real distinction:** leave the engine and the entry as they are. Write up the rule text that supports it and what the engine would change. Report it for the user to decide.

## Part B: "you and that player each draw" for one player (a decision and an engine change)

**Decided by the user (Nov 7):** "I feel like the Diviner Spirit ruling does not contradict the Angel of Destiny ruling at all. They both seem to imply a single instance, and that's what I would go with."

**Change.**
- Rewrite register entry `player-named-twice-draws-twice` as decided, with the opposite decision. When "you" and "that player" of "you and that player each draw that many cards" are the same player, that player is given **one** instruction to draw twice that many. The same goes for gaining life (Angel of Destiny's ruling) and any other "you and that player each [do N]" whose referents coincide.
- So a replacement effect that refers to the number of cards drawn (121.2a; e.g. Alms Collector) applies once to the combined draw. With an opponent's Alms Collector, Diviner Spirit's 2 combat damage redirected to its controller has that player and Alms Collector's controller each draw one card.
- Change the engine generally, with no card-name special cases. Update the tests: the old two-instruction tests become one-instruction tests citing the Diviner Spirit and Angel of Destiny rulings, and there is an Alms Collector test.
- The life-gain side should already behave this way; check it and test it.
- Regenerate INTERPRETATIONS.md.

DONE when Part A has an outcome as above, Part B's behavior and tests are in, and `cargo test --workspace` passes.
