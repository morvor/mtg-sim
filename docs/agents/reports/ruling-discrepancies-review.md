# Ruling discrepancy log: review summary for the user (Nov 9, 2026)

Written by a subagent of the `finish-line-plan` planner from docs/RULING_DISCREPANCIES.md at commit ad23a7a; every quote was checked against data/. Summarized in docs/agents/finish-line-plan.md §4. The user marks each entry's status in the log itself.


Source: `docs/RULING_DISCREPANCIES.md` (603 lines, 48 entries). Each entry is a `###`
heading with `- Kind:` / `- Status:` / one `- Ruling, <card> (<date>):` line per ruling /
`- Conflicts with:` / `- Evidence:` / `- Engine:` / `- Recorded:`. The "Not yet investigated"
section at the end is a bullet list, not entries. Every ruling quoted below was checked
word for word against `data/rulings.jsonl.gz` (all `source: wotc`); Oracle texts against
`data/oracle_cards.jsonl.gz`; CR text against `data/comprehensive-rules.txt`.

## 1. Counts

Entries by kind (48 `###` entries, 48 `- Kind:` lines, one per entry):

| Kind | Entries | Rulings quoted |
|---|---|---|
| template error | 6 | 6 |
| copying error | 2 | 2 |
| inconsistent | 5 | 5 |
| narrower than stated | 4 | 9 (Tempt: 6 cards; Lightning: 2 rulings) |
| misattached | 6 | 15 (Clive: 7; Squall: 3; Gut: 2) |
| outdated | 25 | 36 (MDFC: 2; Oculus etc.: 3; Sieges: 4; Kurkesh/Bracers: 2; Norn's/Corruption: 2; Lich: 2) |
| **total** | **48** | |

Entries by status: **48 open for review, 0 reported, 0 confirmed.** Nothing has been sent
to Wizards yet. One "open" entry (Share the Spoils) is also marked internally as pending a
coordinator decision: neither cited nor exempt.

"Not yet investigated" (not entries): 3 possible instances + 3 bookkeeping bullets (see §4).

## 2. Cross-check against docs/rulings-exemptions/*.tsv

103 exemption files, 195 lines. Lines whose reason names a policy word the completeness
test looks for (`crates/mtg-data/src/discrepancies.rs`: outdated / supersed- / overrid- /
inconsistent / misattach- / "template error" / "copying error" / the register id
`ruling-contradicting-oracle-gives-way`, matched as word beginnings): **41 lines**.

| Reason says | Exemption lines | Log entries | Why they differ |
|---|---|---|---|
| template error (all also cite the policy id) | 6 | 6 | 1:1 |
| copying error (all also cite the policy id) | 3 | 2 | Run Out of Town has two exempted sentences (P140 lines 1–2) |
| outdated / superseded | 28 | 25 | multi-ruling entries (Lich 2, MDFC 2, Oculus/Mutagen 2, Sieges 4); three outdated entries have **no** exemption and stay cited (Kurkesh/Bracers, Norn's Inquisitor/Corruption "Only transforming...", Golden-Gear Colossus) |
| inconsistent | 4 | 5 | Share the Spoils has no exemption (pending) |
| misattached | 0 | 6 | policy: never exempt (Astarion's earlier P194 exemption withdrawn) |
| overridden | 0 | — | |
| policy id cited | 9 | 8 | = the 6 template + 3 copying lines |

Housekeeping noticed while counting (not errors in the rulings): the Oculus Whelp entry
quotes Invasion of Pyrulea's ruling too, but `rulings-P225.tsv` has lines only for Oculus
Whelp and Mutagen Connoisseur; the Eggs entry covers five Eggs but `rulings-S22.tsv` exempts
only Mossfire Egg; Dance of the Dead's exemption is in `gap-zone-moves.tsv`, not a
`rulings-P*.tsv`. `cargo test -p mtg-data` was not run (read-only task).

## 3. The ten most consequential entries

Ranked by how wrong the stated outcome is and how often the card is played. Template and
copying errors first; two inconsistent and two outdated entries are included because they
state a false outcome for cards people play. Sanguine Spy, Rimescale Dragon, Citanul Druid
(template/copying errors on cards whose own text makes the error obvious or that are rarely
played), Fiery Annihilation, Lich and Gnoll War Band are the next tier.

### 1. Combat Celebrant — template error (open)
- Ruling (2017-04-18): "Combat Celebrant's ability untaps all of your creatures, not just the ones that are attacking."
- Oracle: "If this creature hasn't been exerted this turn, you may exert it as it attacks. When you do, untap all other creatures you control and after this phase, there is an additional combat phase. (An exerted creature won't untap during your next untap step.)"
- Origin per log: in the Amonkhet release notes under the card (2017-04-14); likely adapted from Lightning Runner's ruling "You untap all creatures you control, including ones that aren't attacking." (2017-02-09), whose card lacks "other". Marked "an inference" in the log.
- Engine: untaps every other creature you control; the exerted Celebrant stays tapped.
- Tell Wizards: the ruling should read "all other creatures you control"; as published it implies the Celebrant untaps itself (so it could block, or be untapped for a second Celebrant's combat), which the card's "other" forbids.

### 2. Homicide Investigator — template error (open)
- Ruling (2024-04-12): "If Homicide Investigator dies at the same time as one or more other nontoken creatures you control, Homicide Investigator's ability triggers for each of them."
- Oracle: "Whenever one or more nontoken creatures you control die, investigate. This ability triggers only once each turn." CR 603.2c: "An ability triggers only once each time its trigger event occurs. However, it can trigger repeatedly if one event contains multiple occurrences."
- Origin per log: Midnight Reaper's ruling word for word with the name swapped: "If Midnight Reaper dies at the same time as one or more other nontoken creatures you control, Midnight Reaper's ability triggers for each of them." (2018-10-05); the Investigator's other ruling that day is Midnight Reaper's companion ruling in the same order. G'raha Tia's ruling on the same template says the opposite: "If G'raha Tia and one or more other creatures and/or artifacts you control die at the same time, its last ability will trigger once (as long as it hasn't already triggered this turn)." (2025-06-06). Strong evidence.
- Engine: one Clue for creatures dying together, at most one per turn.
- Tell Wizards: a board wipe gives one Clue, not one per creature; the ruling was copied from Midnight Reaper, whose trigger is per creature and has no once-per-turn clause.

### 3. Run Out of Town — copying error (open)
- Ruling (2022-04-29): "That permanent's controller chooses whether to put it on the top or bottom of the library. If the target permanent is made up of multiple cards (due to mutate or meld, for example), that permanent's controller puts all of them on the top or all of them on the bottom. They do not need to reveal the order those cards were put into the library this way."
- Oracle: "The owner of target nonland permanent puts it on their choice of the top or bottom of their library."
- Origin per log: the Streets of New Capenna release notes (2022-04-20, ormos mirror) have the same note with "owner" both times: "That permanent's owner chooses whether to put it on the top or bottom of the library. If the target permanent is made up of multiple cards (due to mutate or meld, for example), that permanent's owner puts all of them on the top or all of them on the bottom. ..." Every other card with this instruction (Aether Gust, Desynchronize, Unlucky Drop, ... 10 cards 2019–2026) has the owner choose. Strong evidence: the published text differs from Wizards' own release notes.
- Engine: the owner chooses; the controller isn't asked.
- Tell Wizards: Gatherer's text says "controller" twice where the release notes and the card say "owner"; matters whenever a stolen permanent is targeted.

### 4. Share the Spoils — inconsistent (open; no exemption, pending a decision)
- Ruling (2021-07-23): "If you control an opponent's Share the Spoils and that player loses the game, Share the Spoils leaves the game with them and the first ability won't trigger."
- CR 603.10f: "Abilities that trigger when a player loses the game look back in time." CR 603.3a: "A triggered ability is controlled by the player who controlled its source at the time it triggered, unless it's a delayed triggered ability. ..." Card: "When this enchantment enters and whenever an opponent loses the game, exile the top card of each player's library. ..."
- Origin per log: in the AFR release notes under the card (2021-07-12), so not a copying error; contradicts Sanctum of Serra "If you planeswalk away from Sanctum of Serra because the player who owns Sanctum of Serra leaves the game, its first ability will still trigger and resolve." (2009-10-01), the God-Eternals (2019-05-03), Thieving Amalgam and Grismold (2019-08-23). Likely origin: the writer checked after the event, missing 603.10f (an inference; no supporting text found by an investigator and a skeptic).
- Engine: follows 603.10f; the ability triggers under the controller's control and exiles the top cards; nobody can play them since the enchantment is gone.
- Tell Wizards: under 603.10f the loss trigger looks back, so a borrowed Share the Spoils does trigger when its owner loses; ask whether the ruling or the look-back rule is intended.

### 5. Sakashima's Protege — template error (open)
- Ruling (2020-11-10): "If Sakashima's Protege somehow enters the battlefield at the same time as another creature, it can't become a copy of that creature. You may choose only a creature that's already on the battlefield and entered the battlefield this turn."
- Oracle: "You may have this creature enter as a copy of any permanent that entered this turn." Its own same-day ruling covers copying a planeswalker (quoted in the log).
- Origin per log: the Clone template (Progenitor Mimic 2013-04-15: "If Progenitor Mimic somehow enters the battlefield at the same time as another creature, it can't become a copy of that creature. You may choose only a creature that's already on the battlefield.", 24 more cards) with "and entered the battlefield this turn" appended; the Commander Legends release notes have the same words, so Wizards wrote it that way. Strong evidence (the card's own planeswalker ruling contradicts it).
- Engine: may copy any permanent that entered this turn and is already on the battlefield, noncreature included; not one entering with it.
- Tell Wizards: the second sentence should say "permanent"; as published it tells players they can't copy an artifact, enchantment, planeswalker or land that entered this turn.

### 6. Acolyte Hybrid — template error (open)
- Ruling (2022-10-07): "If the target artifact has indestructible, it isn't destroyed this way and you won't draw a card. If it is destroyed but put into a zone other than a graveyard, you'll draw a card."
- Oracle: "Heavy Rock Cutter — Whenever this creature attacks, destroy up to one target artifact. If an artifact is destroyed this way, its controller draws a card."
- Origin per log: the "destroy, then if destroyed this way" template with "you" kept from Noxious Gearhulk's "If the target creature has indestructible, it isn't destroyed this way and you won't gain life. If it is destroyed but put into a zone other than a graveyard, you'll gain life." (2016-09-20); the Warhammer 40,000 release notes (2022-09-19) carry it word for word, so Wizards wrote it. Rampage of the Clans' version of the template says "its controller". Good evidence.
- Engine: the destroyed artifact's controller draws (also when exiled instead by Rest in Peace); the Hybrid's controller draws nothing for an opponent's artifact.
- Tell Wizards: "you" should be "its controller"; as published it says the attacker draws when destroying an opponent's artifact, the opposite of the card's drawback.

### 7. Deranged Assistant — outdated (open)
- Ruling (2025-01-24): "Once Deranged Assistant’s ability has been activated, it can’t be reversed for any reason. If you activate it while casting a spell and discover you can’t produce enough mana to pay that spell’s costs, the spell is reversed. The spell returns to whatever zone you were casting it from. You may reverse other mana abilities you activated while casting the spell, but Deranged Assistant’s ability can’t be reversed. You’ll still have any mana the ability produced, and the milled card will still be in your graveyard."
- CR 605.1a (effective 2026-08-07 revision): "An activated ability is a mana ability if it meets all of the following criteria: it doesn’t require a target (see rule 115.6), it could add mana to a player’s mana pool when it resolves, it’s not a loyalty ability (see rule 606, “Loyalty Abilities”), and its cost and effect don’t move any card to or from a library. ..." Oracle now: "{T}, Mill a card: Add {C}. (Activate only as an instant. ...)". Millikin (2026-08-05): "Millikin’s ability is not a mana ability. Players can respond to it, and it can’t be activated in the middle of paying for a cost. A recent update to rule 605.1a has made any abilities that move cards to or from libraries no longer mana abilities."
- Origin per log: the ruling (same content in release notes since 2018-11-29) describes a mana ability activated mid-cast; the library clause came in 2026-08-07 and Wizards issued new rulings for Millikin, Chromatic Sphere and Selvala but left this one published. Strong evidence (dated rules change plus Wizards' own new rulings).
- Engine: not a mana ability; uses the stack; can't be activated while casting.
- Tell Wizards: this ruling (and the Eggs' 2008 "This is a mana ability", Charmed Pendant's 2004 "still a mana ability") should be replaced by the Millikin-style note.

### 8. Valakut Awakening // Valakut Stoneforge and 72 other MDFCs — outdated (open)
- Ruling (2020-09-25; the same text, straight or curly apostrophe, on 73 cards dated 2020-09-25 to 2024-11-08): "A modal double-faced card can't be transformed or be put onto the battlefield transformed. Ignore any instruction to transform a modal double-faced card or to put one onto the battlefield transformed."
- CR 701.27a: "To transform a permanent, turn it over so that its other face is up. Only permanents represented by double-faced tokens and double-faced cards can transform. (See rule 712, “Double-Faced Cards.”)" CR 712.14a: "If a spell or ability puts a double-faced card onto the battlefield “transformed” or “converted,” it enters the battlefield with its back face up. ..." Gwen Stacy // Ghost-Spider (2025-09-19): "A modal double-faced card can be transformed or be put onto the battlefield transformed. This is a change from previous rules. ..."
- Origin per log: the Marvel's Spider-Man CR update (effective 2025-09-19). Strong evidence. The same change outdates the Oculus Whelp/Mutagen Connoisseur/Invasion of Pyrulea "never transformed permanents", Corruption of Towashi "can't enter transformed", the 35 Sieges' copy rulings, the 24 craft cards' copy ruling, Norn's Inquisitor and The Golden-Gear Colossus (all logged as outdated).
- Engine: an MDFC transforms if its other face is a permanent face; one put onto the battlefield transformed enters back face up.
- Tell Wizards: 73 Zendikar Rising / Kaldheim / MH3 / Foundations cards still carry a ruling the 2025-09-19 rules change reversed, as do ~60 Siege/craft/MOM cards.

### 9. Araumi of the Dead Tide — inconsistent (open)
- Ruling (2020-11-10): "If the creature card has {X} in its mana cost, you can choose any value for X, but the value you choose won't apply to the tokens that are created. X will be 0 for those tokens."
- CR 107.3h: "If an effect instructs a player to pay an object’s mana cost that includes {X}, the value of X is treated as 0 unless the object is a spell on the stack. In that case, the value of X is the value chosen or determined for it as the spell was cast." Varolz, the Scar-Striped (2013-04-15), same wording for scavenge: "If the creature card you scavenge has {X} in its mana cost, X is 0." Card: "... The encore cost is equal to its mana cost. ..."
- Origin per log: in the Commander Legends release notes as published; likely a transfer of the flashback-"equal to its mana cost" rulings (Snapcaster Mage, Past in Flames), where the card is cast so X is chosen — an inference. 107.3h predates the ruling (2015). Moderate evidence (the CR text is clear; the origin is a guess).
- Engine: X is 0 for the encore cost and the tokens.
- Tell Wizards: 107.3h makes X 0 for an encore cost paid off the stack; the first clause lets players pay extra mana for nothing or, worse, believe they may choose X for a card with {X} they could otherwise encore for 0 — ask which is intended.

### 10. Sanguine Spy — copying error (open)
- Ruling (2022-04-29): "Sanguine Spy's triggered ability checks the number of mana values among cards you control both when the ability triggers and when it resolves. If you no longer have five or more mana values among those cards when the ability resolves, you may not pay 2 life."
- Oracle: "At the beginning of your end step, if there are five or more mana values among cards in your graveyard, you may pay 2 life. If you do, draw a card." CR 603.4 (intervening "if").
- Origin per log: the SNC release notes (2022-04-20) say "among cards in your graveyard"; the data's text differs from its own source. Strong evidence.
- Engine: counts mana values in the controller's graveyard at trigger and resolution.
- Tell Wizards: Gatherer's text says "cards you control" where the release notes and card say "cards in your graveyard".

Next tier, briefly: **Rimescale Dragon** (2006-07-15) "Rimescale Dragon’s last ability affects all permanents with ice counters on them, whether or not it put the ice counters on them." vs Oracle "Creatures with ice counters on them don't untap during their controllers' untap steps." — Rimefeather Owl's ruling with the name swapped (strong), but an obscure card. **Fiery Annihilation** (2024-11-08) "If the target creature is an illegal target when Fiery Annihilation begins to resolve, the target Equipment (if any) won't be exiled." vs Yosei's "If the player becomes an illegal target, the target permanents are tapped, but the player doesn't skip their untap step." (2017-11-17) — Standard-legal, but the CR is silent and only Yosei's ruling is squarely contrary; the owner already decided it Nov 7. **Lich** (2010-08-15) "... at the time the Lich leaves the battlefield ..." vs Oracle "When this enchantment is put into a graveyard from the battlefield, you lose the game." — Wizards reworded the 2011-01-01 copy but left the 2010 one. **Citanul Druid** (2004-10-04) "In multiplayer games it affects all players." — harmless in practice.

## 4. "Not yet investigated" section

Three possible instances and three bookkeeping bullets (none are entries):

1. **Spatial Contortion, Circle of Flame, Hada Freeblade** (Scryfall-authored note, 2021-02-01): says each is "banned in Pauper (not just non-legal)" while the snapshot's legalities say `not_legal`. Needs: when and why Scryfall changed the legality (scryfall.com and magic.wizards.com were outside allowed hosts); a report would go to Scryfall, not Wizards. Engine unaffected.
2. **The Dragon-Kami Reborn // Dragon-Kami's Egg** (2022-02-18): ruling says "hatchling counters"; the card has hatching counters; the NEO release notes have the same typo, so it's Wizards'. Needs: only a decision whether a typo is worth reporting (no kind fits; nothing depends on it).
3. **Geier Reach Sanitarium** (2016-07-13: "When you activate ... first each player draws a card") and **Kami of the Crescent Moon** (2005-06-01: "The additional draw triggers just after the normal draw for the turn happens."): wording puts an event at the wrong moment (activation vs resolution; trigger vs stack placement) though the intended outcome matches the CR. Needs: decide whether either is worth reporting; check whether Kami's matches a pre-2009 draw-step rule.
4. Bookkeeping: ruling-conflicts-3 compared 69 release-notes files (2013–2022) against the data and hand-read ~25 who/where/what word changes; only Run Out of Town and Sanguine Spy contradict their card. **The rest of the ~1,100 changed notes were not read one by one** — a possible source of more copying errors. ruling-conflicts-4 and user-questions-nov7 bullets record decisions already in the entries above (Fiery Annihilation, Stone Calendar, Memory's Journey).

## 5. Entries whose evidence is weak — read these most carefully

- **Combat Celebrant**: the Lightning Runner origin is explicitly "an inference"; the release notes were "not re-read here". Still, the Oracle "other" is unambiguous, so the contradiction stands even if the origin is wrong.
- **Araumi of the Dead Tide**: origin (flashback template) is a guess; the CR reading rests on 107.3h, which is clear, but Wizards published the ruling deliberately in the release notes, so it may reflect intent rather than a copying slip.
- **Share the Spoils**: no origin at all ("the usual check after the event, missing 603.10f", an inference); the ruling appears nowhere else. Wizards may defend it; pending a coordinator decision, no exemption.
- **Gnoll War Band**: origin (encore template) "an inference"; the CR (800.4i) leaves the question open, and the entry leans entirely on other rulings.
- **Fiery Annihilation**: the longest entry; only one post-2015 ruling (Yosei) is squarely contrary, the CR draws no line, and the "Possible origin" is an inference. Decided by the owner Nov 7 as inconsistent, but Wizards could plausibly stand by it.
- **Arbiter of Knollridge**: a 2007 example about range of influence; the log's reading of 801.10/801.11 is sound, but the ruling could be arguing a different model of simultaneous effects; nobody plays with range of influence.
- **Citanul Druid**: the printed Antiquities text was taken from third-party databases in search results, "no scan was checked"; the claim of a template (Malignant Growth's sentence with "players" for "opponents") is a pattern argument.
- **Stone Calendar**: the "up to" Oracle history comes from Forge card scripts and a third-party MTGJSON index, not from Wizards; "Wizards' own note on the Bloomburrow Oracle changes wasn't read". The conclusion (reduction now mandatory) follows from the current Oracle text regardless.
- **Scuttletide** (misattached): the card the ruling describes was never identified; Search the Premises is a stand-in.
- **Astarion, the Decadent** (misattached): the pairing via flavor text is "an inference", though the release notes placing it under Astarion's Thirst are solid.

Solid evidence (release notes or an identical sibling ruling, or a dated CR change): Homicide
Investigator, Run Out of Town, Sanguine Spy, Sakashima's Protege, Acolyte Hybrid, Rimescale
Dragon, Deranged Assistant / Eggs / Charmed Pendant, all MDFC/Siege/craft/transform entries,
Blitz Hellion, Aetherplasm, Green Dragon, Back from the Brink, Sarkhan the Mad, Manticore,
The Fallen, Lich, Cyclopean Tomb, Jade Statue, Krosan Adaptation, Dance of the Dead, Will
Kenrith, Clive, Squall, Gut.
