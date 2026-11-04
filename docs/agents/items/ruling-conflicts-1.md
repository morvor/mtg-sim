# ruling-conflicts-1: resolve every apparent conflict between the CR, the rulings and the Oracle text

WHY: on Nov 4 the user decided:

> "The CR and rulings should NOT contradict. If they seem to, we need to examine these cases much more closely. Take the items that currently are listed as having a contradiction (as well as any that may have been decided in the past) and ensure they are investigated to try to reach a conclusion that resolves the sources."

Until now, when the CR text and a Scryfall ruling seemed to disagree, workers followed one source and exempted or overrode the other. The same happened for two rulings that disagreed, or a ruling and the card's Oracle text. This item re-examines every such case and looks for the conclusion that makes the sources agree.

**Known cases (12).**

These register entries stay provisional until this item concludes:
1. `put-onto-battlefield-blocking-retriggers-becomes-blocked-by`: CR 509.3d against Aetherplasm's 2005 ruling about Deathgazer. The ruling is exempt as superseded in docs/rulings-exemptions/rulings-P006.tsv.
2. `history-counts-players-who-left`: Gnoll War Band's ruling against CR 800.4i and the rulings of Belbe, Teysa, Kaito, Moonshae Pixie and others. It is exempt as overridden in rulings-P186.tsv. Read 800.4i closely: it speaks of "actions taken by a player" who has left, and being dealt damage or losing life is not plainly an action that player took.
3. `self-shuffle-gone-library-still-shuffled`: CR 701.24c against Blitz Hellion's 2009 ruling.
4. `room-door-unlock-timestamp`: the Roaring Furnace // Steaming Sauna ruling against a literal reading of CR 613.7a.
5. `ice-cauldron-last-exiled`: the ruling ("the card that was exiled ... when the most recent charge counter was put there") against the Oracle text ("the last card exiled with this artifact").
6. `cyclopean-tomb-land-choice`: the ruling ("you must remove a mire counter") against the Oracle text ("remove all mire counters").

These rulings were exempted earlier as superseded by the current CR, without the user:
7. Krosan Adaptation: "token copies ... considered to have been created", against CR 111.13 and 608.3f.
8. Jade Statue: "gain the creature type Golem in addition to those creature types", against CR 205.1b.
9. Deranged Assistant: "can't be reversed", against CR 605.1a.
10. Mossfire Egg: "This is a mana ability", against CR 605.1a.
11. Oculus Whelp: "modal double-faced permanents ... are never transformed permanents", against CR 701.27a and 701.27g.
12. Corruption of Towashi: "modal double-faced permanents can't enter transformed", against CR 701.27a and 701.27g.

**Tasks:**
1. **Inventory.** Confirm the list and find any other cases:
   - every register entry, whatever its status (docs/interpretations);
   - every rulings exemption whose reason relies on the CR or another ruling over the ruling (docs/rulings-exemptions);
   - test comments: `rg -i "supersed|outdated|overrid|predates|contradict" crates/mtg-engine/tests`;
   - rulings cited only in part because another part disagrees, such as the Blitz Hellion test.
2. **Investigate each case.** Use an independent investigator subagent for each case, then an adversarial skeptic that tries to refute its conclusion. Run at most two subagents at a time. For each case:
   - Quote exactly: the current CR text (data/comprehensive-rules.txt, effective 2026-09-25); the ruling with its `published_at` date (data/rulings.jsonl.gz); the card's Oracle text.
   - First, look for a reading under which the sources agree. The ruling may address a narrower situation, use a term in its CR sense, describe a card whose Oracle text has since changed, or agree with a closer reading of the rule.
   - If both cannot be true, establish which is current, with dated evidence:
     - when the rule or its wording changed (the Comprehensive Rules update notes in release notes and update bulletins, CR change lists, the MTG wiki);
     - when the ruling or the Oracle text was published or changed.
   - Use WebSearch and WebFetch where they work, and record every source with its date. A claim that a ruling predates a rules change must name the change and its date.
   - Reach one of these conclusions:
     - **A, reconciled:** the sources agree under a stated reading.
     - **B, outdated:** the ruling or Oracle wording is older than a named, dated rules change.
     - **C, settled by the ruling:** the CR is silent or ambiguous and the ruling decides it.
     - **D, unresolved:** there is evidence on both sides, and the user must decide.
3. **Act on each conclusion.**
   - **A, B and C:** make the engine follow the conclusion, with tests when behavior changes. Cite the ruling with `ruling!` wherever it agrees with the conclusion, and in that case remove its exemption. Keep an exemption only for B, with the change named and dated. Rewrite the register entry with the evidence; it stays provisional, because the coordinator takes the conclusions to the user.
   - **D:** leave the engine as it is, and write up both sides with a recommendation.
4. **Report.** One short paragraph per case:
   - the sources, briefly quoted;
   - the conclusion type;
   - the evidence, with dates;
   - any engine change;
   - what the user needs to decide.

DONE when every case has a conclusion backed by quoted sources (or a D write-up), the engine and tests follow the A, B and C conclusions, the register entries and exemptions say why, INTERPRETATIONS.md is regenerated, and `cargo test --workspace` passes.
