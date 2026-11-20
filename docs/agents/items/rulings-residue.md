# rulings-residue: the open rulings left behind in batches already worked

WHY: docs/agents/finish-line-plan.md, phase A, item A3. 114 batches that earlier rounds worked
still hold 1,599 OPEN ruling texts between them (Nov 9; `python3 scripts/rulings_batches.py list`
shows each batch's OPEN count). Earlier rounds (the keyword sweeps, the rounds before "drive OPEN
to zero") left the harder or then-unsupported part; the 23 batches landed since Nov 5 left one
OPEN text in total. These leftovers are where the interaction bugs are (copies, counters,
reflexive triggers, type changes, forced attacks and blocks), so they are worked as themed
items, not batch by batch.

Each item is a rulings batch in every respect (the "Rulings batch" brief in
docs/agents/worker.md: cited / exempt / uncovered, the discrepancy policies, the report counts),
except that its open rulings come from several batches. Get them with
`python3 scripts/rulings_batches.py show PNNN --open` for each batch named; tests go in
`crates/mtg-engine/tests/rulings/r_residue_<topic>.rs`. Do the interaction rulings first. A
ruling left UNCOVERED for a named reason (the script has no such status; it stays OPEN) also
goes into docs/rulings-uncovered.tsv (`Card<TAB>substring<TAB>reason`), the list criterion C1
reconciles against the OPEN count. These items are the plan's stopping signal for the rulings
oracle (finish-line-plan §1.1), so the reviewer appends the item's row to docs/agents/yield.tsv
and reports how many of its rulings were interactions rather than restatements.

## Items (slug: batches; approximate OPEN count on Nov 9)

| Slug | Batches | OPEN | Topic |
|---|---|---|---|
| `rulings-residue-copy-creature` | P020 P021 P023 | 116 | creatures that copy, Clone exceptions, copies entering with choices |
| `rulings-residue-enchant-equip` | P207 P208 P209 P210 P211 P212 P213 | ~130 | Auras and Equipment keyword leftovers: attach legality, enchant restrictions, equip timing |
| `rulings-residue-counters` | P035 P119 P120 P121 | ~90 | counters: placement, removal, proliferate, doubling |
| `rulings-residue-reflexive-refund` | P129 P130 | ~65 | reflexive triggers and refunds |
| `rulings-residue-type-change` | P187 P188 | 58 | type-changing effects and layers |
| `rulings-residue-hate-attack-block` | P079 P080 P082 | 63 | attack and block requirements, restrictions, punishers |
| `rulings-residue-copy-instant` | P028 P030 P032 | 47 | copying spells on the stack |
| `rulings-residue-keywords` | P217 P214 P205 P226 P222 P215 | ~115 | other keyword leftovers |
| `rulings-residue-burn-freeze` | P009 P062 | ~60 | damage spells, tap-and-freeze |
| `rulings-residue-shapechange-typal` | P160 P161 P184 | ~75 | changelings, shapeshifters, typal payoffs |
| `rulings-residue-misc-1`, `-misc-2` | every other batch with OPEN texts, split in two by id | ~770 | the small residues (1–20 a batch) |

Before starting an item, re-run `list`: another item may have cited some of its rulings
(rulings shared by several cards are covered by a test on any of them). Take only batches no
other running item has taken; say which in the report.

Also folded into `rulings-residue-misc-2`, if the user wants the discrepancy log complete before
reporting (finish-line-plan decision D2): finish the release-notes comparison that
`ruling-conflicts-3` began (the ormos mirror, https://github.com/maxmakesmagic/ormos, 2013–2022;
if the host is unreachable from the build machine, say so and leave it): it hand-read about 25 of
the ~1,100 passages whose words differ from the data; read the rest for who/where/what word
changes, and log any new copying error in docs/RULING_DISCREPANCIES.md.

DONE per item when every ruling of its batches is CITED, EXEMPT with a reason or UNCOVERED with a
named reason in docs/rulings-uncovered.tsv. Report counts: cited, exempt, uncovered, bugs fixed
(each with its rule), interactions against restatements; and the row in docs/agents/yield.tsv.

## The misc split (Nov 20, by `rulings-residue-misc-1`)

`list` on Nov 20 showed 860 OPEN texts in the batches no themed item covers, leaving out the
batches of the themed items already landed and those of the two running then
(`rulings-residue-burn-freeze`: P009 P062; `rulings-residue-shapechange-typal`: P160 P161 P184).
Split in two by batch id (OPEN counts on Nov 20 in brackets):

- **`rulings-residue-misc-1`** (429): P002 P005 P011 P012 P015 P016 P017 P037 P039 P044 P045 P051
  P052 P053 P056 P057 P058 P064 P065 P066 P073 P074 P076 P085 P092 P094 P098 P099 P101 P104 P107
  P108 P109 P112 P113 P115 P116.
- **`rulings-residue-misc-2`** (431): P117 P122 P124 P125 P126 P133 P134 P136 P137 P141 P142 P143
  P145 P146 P148 P150 P151 P156 P162 P163 P165 P166 P168 P171 P172 P176 P177 P180 P185 P186 P189
  P190 P193 P194 P201 P202 P203 P204 P206 P216 P218 P219 P221 P223 P224 P225.

## Open questions the items found (for the coordinator)

- **"If it's the first time ... this turn" checked again on resolution** (found by
  `rulings-residue-counters`, Nov 19; no reading chosen yet, and no test pins either). CR 603.4
  checks an intervening "if" again as the ability resolves; neither the CR nor any ruling says
  whether "it's the first time [something happened to] that permanent this turn" can stop being
  true once the event that triggered the ability was the first one. The engine reads the same
  wording two ways:
  - Stalwart Successor (`oracle/patterns/history_conditions_4.rs`, `first_time_counters`) makes
    it part of the trigger (`TriggerCond::FirstTimeEachTurn`), checked only as the ability
    triggers: the triggering event stays "the first time" as it resolves. Vengevine's "if it's
    the second creature spell you cast this turn" behaves the same way (a third creature spell
    cast in response doesn't stop it from returning).
  - Botanical Brawler (`oracle/patterns/history_conditions.rs`, `nth_time_counters`) and Captain
    America, Living Legend (`history_conditions_4.rs`, `first_time_referent`) count this turn's
    events again as the ability resolves (`== 1`). With Cursed Wombat, a +1/+1 counter put on a
    Grizzly Bears triggers Brawler and the Bears' Wombat-granted ability; if the Wombat ability
    resolves first (a second counter on the Bears), Brawler's ability then does nothing, where
    Stalwart Successor's in the same spot still puts its counter. Captain America's ability
    doesn't untap a creature that was untapped and tapped again while it waited on the stack.

  Settling it needs a register entry (a reasonable judge could go either way: provisional) and
  the three patterns made to agree, with tests of the in-between event for each wording.
  **Taken up by item ruling-conflicts-12** (see its STATUS): all the patterns now read the
  condition as about the trigger event (`first-time-is-about-the-trigger-event`, provisional, then
  **decided** by the user Nov 20, work item user-decisions-nov20).
- **Several permanents entering at once apply their "as [this] enters" replacement effects in the
  order the moves are given** (per player, APNAP; `Game::move_objects` in actions.rs), not in an
  order their controller chooses. Thief of Blood's ruling lets the controller apply any of the
  entering Thieves' effects first; for identical Thieves every choice gives the same game (their
  timestamp order is still the player's choice, CR 613.7m), so its test doesn't pin the order,
  but for different permanents whose "as enters" effects interact the choice would matter.
  **Taken up by item ruling-conflicts-12** (see its STATUS): the controller now chooses where
  the order can matter (decided `simultaneous-entries-ordered-by-controller`).
- **A creature type added to a noncreature, once a later effect makes it a creature** (found by
  `rulings-residue-type-change`'s review, Nov 19; pre-existing behavior, no test pins it).
  Shadow Puppeteers' trigger resolving for a creature that Swift Reconfiguration made a
  noncreature Vehicle makes it only red: its ruling says "Since it's not a creature, it won't
  gain the Dragon subtype, and it won't have base power and toughness 4/4." Crewed later that
  turn, it's a 4/4 (CR 208.3a says so for the base power and toughness) and, in the engine, a
  Dragon too: layer 4 drops the subtypes that don't match the card types only after applying
  every type-changing effect (`layers.rs`, CR 205.3d), as Terror of Towashi's ruling wants for
  a type given while it *was* a creature. Applying 205.3d as each effect applies in timestamp
  order (the crew effect is later) would leave it no Dragon. `tests/rulings/
  r_residue_type_change_layers.rs` (`shadow_puppeteers_on_a_creature_that_stopped_being_a_creature`)
  asserts the 4/4 but not the Dragon. Settling it needs a register entry (provisional).
  **Decided** by the user, Nov 19 (work item user-decisions-nov19c), for the alternative: a
  subtype a resolving effect adds that doesn't correspond to the object's types as the effect
  begins is never gained by it, even once the object is crewed (register
  `subtype-gained-only-if-it-corresponds-as-the-effect-begins`, `src/subtype_gain.rs`; also
  for what an effect says a permanent it puts onto the battlefield "is", CR 611.2e, such as
  Rise from the Grave's Zombie on a returned noncreature God); the crewed Vehicle is red and
  no Dragon. Its base 4/4 still applies once it's a creature (CR
  208.3a; the brief asked for the 1/2, which would deviate from 208.3a: register
  `base-pt-set-on-a-noncreature-applies-once-it-is-a-creature`, provisional, then **decided** by the
  user Nov 20 as recommended, work item user-decisions-nov20; the user called the Nov 19 subtype
  decision one of their lower-confidence ones: docs/agents/finish-line-plan.md §5, "Revisit later").
- **Creature types after a card-type-removing effect, once a later effect makes the object a
  creature again** (found by user-decisions-nov19c, Nov 19; pre-existing behavior, no test pins
  it). CR 205.1a: "If an object's card type is removed, the subtypes correlated with that card
  type will remain if they are also the subtypes of a card type the object currently has;
  otherwise, they are also removed for the entire time the object's card type is removed."
  Swift Reconfiguration on a Storm Crow (a Bird) that Shadow Puppeteers made a Dragon: crewed
  later that turn, the engine gives it no creature types at all (`Modification::SetTypes` and
  `RemoveTypes` drop the uncorrelated subtypes as they apply, in timestamp order, so the later
  crew effect adds the creature type without them). Reading "the entire time the object's card
  type is removed" as ending when the object has the card type again would give it back Bird
  and Dragon. Settling it needs a register entry (provisional).
  **Decided** by the user, Nov 20 (work item user-decisions-nov20), for the engine's reading: "I agree
  with this - the equipment ruling on this seems to reinforce this idea" (register
  `subtypes-removed-with-a-card-type-stay-removed-once-it-returns`; pinned by
  `tests/rulings/subtypes_after_card_type_removed.rs`: Grizzly Bears under Swift Reconfiguration,
  crewed, is an artifact creature Vehicle with no creature types).
- **Rulings older than the Dominaria planeswalker errata that say "a player" where the card now
  says "target player or planeswalker"** (found by `rulings-residue-burn-freeze`'s review,
  Nov 20; no exemption made: they are cited for player targets, as the earlier P009 rounds
  cited the same class, Punish the Enemy's in `tests/rulings/r_p009_targets.rs` and
  Blightning's in `tests/rulings/r_p009_replacement.rs`). The Dominaria Oracle Changes (Eli Shiffrin, 2018-04-13;
  ormos mirror `archive/en/articles/archive/news/2018/04/dominaria-oracle-changes-2018-04-13.md`,
  Wayback capture 2018-04-13 16:31:32; the Oracle update that went with the CR effective
  2018-04-27): "Abilities that damage or prevent damage to "target player" have been changed to
  "target player or planeswalker."", listing each card below. Soul of Shandalar (2014-07-18):
  "To activate either ability, you must target a player, but you can choose to not target any
  creatures."; Chandra, Pyromaster (2014-07-18): "To activate the first ability, you must
  target a player but you can choose to not target any creatures."; Searing Blaze
  (2010-03-01): "You must choose two targets as you cast Searing Blaze: a player and a creature
  that player controls. ..."; Sarkhan the Mad (2010-06-15): "You choose a single target as you
  activate the third ability. All your Dragons deal damage to that player."; Vengeful Archon
  (2010-08-15): "The only target of Vengeful Archon’s ability is the player it may deal damage
  to. ..."; Flames of the Blood Hand (2005-02-01): "The damage is unpreventable, and the
  targeted player’s life gain that turn is replaced with nothing." and "The replacement effect
  applies only to the target player, ..."; cited earlier, Punish the Enemy (2013-04-15): "You
  must be able to target a player and a creature in order to cast Punish the Enemy." and
  Blightning (2008-10-01): "The targeted player will discard two cards even if ...". The engine
  follows the Oracle text (a planeswalker may be the target). Under the brief's "outdated"
  policy their "a player" parts would be exempt in part, each with an entry in
  docs/RULING_DISCREPANCIES.md; a class decision like `pre-battle-rulings-give-way-to-the-cr`
  would settle them all (the P008/P009 "burn planeswalker" batches hold more of them).
  **Decided** by the user, Nov 20 (work item user-decisions-nov20): treat them like the battle rulings
  (register `pre-dominaria-player-target-rulings-give-way`, docs/interpretations/sources.md): 24
  rulings on 22 cards exempt in part as outdated (`docs/rulings-exemptions/pre-dominaria-player-rulings.tsv`),
  one grouped entry in docs/RULING_DISCREPANCIES.md, a planeswalker-target test per card
  (`tests/rulings/pre_dominaria_player_target_rulings.rs`).
- **Swift Reckoning cast from a graveyard: does the card count itself?** (found by
  `rulings-residue-misc-1`, Nov 20; a conflict case, the ruling left uncited, not exempt).
  Swift Reckoning: "Spell mastery — If there are two or more instant and/or sorcery cards in
  your graveyard, you may cast this spell as though it had flash." Its ruling (2018-12-07):
  "If an effect allows you to cast Swift Reckoning from your graveyard, you need two other
  instant and/or sorcery cards in your graveyard to cast it as though it had flash." That
  follows from 601.3 as it read in the CRs effective 2016-07-22 to 2019-08-23 ("... If that
  player is no longer allowed to cast that spell after completing its proposal, the casting
  of the spell is illegal and the game returns to the moment before the casting of that
  spell was proposed ..."), which the CR effective 2019-10-04 replaced with today's "A
  player can begin to cast a spell only if a rule or effect allows that player to cast it
  and no rule or effect prohibits that player from casting it." Graveyard Shift's ruling
  (2022-04-29), under today's rules: "If an effect allows you to cast Graveyard Shift from
  your graveyard, you may cast it any time you could cast an instant if you have five or more
  mana values among cards in your graveyard immediately before you begin to cast it, even if
  moving it to the stack would reduce that number to four." The case for the ruling still
  holding: 601.2e ("The game checks to see if the proposed spell can legally be cast.") and
  the cards' different wordings (601.3b's "allows a player to cast a spell ... as though it
  had flash" against 601.3d's "would have flash only if certain conditions are met"). The
  engine had followed the ruling while its list of legal actions offered the cast it then
  refused; it now counts the card itself as the casting begins, so the list and the casting
  agree (provisional `flash-permission-conditions-judged-as-casting-begins`). To settle:
  either the ruling is outdated (an outdated entry in docs/RULING_DISCREPANCIES.md, where its
  note under "Not yet investigated" has the quotes, and an exemption in
  `docs/rulings-exemptions/rulings-P011.tsv`; the register entry decided), or it stands (the
  register entry reversed: the effects judged again with the card on the stack, and the list
  of legal actions made to judge them so too).
