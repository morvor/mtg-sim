# ruling-conflicts-2: follow-up conflict cases found by rulings batches

Same method, conclusion types and actions as `ruling-conflicts-1.md` (an independent investigator and an adversarial skeptic for each case, at effort max; quote the sources with dates; conclusions A/B/C/D; at most two subagents at a time). Add a STATUS section at the end when done.

**Cases (Nov 5).**
1. **Ghost Council of Orzhova** against CR 400.7j. Reported by rulings-1 while landing P128/P091 (d948fbee, ccb35825). Find the ruling with `zcat data/rulings.jsonl.gz | grep -i "ghost council"`.
2. **Homicide Investigator.** The 2024-04-12 ruling says the ability "triggers for each of them". The Oracle text says "Whenever one or more nontoken creatures you control die, investigate. This ability triggers only once each turn." The engine follows the Oracle text (844da5ba, rulings-P005), and the ruling is uncited.
3. **Others from rulings-P005.** Any further rulings the rulings-2 worker left uncited as apparent conflicts, listed in docs/agents/engine-gaps.md under "Apparent conflicts (Nov 5)". If that section doesn't exist yet when you start, check `git log --grep '^rulings-P005:'` and the uncited rulings of batch P005 (`python3 scripts/rulings_batches.py show P005 --open`).

New cases reported by later batches are added here by the coordinator.

DONE when every case has a conclusion as in ruling-conflicts-1, the engine and tests follow A/B/C, the register and exemptions say why, and `cargo test --workspace` passes.

## STATUS

Done (Nov 5). Conclusion types: A reconciled, B outdated, C settled by the ruling, D unresolved.
Each case had an independent investigator and an adversarial skeptic; the skeptic's corrected
actions were followed (case 4: D in part, not the investigator's A; case 5: cite, not exempt).
Every new register entry is provisional for the user. Brief case 3 ("others from rulings-P005",
the "Apparent conflicts (Nov 5)" list in `docs/agents/engine-gaps.md`) became cases 3 and 4;
the worker added case 5 from rulings-P110.

| # | Case | Conclusion | Commit(s) |
|---|---|---|---|
| 1 | Ghost Council of Orzhova against CR 400.7j (`cost-moved-source-found-only-by-own-cost`) | C: the cost clause (400.7g in the CR of 2011-09-30, current wording 2016-04-08) doesn't say which references find what the cost moved; six discriminating rulings (Ghost Council re-published in the MM2 notes 2015-05-12, Nantuko Husk 2015, Krav 2018, Bloodflow Connoisseur 2018, Falkenrath Aristocrat 2020, Defiant Salvager 2020) and none against: the source is followed only when a cost part about the source itself moved it (Plague Reaver). Engine fix: about 40 "Sacrifice a creature: ... ~" cards no longer put counters, bonuses or keywords on their own card in the graveyard | b383b1fe, f147ba08 |
| 2 | Homicide Investigator, "triggers for each of them" (`homicide-investigator-triggers-once`) | D: the 2024-04-12 ruling is Midnight Reaper's (Guilds of Ravnica release notes, 2018-09-20) with the name swapped, against both sentences of an Oracle text unchanged since its first preview; no dated change explains it. Engine unchanged (the Oracle text); the ruling stays OPEN | b58a55b6 |
| 3 | Lich, "leaves the battlefield" (rulings-P096) | B in part: the September 2010 Update Bulletin (Matt Tabak, 2010-09-22; Oracle update announced for 2010-09-24) narrowed "leaves the battlefield" to "is put into a graveyard"; the two outdated clauses are exempt in `rulings-P096.tsv`, the rest of all 9 rulings is cited; new test for exile, bounce and Rest in Peace | a781895e |
| 4 | Rimescale Dragon, "all permanents" (rulings-P096; `rimescale-dragon-creatures-only`) | D in part: the ruling is Rimefeather Owl's same-day ruling (2006-07-15) word for word; the card was printed and is Oracle "Creatures". Its "whether or not it put the ice counters on them" is cited; its "all permanents" is not followed (a Forest, and Mishra's Factory once it's no longer a creature, untap) | b58a55b6 |
| 5 | Clive's Hideaway rulings attached to Clive, Ifrit's Dominant (added by the worker from rulings-P110) | A (not applicable, misattached): three FINAL FANTASY release-note rulings (2025-06-06) about Clive's Hideaway's last ability are filed under Clive; read for the Hideaway they agree with the CR (602.1b, 608.2h, 608.2g, 305.2a/b, 305.3) and the engine. Cited under Clive in a Clive's Hideaway test in which Clive is one of the four legendary creatures; no exemption | 88f02d3d |

**For the user.**
1. **Cases 2 and 4, together.** When a Wizards ruling's words contradict the card's own Oracle text
   and no rules or Oracle change explains it, may the ruling's wrong part give way to the Oracle
   text (CR 108.1)? Recommendation: yes, as Gnoll War Band's inconsistent ruling was set aside
   (ruling-conflicts-1 case 2, `rulings-P186.tsv`). If yes, both register entries become decided, and:
   - Homicide Investigator: a new `docs/rulings-exemptions/rulings-P005.tsv` line with the needle
     "Homicide Investigator's ability triggers for each of them" (straight apostrophe), its reason in
     the Gnoll War Band style (conclusion D, the decision and its date, the register id, the Midnight
     Reaper origin, G'raha Tia's ruling of 2025-06-06).
   - Rimescale Dragon: a `rulings-P096.tsv` line "Inconsistent in part with the card's text
     (conclusion D, ...)" with the needle "Rimescale Dragon’s last ability affects all permanents"
     (the shorter "affects all permanents with ice counters on them" would also match the Owl's
     ruling), keeping the citation of the "whether or not" clause.
   - The same answer covers Combat Celebrant (P056: "untaps all of your creatures" for "untap all
     other creatures you control", cited in full, its "other" untested) and Citanul Druid (P178,
     OPEN: "In multiplayer games it affects all players.").
2. **Case 1.** Accept the reading of `cost-moved-source-found-only-by-own-cost`: 400.7j's plain words
   (with 201.5) would let "Exile Ghost Council of Orzhova" find the card in the graveyard; the
   rulings settle it the other way. (It is also arguable as A, through the CR's "the zone it’s
   expected to be in"; the engine is the same either way.)
3. **Optional: misattached rulings.** A policy such as "a ruling the data attaches to the wrong card
   is tested on the card it describes and cited under the card it's attached to, with a comment;
   the attached card is set up in the test when it can take part; being misattached is no reason to
   exempt". It would cover Clive ← Clive's Hideaway (case 5, cited so), Squall, SeeD Mercenary ←
   Starting Town (three texts cited in P097/P098 without Squall in the setup), Astarion, the
   Decadent ← Astarion's Thirst (exempt in `rulings-P194.tsv`, ruling-conflicts-1 case 18; could be
   converted) and Gut, True Soul Zealot ← Halsin, Emerald Archdruid (two texts OPEN in P114).

**Left open.** An action on "the sacrificed creature" (kept as last known information) when the
source was sacrificed to a generic cost wouldn't find the card in the graveyard; no card in the pool
does that ("the sacrificed card" of Rescue from the Underworld goes through the cost's moved
objects and is tested). The same question without a cost (Harvester Troll, no ruling) is out of the
entry's scope.

**Not in this item.** While it was landing, later batches added two more entries to "Apparent
conflicts (Nov 5)": Run Out of Town (rulings-P140: the ruling has the permanent's controller choose
top or bottom, the Oracle text its owner) and Caller of the Untamed (rulings-P025: the ruling chooses
the card to copy during activation, the decided `x-defined-by-target` as the ability resolves). They
are new cases for the coordinator to add.
