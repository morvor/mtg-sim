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

## STATUS

Done (Nov 5). Conclusion types: A reconciled, B outdated, C settled by the ruling, D unresolved.
Every new or rewritten register entry is provisional for the user.

| # | Case | Conclusion | Commit(s) |
|---|---|---|---|
| 1 | Aetherplasm vs Deathgazer (`put-onto-battlefield-blocking-retriggers-becomes-blocked-by`) | B: old single CR 309 "becomes blocked" rule, split into 509.3a-d by Feb 2011 (probably M10, 2009-07-10) | ea261eab |
| 2 | Players who left the game (`history-counts-players-who-left`) | C: ~15 rulings 2007-2024 count them; Gnoll War Band the outlier; history counts fixed to include them | 785d9ef1, 12748b8f |
| 3 | Blitz Hellion shuffle (`self-shuffle-gone-library-still-shuffled`) | B: Magic Origins CR update 2015-07-16 | 785d9ef1 |
| 4 | Room door timestamp (`room-door-unlock-timestamp`) | D (leaning C): rulings kept as a provisional deviation from 613.7a; user decides | 6d149a6f |
| 5 | Ice Cauldron (`ice-cauldron-last-exiled`) | A | c92b1d57 |
| 6 | Cyclopean Tomb (`cyclopean-tomb-land-choice`) | B in part: Update Bulletins 2008-07-09 / 2008-09-24 | dd67c8ca |
| 7 | Krosan Adaptation, token copies "created" | B: Zendikar Rising CR changes 2020-09-22 | e6421907 |
| 8 | Jade Statue, "in addition to those creature types" | B: Throne of Eldraine CR update 2019-09-27 (effective 2019-10-04) | f57418e2 |
| 9 | Deranged Assistant, "can't be reversed" | B: 605.1a revision effective 2026-08-07 | 906ffd16 |
| 10 | Mossfire Egg and the other Eggs, "This is a mana ability" | B: 605.1a revision effective 2026-08-07 | a2c501b6 |
| 11 | Oculus Whelp / Mutagen Connoisseur / Invasion of Pyrulea, MDFCs "never transformed" | B: Marvel's Spider-Man CR update 2025-09-19 | 86f1fb6d |
| 12 | Corruption of Towashi, MDFCs "can't enter transformed" | B: CR update 2025-09-19; provisional `mdfc-back-face-up-enters-transformed` | 86f1fb6d |
| 13 | Cast grants when the source leaves (Wort/Imoti/Party Thrasher vs 610.5) | A: 610.5 (CR 2026-01-16) covers only "gain ... as you cast them"; engine fix; provisional `cast-grant-have-after-cast` | ccb59166, 924f907b, 60501ff1 |
| 14 | Dance of the Dead, two on one creature | B: Masters Edition Update Bulletin 2007-08-29 | 63d820ca |
| 15 | Library arranged by owner vs instructing player (see below) | A: resolved by the user's register decision of Nov 4, applied in item register-fixes-1 | 178596b8 (register-fixes-1) |
| 16 | Tempting offer order off-turn (CR 101.4) | A: the 2013 rulings describe the caster's own turn; Tempt with Mayhem 2024-06-07 starts with the active player | 4a626295 |
| 17 | Jailbreak (`when-that-permanent-enters-as-it-is-created`) | A: 603.12 since the CR of 2019-01-25 | 80061407 |
| 18 | Astarion, the Decadent, ruling about X | A (not applicable): the ruling is misattached; it belongs to Astarion's Thirst (CLB release notes 2022-06-01) | d909f382 |
| 19 | Deathtouch noncombat excess damage | A: CR 120.4a | d909f382 |
| 20 | Charmed Pendant, "still a mana ability" (folded into case 10) | B: 605.1a revision effective 2026-08-07 | a2c501b6 |
| 21 | Two Siege rulings on MDFCs | B in part: CR update 2025-09-19 | 86f1fb6d, 60501ff1 |

**Case 15 in full.** Register entry `library-arranged-by-owner-not-instructing-player` (zones-objects.md,
then provisional, Kind: deviation), now `library-in-any-order-arranged-by-instructing-player`.
- Question: when a spell puts cards another player owns into that player's library "in any order"
  (Misinformation, Drafna's Restoration, Lodestone Bauble, Agonizing Memories), who arranges them?
- Sources: CR 401.4 "If an effect puts two or more cards in a specific position in a library at the same
  time, the owner of those cards may arrange them in any order." CR 101.1 "Whenever a card’s text directly
  contradicts these rules, the card takes precedence." Misinformation ruling (2008-10-01): "You don’t need to
  show the opponent what order you put the cards on top of their library." Sealed Fate ruling (2004-10-04):
  "The controller of this spell decides the order of the cards on the library." Oracle, Misinformation: "Put
  up to three target cards from an opponent's graveyard on top of their library in any order."
- The engine had let the owner arrange such cards (401.4), against the rulings.
- docs/agents/verification-plan.md (Nov 4) assigned the entry to item register-fixes-1, so this item's
  investigator/skeptic pipeline did not take it up separately. The user decided it on Nov 4 (register
  decisions): the player the text instructs arranges them.
- Conclusion **A, reconciled**: "in any order" in the card's text instructs its controller to arrange the cards,
  which takes precedence over 401.4's default (101.1); 401.4 still applies without that wording (Hallowed
  Burial) and to cards a replacement effect sends there.
- Change (register-fixes-1, 178596b8): `Destination::arranged_by` / `EtbInfo::arranged_by` carry the
  instructed player to `zones::order_simultaneous`; the entry was rewritten and marked decided; tests in
  `tests/rulings/r_p147_restock.rs` and `tests/cards/m_lodestone_bauble.rs`. Both rulings are CITED.
