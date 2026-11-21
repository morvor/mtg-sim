# cards4-paper-plan: plan the paper cards the tools left out (117 texts)

WHY: the tools counted about 1,060 paper cards as digital-only, because their Scryfall oracle record is an online-only reprint (item `paper-scope-1` fixes that). 111 of those cards are unsupported, with 117 texts, and no plan places them: docs/agents/round4/paper-scope-gap.tsv. On Oct 28 the user said "get to work on the 117 texts". This item plans them the way `cards4-plan` planned round 4 (docs/agents/card-support-plan-4.md, "Method"). It writes documents and changes no engine code.

1. **Probe.** Start from round4/paper-scope-gap.tsv. When `paper-scope-1` lands, take its refreshed copy of the file and place any text it adds. Probe each text one component at a time with round3/probe.py and autoprobe.py, as round 4 did, to find the construct the compiler or the engine lacks. Most are old-card wording (Masters Edition, Tempest).
2. **Group the texts into families by that missing construct.**
   - Shared grammar comes first: a construct two or more in-scope cards share must be compiled (CLAUDE.md).
   - Give every one-off claim the usual check against all in-scope paper cards, now including these.
   - Where a family matches one in a cards4 item no worker has started, add its texts to that item's .texts.tsv and brief. Otherwise create new items, `cards4-paper-<area>`: one parser or engine area each, the cards4 brief format, about 40–100 texts each.
   - Texts that fall under DEFERRED.md go to round4/deferred.tsv.
3. **Update the plan:**
   - a section in docs/agents/card-support-plan-4.md: counts, the item table and the projection;
   - round4/check_plan.py: it now also checks paper-scope-gap.tsv, so that every text sits in exactly one place.
4. **Review.** An adversarial reviewer samples every family and every one-off claim, as in round 4.

DONE when every text in the refreshed paper-scope-gap.tsv sits in exactly one item or list and check_plan.py passes. Report the item table.
