# ruling-conflicts-10: who orders one player's per-object processes (Chaotic Transformation vs CR 608.2f)

Run at effort max, with an independent investigator subagent and a separate adversarial skeptic subagent, as in ruling-conflicts-1. The user's standing rule: the CR and rulings should not contradict; apparent conflicts are investigated before the user decides.

**Raised by:** rulings-P135 (landed f1d4559). The Chaotic Transformation ruling is left OPEN; logged in docs/RULING_DISCREPANCIES.md under "Not yet investigated".

**The conflict.**
- Chaotic Transformation (Oracle): "Exile up to one target artifact, up to one target creature, up to one target enchantment, up to one target planeswalker, and/or up to one target land. For each permanent exiled this way, its controller reveals cards from the top of their library until they reveal a card that shares a card type with it, puts that card onto the battlefield, then shuffles."
- Ruling (2022-09-09): "If a player controls multiple permanents that were exiled this way, that player first chooses which target they're considering, then they reveal cards until they reveal a card that shares a type with that permanent as it last existed on the battlefield, then they put that card onto the battlefield and shuffle. They then repeat this process for the remaining targets they controlled."
- CR 608.2f (CR effective 2026-09-25): "... APNAP order is used to make the primary determination of the order of those actions. Secondarily, if the action is to be taken on both a player and an object they control or on multiple objects controlled by the same player, the player who controls the resolving spell or ability chooses the relative order of those actions."
- They agree when the caster's own permanents are exiled; for another player's permanents the ruling has that player choose the order, the CR has the caster choose. The engine asks no one (a fixed order via simultaneous::for_each_object).

**Survey.** Find when 608.2f's "Secondarily" sentence was added (CR change history in data/ or release notes; the P135 reviewer's unverified recollection: around Soulfire Eruption, 2020) and whether the ruling predates or postdates it. Collect other rulings on the order of per-object processes for one player (Soulfire Eruption, Mythos/"for each" reveal-until cards, Proteus/Polymorph-style mass effects, Transmogrify-likes, Synthetic Destiny, Wild Swing etc.), with dates, and note who chooses in each. Check whether the ruling can be read as consistent (e.g. the ruling's "that player first chooses which target they're considering" read as the order the caster lets them, or the per-object action being that player's own action under another rule).

**Outcome.**
- **Reconciled** (a reading under which both hold, or a later CR change superseding the ruling with a dated source): implement 608.2f's order choice generally in simultaneous.rs (the order chooser asked per player's set of objects), cite the ruling (or exempt the superseded part per `ruling-contradicting-oracle-gives-way`-style policy only if justified), add or cite a register entry, log in docs/RULING_DISCREPANCIES.md.
- **They contradict:** change nothing in behavior beyond what both agree on; report both readings with every source quoted verbatim with dates, with a recommendation, for the user to decide.
- Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.

## STATUS

Investigated (Nov 18) by an independent investigator; an adversarial skeptic reviews it next. **Conclusion D (unresolved): the user decides.**

**The sources don't settle it.**
- **Where they agree:** when the caster controlled the exiled permanents, and for processes the caster performs (Soulfire Eruption, Kaboom!).
- **Where they differ:** two or more of another player's permanents. CR 608.2f's words give the caster the order; the ruling gives it to that player.
- **A reconciling reading (R2, below):** 608.2f's "Secondarily" sentence doesn't reach an action that a player takes on their own library once for each of their former permanents. On that reading the CR leaves the order open, the ruling decides it, and every source found agrees. The plain reading (R1) is just as defensible, and the ruling's own words lean toward it.
- **The ruling:** left OPEN, uncited and not exempted.
- **Recommendation:** reading R2, following the ruling (see "For the user").

**Ruled out.**
- **Not outdated:** 608.2f's sentence is 21 months older than the ruling. Its only change since is "mana value" in its example.
- **Not a template or copying error:** the ruling agrees with the card's Oracle text, and the Dominaria United Release Notes have it word for word.
- **Not a ruling superseded by a dated change:** nothing in the CR has changed on this since 2020-11-20.

**Engine: no behavior change.**
- It asks no one: within one player's group the permanents go in target-slot order.
- Both readings give a player the choice, so this is a deviation from both. It is now registered as the provisional deviation `per-object-actions-in-given-order` (docs/interpretations/stack-casting-costs.md), pinned by two new tests.

**Sources** (verbatim; from `data/` unless stated).
- **Chaotic Transformation** (Dominaria United, released 2022-09-09), Oracle text: "Exile up to one target artifact, up to one target creature, up to one target enchantment, up to one target planeswalker, and/or up to one target land. For each permanent exiled this way, its controller reveals cards from the top of their library until they reveal a card that shares a card type with it, puts that card onto the battlefield, then shuffles."
- **Its rulings, both dated 2022-09-09:**
  - "The target permanents are exiled all at the same time." CITED: `tests/cards/c4_reveal_until.rs`, `chaotic_transformation_each_controller_reveals_for_each_exiled_permanent`.
  - "If a player controls multiple permanents that were exiled this way, that player first chooses which target they’re considering, then they reveal cards until they reveal a card that shares a type with that permanent as it last existed on the battlefield, then they put that card onto the battlefield and shuffle. They then repeat this process for the remaining targets they controlled." OPEN.
- **Dominaria United Release Notes** (Jess Dunks, posted 2022-08-26; ormos mirror, https://github.com/maxmakesmagic/ormos, commit bd11dee2):
  - File: `archive/en/articles/archive/feature/2022/08/dominaria-united-release-notes-2022-08-26.md`.
  - Wayback capture 2022-08-26 15:02:32: https://web.archive.org/web/20220826150232/https://magic.wizards.com/en/articles/archive/feature/dominaria-united-release-notes-2022-08-26.
  - Both bullets are word for word as the data has them; the only difference is a straight apostrophe in "they're".
- **The current CR** (effective 2026-09-25):
  - **608.2f:** "Some spells and abilities include actions taken on multiple players and/or objects. In most cases, each such action is processed simultaneously. If the action can’t be processed simultaneously, it’s instead processed considering each affected player or object individually. APNAP order is used to make the primary determination of the order of those actions. Secondarily, if the action is to be taken on both a player and an object they control or on multiple objects controlled by the same player, the player who controls the resolving spell or ability chooses the relative order of those actions."
    - Example: "Blatant Thievery says “For each opponent, gain control of target permanent that player controls.” As Blatant Thievery resolves, its controller gains control of all permanents chosen as targets simultaneously."
    - Example: "Soulfire Eruption says, in part, “Choose any number of target creatures, planeswalkers, and/or players. For each of them, exile the top card of your library, then Soulfire Eruption deals damage equal to that card’s mana value to that permanent or player.” A player casts Soulfire Eruption targeting an opponent and a creature that opponent controls. As Soulfire Eruption resolves, the player can’t exile the top card of their library multiple times at the same time, so they first choose which target they are considering, then they exile the top card of their library, and finally Soulfire Eruption deals damage to that target. They then repeat this process for the remaining target."
  - **608.2e:** "Some spells and abilities have multiple steps or actions, denoted by separate sentences or clauses, that involve multiple players. In these cases, the choices for the first action are made in APNAP order, and then the first action is processed simultaneously. Then the choices for the second action are made in APNAP order, and then that action is processed simultaneously, and so on. See rule 101.4."
  - **608.2c:** "The controller of the spell or ability follows its instructions in the order written. ..."
  - **608.2h:** "... If the effect requires information from a specific object, including the source of the ability itself, the effect uses the current information of that object if it’s in the public zone it was expected to be in; if it’s no longer in that zone, or if the effect has moved it from a public zone to a hidden zone, the effect uses the object’s last known information. ..."
  - **101.4:** "If multiple players would make choices and/or take actions at the same time, the active player (the player whose turn it is) makes any choices required, then the next player in turn order (usually the player seated to the active player’s left) makes any choices required, followed by the remaining nonactive players in turn order. Then the actions happen simultaneously. ..."
  - **101.4c:** "If a player would make more than one choice at the same time, the player makes the choices in the order specified. If no order is specified, the player chooses the order." In substance in every CR since the one effective 2009-07-11; in these words since 2019-05-03.
  - **108.4:** "A card doesn’t have a controller unless that card represents a permanent or spell; in those cases, its controller is determined by the rules for permanents or spells. ..."
  - **108.4a:** "If anything asks for the controller of a card that doesn’t have one (because it’s not a permanent or spell), use its owner instead."
  - **Rules where a player orders their own things:**
    - 405.3: "... If a player controls more than one of these objects, that player chooses their relative order on the stack."
    - 401.4: "If an effect puts two or more cards in a specific position in a library at the same time, the owner of those cards may arrange them in any order. ..."
    - 616.1: "If two or more replacement and/or prevention effects are attempting to modify the way an event affects an object or player, the affected object’s controller (or its owner if it has no controller) or the affected player chooses one to apply, ..."
  - **701.23h**, the CR's own case of a player repeating a per-object library action: "An effect may instruct a player to search a library for one or more cards more than once before instructing a player to shuffle that library. This is the same as a single instruction for that player to search that library for all those cards. The player searches that library only once."
- **Soulfire Eruption** (Commander Legends, released 2020-11-20). Oracle text: "Choose any number of target creatures, planeswalkers, and/or players. For each of them, exile the top card of your library, then Soulfire Eruption deals damage equal to that card's mana value to that permanent or player. You may play the exiled cards until the end of your next turn."
  - Ruling (2020-11-10): "As Soulfire Eruption resolves, if you have chosen multiple targets (and we really hope you have), those targets will be handled one at a time in turn order. Start with targets that are either the player whose turn it is (probably you) or permanents that player controls. You choose the relative order of targets within that group. For whichever target you select first, exile the top card of your library, then Soulfire Eruption deals damage to that target. Repeat this process for each subsequent target in the group. After that group is done, move on to targets that are either the next player in turn order or permanents they control. Again, you choose the relative order of targets in that group. Repeat the whole process for each other player."
  - The same words are in the Commander Legends Release Notes (Eli Shiffrin, posted 2020-11-06; ormos, Wayback capture 2020-11-07 11:01:36), with the card's other rulings. One of those has the caster choose each next target as the spell resolves: "Notably, you'll see which card is on top of your library before selecting from among the targets in each group."
- **Kaboom!** (Onslaught, 2002-10-07). Oracle text: "Choose any number of target players or planeswalkers. For each of them, reveal cards from the top of your library until you reveal a nonland card, Kaboom! deals damage equal to that card's mana value to that player or planeswalker, then you put the revealed cards on the bottom of your library in any order."
  - Ruling (2022-09-09): Soulfire Eruption's, word for word, but for "Kaboom!" in its first sentence and without "(and we really hope you have)". So it says "You choose the relative order of targets within that group", and also keeps "exile the top card of your library, then Soulfire Eruption deals damage to that target", which isn't what Kaboom! does.
  - It isn't in the Dominaria United Release Notes or the Dominaria United Commander decklists (ormos), so where Wizards published it wasn't found.

**608.2f's history** (archived Comprehensive Rules).
- **Source:** https://github.com/pit142857/mtg-cr, commit 8351b94f. 608.2e and 608.2f were extracted by script from every file from 2020 to 2026, each dated by the effective date it states.
- **Effective 2020-09-25 (Zendikar Rising), the last CR without it.**
  - 608.2e was as today.
  - 608.2f was "If an effect gives a player the option to pay mana, they may activate mana abilities before taking that action. ...", today's 608.2g.
  - No rule then said who orders a spell's actions on several objects; only 405.3 ordered objects, and only those put on the stack. The two CRs differ in this respect by 608.2f alone (searched for "relative order", "individually", "one at a time" and "processed simultaneously").
- **Effective 2020-11-20 (Commander Legends): today's 608.2f added.**
  - The whole rule came in at once, with both examples, the Soulfire Eruption one saying "converted mana cost".
  - It came two weeks after Soulfire Eruption's release-notes ruling (2020-11-06), which it generalizes.
  - The CRs effective 2021-02-05 and 2021-02-15 have the same text.
- **Effective 2021-04-22 (Strixhaven):** "mana value" replaced "converted mana cost" in the example. Nothing else changed.
  - It is unchanged in every archived CR since, through the one effective 2026-04-17, and in the repo's (2026-09-25).
- **When the ruling was written and published, the sentence read as today.**
  - The CR effective 2022-07-08 was in force when the release notes were written.
  - The CR effective 2022-09-08 was in force when the ruling was published.
  - So the ruling postdates the sentence by 21 months (release notes 2022-08-26; Scryfall 2022-09-09).
- **No Wizards article explaining the addition was found.**
  - ormos has no rules-change article or update bulletin for Commander Legends: its 2020/11 news folder has none.
  - The Zendikar Rising (2020-09-22) and Kaldheim (2021-02-02) rules-change articles say nothing on 608.2f.
  - Web search found nothing either.
  - magic.wizards.com and mtg.wiki are outside this session's allowed hosts (WebFetch: not resolvable).
  - A search engine's summary of the MTG Wiki's change list names only the Commander Draft format and encore for Commander Legends. That summary is unverified.

**Survey of rulings on ordering per-object processes.** The whole rulings file was searched for "relative order", "chooses the order" and "which target", for "one at a time", "repeat this process" and "considering", and for every Oracle text with "for each ..., its controller/owner ..." or a multi-object reveal-until.

| Card | Ruling date (first published) | Who performs each object's process | Who orders one player's set |
|---|---|---|---|
| Soulfire Eruption | 2020-11-10 (Commander Legends Release Notes, 2020-11-06) | the caster | the caster: "You choose the relative order of targets within that group." |
| Kaboom! | 2022-09-09 (Soulfire Eruption's text) | the caster | the caster: the same words |
| Chaotic Transformation | 2022-09-09 (Dominaria United Release Notes, 2022-08-26) | each permanent's controller | that player: "that player first chooses which target they’re considering" |
| Divergent Transformations | 2016-11-08 (Commander 2016 Release Notes, Eli Shiffrin, 2016-10-28) | each creature's controller | not stated: "If one player's creatures are exiled this way, that player repeats this process twice." Both processes are the same, so the order can't matter. Two players: "start with the player whose turn it is". |
| Indomitable Creativity | 2017-02-09 (Aether Revolt Release Notes, 2017-01-06) | each permanent's controller | not stated (the same process for each): "All of the exiled cards are put onto the battlefield at the same time." |
| Chaos Mutation | 2022-10-07 | each creature's controller | can't arise: "The phrase "target creatures controlled by different players" means that no two targets may be controlled by the same player." |
| Mystic Confluence, Eldrazi Confluence, Rootcast Apprenticeship | 2021-03-19, 2024-06-07, 2024-07-26 | a mode chosen more than once | the caster: Mystic Confluence's "you choose the relative order to return the target creatures"; the other two "as you cast the spell" (CR 700.2d's mode order, not 608.2f), even when opponents act: "you choose the relative order in which the target opponents sacrifice nontoken artifacts" |
| Aetherspouts | 2014-07-18 | each attacker's owner | no order: owners choose top or bottom in APNAP order and the cards move together; the owner arranges them (401.4) |
| Killing Wave, Fade Away | 2012-05-01, 2004-10-04 | each creature's controller | no order: choices in APNAP order, then the actions happen at the same time |

The other cards the brief named say nothing on this.
- **One process:** Polymorph, Transmogrify, Proteus Staff, Shape Anew and Reweave have one target. Synthetic Destiny and Mass Polymorph have one process, the caster's ("The creature cards will enter the battlefield simultaneously.").
- **No process:** Wild Swing picks one target at random.
- **No other card** found has another player perform different processes for two or more of their own objects. Divergent Transformations and Indomitable Creativity perform the same process for each, and Chaos Mutation forbids two of one player's. So Chaotic Transformation is the only card where the two readings play differently.

**When the order matters** (one player's two or more permanents exiled by Chaotic Transformation).
- **A known library top.** The first process reveals from the library as it is; the later ones from a shuffled library. So the order matters when the top is known or arranged: after a scry, with Courser of Kruphix ("Play with the top card of your library revealed."), or with an artifact creature on top that either of an exiled artifact's and an exiled creature's processes would take.
- **A permanent that changes how a later card can enter.** Grafdigger's Cage ("Creature cards in graveyards and libraries can't enter the battlefield."), found first for an exiled artifact, stops the creature card found for an exiled creature. Containment Priest ("If a nontoken creature would enter and it wasn't cast, exile it instead."), found first for an exiled creature, exiles an artifact creature card found later.
- **Different players' processes:** both readings order them the same way (APNAP).

**Readings tried.**
- **R1, the words (they contradict).** "For each permanent exiled this way" iterates over the exiled permanents, and 608.2f processes an action "considering each affected player or object individually". The ruling uses the same frame. Its condition, "a player controls multiple permanents that were exiled this way", is 608.2f's "multiple objects controlled by the same player", with control read from the permanents as they last existed (608.2h). Its choice, "which target they’re considering", is the example's words. So 608.2f's chooser, "the player who controls the resolving spell or ability", is the caster, and the ruling's "that player" contradicts it.
- **R2, "taken on" (they agree).**
  - **The reading.** 608.2f's words "actions taken on" and "each affected player or object" are about what an action is done to, and both its examples fit that: control of the permanents is gained; damage is dealt to the target. Chaotic Transformation's second sentence acts on each controller's library and on the card put onto the battlefield. The exiled permanent is only the reference ("shares a card type with it"), so the action is taken on the same player once per permanent.
  - **Why the CR is then silent.** The "Secondarily" sentence names two cases: "both a player and an object they control", and "multiple objects controlled by the same player". The same player several times is neither, so the CR leaves the order of one player's repetitions open, and the ruling decides it.
  - **Fit with the rest of the CR.** A player orders their own choices (101.4c), their own objects put on the stack (405.3) and their cards put into their library (401.4), and the affected player orders replacement effects (616.1). The CR also merges a player's repeated per-object searches into one (701.23h).
  - **Fit with the rulings.** Soulfire Eruption's and Kaboom!'s processes deal damage to the targets, so the targets are what the action is taken on, and the caster orders, as their rulings and the rule say. Divergent Transformations and Indomitable Creativity agree too.
  - **Against it:**
    - The ruling frames the order in 608.2f's own words, as though the permanents were the objects considered.
    - 608.2f's first sentence is usually read to cover any "for each [object]" instruction, whatever it acts on.
    - 608.2c has the spell's controller follow its instructions.
  - **For it:**
    - The ruling was written knowing 608.2f: it borrows the example's words but names another chooser. Its compiler, Jess Dunks, also wrote that release's Comprehensive Rules changes article (2022-09-02; ormos).
    - Kaboom!'s ruling, published the same day, kept the caster for a process the caster performs.
    - The ruling hasn't been changed in the four years since.
- **R3, delegation (doesn't reconcile).** The caster orders, and lets that player choose. No rule provides for handing the choice over, and the ruling has "that player" make it.
- **R4, the caster's own permanents only (doesn't reconcile).** "If a player controls ..." covers any player; release notes say "you" for the caster. They agree in that case only.
- **R5, no controller in exile (doesn't reconcile).** Once exiled, the cards have no controller (108.4), so "objects controlled by the same player" never holds. But 108.4a substitutes the owner, and the card's "its controller" is read from the permanent as it last existed (608.2h), so the clause would still name the caster. This is part of why the clause's reach is unclear, not a reconciliation.

**Engine** (checked; no change).
- **Compilation.** Chaotic Transformation compiles (`mtg-tools unsupported --text ... --ast`) to an exile of the five targets, then `ForEach { sel: Matching(Before(Var(0)), Permanent), effect: AsPlayer { who: ControllerOf(each), effect: Seq[RevealUntil (shares a card type with each), Move to the battlefield, Shuffle] } }`.
- **The order.** In `simultaneous::for_each_object`, the frames are sorted stably by the APNAP position of each object's controller (`Frames::new`, and `deciding_player` for `AsPlayer` bodies). So one player's permanents keep the order given, the target slots' order (artifact, creature, enchantment, planeswalker, land). A reveal-until isn't performed for all the frames at once, so the frames go one after another, and no decision is asked.
- **Kaboom!** (`oracle/patterns/card_flow_5.rs`, `for_each_target_reveal_damage`) is a `ForEach` over its target slot. Within a player's group its targets go in the order the caster chose them as the spell was cast, not in an order chosen as it resolves.
- **Soulfire Eruption's hand-written ability** (`src/cards/soulfire_eruption.rs`) asks its controller to "Choose the next target" within each group, as 608.2f's example has it.
- **Pinning tests.** `tests/rulings/per_object_order.rs` (new):
  - `chaotic_transformation_takes_one_players_permanents_in_target_order`. P1's Sol Ring and Grizzly Bears are exiled, with P1's library topped by Ornithopter, Hill Giant and Mind Stone. P1 gets Ornithopter and Hill Giant, since the artifact's process comes first; the other order would give Mind Stone. No order or choice decision is asked.
  - `kaboom_takes_one_players_targets_in_the_order_they_were_chosen`. The targets are chosen Jace Beleren first, then P1: Jace takes 2 and survives with 1 loyalty, and P1 takes 4. Nothing is asked as it resolves.
  - Both cite `interp!("per-object-actions-in-given-order")` and will fail once the engine asks, which is the point: whoever implements the user's decision updates them.
  - Checked against a mutant: with the order within each player's group reversed in `Frames::new` (a temporary `items.reverse()`, since reverted), both tests fail. Hill Giant isn't found, and Jace dies.
- **The earlier tests stay as they are.** P135's `chaotic_transformation_handles_each_exiled_permanent_of_a_player_in_turn` is order-independent.

**Docs.**
- **docs/RULING_DISCREPANCIES.md:**
  - A new inconsistent entry, open for review, replaces the "Not yet investigated" bullet.
  - A new "Not yet investigated" bullet for Kaboom!'s ruling: a template error by the log's definition, whose caster's choice isn't implemented. It is left for the change that follows the user's decision, since that change asks the caster there under either reading.
- **Register:** the new entry `per-object-actions-in-given-order` (deviation, provisional). docs/INTERPRETATIONS.md is regenerated.

**Noticed, not investigated.**
- **Two players' processes at once?** Divergent Transformations' rulings (2016-11-08) put two players' processes one after the other, starting with the active player ("The two creatures that are put onto the battlefield are put onto the battlefield sequentially."). 608.2f, added in 2020, processes an action simultaneously "in most cases", and individually only when it "can’t be processed simultaneously". Whether two players' reveal-until processes can be performed at once wasn't investigated; if they can, the 2016 rulings would be outdated. The engine follows the 2016 rulings (cited) and does the same for Chaotic Transformation's different players. Chaotic Transformation's ruling doesn't address different players.
- **Where Kaboom!'s 2022-09-09 ruling first appeared** wasn't found.

**For the user.** When Chaotic Transformation exiles two or more permanents of a player other than its caster, who chooses the order of that player's reveal-until processes?
- **(1) The caster, by CR 608.2f's words.**
  - Set aside the ruling's "that player first chooses which target they’re considering" as inconsistent, as Arbiter of Knollridge's and Headliner Scarlett's were set aside.
  - Cite the rest of it: "then they reveal cards until they reveal a card that shares a type with that permanent as it last existed on the battlefield, then they put that card onto the battlefield and shuffle. They then repeat this process ...". The engine already does this.
  - The engine would ask the controller of the resolving spell or ability.
- **(2) That player, as the ruling says, under reading R2.** 608.2f's "Secondarily" sentence covers an action taken on a player and their objects, or on several objects of one player. It doesn't cover an action a player takes on their own library once for each of their former permanents. Cite the ruling. The engine would ask the player the instruction has perform the process ("its controller reveals ...") to order their own repetitions, and the caster otherwise.

Recommendation: **(2).**
- It is the one reading under which the CR and every ruling found agree, which is the user's standing rule.
- It matches the CR's other allocations of a player's own orderings (101.4c, 405.3, 401.4, 616.1).
- The alternative sets aside a card-specific ruling written knowing 608.2f (the example's words, with another chooser named) by the author of that release's rules-changes article, and unchanged since 2022.
- It is a judgment call: a judge reading 608.2f's "multiple objects controlled by the same player" as the exiled permanents would give the choice to the caster.

Either way, the follow-up change is the same in outline:
- **The question asked:** `simultaneous.rs` asks, as the instruction is performed, for each next object within a player's group (Soulfire Eruption's ruling: the caster sees the new top card before choosing the next target). Only the chooser differs between (1) and (2). The default is the order given (`default-order-as-listed`).
- **Kaboom!** gets the caster's choice under both readings, and its template-error sentence is logged.
- **The register:** `per-object-actions-in-given-order` is rewritten as decided, and its two tests updated.
- **Merging:** another session is mutation-testing `simultaneous.rs`, so the change should merge with it.
