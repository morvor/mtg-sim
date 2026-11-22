# cards3-plan: plan round 3 of card support (every text still unsupported after round 2)

WHY: round 2 (docs/agents/card-support-plan-2.md) is nearly done: about 29,900 of 31,051 paper cards are fully supported. Round 3 has to place every text that is still unsupported, compile-first (CLAUDE.md), so the card workers have well-scoped items again. This item only plans. It writes documents and changes no engine code.

1. **Inputs:**
   - every unsupported text on legal paper cards (`mtg-tools unsupported --paper`, by card as in round 2);
   - docs/agents/round2/ (gap-owned.tsv, pending-recheck.tsv, deferred.tsv);
   - the round-2 items' `.texts.tsv` files;
   - docs/DEFERRED.md.

   Leave texts owned by round-2 items that are still in flight to those items, and list them: `cards2-copies-tokens-counters`, `cards2-triggers-delayed` and `cards-manual-defs-2`, or whichever haven't landed when you start (check `git log`).
2. **Find why each text fails.** Probe each remaining text the way round 2 did (card-support-plan-2.md, "Method"): one component at a time, to find the construct the compiler is missing.
3. **Group the texts into families by that missing construct.** Shared grammar comes first: a construct that two or more cards share must be compiled, not hand-written. Give every one-off claim the round-2 check: search all in-scope paper cards for the same construct before calling a text a genuine one-off. Genuine one-offs go to a manual-definitions item. Texts that are out of scope under DEFERRED.md are listed, not planned.
4. **Outputs:**
   - docs/agents/card-support-plan-3.md, with the same sections as round 2: baseline counts, method, a work-item table ordered by cards unlocked per unit of effort, and a projection.
   - One brief per item, `docs/agents/items/cards3-<slug>.md` with a `.texts.tsv` (about 100–200 texts each, one parser area per item so that items rarely edit the same files). Use the same brief format as the cards2 items.
   - docs/agents/round3/ for the re-check list (texts waiting on an in-flight item) and the deferred list.
5. **Process:** split the text list into shards and run subagents on them in parallel. Then have an adversarial reviewer sample every family, to confirm its texts really share the gap, and every one-off claim.

DONE when every unsupported text sits in exactly one place: a cards3 item, the manual item, an in-flight round-2 item, the re-check list or the deferred list. A script check like round 2's `check_plan.py` must confirm this. Report the counts and the item table.
