# cards5-plan: plan round 5 of card support (the long tail left after round 4)

WHY: round 4 (docs/agents/card-support-plan-4.md, with its "Paper-scope texts" section) is nearly done. On Oct 29, 31,667 of 32,111 paper cards were fully supported (98.6%), counted with the paper rule from paper-scope-1. Each round-4 item leaves some texts unsupported, with a reason in its report. Round 5 places every text that is still unsupported, compile-first (CLAUDE.md), so the card workers have well-scoped items when round 4 ends. This item only plans. It writes documents and changes no engine code.

1. **Inputs:**
   - every unsupported text on legal paper cards (`mtg-tools unsupported --paper --by-card`, built from the current branch);
   - docs/agents/card-support-plan-4.md and docs/agents/round4/: deferred.tsv, paper-scope-gap.tsv, drills.tsv and check_plan.py; the probe scripts are in round3/;
   - the round-4 items' briefs and `.texts.tsv` files, and their landing commits (`git log --grep '^cards4-<slug>:'`). The commits and the items' reports say which texts each item left and why;
   - docs/DEFERRED.md.

   Leave texts owned by round-4 items that are still in flight to those items, and list them. When this brief was written that was `cards4-triggers-durations`, `cards4-costs-mana-replacements`, `cards4-manual-defs`, `cards4-paper-effects` and `cards4-paper-combat-statics`; check `git log` for whichever haven't landed when you start.
2. **Find why each text fails.** Re-probe each remaining text the way round 4 did (card-support-plan-4.md, "Method"): one component at a time, to find the construct the compiler or the engine lacks. Many of these texts have been tried by two or three rounds, so read the last item's reason first. Name the engine work as well as the grammar when an item left a text because of an engine gap.
3. **Group the texts into families by that missing construct.**
   - Shared grammar comes first: a construct that two or more in-scope cards share must be compiled, not hand-written.
   - Give every one-off claim the usual check: search all in-scope paper cards for the same construct before calling a text a genuine one-off. Genuine one-offs go to a manual-definitions item.
   - Texts that are out of scope under DEFERRED.md are listed, not planned.
   - Texts that need a user decision are listed with a recommendation each, in a "Decisions needed" section of the plan, not planned.
4. **Outputs:**
   - docs/agents/card-support-plan-5.md, with the same sections as round 4: baseline counts, method, a work-item table ordered by cards unlocked per unit of effort, and a projection.
   - One brief per item, `docs/agents/items/cards5-<slug>.md` with a `.texts.tsv`. Aim for about 40–120 texts per item, one parser or engine area per item, so that items rarely edit the same files. Use the cards4 brief format.
   - docs/agents/round5/ for the snapshot, the re-check list (texts waiting on an in-flight item), the deferred list and the check script.
   - In the plan, an estimate of what is left after round 5. That is the long tail: texts no round could compile, one-offs, and engine gaps. Say what kind of work finishing it would take.
5. **Process:** split the text list into shards and run subagents on them in parallel. Then have an adversarial reviewer sample every family, to confirm its texts really share the gap, and check every one-off claim.

DONE when every unsupported text sits in exactly one place: a cards5 item, the manual item, an in-flight round-4 item, the re-check list or the deferred list. A script check like round 4's `check_plan.py` must confirm this. Report the counts, the item table and any decisions needed.
