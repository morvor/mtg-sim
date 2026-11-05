# ruling-conflicts-3: follow-up conflict cases (Nov 5, second set)

Same method, conclusion types and actions as `ruling-conflicts-1.md` and `ruling-conflicts-2.md` (an independent investigator and an adversarial skeptic for each case, at effort max; quote the sources with dates; at most two subagents at a time). Add a STATUS section at the end when done.

**Cases.** Both come from the "Apparent conflicts (Nov 5)" list in docs/agents/engine-gaps.md.
1. **Run Out of Town** (rulings-P140). The ruling has the permanent's controller choose top or bottom; the Oracle text has its owner choose.
2. **Caller of the Untamed** (rulings-P025). The ruling chooses the card to copy during activation. The decided register entry `x-defined-by-target` chooses it as the ability resolves.
3. **Wastes in limited** (rulings-P192, register entry `limited-free-basics-have-a-basic-land-type`). CR 100.2b lets a limited deck add "basic land cards" beyond the product. The Wastes ruling (2021-03-19) says Wastes must come from your card pool. The engine follows the ruling. Look into tournament rules (MTR) and release notes (Oath of the Gatewatch, 2016) as sources, and whether 100.2b's "basic land cards" is meant as the five basic land types.
4. Any further entries added to that list after ruling-conflicts-2 landed, other than Homicide Investigator, Lich and Rimescale Dragon.

The user has not yet answered ruling-conflicts-2's questions about rulings that contradict the Oracle text, so a case concluded D stays as it is: engine on the Oracle text, ruling OPEN.

DONE when every case has a conclusion as in ruling-conflicts-1, the engine and tests follow A/B/C, the register and exemptions say why, and `cargo test --workspace` passes.

## STATUS

Done (Nov 5). Conclusion types: A reconciled, B outdated, C settled by the ruling, D unresolved.
This session had no Agent tool, so each case's investigator and adversarial skeptic were run in
sequence by the item's worker (each case investigated, then re-examined against its strongest
counter-reading), not as separate subagents. Sources outside `data/` were the archived CR texts
(https://github.com/pit142857/mtg-cr) and the ormos mirror of the Wizards article archive
(https://github.com/maxmakesmagic/ormos, Wayback captures, up to 2022) for release notes and
update bulletins; the Magic Tournament Rules only through search results (media.wizards.com,
magic.wizards.com and web.archive.org were outside the allowed hosts). Following the user's Nov 5
policies, which superseded this brief's "a case concluded D stays as it is", rulings that
contradict the card were exempted under `ruling-contradicting-oracle-gives-way` and every finding
was logged in `docs/RULING_DISCREPANCIES.md`. Every new or rewritten register entry is
provisional for the user, except `x-defined-by-target` (decided Nov 4), whose Affects line was
corrected for Caller of the Untamed without touching its decision.

| # | Case | Conclusion | Commit(s) |
|---|---|---|---|
| 1 | Run Out of Town: the ruling's "controller" chooses top or bottom, the Oracle text's owner | D, explained (transcription error): no reading reconciles them, and the Streets of New Capenna Release Notes (2022-04-20) have the same note with "owner" for "controller", twice; set aside under `ruling-contradicting-oracle-gives-way` (`rulings-P140.tsv`), last sentence cited, new test with a melded permanent another player controls | 88e2a1e6 |
| 2 | Caller of the Untamed: the card to copy chosen during activation | A: CR 601.2b's sentence for an X defined by a later choice (CR effective 2016-09-30; the Kaladesh Update Bulletin, 2016-09-30, names Caller of the Untamed) agrees with the ruling. The engine chose the card as the ability resolved, as the Conspiracy: Take the Crown Release Notes (2016-08-22) had it; it now chooses as X is announced (`kw/choice_defining_x.rs`); ruling cited | 88e2a1e6 |
| 3 | Wastes in limited (`limited-free-basics-have-a-basic-land-type`) | A: 100.2b describes limited play without saying which basic land cards; 100.6 leaves tournaments' further rules to the MTR, whose 7.2 adds only cards named Plains, Island, Swamp, Mountain or Forest; the Wastes, Snow-Covered Wastes and Kaldheim snow-land rulings say the same. "and basic land cards" came into 100.2b in July 2014 (Magic 2015 Update Bulletin, 2014-07-17), before Wastes. Engine unchanged; entry rewritten, provisional | 9b1ac9a5 |
| 4 | The craft copy ruling (24 cards, 2023-11-10) and modal double-faced cards | B in part: it restates 712.14a of 2023 ("transforming double-faced card"); since the CR effective 2025-09-19 a modal double-faced copy returns as its own back face unless that face is an instant or sorcery. Exempt for modal cards (`rulings-P204.tsv`), still cited for single-faced ones; new test (Eddie Brock returns as Venom, Delver as Insectile Aberration, Pestilent Cauldron stays in exile) | 88e2a1e6 |
| 5 | Sanguine Spy: "cards you control" for "cards in your graveyard" (found comparing case 1's release notes with the data; rulings-P093) | D, explained (transcription error): the same release notes say "cards in your graveyard"; set aside under the policy (`rulings-P093.tsv`), the rest cited, new test | 88e2a1e6 |
| 6 | Araumi of the Dead Tide (added by the coordinator; rulings-P024): "you can choose any value for X" for its encore cost "equal to its mana cost" | D: against CR 107.3h (CR effective 2015-01-23; Fate Reforged Update Bulletin: "Any other time, X is 0."), 702.1a's Varolz example with 107.3g, and the Varolz (2013-04-15) and Young Deathclaws (2024-03-08) rulings on the same wording for scavenge; the Commander Legends Release Notes (2020-11-06) have it as is, and no change since explains it. Engine unchanged (X is 0); new provisional entry `x-in-a-cost-equal-to-a-cards-mana-cost-is-0`; its second sentence cited; logged as inconsistent, no exemption until the user decides | 9b1ac9a5 |
| 7 | Arbiter of Knollridge (added to the list by rulings-P014 while this item ran; in the brief's "any further entries"): the ruling's example gives each player the highest life total within that player's own range of influence | D: against CR 801.10 and 801.11, which the CR current as of 2007-10-01 (the ruling's date) already had word for word as 601.10 and 601.11 (its Coat of Arms example was then narrower still: only players in range of both), and against the Imperial Mask (2007-05-01) and Benediction of Moons (2006-02-01) rulings, which use the controller's range; no controller fits its example. Engine unchanged (the CR; rulings-P014's test); logged as inconsistent; the ruling stays uncited until the user decides | 0e5755c2 |

**For the user.**
1. **Arbiter of Knollridge (case 7).** May its ruling be set aside as inconsistent with CR 801.10
   and 801.11 and the other range-of-influence rulings? Recommendation: yes (an exemption line
   citing the decision, as for Gnoll War Band); the engine already follows the CR.
2. **Araumi of the Dead Tide (case 6).** May its "you can choose any value for X" be set aside as
   inconsistent with CR 107.3h and the scavenge rulings, as Gnoll War Band's was? Recommendation:
   yes; then `x-in-a-cost-equal-to-a-cards-mana-cost-is-0` becomes decided and the clause gets an
   exemption line citing that register decision.
3. **A new log kind, "transcription error"** (cases 1 and 5): a ruling whose first publication, in
   the set's release notes, agrees with the card while the data's text doesn't. It is handled under
   `ruling-contradicting-oracle-gives-way`; the policy entry's Affects line names both cards.
4. **`x-defined-by-target`'s Affects line** now has Caller of the Untamed's card chosen as X is
   announced (601.2b, as its ruling says), not as the ability resolves; the decision (Reflecting
   Mirror's X announced, the target limited to it) is unchanged. Please confirm.
5. **`limited-free-basics-have-a-basic-land-type`**: accept as reconciled (A), the engine modelling
   tournament limited play.

**Also found.** Comparing the 69 release-notes files in the ormos mirror (2013 to 2022) with the
data card by card turned up about 1,100 notes the data has with up to four words changed, mostly
grammar or later updates; of the 25 or so whose changed words are about who, where or what, read
by hand, only Run Out of Town's and Sanguine Spy's contradict the card (the others are rewordings
or corrections toward the card, such as Failure // Comply's "Opponents" for "Players"). The rest
weren't read one by one; a rulings batch could run the same comparison for its own cards.
