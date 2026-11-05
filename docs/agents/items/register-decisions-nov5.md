# register-decisions-nov5: record the user's Nov 5 decisions, and start the log of erroneous rulings

WHY: on Nov 5 the user answered every question from ruling-conflicts-1 and ruling-conflicts-2, and the Grip of Chaos question:

> "It looks like some rulings are applied as a template, and are therefore occasionally incorrect. This is the case for Homicide Investigator and the similar cases you mentioned. I agree with all of your recommendations. We should still keep a detailed account of all of the instances though, so they can be checked by a human, and reported to Wizards of the Coast if erroneous. For the Grip of Chaos question, I believe you are correct in that it does not trigger ward and 'whenever this becomes the target' abilities again."

This item is mostly records: the register, the exemptions, a new log and the guides. Engine changes are needed only where a test is missing (part 3). Run the judgment parts at effort max.

## 1. Register: these entries become decided

For each entry:
- set `Status: decided`;
- start the Decision with "Decided by the user, Nov 5.", keeping the decision as written unless noted;
- keep its evidence, and the `interp!` citations of the tests that pin it.

The entries:
- `history-counts-players-who-left`: keep counting players who left. Gnoll War Band's ruling is set aside as inconsistent with the other rulings.
- `room-door-unlock-timestamp`: follow the rulings. An unlocked door's abilities are timestamped from the unlocking. Kind stays a deviation from a literal reading of 613.7a.
- `mdfc-back-face-up-enters-transformed`: yes. Any double-faced permanent that enters with its back face up enters transformed.
- `cast-grant-have-after-cast`: keep the simplification. Record the follow-up: revisit when Shadow the Hedgehog becomes supported, and decide the Henzie edge case (611.3d) then.
- `cyclopean-tomb-land-choice`: accept.
- `when-that-permanent-enters-as-it-is-created` (Jailbreak): accept, as reconciled (A).
- `put-onto-battlefield-blocking-retriggers-becomes-blocked-by` (Aetherplasm): accept, as outdated (B). Name the rules change when it can be dated: compare the CR of 2009-07-10 with the last CR that has the old rule 309.
- `ice-cauldron-last-exiled`: accept, as reconciled (A).
- `self-shuffle-gone-library-still-shuffled` (Blitz Hellion): accept, as outdated (B).
- `random-reselection-same-target` (Grip of Chaos, Chef's Kiss): No. A target reselected as itself doesn't become a target again, so ward and "becomes the target" abilities don't trigger.
- `cost-moved-source-found-only-by-own-cost` (Ghost Council of Orzhova): accept the rulings' reading.
- `homicide-investigator-triggers-once` and `rimescale-dragon-creatures-only`: decided under the policy in part 2.

Don't touch `entered-blocker-never-blocked` or any other provisional entry the user hasn't seen. The coordinator takes those to the user.

## 2. Policy: the card's text wins over a ruling that contradicts it

**Decided by the user, Nov 5.** A Wizards ruling whose words contradict the card's own Oracle text, with no dated rules or Oracle change to explain it, gives way to the Oracle text (CR 108.1). Such rulings are often a template from a similar card, applied to the wrong card. The parts of the ruling that agree with the Oracle text stay cited.

Add this to the register as an entry with id `ruling-contradicting-oracle-gives-way`, kind policy, in the area file that fits. Then apply it, following ruling-conflicts-2's STATUS ("For the user", item 1):
- Homicide Investigator: a line in `docs/rulings-exemptions/rulings-P005.tsv`.
- Rimescale Dragon: a line in `rulings-P096.tsv`, keeping the citation of the "whether or not" clause.
- Combat Celebrant (P056): add a test that pins "other" (Combat Celebrant itself doesn't untap). The ruling's "untaps all of your creatures" gives way; keep what else is cited.
- Citanul Druid (P178, OPEN): read the Oracle text and the ruling first. If the ruling contradicts the card, exempt it under this policy. If it is consistent (multiplayer: every opponent), cite it with a test instead.

Every exemption reason names the policy, the register id, the date of the decision and the ruling's likely origin.

## 3. Policy: misattached rulings

**Decided by the user, Nov 5.** When the data attaches a ruling to the wrong card:
- test the ruling on the card it describes;
- cite it with `ruling!` under the card it is attached to (the macro needs that), with a comment naming the card it describes;
- set up the attached card in the test when it can take part.

Being misattached is never a reason to exempt.

Apply it:
- Astarion, the Decadent ← Astarion's Thirst: convert the exemption in `rulings-P194.tsv` to a test and citation.
- Squall, SeeD Mercenary ← Starting Town (P097/P098): add Squall to the setup where it can take part.
- Gut, True Soul Zealot ← Halsin, Emerald Archdruid (P114): cite the two OPEN texts with tests, if the cards are supported.
- Clive, Ifrit's Dominant ← Clive's Hideaway: already done this way; just check it.

## 4. A log of erroneous rulings, for human review

Create `docs/RULING_DISCREPANCIES.md`. It is the place a human checks the findings and reports them to Wizards of the Coast. For each instance, record:
- the card the ruling is attached to;
- the ruling text, verbatim, with its `published_at` date;
- the conflicting source, quoted with its date: the Oracle text, the CR rule, or the other rulings;
- the kind:
  - **template error:** the ruling contradicts the card's text, and is a copy of a similar card's ruling;
  - **misattached:** the ruling describes another card;
  - **inconsistent:** the ruling contradicts other rulings on the same question;
  - **outdated:** the ruling is older than a dated rules or Oracle change, but is still published;
- the evidence: the likely origin, such as the card whose ruling it copies, with that ruling's date, and any release notes;
- what the engine does;
- where it is recorded: the exemption file, the test, the register id;
- status: open for review, reported, or confirmed.

Start with every instance known so far:
- **template errors and inconsistent rulings:** Homicide Investigator (Midnight Reaper's ruling), Rimescale Dragon (Rimefeather Owl's ruling), Combat Celebrant, Citanul Druid (if it qualifies), Gnoll War Band, and Tempt with Glory's "starting with the opponent on your left" if it is wrong for off-turn casts (see ruling-conflicts-1 case 16; record it as "narrower than stated" if so);
- **misattached rulings:** Astarion, Clive, Squall, Gut;
- **outdated rulings:** every B conclusion of ruling-conflicts-1 and ruling-conflicts-2 (Aetherplasm, Blitz Hellion, Cyclopean Tomb, Krosan Adaptation, Jade Statue, Deranged Assistant, the Eggs, Charmed Pendant, Oculus Whelp and the other MDFC rulings, Corruption of Towashi, Dance of the Dead, Lich), plus every other rulings exemption whose reason calls the ruling superseded, outdated or overridden (`rg -il "supersed|outdated|overrid" docs/rulings-exemptions`).

Add a check so the log stays complete. For example, a test in `cargo test -p mtg-data`, or an `mtg-tools rulings-coverage --check` rule: every exemption whose reason cites the Oracle-text policy, misattachment, inconsistency or an outdated ruling must have a log entry naming that card and needle.

## 5. Guides

- docs/agents/worker.md, rulings brief: describe both policies. A worker who finds a ruling that contradicts the card's text, is misattached, contradicts other rulings or is outdated adds it to `docs/RULING_DISCREPANCIES.md`. A contradiction with no clear explanation still goes to the coordinator as a conflict case.
- CLAUDE.md, "Tests: citing rules and rulings": one or two sentences on the two policies and the log.

DONE when:
- the entries in part 1 are decided;
- the two policies are recorded and applied;
- the log exists with every known instance, and the check enforces it;
- INTERPRETATIONS.md is regenerated, `cargo run --release -q -p mtg-tools -- rulings-coverage --check` passes, and `cargo test --workspace` passes.
