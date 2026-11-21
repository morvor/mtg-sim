# cards6-plan: plan round 6 of card support (the last unsupported cards)

WHY: round 5 has landed in full. On Nov 1, 32,040 of 32,111 paper cards were fully supported (99.8%). Of the 71 unsupported cards, 35 are deferred under DEFERRED.md, which leaves about 36.

Plan 5 predicted this long tail ("What is left after round 5"):
- texts no round could compile;
- one-off cards;
- engine gaps that need a whole feature: history across turns per object and per source spell, choices made by players other than the controller, manifesting from a hand, copies of cards in hidden zones, notes kept on objects across turns, durations that end on events other than steps, and draft-table rules.

Plan round 6 the way cards5-plan planned round 5, but expect each item to be one engine feature rather than a family of grammar. This item only plans: it writes documents and changes no engine code.

1. **Inputs:**
   - `mtg-tools unsupported --paper --by-card`, built from the current branch;
   - docs/agents/card-support-plan-5.md with its "Update" sections, and the round5/ lists;
   - the cards5 items' landing commits (`git log --grep '^cards5-<slug>:'`) and their briefs;
   - docs/DEFERRED.md.
2. **Find why each text fails.** Re-probe each remaining text one component at a time (round3/probe.py, round3/autoprobe.py, then drills), as round 5 did. Name the missing engine feature where that is the gap, not only the grammar.
3. **Group the texts into items:**
   - Shared constructs come first: a construct that two or more in-scope cards share must be compiled, not hand-written (CLAUDE.md).
   - Give every one-off claim the usual search over all in-scope paper cards. Genuine one-offs go to a manual-definitions item.
   - List texts that fall under DEFERRED.md; don't plan them.
   - List anything only the user can decide, each with a recommendation, in a "Decisions needed" section. An example is deferring a card whose support needs an engine feature out of proportion to one card.
4. **Outputs:**
   - docs/agents/card-support-plan-6.md: baseline counts, method, a work-item table ordered by cards unlocked per unit of effort, and a projection;
   - one brief per item, `docs/agents/items/cards6-<slug>.md` with a `.texts.tsv`, in the cards5 brief format;
   - docs/agents/round6/: the snapshot, the deferred list and a check script like round 5's `check_plan.py`.
5. **Review.** An adversarial reviewer samples every family and checks every one-off claim and every proposed deferral.

DONE when every unsupported text sits in exactly one place (a cards6 item, the manual item or the deferred list) and the check script confirms it. Report the counts, the item table and any decisions needed.
