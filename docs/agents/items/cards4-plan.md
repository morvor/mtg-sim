# cards4-plan: plan round 4 of card support (every text the landed round-3 items left unsupported)

WHY: round 3 (docs/agents/card-support-plan-3.md) is nearly done. Six of its nine items have landed, and 30,486 of 31,051 paper cards are fully supported (98.2%). Each landed item left some texts unsupported, with a reason in its report. Round 4 has to place every text that is still unsupported, compile-first (CLAUDE.md), so the card workers have well-scoped items when round 3 ends. This item only plans. It writes documents and changes no engine code.

1. **Inputs:**
   - every unsupported text on legal paper cards (`mtg-tools unsupported --paper --by-card`);
   - docs/agents/card-support-plan-3.md and docs/agents/round3/: deferred.tsv, and the scripts probe.py, autoprobe.py and check_plan.py;
   - the round-3 items' briefs and `.texts.tsv` files, and their landing commits (`git log --grep '^cards3-<slug>:'`). The commits and the items' reports say which texts each item left and why;
   - docs/DEFERRED.md.

   Leave texts owned by round-3 items that are still in flight to those items, and list them. When this brief was written that was `cards3-followups-choices`, `cards3-game-rules-dice` and `cards3-manual-defs`; check `git log` for whichever haven't landed when you start.
2. **Find why each text fails.** Re-probe each remaining text the way round 3 did (card-support-plan-3.md, "Method"; round3/autoprobe.py): one component at a time, to find the construct the compiler or the engine is missing. For a text that a round-3 item tried and left, read that item's reason first. If the item left it because of an engine gap, the plan must name the engine work, not only the grammar.
3. **Group the texts into families by that missing construct.**
   - Shared grammar comes first: a construct that two or more in-scope cards share must be compiled, not hand-written.
   - Give every one-off claim the round-3 check: search all in-scope paper cards for the same construct before calling a text a genuine one-off. Genuine one-offs go to a manual-definitions item.
   - Texts that are out of scope under DEFERRED.md are listed, not planned.
   - Texts that need a user decision are listed with a recommendation each, in a "Decisions needed" section of the plan, not planned. One example: the five Conspiracy cards whose draft abilities turn them face down or change how packs are drafted, if cards3-game-rules-dice leaves them.
4. **Outputs:**
   - docs/agents/card-support-plan-4.md, with the same sections as round 3: baseline counts, method, a work-item table ordered by cards unlocked per unit of effort, and a projection.
   - One brief per item, `docs/agents/items/cards4-<slug>.md` with a `.texts.tsv` (about 60–150 texts each, one parser or engine area per item so that items rarely edit the same files). Use the same brief format as the cards3 items.
   - docs/agents/round4/ for the re-check list (texts waiting on an in-flight item), the deferred list (round3/deferred.tsv carried over, plus anything new), the snapshot and the check script.
5. **Process:** split the text list into shards and run subagents on them in parallel. Then have an adversarial reviewer sample every family, to confirm its texts really share the gap, and check every one-off claim.

DONE when every unsupported text sits in exactly one place: a cards4 item, the manual item, an in-flight round-3 item, the re-check list or the deferred list. A script check like round 3's `check_plan.py` must confirm this. Report the counts, the item table and the decisions needed.
