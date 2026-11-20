# ruling-research-2: "that player" / "that opponent" of a trigger after the objects change control

Research item (brief: `docs/agents/items/ruling-research-2.md`), effort max, with an independent
investigator and a separate adversarial skeptic (Nov 19–20). Register entry:
`that-opponent-fixed-as-it-triggers` (`docs/interpretations/triggers-durations.md`), which stays
**provisional**. Every quote below is verbatim; CR quotes are from `data/comprehensive-rules.txt`
(effective 2026-09-25), rulings from `data/rulings.jsonl.gz` (Scryfall bulk data; dates as
published), and each was checked again for this report (grep / `zcat data/rulings.jsonl.gz`).

## 1. The question

Norn's Decree: "Whenever one or more creatures an opponent controls deal combat damage to you,
that opponent gets a poison counter." Coveted Jewel: "Whenever one or more creatures an opponent
controls attack you and aren't blocked, that player draws three cards and gains control of this
artifact. Untap it." Between the trigger event and the ability's resolution the creatures change
control (the Jewel's controller gains control of an unblocked attacker in response) or leave the
battlefield. Which player is "that opponent" / "that player"?

- **F**: fixed as the ability triggers: the opponent the trigger condition named.
- **R**: the creatures' controller as the ability resolves (CR 608.2h's current information), or
  as they last existed if they left the battlefield.

Only a control change separates the two readings: a creature that merely leaves the battlefield
last existed under that opponent's control, so both give that opponent.

## 2. Outcome

- **No source decides the case.** No CR rule or example, no ruling in the data, no release note
  that could be reached and no web source addresses "that player" / "that opponent" of a
  "[creatures] an opponent controls" trigger after the creatures change control.
- **F is the better-supported reading**: the investigator put it at about 70%, the adversarial
  skeptic at about 60–65% (verdict **UPHELD**: "F (fixed as the ability triggers) is still the
  better-supported reading. `that-opponent-fixed-as-it-triggers` should stay PROVISIONAL.").
- **The rulings split by wording, and that is the distinguishing principle the engine now
  applies everywhere:**
  - A player-referring **"that player" / "that opponent" / "they"** whose antecedent is the
    trigger condition's "an opponent" (or "a player") is the player the condition named as the
    ability triggered. Nothing checks a trigger condition again as the ability resolves (CR
    603.4 re-checks only an intervening "if"), and the closest rulings on a "that player" whose
    description later fits someone else keep the first player (the monarch draw ruling,
    Emberwilde Captain).
  - An **object-based "its controller" / "that creature's controller" / "that source's
    controller" / "that spell's controller"** is the object's controller as the ability
    resolves, or as the object last existed (CR 608.2h), as every ruling on such a wording
    says (Tainted Aether, Ankh of Mishra, Celestial Mantle, Relic Putrescence, Archfiend of
    Spite), even when the trigger condition also names the controller (Archfiend of Spite's
    "a source an opponent controls").
- **The skeptic's main finding was on the engine side**: the same construction was read both
  ways (F for Norn's Decree, Coveted Jewel and ward; R for Michiko Konda and the "[a permanent] an
  opponent controls enters ..." triggers). The engine now follows the principle above for every
  card (section 7).

## 3. Why F (the weighing)

For F:
- the plain grammar of a demonstrative back-reference ("that opponent") whose antecedent sits in
  the trigger condition, which describes the event: "creatures an opponent **controls** deal
  combat damage" is tied to the event's time, while object-based wordings put "controller" in the
  effect, tied to resolution (the skeptic's linguistic point, matching the Lightmine Field
  rulings: referents the trigger picked out are fixed, fresh descriptions in the effect are read
  on resolution);
- CR 608.2k's back-reference principle (Wall of Tears), for objects;
- the monarch draw and Emberwilde Captain rulings ("that player" stays the first player after
  the condition's description fits someone else), though they concern a designation and a
  player-subject event, not control of objects (the skeptic's E4/E8: weaker than the
  investigator first claimed);
- the text: after the Decree's controller steals the creatures, R makes "that opponent" a player
  who isn't an opponent; for the Jewel, R makes its own controller draw three cards. No rule
  supports patching that.

For R:
- CR 608.2h's default (current information of a specific object) with the glossary's "An
  object's 'controller' is the player who currently controls it.";
- the uniform object-based rulings (section 4.4.2);
- WotC's present-tense paraphrases of "that player" as the controller (Bronze Guardian, Hall of
  Storm Giants, Skulking Killer);
- CR 726.2's own rendering of the same construction as "the controller of those creatures"
  (weakened by the skeptic's drafting explanation: its condition names two players, so "that
  player" would have been ambiguous; the release notes paraphrase it "the first player");
- the CR's 2017 rewrite of the defending-player rule from "at the time it became an attacking
  creature" to current information with a last-known fallback (section 4.2);
- the user's Mutiny rationale, read generally (section 6).

What would move it: any ruling or judge answer on one of these wordings after a control change
(ward with a stolen spell, Blood Seeker or Suture Priest with a flash steal, Michiko Konda, the
initiative trigger), the Phyrexia: All Will Be One Commander release notes for Norn's Decree
(blocked, section 9), or the user saying whether the Mutiny rationale was meant generally.

## 4. Sources

### 4.1 Comprehensive Rules (data/comprehensive-rules.txt, effective 2026-09-25)

Triggering and what the condition fixes:
- 603.2 "Whenever a game event or game state matches a triggered ability’s trigger event, that ability automatically triggers. The ability doesn’t do anything at this point." (F, structural: the condition is matched once, at the event.)
- 603.2c "An ability triggers only once each time its trigger event occurs. However, it can trigger repeatedly if one event contains multiple occurrences." (The per-opponent instances of `one-or-more-an-opponent-controls-per-opponent`.)
- 603.3a "A triggered ability is controlled by the player who controlled its source at the time it triggered, unless it’s a delayed triggered ability. To determine the controller of a delayed triggered ability, see rules 603.7d–f." (A player fixed as it triggers; an analogy only.)
- 603.4 "A triggered ability may read “When/Whenever/At [trigger event], if [condition], [effect].” When the trigger event occurs, the ability checks whether the stated condition is true. The ability triggers only if it is; otherwise it does nothing. If the ability triggers, it checks the stated condition again as it resolves. If the condition isn’t true at that time, the ability is removed from the stack and does nothing. Note that this mirrors the check for legal targets. This rule is referred to as the “intervening ‘if’ clause” rule. (The word “if” has only its normal English meaning anywhere else in the text of a card; this rule only applies to an “if” that immediately follows a trigger condition.)" (Only an intervening "if" is checked again; neither card has one. The skeptic, E3: R doesn't re-evaluate the condition either, it reads "that opponent" as information about "those creatures", so 603.4 doesn't decide between them.)
- 805.4d "An ability that triggers at the beginning of a step or phase may trigger multiple times if it triggers at the beginning of “each player’s” or “each opponent’s” step or phase. These abilities trigger once for each appropriate player if the ability’s trigger condition, effect, or intervening “if” clause refers to “that player,” “that opponent,” or similar." (A "that player" ties each instance to one player picked out as it triggers; a different trigger type.)
- 109.5 "The words “you” and “your” on an object refer to the object’s controller, its would-be controller (if a player is attempting to play, cast, or activate it), or its owner (if it has no controller). For a static ability, this is the current controller of the object it’s on. For an activated ability, this is the player who activated the ability. For a triggered ability, this is the controller of the object when the ability triggered, unless it’s a delayed triggered ability. To determine the controller of a delayed triggered ability, see rules 603.7d–f." ("You" is fixed as it triggers; the CR spells this out for "you" and says nothing for "that player".)

Information read during resolution:
- 608.2b "If the spell or ability specifies targets, it checks whether the targets are still legal. A target that’s no longer in the zone it was in when it was targeted is illegal. Other changes to the game state may cause a target to no longer be legal; for example, its characteristics may have changed or an effect may have changed the text of the spell. ..." (Re-checks targets: the basis of the Mutiny decision, section 6.)
- 608.2h "If an effect requires information from the game (such as the number of creatures on the battlefield), the answer is determined only once, when the effect is applied. If the effect requires information from a specific object, including the source of the ability itself, the effect uses the current information of that object if it’s in the public zone it was expected to be in; if it’s no longer in that zone, or if the effect has moved it from a public zone to a hidden zone, the effect uses the object’s last known information. See rule 113.7a. If an ability states that an object does something, it’s the object as it exists—or as it most recently existed—that does it, not the ability." (R's main rule, if "that opponent" is information from the creatures; the rule for every object-based wording.)
- 608.2i "Some effects look back in time and require information about previous game states and actions rather than considering the current game state. If such an effect requires information from the game about an object or group of objects, and that effect is not taking any actions on those objects, they don’t need to be currently in the zone they were in at the time of that previous game state or action, nor do they need to currently meet the criteria described in the action, as long as they did so at the specified time. This is an exception to 608.2h." (Weakly F, if identifying whose creatures dealt the damage counts as looking back.)
- 608.2k "If an ability’s effect refers to a specific untargeted object that has been previously referred to by that ability’s cost or trigger condition, it still affects that object even if the object has changed characteristics." Example: "Wall of Tears says “Whenever this creature blocks a creature, return that creature to its owner’s hand at end of combat.” If Wall of Tears blocks a creature, then that creature ceases to be a creature before the triggered ability resolves, the permanent will still be returned to its owner’s hand." (F's best structural analogy; it covers objects and characteristics, and control isn't a characteristic, 109.3.)
- 700.7 "If an ability uses a phrase such as “this [something]” to identify an object, where [something] is a characteristic or other quality, it is referring to that particular object, even if it isn’t the appropriate quality at the time."
- 800.4i "If an effect requires information about a specific player, the effect uses the current information about that player if they are still in the game; otherwise, the effect uses the last known information about that player before they left the game. ..." (A player named by an effect is a specific referent; very weak F.)

Combat:
- 506.4 "A permanent is removed from combat if it leaves the battlefield, if its controller or protector changes, ..." (A stolen attacker stops being attacking and unblocked; neither card re-checks its condition, so the ability still resolves.)
- 508.3d "An ability that reads “Whenever [a player] attacks, . . .” triggers if one or more creatures that player controls are declared as attackers." and 508.3e "An ability that reads “Whenever [a player] attacks [another player], . . .” triggers if one or more creatures the first player controls are declared as attackers attacking the second player. ..." ("[a player] attacks" is defined as "creatures [that player] controls" attacking, the Jewel's subject; the Jewel's ruling speaks of an "opponent [who] attacks you".)
- 508.5 (now) "If an ability of an attacking creature refers to a defending player, or a spell or ability refers to both an attacking creature and a defending player, then unless otherwise specified, the defending player it’s referring to is the player that creature is attacking, the controller of the planeswalker that creature is attacking, or the protector of the battle that creature is attacking. If that creature is no longer attacking, the defending player it’s referring to is the player that creature was attacking before it was removed from combat, ..." (History in 4.2: mixed.)

Inherent triggers with the same structure:
- 725.2 "There are two inherent triggered abilities associated with being the monarch. These triggered abilities have no source and are controlled by the player who was the monarch at the time the abilities triggered. This is an exception to rule 113.8. The full texts of these abilities are “At the beginning of the monarch’s end step, that player draws a card” and “Whenever a creature deals combat damage to the monarch, its controller becomes the monarch.”"
- 726.2 "There are three inherent triggered abilities associated with having the initiative. These triggered abilities have no source and are controlled by the player who had the initiative at the time the abilities triggered. This is an exception to rule 113.8. The full text of these abilities are “At the beginning of the upkeep of the player who has the initiative, that player ventures into Undercity,” “Whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the controller of those creatures takes the initiative,” and “Whenever a player takes the initiative, that player ventures into Undercity.” ..." The Oracle text of the designation card Undercity // The Initiative (https://scryfall.com/card/tclb/20/undercity-the-initiative) words the same ability "Whenever one or more creatures a player controls deal combat damage to you, that player takes the initiative." (R's strongest CR text; the skeptic, §4e: the CR's condition names two players, "creatures a player controls" and "the player who has the initiative", so "that player" would be ambiguous there and "the controller of those creatures" disambiguates, as the release notes' "the first player" does the other way.)
- 702.21a "Ward is a triggered ability. Ward [cost] means “Whenever this permanent becomes the target of a spell or ability an opponent controls, counter that spell or ability unless that player pays [cost].”" (The same "[object] an opponent controls ..., that player" structure on a keyword; no rule or ruling covers a stolen spell.)

No CR example involves a control change between trigger and resolution (the investigator grepped every example for "that player", "its controller", "that creature's controller", "those creatures", "controller of th" and "whenever ... opponent").

### 4.2 CR history (archived texts, https://github.com/pit142857/mtg-cr)

- The defending-player rule was rewritten from "fixed at the event" to "current, with a
  last-known fallback" between the CRs of 2017-07-07 and 2017-08-25 (found by the skeptic;
  re-checked for this report in the archive's files):
  - CR 2017-07-07, 508.4 (https://github.com/pit142857/mtg-cr/blob/main/Comprehensive%20Rules/CR-2017-07-07.txt): "508.4. If an ability of an attacking creature refers to a defending player, or a spell or ability refers to both an attacking creature and a defending player, then unless otherwise specified, the defending player it’s referring to is the player that creature was attacking at the time it became an attacking creature that combat, or the controller of the planeswalker that creature was attacking at the time it became an attacking creature that combat."
  - CR 2017-08-25, 508.5 (https://github.com/pit142857/mtg-cr/blob/main/Comprehensive%20Rules/CR-2017-08-25.txt): "508.5. If an ability of an attacking creature refers to a defending player, or a spell or ability refers to both an attacking creature and a defending player, then unless otherwise specified, the defending player it’s referring to is the player that creature is attacking, or the controller of the planeswalker that creature is attacking. If that creature is no longer attacking, the defending player it’s referring to is the player that creature was attacking before it was removed from combat or the controller of the planeswalker that creature was attacking before it was removed from combat."
  - The old text is 508.5 in CR 2016-08-26 and 508.4 from 2016-09-30 through 2017-07-07; the new one is in every later version checked. An R-direction precedent for an object-derived player; but since a control change removes the creature from combat (506.4), the fallback keeps the original defending player, which fits F (the skeptic: "better scored as mixed").
- 608.2h's principle exists since CR 2004-10-01 (413.2f: "... The effect uses the current information of a specific permanent if that permanent is still in play, or of a specific card in the stated zone; otherwise, the effect uses the last known information the object had before leaving that zone."). 608.2k first appears in CR 2007-07-13 as 413.2h (absent in 2007-02-01). 805.4d's "that player," "that opponent," or similar is absent in CR 2019-10-04 and present in 2020-04-17. 725.2 (as 716.2 since 2016-08-26) and 726.2 (as 721.2 since 2022-06-10) haven't changed in wording.
- No CR version checked (2004-10-01 through 2026-04-17, about twenty versions) has a rule or example on what "that player" in a triggered ability refers to after an object changes control.

### 4.3 Release notes (magic.wizards.com, read through the Wayback captures mirrored at https://github.com/maxmakesmagic/ormos)

- Commander 2018 Release Notes (2018-07-27; https://magic.wizards.com/en/articles/archive/feature/commander-2018-edition-release-notes-2018-07-27, mirror `archive/en/articles/archive/feature/2018/07/commander-2018-edition-release-notes-2018-07-27.md`), Coveted Jewel: identical to its four rulings, e.g. "If more than one opponent attacks you at the same time, Coveted Jewel's last ability triggers for each of them. You choose which player ends up with Coveted Jewel, but each of them draws three cards and has a chance to activate its mana ability."
- Commander 2019 Release Notes (2019-08-09; https://magic.wizards.com/en/articles/archive/feature/commander-2019-edition-release-notes-2019-08-09, mirror `.../2019/08/commander-2019-edition-release-notes-2019-08-09.md`), Archfiend of Spite: "The player who loses life or permanents is the player who controls the source as Archfiend of Spite's triggered ability resolves. In some rare cases, this may not be the same player who controlled the source as it dealt damage. If the source is no longer in the zone it was in as it dealt damage, the source's last known information is used to determine who controlled it."
- Conspiracy: Take the Crown Release Notes (2016-08-22; https://magic.wizards.com/en/articles/archive/feature/conspiracy-take-crown-release-notes-2016-08-22, mirror `.../2016/08/conspiracy-take-crown-release-notes-2016-08-22.md`): "If the triggered ability that causes the monarch to draw a card goes on the stack, and a different player becomes the monarch before that ability resolves, the first player will still draw the card." The Commander Legends Release Notes (2020-11-06; https://magic.wizards.com/en/articles/archive/feature/commander-legends-release-notes-2020-11-06, mirror `.../2020/11/commander-legends-release-notes-2020-11-06.md`) repeat it (without the comma) and give Emberwilde Captain's rulings (4.4.1).
- Commander Legends: Battle for Baldur's Gate Release Notes (2022-06-01; https://magic.wizards.com/en/articles/archive/feature/commander-legends-battle-baldurs-gate-release-notes-2022-06-01, mirror `.../2022/06/commander-legends-battle-baldurs-gate-release-notes-2022-06-01.md`): "Second, whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the first player takes the initiative."
- The mirror ends in October 2022, so it has no Phyrexia: All Will Be One Commander (ONC) notes for Norn's Decree; magic.wizards.com and media.wizards.com were blocked (section 9).

### 4.4 Rulings (data/rulings.jsonl.gz)

#### 4.4.1 "That player" / "that opponent" of a player the trigger picked out: kept

- Monarch draw (Canal Courier, https://scryfall.com/card/cn2/28/canal-courier, 2016-08-23; 52 times on 50 cards in all, through 2025-11-17, e.g. Garland, Royal Kidnapper 2025-10-02): "If the triggered ability that causes the monarch to draw a card goes on the stack, and a different player becomes the monarch before that ability resolves, the first player will still draw the card." (The closest ruling on a "that player" whose description later fits someone else; the description is a designation, not control of objects.)
- Emberwilde Captain (https://scryfall.com/card/cmr/175/emberwilde-captain, 2020-11-10; "Whenever an opponent attacks you while you're the monarch, this creature deals damage to that player equal to the number of cards in their hand."): "If another player becomes the monarch after Emberwilde Captain's last ability triggers but before it resolves, the ability will resolve as normal. The player who attacked you will be dealt damage, even if that player was the one who became the monarch." and "Emberwilde Captain's last ability triggers once for each player who attacks you, no matter how many creatures they attack with beyond the first."
- Struggle for Project Purity (https://scryfall.com/card/pip/39/struggle-for-project-purity, 2024-03-08; "Enclave — Whenever a player attacks you with one or more creatures, that player gets twice that many rad counters."): "The number of rad counters the attacking player gets when the “Enclave” triggered ability resolves is based on the number of creatures they declared as attackers. Removing creatures from combat won’t reduce the number of rad counters they get. ..." (The count, not the player; a player-subject trigger: the skeptic's E4.)
- Tahngarth, First Mate (https://scryfall.com/card/c19/50/tahngarth-first-mate, 2019-08-23): "If an opponent attacks a player or planeswalker but each creature attacking that player or planeswalker is no longer attacking that player or planeswalker by the time Tahngarth's ability resolves (most likely because they've left the battlefield), that player or planeswalker can't be chosen for Tahngarth to attack. ..." ("That opponent" stays the attacking opponent; a description in the effect is read on resolution.)
- Garland, Royal Kidnapper (https://scryfall.com/card/fic/442/garland-royal-kidnapper, 2025-10-02; "Whenever an opponent becomes the monarch, gain control of target creature that player controls for as long as they're the monarch."): "While Garland's second ability is on the stack, if that opponent stops being the monarch before it resolves, you won't gain control of the target creature at all."
- Archaeomancer's Map (https://scryfall.com/card/soc/136/archaeomancers-map, 2021-04-16; "Whenever a land an opponent controls enters, if that player controls more lands than you, ..."): "The last ability triggers only if that opponent controls more lands than you after the land enters the battlefield. The ability will check again as it tries to resolve. If that opponent doesn't still control more lands than you at that time, the ability won't resolve." (The same player, "still", in a re-checked intervening "if"; no control change of the land.)
- Lightmine Field (https://scryfall.com/card/roe/32/lightmine-field, 2010-06-15): "Which creatures are dealt damage is based on which creatures were declared as attackers during the declare attackers step." and "How much damage each of those creatures is dealt is based how many creatures are attacking at the time the ability resolves." (Back-references to what the trigger picked out are fixed; fresh descriptions in the effect are read on resolution. Objects.)
- Coveted Jewel (https://scryfall.com/card/msc/196/coveted-jewel, 2018-07-13): "If more than one opponent attacks you at the same time, Coveted Jewel's last ability triggers for each of them. You choose which player ends up with Coveted Jewel, but each of them draws three cards and has a chance to activate its mana ability." (Weakly F: "that player" is the opponent the instance was created for.)
- Tunnel Ignus (https://scryfall.com/card/som/105/tunnel-ignus, 2011-01-01): "Whenever a land an opponent controls enters, this ability triggers only if another land had already entered the battlefield under that opponent's control this turn. It doesn't matter if that other land is still on the battlefield, is still under that player's control, or is still a land." (About the other land; no control change of the triggering one.)
- "You" fixed as the ability was activated or put on the stack (109.5 applications; weak analogies): Goblin Festival (https://scryfall.com/card/uds/83/goblin-festival, 2004-10-04) "If this card changes controllers before the ability resolves, you still pick an opponent of the player who controlled it when it was activated." and Purgatory (https://scryfall.com/card/mir/275/purgatory, 2008-08-01) "If this card changes controllers between putting the upkeep triggered ability on the stack and resolving it, the creature is put onto the battlefield under the control of the player who controlled this card when the ability was put on the stack."
- A description of a player in the effect is read again (not an anaphor): Wild Dogs (https://scryfall.com/card/dmr/183/wild-dogs, 2022-12-08) "... The player who had the most life as the ability triggered doesn't have to be the same player as the one who has the most life as the ability resolves."
- Norn's Decree (https://scryfall.com/card/onc/9/norns-decree) has one ruling, irrelevant here (2023-02-04): "A player is poisoned if that player has one or more poison counters."

#### 4.4.2 Object-based wording: the object's controller as the ability resolves, or as it last existed

- Ankh of Mishra (https://scryfall.com/card/vma/263/ankh-of-mishra, 2004-10-04; "... deals 2 damage to that land's controller."): "It determines the land's controller at the time the ability resolves. If the land leaves the battlefield before the ability resolves, the land's last controller before it left is used."
- Tainted Aether (https://scryfall.com/card/7ed/167/tainted-aether, 2004-10-04; "Whenever a creature enters, its controller sacrifices a creature or land of their choice."): "If the creature changes controllers before this ability resolves, its new controller has to sacrifice something."
- Plague Fiend (https://scryfall.com/card/pcy/73/plague-fiend, 2004-10-04): "The creature’s controller gets the option to pay when the triggered ability resolves."
- Celestial Mantle (https://scryfall.com/card/jmp/96/celestial-mantle, 2009-10-01; "Whenever enchanted creature deals combat damage to a player, double its controller's life total."): "At the time the ability triggers, determine which creature Celestial Mantle is enchanting. As the ability resolves, determine who currently controls that creature (or, if it’s no longer on the battlefield, determine who controlled it when it left). That’s the player whose life total is doubled. It doesn’t matter who controls Celestial Mantle, who controlled the creature at the time the ability triggered, or what creature Celestial Mantle enchants at the time the ability resolves."
- Relic Putrescence (https://scryfall.com/card/som/77/relic-putrescence, 2011-01-01; "Whenever enchanted artifact becomes tapped, its controller gets a poison counter."): "When the enchanted artifact becomes tapped, Relic Putrescence’s ability triggers. The player who gets the poison counter is the player who, at the time the ability resolves, controls the artifact that became tapped. If that artifact is no longer on the battlefield, its last existence on the battlefield is checked to determine its controller. It doesn’t matter whether Relic Putrescence is still on the battlefield as the ability resolves, what artifact it’s enchanting at that time, who controlled the artifact at the time it became tapped, or who tapped it."
- Archfiend of Spite (https://scryfall.com/card/c19/14/archfiend-of-spite, 2019-08-23; "Whenever a source an opponent controls deals damage to this creature, that source's controller loses that much life unless they sacrifice that many permanents of their choice."): "The player who loses life or permanents is the player who controls the source as Archfiend of Spite's triggered ability resolves. In some rare cases, this may not be the same player who controlled the source as it dealt damage. If the source is no longer in the zone it was in as it dealt damage, the source's last known information is used to determine who controlled it." (The closest structural analog: the condition names the controller, "a source an opponent controls", and the object-based effect is still read on resolution. Michiko Konda, Truth Seeker has the same condition with "that player sacrifices a permanent of their choice" and no ruling.)
- Gloom Sower (https://scryfall.com/card/m21/100/gloom-sower, 2020-06-23): "If the creature that blocked Gloom Sower leaves the battlefield before Gloom Sower's triggered ability resolves, the player who controlled it before it left is the player who loses 2 life."
- Call to Heel (https://scryfall.com/card/bbd/114/call-to-heel, 2008-10-01, a spell): "The player who draws a card is the player who controls the creature when Call to Heel resolves. This may not be the player whose hand the creature is put into."
- Contested War Zone (https://scryfall.com/card/mbs/144/contested-war-zone; "Whenever a creature deals combat damage to you, that creature's controller gains control of this land."), the Jewel's closest functional twin, 2011-06-01: "In some multiplayer formats, creatures controlled by different players may deal combat damage to you simultaneously. If this happens, you control each triggered ability and can put them on the stack in any order. The last ability to resolve will determine who ends up controlling Contested War Zone." (No ruling on a control change.)

#### 4.4.3 Paraphrases that treat the wordings as interchangeable (no control change involved)

- Bronze Guardian (https://scryfall.com/card/c21/13/bronze-guardian, 2021-04-16), for ward's "that player": "Once a ward ability has triggered, it doesn't matter if the artifact loses ward (most likely because Bronze Guardian leaves the battlefield in response) before the ability resolves. The controller of the spell or ability will still have to pay {2} or the spell or ability will be countered."
- Hall of Storm Giants (https://scryfall.com/card/afr/257/hall-of-storm-giants, 2021-07-23): "... Each of those instances will trigger separately and ask the opponent who controls the spell or ability to pay {3}. ..."
- Elesh Norn // The Argent Etchings (https://scryfall.com/card/mom/12/elesh-norn-the-argent-etchings, 2023-04-14), for "that source's controller": "... Each of those abilities will cause that opponent to lose 2 life unless they pay {1}."
- Kazuul, Tyrant of the Cliffs (https://scryfall.com/card/mkc/157/kazuul-tyrant-of-the-cliffs, 2010-03-01), for "that creature's controller": "... The attacking player chooses whether to pay {3} or let you have an Ogre token each time one of those abilities resolves."
- Emberwilde Captain (2020-11-10), for the monarch's "its controller": "If combat damage dealt to the monarch causes that player to lose the game, the triggered ability that causes the controller of the attacking creature to become the monarch doesn't resolve. ..."
- The initiative, ruling on 23 CLB cards (e.g. Explore the Underdark, https://scryfall.com/card/clb/232/explore-the-underdark, 2022-06-10): "... Second, whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the first player takes the initiative. ..."; Nelly Borca, Impulsive Accuser (https://scryfall.com/card/mkc/4/nelly-borca-impulsive-accuser) words Norn's Decree's condition with the object-based "you and the controller of those creatures each draw a card" (no ruling on control).
- These show the phrasings name the same player in ordinary cases and settle nothing about the control-change case.

#### 4.4.4 Target restrictions (the Mutiny contrast)

- Mutiny (https://scryfall.com/card/rix/106/mutiny, 2018-01-19): "If either or both targets are illegal when Mutiny resolves, no creature will deal or be dealt damage."
- Dwarven Sea Clan (https://scryfall.com/card/hml/71/dwarven-sea-clan, 2008-08-01): "If control of the targeted creature changes between the activating and resolution of the ability (ie, while it's on the stack), then the player who must control an Island as the ability resolves is the creature's new controller."
- Seasinger (https://scryfall.com/card/me1/49/seasinger, 2007-09-16): "... Whether that player controls an Island is checked both when Seasinger’s ability is activated and when it resolves. ..."
- Simic Guildmage (https://scryfall.com/card/rvr/223/simic-guildmage, 2006-05-01): "For the first ability, if the two target creatures aren’t controlled by the same player when the ability resolves, the ability does nothing. The player who controls the two creatures doesn’t have to be the same player who controlled them when the ability was activated, ..."
- Bioshift (https://scryfall.com/card/gtc/214/bioshift, 2013-01-24): "If one of the two creatures is an illegal target when Bioshift tries to resolve, or if the creatures are controlled by different players at that time, no counters will move."
- Skulking Killer (https://scryfall.com/card/vow/130/skulking-killer, 2021-11-19; "target creature an opponent controls gets -2/-2 until end of turn if that opponent controls no other creatures"): "... If the targeted creature's controller controls any other creatures as the ability resolves, the ability won't do anything. ..."
- All support re-checking a controller-based target restriction on resolution (the user's Mutiny decision); for triggers they are distinguishable (section 6).

### 4.5 The web

Searches (WebSearch, standard and restricted to cranial-insertion.com, boardgames.stackexchange.com,
mtgsalvation.com, blogs.magicjudges.org, judgeacademy.com, apps.magicjudges.org) for Coveted
Jewel with the attacker stolen in response, Norn's Decree after a control change, ward with
Commandeer or Aethersnatch, the initiative and monarch triggers after a control change, Michiko
Konda, Blood Seeker and Suture Priest, Contested War Zone, Edric, Mutiny in multiplayer and
general "that player locked in" questions found **nothing on a control change between trigger and
resolution**. They returned card pages, the official rulings quoted above, Cranial Insertion
articles 2760/2763 (Coveted Jewel in team games; the search summary's quotation is unverified, the
pages couldn't be opened), a Magic Judges forum topic 49911 ("Coveted Jewel and creatures not
declared attacking", 2019), Cardmarket's "#CMAskTheJudge Episode 66: A Real Jewel" (Coveted Jewel
and removal before blocks), tappedout.net threads on the Jewel dying before blockers, and an MTG
Salvation thread on Coveted Jewel and Saheeli's Directive. Several summaries offered the search
model's own guesses, both ways; those aren't sources. Only github.com and raw.githubusercontent.com
could be fetched (the CR archive and the ormos mirror).

## 5. The skeptic's corrections to the investigator (all adopted here)

- E1 (engine scope): the engine read the construction both ways (section 7); fixed in this item.
- E2: the 508.5 history hides a 2017 move toward current information (section 4.2): mixed, not F.
- E3: 603.4 is neutral (R reads "that opponent" as information about the creatures, not as the condition re-evaluated).
- E4: Struggle for Project Purity's ruling addresses the count, and it, Emberwilde Captain and Tahngarth are player-subject triggers: weaker analogies.
- E5: the monarch draw ruling appears 52 times (15 on 2016-08-23, 37 from 2020-11-10 through 2025-11-17), on 50 cards (two have both versions).
- E6: the initiative's Two-Headed Giant text is a 2022-06-10 ruling on 23 CLB cards, not on Undercity // The Initiative (which has no rulings).
- E7: Meathook Massacre II and Charismatic Conqueror say "they", not "that player" (the same anaphor).
- E8: the monarch draw ruling concerns a designation, which no object carries; it refutes "re-evaluate the trigger's description" but not "read the creatures' controller".
- New sources: the 2017 508.4/508.5 texts, Lightmine Field, Wild Dogs, Goblin Festival and Purgatory, Archfiend of the Dross ("Whenever a creature an opponent controls dies, its controller loses 2 life." beside Massacre Wurm's "that player loses 2 life": WotC uses both wordings for one condition), 800.4i.

## 6. The Mutiny decision (`target-player-reference-rechecked`, Nov 5)

The user decided that Mutiny's "another target creature that player controls" is checked against
the first target's controller as the spell resolves. Its stated reason: "608.2b checks each target
against the current game state, and the restriction's referent is part of that state; nothing in
the rules locks the referent in when targets are chosen."

- **Operative basis: distinguishable.** Mutiny's antecedent sits in a target restriction, which
  608.2b checks again as the spell resolves; the target rulings above agree (Dwarven Sea Clan,
  Seasinger, Simic Guildmage, Bioshift, Skulking Killer). Norn's Decree's and Coveted Jewel's
  antecedent sits in a trigger condition, which nothing checks again (603.4 checks only an
  intervening "if"). The investigator's unifying principle (an inference): an anaphor is re-bound
  exactly when the clause holding its antecedent is re-evaluated; target restrictions are (608.2b),
  an intervening "if" is (603.4), a plain trigger condition never is. The skeptic tested it
  against every ruling found and it fits all of them.
- **Stated reason: points to R if taken generally.** "Nothing in the rules locks the referent in"
  is equally true when the ability triggers. **Question for the user**: was the Mutiny rationale
  meant generally ("controller-derived player references are read currently"), which would
  overturn F, or as a rule about re-checking targets, which doesn't reach triggers?

## 7. Engine: which wording reads which way

Before this item the engine read "that player" of these triggers as the event's player (fixed) for
Norn's Decree, Coveted Jewel, ward and the "a spell or ability an opponent controls, that player"
cards, but as the object's controller on resolution for Michiko Konda and the "[a permanent] an
opponent controls enters / is dealt damage / becomes tapped ..." triggers; and it fixed the
object-based "that spell's controller" / "its controller" of the becomes-the-target triggers and
the monarch's "its controller". Now:

| Wording | Reading | Cards (examples) | Code |
|---|---|---|---|
| "[creatures] an opponent controls deal combat damage to you / attack you and aren't blocked, that opponent / that player" | F (unchanged) | Norn's Decree, Coveted Jewel | `BatchPer::OtherController` / `ObjectController` record the group's player; `PlayerRef::TriggerPlayer` |
| "[a permanent] an opponent controls enters / a land enters under an opponent's control / a creature enchanted player controls enters / a source an opponent controls deals damage to you ..., that player / they" | **F (was R)** | Blood Seeker, Suture Priest, Gideon the Oathless, Polluted Bonds, Nightshade Harvester, Sire of Stagnation, Invader Parasite, Archaeomancer's Map, Confounding Conundrum, Tunnel Ignus, Charismatic Conqueror, Michiko Konda, Truth Seeker, Trespasser's Curse (added by the review) (and any "blocks / is dealt damage / becomes tapped / untapped / phases in" trigger worded so) | `oracle/patterns/that_player_named_controller.rs`; `PlayerRef::ControllerAsTriggered` reads `EventInfo::object_controller` / `other_controller`, recorded by `Game::trigger_matches_ctx` |
| "... a spell or ability an opponent controls, that player ..." | F (unchanged) | ward (all ward cards), Ashenmoor Liege, Black Bolt, Scalelord Reckoner, Thunderbreak Regent, Leyline of Combustion, Parnesse, Karmic Justice, Psychic Purge, Mana Web | `PlayerRef::TriggerPlayer` (the event's player) |
| "[a permanent] an opponent controls dies / is put into a graveyard ..., that player" | the same either way (LKI) | Massacre Wurm, Assault Intercessor, Shriek, Snarlfang Vermin, Pain Distributor, Sardian Avenger, Prince of Thralls, Meathook Massacre II | `ControllerOf(TriggerLki)` |
| "its controller", "that land's / creature's / source's controller" | R (unchanged) | Tainted Aether, Ankh of Mishra, Celestial Mantle, Relic Putrescence, Archfiend of Spite, Elesh Norn, Flameblade Angel, Kazuul, Tetsuko Umezawa, Contested War Zone, Nelly Borca ("the controller of those creatures") | `PlayerRef::ControllerOf` |
| becomes-the-target triggers: "that spell's controller", "that spell or ability's controller", "counter that spell or ability unless its controller pays / discards" | **R (was F)** | Frost Titan, Bonecrusher Giant, Lava Runner, Retromancer, Forsaken Wastes, Fractured Loyalty, Reality Smasher, Diffusion Sliver, Unsettled Mariner, Amulet of Safekeeping, Reidane // Valkmira | `oracle/patterns/sweep_long_tail_targeted.rs`: `ControllerOf(Sel::TriggerSpell)` |
| CR 725.2 "its controller becomes the monarch" | **R (was F)** | the monarch | `monarch_initiative.rs` |
| CR 726.2 "the controller of those creatures takes the initiative" / The Initiative's "that player takes the initiative" | F (unchanged; open question) | the initiative | `monarch_initiative.rs` |

The initiative is the one place the principle doesn't settle: the CR's text is object-based ("the
controller of those creatures"), while the designation card says "that player takes the
initiative" and the release notes and rulings "the first player takes the initiative"; R would
also need a rule for those creatures ending up under different controllers. The engine keeps the
player the trigger was created for (the card's and the release notes' wording) and asks the user.

Tests: `crates/mtg-engine/tests/rulings/r_that_opponent_fixed_as_it_triggers.rs` (control passing
to another opponent in a free-for-all game, only some of the creatures changing control, a
creature leaving the battlefield, with and without a control change first, for both cards; Blood
Seeker; Tunnel Ignus's "enters under an opponent's control" with its intervening "if"; Michiko
Konda; ward with the spell stolen in response; Frost Titan's and Bonecrusher Giant's object-based
wordings; the monarch), plus the two-player tests in `r_residue_reflexive_refund_3.rs`. Each fails
under the other reading (checked by switching the engine's reading, Nov 20): disabling the fix of
`that_player_named_controller.rs` fails Blood Seeker, Tunnel Ignus and Michiko Konda; reading Norn's
Decree's and Coveted Jewel's player on resolution fails their eight tests; fixing the
becomes-the-target object wordings fails Frost Titan and Bonecrusher Giant; fixing the monarch's
"its controller" fails the monarch test; reading ward's "that player" on resolution fails the ward
test.

No card in any of these families has a ruling that settles its own "that player" or "its
controller" after a control change (the skeptic listed all 123 cards with a "[object] an opponent
controls / a player controls ..., ... that player / that opponent" trigger, with their rulings;
this item re-checked the cards whose reading changed).

## 8. Questions for the user

1. F or R for "that player" / "that opponent" of a trigger condition's "an opponent" (the entry,
   provisional). The research favors F, without a decisive source.
2. The Mutiny rationale (section 6): general, or about target re-checks only?
3. The initiative (section 7): keep "that player" fixed (the designation card's wording), or read
   CR 726.2's "the controller of those creatures" on resolution (and then decide what happens if
   those creatures end up under different controllers)?
4. Confirm the two consistency changes to R that follow from the principle: the becomes-the-target
   triggers' "that spell's controller" / "its controller" (a stolen spell's new controller pays,
   or is dealt the damage) and the monarch's "its controller" (a creature stolen after dealing
   combat damage makes its new controller the monarch).

## 9. Blocked hosts

- **magic.wizards.com** (the Phyrexia: All Will Be One and ONC release notes, which would cover
  Norn's Decree) and **media.wizards.com** (the release-notes PDF): WebFetch refused,
  "getaddrinfo ENOTFOUND ..., outside the session's allowed network domains".
- **apps.magicjudges.org** (forum topic 49911) and **www.cardmarket.com** ("#CMAskTheJudge
  Episode 66"): the same refusal.
- **reddit.com** (r/mtgrules): refused by the search tool itself ("The following domains are not
  accessible to our user agent: ['reddit.com']").
- Not tried after those refusals (the tool asked to stop trying new hosts; the environment's
  network is limited to package registries and MCP servers): cranial-insertion.com,
  blogs.magicjudges.org, judgeacademy.com, boardgames.stackexchange.com, mtgsalvation.com,
  tappedout.net.
- Nothing was routed around the blocks; a Hugging Face copy of r/mtgrules Q&A that a search
  surfaced was deliberately not used.

## 10. Review (Nov 20)

- **Scope of the change, measured**: every card compiled with the item's three changes reverted
  and with them (a temporary dump of every triggered ability's AST): exactly the 23 abilities
  listed in section 7 changed, each only by the intended replacement (`ControllerOf` of the
  trigger object or source to `ControllerAsTriggered` for the 12 "that player" / "they"
  abilities, the event's player to `ControllerOf(Sel::TriggerSpell)` for the 11 becomes-the-target
  abilities), and no card's unsupported text changed. In all 12 the antecedent of "that player"
  / "they" is the condition's controller phrase; the central pattern catches nothing else ("a
  player" as an event's subject and "target player" never reach it: their "that player" isn't
  the object's controller).
- **Rulings**: every ruling of the 23 cards (and of Norn's Decree, Coveted Jewel, the ward and
  "that player" becomes-the-target cards, the monarch and the initiative) re-read: none
  addresses a control change before the ability resolves, and none contradicts the reading
  given (Archaeomancer's Map's "If that opponent doesn't still control more lands than you"
  and Charismatic Conqueror's "the opponent can't choose to tap it" fit F). A search of all
  rulings for a control change while an ability waits to resolve found only rulings about the
  source or the ability's controller (Rainbow Vale, evoke, Goblin Festival, the "you won't gain
  control of ... at all" rulings), about targets, and Tainted Aether's and Garland's (both
  quoted above).
- **Added**: Trespasser's Curse ("Whenever a creature enchanted player controls enters, that
  player loses 1 life and you gain 1 life") read "that player" on resolution, the one remaining
  "[object] [player] controls [event], that player" trigger; "enchanted player controls" now
  names the player as "an opponent controls" does (`names_other_controller`), with a test. The
  initiative's kept reading (fixed, open) is now pinned by a test.
- **Tests**: each new test fails under the other reading (deliberate engine breaks: the three
  changes reverted, then ward, Norn's Decree and Coveted Jewel read on resolution, then the
  enchanted-player case and the initiative switched). Two tests cited 608.2h for a creature
  leaving the battlefield although "that player" there is the recorded player (with the
  creature stolen first, the opposite of its last known controller): the citation was dropped.
