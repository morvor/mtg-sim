# Ruling discrepancies

Wizards rulings (Scryfall's `data/rulings.jsonl.gz`) that the engine doesn't follow in full,
or tests on another card than the one the data attaches them to. This is where a human
checks each finding and reports it to Wizards of the Coast; nothing here changes how the
engine plays. The user's decision of Nov 5: "It looks like some rulings are applied as a
template, and are therefore occasionally incorrect. ... We should still keep a detailed
account of all of the instances though, so they can be checked by a human, and reported to
Wizards of the Coast if erroneous."

**Kinds.**
- **template error**: the ruling's words contradict the card's own Oracle text, and it is
  (most likely) a copy of a similar card's ruling. Policy `ruling-contradicting-oracle-gives-way`
  (`docs/interpretations/sources.md`, decided by the user Nov 5): the Oracle text wins
  (CR 108.1); the parts of the ruling that agree with it stay cited.
- **copying error** (a kind decided by the user Nov 5): the ruling's words contradict the
  card's own Oracle text, and the same ruling as Wizards first published it (in the set's
  release notes) has words that agree with the card: the data's text differs from its own
  source, as though miscopied. It falls under the same policy,
  `ruling-contradicting-oracle-gives-way`.
- **misattached**: the ruling describes another card. Policy (decided by the user Nov 5):
  it is tested on the card it describes and cited under the card it is attached to (the
  `ruling!` macro needs that), with a comment naming the card it describes; the attached
  card is set up in the test when it can take part. Being misattached is never a reason to
  exempt a ruling.
- **inconsistent**: the ruling contradicts other rulings on the same question.
- **outdated**: the ruling is older than a dated rules change or Oracle change, but is
  still published.
- **narrower than stated**: the ruling is right in the situation it was written for, but its
  words state a rule that is wrong elsewhere.

**Statuses.** *open for review*: not yet checked by a human. *reported*: reported to Wizards
of the Coast (say when and where). *confirmed*: Wizards changed or confirmed the ruling (say
how).

**Entries.** Each entry gives the card the ruling is attached to and the ruling, verbatim
with its `published_at` date (`- Ruling, <card> (<date>): "<text>"`, one line each); the
source it conflicts with, quoted and dated; the evidence (the likely origin, release notes,
rules changes); what the engine does; and where it is recorded (exemption files, tests and
register ids, in backticks; test paths are relative to `crates/mtg-engine`). Oracle texts
and rulings are quoted from Scryfall's bulk data in `data/` (`oracle_cards.jsonl.gz` and
`rulings.jsonl.gz`, both updated 2026-09-24, per `data/manifest.json`); an Oracle text
quoted without a date is the text of that snapshot. Rules text is quoted from
`data/comprehensive-rules.txt` (effective 2026-09-25); earlier rules text from the archived
Comprehensive Rules at https://github.com/pit142857/mtg-cr (commit 8351b94f; the files are
named by upload date and state their own effective dates). Release notes and update
bulletins were outside these sessions' allowed network hosts (magic.wizards.com,
web.archive.org); where an entry cites one, it was read from a mirror of the Wizards article
archive or from search results, as the work item named in the entry says. The mirror is
ormos (https://github.com/maxmakesmagic/ormos, Markdown made from Wayback Machine captures,
each file naming its capture; commit bd11dee2), whose archive ends in 2022.

**Keeping it complete.** `cargo test -p mtg-data` (`ruling_discrepancies_log_is_complete`)
and `cargo run --release -q -p mtg-tools -- rulings-coverage --check` fail when a rulings
exemption whose reason calls the ruling outdated, superseded, overridden, inconsistent,
misattached, a template error or a copying error, or cites
`ruling-contradicting-oracle-gives-way`, has no
entry here quoting that ruling under that card; when a quoted ruling isn't word for word one
of that card's rulings, or has another date; and when a file, test or register id an entry
names doesn't exist (`crates/mtg-data/src/discrepancies.rs`). To add an instance, add an
entry under its kind with every field and run `cargo test -p mtg-data`. A contradiction with
no clear explanation goes to the coordinator as a conflict case first.

## Template errors

### Homicide Investigator: "triggers for each of them"
- Kind: template error
- Status: open for review
- Ruling, Homicide Investigator (2024-04-12): "If Homicide Investigator dies at the same time as one or more other nontoken creatures you control, Homicide Investigator's ability triggers for each of them."
- Conflicts with: the card's Oracle text, printed so in Murders at Karlov Manor (2024-02-09), its only set: "Whenever one or more nontoken creatures you control die, investigate. This ability triggers only once each turn." Creatures dying together are one event for a "one or more" trigger (CR 603.2c: "An ability triggers only once each time its trigger event occurs. However, it can trigger repeatedly if one event contains multiple occurrences."), and the second sentence forbids a second trigger that turn.
- Evidence: the ruling is Midnight Reaper's, word for word with the name swapped: "If Midnight Reaper dies at the same time as one or more other nontoken creatures you control, Midnight Reaper's ability triggers for each of them." (2018-10-05; Guilds of Ravnica release notes, 2018-09-20), right for Midnight Reaper's per-creature trigger ("Whenever a nontoken creature you control dies, ..."). Homicide Investigator's other 2024-04-12 ruling is Midnight Reaper's companion ruling, in the same order ("Midnight Reaper's ability triggers when it dies if it's not a token.", 2018-10-05); it agrees with the card and is cited. The two are the only Murders at Karlov Manor rulings dated after 2024-02-02. Rulings on the same template say the opposite: G'raha Tia ("Whenever one or more other creatures and/or artifacts you control die, draw a card. This ability triggers only once each turn.") "If G'raha Tia and one or more other creatures and/or artifacts you control die at the same time, its last ability will trigger once (as long as it hasn't already triggered this turn)." (2025-06-06). No rules or Oracle change explains it: 603.2c reads the same in the CR effective 2024-04-12 and today. Investigated in work item ruling-conflicts-2, case 2 (conclusion D).
- Engine: one Clue when nontoken creatures you control die together, the Investigator among them, and at most one each turn, as the Oracle text says.
- Recorded: exemption `docs/rulings-exemptions/rulings-P005.tsv`; register `homicide-investigator-triggers-once`, `ruling-contradicting-oracle-gives-way`; test `tests/rulings/r_p005_dies.rs` (`homicide_investigator_investigates_once_for_creatures_dying_together`).

### Rimescale Dragon: "all permanents" for "Creatures"
- Kind: template error
- Status: open for review
- Ruling, Rimescale Dragon (2006-07-15): "Rimescale Dragon’s last ability affects all permanents with ice counters on them, whether or not it put the ice counters on them."
- Conflicts with: the card's Oracle text, printed so in Coldsnap (2006-07-21), its only set: "Creatures with ice counters on them don't untap during their controllers' untap steps." CR 109.2: "If a spell or ability uses a description of an object that includes a card type or subtype, but doesn’t refer to a specific zone or include the word “card,” “spell,” “source,” or “scheme,” it means a permanent of that card type or subtype on the battlefield."
- Evidence: the ruling is Rimefeather Owl's ruling of the same day, word for word with the name swapped: "Rimefeather Owl’s last ability affects all permanents with ice counters on them, whether or not it put the ice counters on them." (2006-07-15), right for the Owl ("Permanents with ice counters on them are snow."). The Dragon was printed "Creatures" and its Oracle text has always said so; its only Oracle change is the reminder text ("snow source", Kaldheim, 2021). No other ruling in the data widens the card types its card affects. Its "whether or not it put the ice counters on them" agrees with the card. Investigated in work item ruling-conflicts-2, case 4 (conclusion D in part).
- Engine: only creatures with ice counters on them don't untap, whoever put the counters there; a Forest with an ice counter untaps, and so does a land iced while it was animated, once it isn't a creature.
- Recorded: exemption `docs/rulings-exemptions/rulings-P096.tsv` (its "all permanents"; the "whether or not" clause stays cited); register `rimescale-dragon-creatures-only`, `ruling-contradicting-oracle-gives-way`; tests `tests/rulings/r_p096_lockdown.rs` (`rimescale_dragon_freezes_creatures_with_ice_counters_from_any_source_but_not_other_permanents`, `rimescale_dragon_lets_a_land_iced_while_animated_untap_once_its_not_a_creature`).

### Combat Celebrant: "untaps all of your creatures" for "untap all other creatures you control"
- Kind: template error
- Status: open for review
- Ruling, Combat Celebrant (2017-04-18): "Combat Celebrant's ability untaps all of your creatures, not just the ones that are attacking."
- Conflicts with: the card's Oracle text: "If this creature hasn't been exerted this turn, you may exert it as it attacks. When you do, untap all other creatures you control and after this phase, there is an additional combat phase. (An exerted creature won't untap during your next untap step.)" "All of your creatures" includes Combat Celebrant; "other" leaves it tapped.
- Evidence: the ruling's point, "not just the ones that are attacking", agrees with the card; only its noun phrase drops "other". It is in the Amonkhet release notes directly under the card's text (2017-04-14, per work item ruling-conflicts-2; not re-read here). Likely origin (an inference): Lightning Runner's Aether Revolt ruling, "You untap all creatures you control, including ones that aren't attacking." (2017-02-09), written for the same design without "other" ("If you pay, untap all creatures you control, and after this phase, there is an additional combat phase."). Combat Celebrant's ruling about exerting several Celebrants in one combat ("... all of your creatures are untapped only during the current combat phase ...", 2017-04-18) is consistent with "other": each Celebrant's ability untaps the others.
- Engine: untaps every other creature you control, attacking or not; the exerted Combat Celebrant, tapped by attacking, stays tapped.
- Recorded: exemption `docs/rulings-exemptions/rulings-P056.tsv` (its "untaps all of your creatures"; "not just the ones that are attacking" stays cited); register `ruling-contradicting-oracle-gives-way`; tests `tests/rulings/ruling_contradicting_oracle.rs` (`combat_celebrant_untaps_all_other_creatures_but_not_itself`), `tests/rulings/r_p056_extra_phases.rs` (`combat_celebrant_untaps_all_other_creatures_and_adds_a_combat_without_a_main_phase`).

### Citanul Druid: "all players" for "an opponent"
- Kind: template error
- Status: open for review
- Ruling, Citanul Druid (2004-10-04): "In multiplayer games it affects all players."
- Conflicts with: the card's Oracle text: "Whenever an opponent casts an artifact spell, put a +1/+1 counter on this creature." CR 102.2: "In a two-player game, a player’s opponent is the other player." CR 102.3: "In a multiplayer game between teams, a player’s teammates are the other players on their team, and the player’s opponents are all players not on their team." Read as written, "all players" includes the Druid's controller and teammates.
- Evidence: the ruling is the multiplayer note of Gatherer's first batch of rulings (2004-10-04), which says "opponents" or "opposing players" on every other card whose ability applies only to opponents: Malignant Growth "In multiplayer games it affects all opponents.", Havoc "In a multiplayer game, it affects all opponents.", Invoke Prejudice "In a multi-player game, it affects all opposing players.", Aura of Silence "Aura of Silence affects all opponents in a multiplayer game.", Lifetap, Lifeblood, Powerleech, Underworld Dreams, Grim Feast, Roots of Life and Misers' Cage (all 2004-10-04), Kismet and Frozen Aether (2007-02-01). It says "all players" only where the ability applies to every player: Thieves' Auction "In multi-player games, it affects all players and players make choices in turn order.", Sphere of Resistance and Spellshock "It affects all players, including you.", Dueling Grounds and Crumbling Sanctuary "The ability affects all players." (all 2004-10-04). Citanul Druid's note is Malignant Growth's sentence with "players" for "opponents". Its evident intent, that every opponent's artifact spells count, is what the card's text already says, and the ruling has no other content. No version of the card's text says "player": it was printed in Antiquities (1994-03-04) as "Druid gains a +1/+1 counter each time opponent casts an artifact." (two-player wording, as quoted by third-party card databases in search results on 2026-11-05; no scan was checked, since the sites that carry printed texts, such as mtgjson.com and Gatherer, were outside this session's allowed network hosts), which the Oracle text renders "an opponent". Decided under the policy by register-decisions-nov5 (Nov 5).
- Engine: every opponent's artifact spells put a counter on it, in free-for-all and team games; its controller's and teammates' don't.
- Recorded: exemption `docs/rulings-exemptions/rulings-P178.tsv`; register `ruling-contradicting-oracle-gives-way`; test `tests/rulings/ruling_contradicting_oracle.rs` (`citanul_druid_grows_for_each_opponents_artifact_spells_but_not_its_controllers`).

## Copying errors

### Run Out of Town: "controller" for "owner"
- Kind: copying error
- Status: open for review
- Ruling, Run Out of Town (2022-04-29): "That permanent's controller chooses whether to put it on the top or bottom of the library. If the target permanent is made up of multiple cards (due to mutate or meld, for example), that permanent's controller puts all of them on the top or all of them on the bottom. They do not need to reveal the order those cards were put into the library this way."
- Conflicts with: the card's Oracle text, "The owner of target nonland permanent puts it on their choice of the top or bottom of their library." (printed in Streets of New Capenna, 2022-04-29, as "The owner of target nonland permanent puts it on the top or bottom of their library.", per the set's release notes). The two differ when a player other than its owner controls the permanent.
- Evidence: the ruling is the card's own note in the Streets of New Capenna Release Notes (compiled by Jess Dunks, posted 2022-04-20; read from the ormos mirror, Wayback capture 2022-04-23), word for word except that the notes say "owner" where the data says "controller", twice: "That permanent's owner chooses whether to put it on the top or bottom of the library. If the target permanent is made up of multiple cards (due to mutate or meld, for example), that permanent's owner puts all of them on the top or all of them on the bottom. They do not need to reveal the order those cards were put into the library this way." Of the notes on that set's cards in those release notes, 277 are in the data word for word, 36 have no close counterpart among the card's rulings, and 11 differ: by later updates (Grime Gorger's list of card types, with battle and kindred; Cemetery Tampering's hideaway reminder, to the hideaway rule of the set's own CR, effective 2022-04-29; Lethal Scheme's two convoke notes, reworded and dated 2024-01-12), by rewording or formatting (Determined Iteration twice, Aven Heartstabber, Evolving Door, an added sentence on Witness Protection), and by two word changes that contradict the card, this one and Sanguine Spy's (below). An edited ruling sometimes keeps its first date (Grime Gorger's 2022-04-29 ruling names battle and kindred, card types of 2023 and 2024) and sometimes gets a new one (Lethal Scheme's, 2024-01-12), so its date doesn't show when "controller" came in. Every other ruling on a card with the same instruction has the owner choose: Aether Gust (2019-07-12), Moonsnare Prototype (2022-02-18), Desynchronize (2022-10-14), Misleading Motes (2023-09-01), Unlucky Drop (2023-11-10), Sudden Setback (2024-02-02), Deem Inferior (2024-06-07), Dire Downdraft (2024-07-26), Lost Days (2025-10-02), Uneasy Partings (2026-06-29); and the Oracle text's "their choice of", added after printing, makes the owner's choice explicit. No rule gives the choice to the controller. Investigated in work item ruling-conflicts-3, case 1.
- Engine: the permanent's owner chooses top or bottom, as for Aether Gust, Desynchronize and Unlucky Drop; a melded permanent's owner puts both cards on top or both on the bottom, in the order they choose, without revealing it. The controller isn't asked.
- Recorded: exemption `docs/rulings-exemptions/rulings-P140.tsv` (the two sentences with "controller"; the third stays cited); register `ruling-contradicting-oracle-gives-way`; tests `tests/rulings/r_p140_tuck.rs` (`run_out_of_town_follows_its_oracle_text`, `run_out_of_town_puts_a_melded_permanent_all_on_top_or_all_on_the_bottom`).

### Sanguine Spy: "cards you control" for "cards in your graveyard"
- Kind: copying error
- Status: open for review
- Ruling, Sanguine Spy (2022-04-29): "Sanguine Spy's triggered ability checks the number of mana values among cards you control both when the ability triggers and when it resolves. If you no longer have five or more mana values among those cards when the ability resolves, you may not pay 2 life."
- Conflicts with: the card's Oracle text: "At the beginning of your end step, if there are five or more mana values among cards in your graveyard, you may pay 2 life. If you do, draw a card." (an intervening "if" clause, CR 603.4: checked as the ability triggers and again as it resolves).
- Evidence: the ruling is the card's own note in the Streets of New Capenna Release Notes (2022-04-20; ormos mirror, as for Run Out of Town above), word for word except for "cards in your graveyard" where the data says "cards you control": "Sanguine Spy's triggered ability checks the number of mana values among cards in your graveyard both when the ability triggers and when it resolves. If you no longer have five or more mana values among those cards when the ability resolves, you may not pay 2 life." The card's other rulings (2022-04-29) and those of the set's other cards with the same condition (Aven Heartstabber, Graveyard Shift, Sewer Crocodile, Snooping Newsie, Syndicate Infiltrator, Tainted Indulgence) all say "cards in your graveyard". Found in work item ruling-conflicts-3 while comparing that set's release notes with the data for case 1 (added as case 5).
- Engine: counts the mana values among the cards in its controller's graveyard, as the ability triggers and as it resolves; permanents with five mana values don't make it trigger, and a graveyard that drops below five mana values before it resolves means no life is paid and no card drawn.
- Recorded: exemption `docs/rulings-exemptions/rulings-P093.tsv` (its "among cards you control"; the rest stays cited); register `ruling-contradicting-oracle-gives-way`; test `tests/rulings/ruling_contradicting_oracle.rs` (`sanguine_spy_counts_mana_values_among_cards_in_your_graveyard`).

## Inconsistent rulings

### Gnoll War Band: opponents who have left the game aren't counted
- Kind: inconsistent
- Status: open for review
- Ruling, Gnoll War Band (2022-06-10): "Opponents who have left the game will not be counted when determining how many opponents have been dealt damage this turn."
- Conflicts with: the other rulings on the same question, which count them: Spinerock Knoll "At the time the last ability resolves, you'll get to play the card if a player who is currently your opponent, or a player who was your opponent at the time they left the game, has been dealt 7 damage over the course of the turn." (2007-10-01); Sygg, River Cutthroat "As the end step begins, Sygg’s ability checks whether a player who is currently your opponent, or a player who was your opponent at the time they left the game, has lost 3 or more life over the course of the turn." (2008-05-01); Moonshae Pixie, from Gnoll War Band's own set with the same structure: "If an opponent was dealt combat damage this turn and subsequently lost the game, Moonshae Pixie's second ability still counts that player when determining how many cards its controller should draw." (2024-04-12); and about a dozen more from 2008 to 2024 (Notorious Throng, Cryptborn Horror, Thought Sponge, Chandra's Incinerator, Belbe, Corrupted Observer, Rakdos, Lord of Riots, Tymna the Weaver, Ludevic, Necro-Alchemist, Dunerider Outlaw, Teysa, Opulent Oligarch, Kaito, Bane of Nightmares). The card: "This spell costs {1} less to cast for each opponent who was dealt damage this turn." The CR leaves the question open (800.4i: "If an effect requires information about a specific player, the effect uses the current information about that player if they are still in the game; otherwise, the effect uses the last known information about that player before they left the game. If an effect requires information from the game about actions players have taken, the effect can find actions that were taken by a player who has left the game. ...").
- Evidence: no wording sets the card apart from the others (any damage or combat damage, a cost reduction or a count; see the register entry), and the rulings before and after it agree with each other, so no rules change explains it: 800.4i's first sentence dates from the CR of 2007-10-01 (as 600.4e) and its second from the CR effective 2019-08-23 (as 800.4h), both before this ruling (archived CR texts). Likely origin (an inference): the template for counts of opponents a player has now, "Opponents who have left the game aren't counted when determining how many tokens to create." (encore cards such as Phyrexian Triniform, 2020-11-10), applied to a count of what happened this turn. Investigated in work item ruling-conflicts-1, case 2 (conclusion C); set aside by the user's decision of Nov 5.
- Engine: counts every opponent who was dealt damage this turn, including those who have since left the game, as for every "for each opponent who ... this turn" count.
- Recorded: exemption `docs/rulings-exemptions/rulings-P186.tsv`; register `history-counts-players-who-left`; test `tests/cards/value_results_history.rs` (`gnoll_war_band_counts_opponents_who_were_dealt_damage`).

### Araumi of the Dead Tide: "you can choose any value for X"
- Kind: inconsistent
- Status: open for review
- Ruling, Araumi of the Dead Tide (2020-11-10): "If the creature card has {X} in its mana cost, you can choose any value for X, but the value you choose won't apply to the tokens that are created. X will be 0 for those tokens."
- Conflicts with: the rulings on cards that give scavenge in the same words ("The scavenge cost is equal to its mana cost."): Varolz, the Scar-Striped (2013-04-15) and Young Deathclaws (2024-03-08), "If the creature card you scavenge has {X} in its mana cost, X is 0."; and CR 107.3h: "If an effect instructs a player to pay an object’s mana cost that includes {X}, the value of X is treated as 0 unless the object is a spell on the stack. In that case, the value of X is the value chosen or determined for it as the spell was cast." The card: "{T}, Exile cards from your graveyard equal to the number of opponents you have: Target creature card in your graveyard gains encore until end of turn. The encore cost is equal to its mana cost. (Exile the creature card and pay its mana cost: ...)".
- Evidence: the ruling is in the Commander Legends Release Notes as the data has it (Eli Shiffrin, 2020-11-06; ormos mirror, Wayback capture 2020-11-07). 107.3h was added, as 107.3g, in the CR effective 2015-01-23, before the ruling, and hasn't changed since but for its number (107.3h from the CR effective 2020-09-25); the Fate Reforged Update Bulletin (Matt Tabak, 2015-01-15) explains it: "Essentially, if the object is a spell on the stack, you pay the actual value of X. Any other time, X is 0.", with manifest's cost to turn a card face up as its first use. CR 702.1a's example (since the CR effective 2016-09-30, the Kaladesh update) calls the scavenge cost Varolz gives "an amount of mana equal to its mana cost", and 107.3g has a card's X be 0 off the stack. Likely origin (an inference): the ruling on cards that give flashback "equal to its mana cost" (Past in Flames, Snapcaster Mage: "If you cast an instant or sorcery with {X} in its mana cost this way, you still choose the value of X as part of casting the spell and pay that cost."), right there because the card is cast and is a spell on the stack, applied to encore, whose card is never cast. Its second sentence (the tokens' X is 0) agrees with the CR and is cited. Investigated in work item ruling-conflicts-3, case 6 (conclusion D); its first clause set aside as inconsistent by the user's decision of Nov 5 (evening; work item register-decisions-nov5b), as Gnoll War Band's ruling was.
- Engine: X is 0 for the encore cost Araumi gives (no X is announced) and for the tokens, as for scavenge.
- Recorded: exemption `docs/rulings-exemptions/rulings-P024.tsv` (its "you can choose any value for X"; the second sentence stays cited); register `x-in-a-cost-equal-to-a-cards-mana-cost-is-0`; tests `tests/rulings/r_p024_x_values.rs` (`araumi_encore_cost_and_tokens_have_x_0`), `tests/rulings/r_s27_mana_value_x.rs` (`a_scavenged_card_s_x_is_0`).

### Arbiter of Knollridge: each player's own range of influence
- Kind: inconsistent
- Status: open for review
- Ruling, Arbiter of Knollridge (2007-10-01): "In other multiplayer formats, each player's life total is set to the highest life total within their range of influence. These changes happen at the same time. For example, if each player has a range of influence of one, and the players in the game are Alice (4 life), Barry (8 life), Carrie (11 life), and Doug (7 life), in that order, then Alice's life total will become 8 and Barry and Doug's life totals will each become 11."
- Conflicts with: CR 801.10: "Spells and abilities can’t affect objects or players outside their controller’s range of influence. The parts of the effect that attempt to affect an out-of-range object or player will do nothing. The rest of the effect will work normally." CR 801.11: "If a spell or ability requires information from the game, it gets only information from within its controller’s range of influence. It doesn’t see objects or events outside its controller’s range of influence." The other rulings on range of influence, which use the controller's: Imperial Mask (2007-05-01) "Only teammates within the range of influence of Imperial Mask's controller will get a token. ..."; Benediction of Moons (2006-02-01) "... In a Free-for-All multiplayer game, players who have been eliminated or who are outside your range of influence don't count toward the total." The card: "When this creature enters, each player's life total becomes the highest life total among all players."
- Evidence: the example uses each affected player's own range: Alice's {Doug, Alice, Barry} gives 8, Barry's {Alice, Barry, Carrie} and Doug's {Carrie, Doug, Alice} give Carrie's 11. No controller fits it under the CR: only Alice has Alice, Barry and Doug in range, and Carrie, whose 11 they get, is outside Alice's range. The CR said the same when the ruling was written (Lorwyn, 2007-10-01; archived CR texts): the CR current as of 2007-10-01 had 801.10 and 801.11 word for word as 601.10 and 601.11, and 601.11's second example then narrowed what an affected player's object sees to "players within range of both Rob and the controller of Coat of Arms" (changed to the controller's whole range in the CR effective 2011-05-01), which gives Barry 8 and Doug 7, not 11; so no rules change explains the ruling. Read alone, its first sentence could mean the controller's range ("their" for the Arbiter's controller), but the example rules that out. Found by work item rulings-P014; investigated in work item ruling-conflicts-3, case 7 (conclusion D); set aside as inconsistent with 801.10, 801.11 and the other rulings by the user's decision of Nov 5 (evening; work item register-decisions-nov5b), as Gnoll War Band's was. Its other two rulings (life-gain triggers; Two-Headed Giant teams, 2010-06-15) are cited.
- Engine: with a limited range of influence, the ability affects only the players within its controller's range and sets each to the highest life total among the players within that range (Alice controlling it: Alice, Barry and Doug become 8; Carrie, out of range, keeps 11).
- Recorded: exemption `docs/rulings-exemptions/rulings-P014.tsv`; register `arbiter-of-knollridge-uses-controllers-range`; test `tests/rulings/r_p014_catch_up.rs` (`arbiter_of_knollridge_with_range_of_influence_uses_its_controllers_range`).

## Narrower than stated

### Tempt with Glory and the other tempting offers: "starting with the opponent on your left"
- Kind: narrower than stated
- Status: open for review
- Ruling, Tempt with Glory (2013-10-17): "Your opponents decide in turn order whether or not they accept the offer, starting with the opponent on your left. Each opponent will know the decisions of previous opponents in turn order when making their decision."
- Ruling, Tempt with Discovery (2013-10-17): "Your opponents decide in turn order whether or not they accept the offer, starting with the opponent on your left. Each opponent will know the decisions of previous opponents in turn order when making their decision."
- Ruling, Tempt with Immortality (2013-10-17): "Your opponents decide in turn order whether or not they accept the offer, starting with the opponent on your left. Each opponent will know the decisions of previous opponents in turn order when making their decision."
- Ruling, Tempt with Reflections (2013-10-17): "Your opponents decide in turn order whether or not they accept the offer, starting with the opponent on your left. Each opponent will know the decisions of previous opponents in turn order when making their decision."
- Ruling, Tempt with Vengeance (2013-10-17): "Your opponents decide in turn order whether or not they accept the offer, starting with the opponent on your left. Each opponent will know the decisions of previous opponents in turn order when making their decision."
- Ruling, Tempt with Bunnies (2024-07-26): "Your opponents decide in turn order whether or not they accept the offer, starting with the next opponent in turn order. Each opponent will know the decisions of previous opponents in turn order when making their decision."
- Conflicts with: CR 101.4: "If multiple players would make choices and/or take actions at the same time, the active player (the player whose turn it is) makes any choices required, then the next player in turn order (usually the player seated to the active player’s left) makes any choices required, followed by the remaining nonactive players in turn order. ..." Tempt with Mayhem (2024-06-07): "Starting with the next opponent in turn order (or the active player, if that player is one of your opponents), each opponent in turn order decides whether or not to copy the spell ...".
- Evidence: the rulings describe the caster's own turn: every one of these cards is a sorcery, so its caster is normally the active player, and the opponent on the left (the next player in turn order) decides first. Cast on another player's turn (with an effect such as Leyline of Anticipation's "You may cast spells as though they had flash."), 101.4 has the active player decide first, as Tempt with Mayhem's ruling says for the one tempting offer that is an instant; Extract a Confession (2024-02-02) and Flare of Malice (2024-06-07) add the same qualification for their sacrifices ("or, if you cast ... on an opponent's turn, starting with the opponent whose turn it is"). The 2013 text (Commander 2013, 2013-10-17) is on five cards; Tempt with Bunnies (2024-07-26) words it "starting with the next opponent in turn order", with the same gap. Investigated in work item ruling-conflicts-1, case 16 (conclusion A: the sources agree for the caster's own turn).
- Engine: the opponents decide in APNAP order (CR 101.4): on the caster's turn the next player in turn order first, on another player's turn the active player first if it's an opponent; each knows the earlier decisions (101.4b).
- Recorded: no exemption: the rulings stay cited for the caster's own turn; tests `tests/rulings/r_s16_tempting_offer.rs` (`opponents_decide_in_turn_order_from_your_left_knowing_earlier_choices`, `off_turn_the_active_opponent_decides_first`, `tempt_with_bunnies_draws_and_makes_rabbits_for_each_player`).

### Lightning, Army of One: "a source you control" for Stagger's "a source"
- Kind: narrower than stated
- Status: open for review
- Ruling, Lightning, Army of One (2025-06-06): "If another effect (or effects) modifies how much damage a source you control would deal—by preventing some of it, for example—the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects (including Lightning's) apply. If all of the damage is prevented, Lightning's effect no longer applies."
- Ruling, Lightning, Army of One (2025-06-06): "If damage dealt by a source you control is being divided or assigned among multiple permanents and/or players, that damage is divided or assigned before doubling. For example, if you attack with a 5/5 creature with trample and it's blocked by a 2/2 creature, you can assign 2 damage to the blocker and 3 damage to the defending player. Those amounts are then doubled to 4 and 6, respectively."
- Conflicts with: nothing they say: both are right for sources Lightning's controller controls. But the card's Oracle text doubles damage from any source: "Stagger — Whenever Lightning deals combat damage to a player, until your next turn, if a source would deal damage to that player or a permanent that player controls, it deals double that damage instead." The rulings' "a source you control" reads as if it applied only to those.
- Evidence: both are Twinflame Tyrant's rulings (Foundations, 2024-11-08), word for word with the name swapped, right for its "If a source you control would deal damage to an opponent or a permanent an opponent controls, it deals double that damage instead."; Lightning's third ruling of the same day ("The damage is dealt by the same source as the original source of damage. ...") is Twinflame Tyrant's too. The same holds for other players' sources: CR 616.1 has the affected player or controller choose the order whoever controls the source, and CR 601.2d and 510.1 divide or assign damage before any replacement applies. Found in work item rulings-P040 (Nov 5); not a contradiction, so nothing is exempted.
- Engine: Stagger doubles damage from any source, including the damaged player's own and a third player's, to that player and their permanents until Lightning's controller's next turn; the damaged player orders it with prevention effects whoever controls the source.
- Recorded: no exemption: the rulings stay cited; tests `tests/rulings/r_p040_multipliers.rs` (`the_player_dealt_damage_orders_multipliers_and_prevention`, `lightning_stagger_doubles_damage_from_any_source_to_that_player_until_your_next_turn`).

### Valki, God of Lies: "you can't attack with him" after he becomes a copy
- Kind: narrower than stated
- Status: open for review
- Ruling, Valki, God of Lies // Tibalt, Cosmic Impostor (2021-02-05): "If Valki becomes a copy of a creature during the same turn Valki enters the battlefield, you can't attack with him or use any {T} abilities he gains."
- Conflicts with: CR 702.10b: "If a creature has haste, it can attack even if it hasn’t been controlled by its controller continuously since their most recent turn began. (See rule 302.6.)" Volrath, the Shapestealer's ruling on the same situation (2019-08-23): "If Volrath copies a creature the same turn Volrath enters the battlefield, you can't attack with it or use any of its {T} abilities (if it gains any) unless the copied creature has haste."
- Evidence: Valki's copy ability ("{X}: Choose a creature card exiled with Valki with mana value X. Valki becomes a copy of that card.") copies the card's haste like any other ability (CR 707.2), and a creature with haste can attack and use its {T} abilities the turn it comes under its controller's control (CR 302.6, 702.10b). The ruling is right for a copied creature without haste, the usual case; it leaves out the exception the older Volrath ruling states. Found in work item rulings-P022.
- Engine: summoning sickness applies to Valki after it becomes a copy (it can't attack or use {T} abilities the turn it entered) unless the copy has haste.
- Recorded: no exemption: the ruling stays cited for a copied creature without haste; test `tests/rulings/r_p022_valki.rs` (`valki_copying_the_turn_it_entered_cant_attack_or_tap`, which also pins the haste case).

### The Phasing of Zhalfir: "while the Saga is on the battlefield" for "for as long as you control this Saga"
- Kind: narrower than stated
- Status: open for review
- Ruling, The Phasing of Zhalfir (2022-09-09): "Permanents that phase out as The Phasing of Zhalfir's first and second chapter abilities resolve don't phase in during their controllers' untap steps while the Saga is on the battlefield. After it leaves the battlefield, they will phase in during their controller's next untap step as normal."
- Conflicts with: nothing in the usual case, but the card's Oracle text ties the effect to control, not to the battlefield: "I, II — Another target nonland permanent phases out. It can't phase in for as long as you control this Saga." CR 611.2b: "Some continuous effects generated by the resolution of a spell or ability have durations worded “for as long as . . . .” ..." If another player gains control of the Saga, the duration is over, and the permanent phases in during its controller's next untap step with the Saga still on the battlefield.
- Evidence: the Dominaria United release notes (Jess Dunks, 2022-08-26; ormos mirror) have the same text under the printed wording "It can't phase in for as long as you control The Phasing of Zhalfir.", so it isn't a transcription error: the ruling describes the usual case, in which the Saga's controller doesn't change while it's on the battlefield. Like the "for as long as" ruling the same release notes give it ("Any continuous effects with a "for as long as" duration ... If ignoring those objects causes the effect's conditions to no longer be met, the duration will expire."), the duration doesn't begin again if the Saga's controller regains control of it. Found in the review of work item rulings-P048 (Nov 6); not a contradiction in the case it was written for, so nothing is exempted.
- Engine: the permanent stays phased out while the chapter ability's controller controls the Saga; once they stop controlling it (another player gains control of it, it leaves the battlefield), it phases in during its controller's next untap step, even if they have regained control of the Saga by then.
- Recorded: no exemption: the ruling stays cited for the usual case; tests `tests/rulings/r_p048_phasing.rs` (`zhalfirs_phased_out_permanents_stay_out_while_the_saga_is_controlled`, `zhalfirs_permanent_phases_in_once_you_stop_controlling_the_saga`).

## Misattached rulings

### Astarion, the Decadent: Astarion's Thirst's ruling about X
- Kind: misattached
- Status: open for review
- Ruling, Astarion, the Decadent (2022-06-10): "Use the creature's power as it last existed on the battlefield to determine the value of X."
- Conflicts with: Astarion, the Decadent has no X and nothing based on power: "Deathtouch, lifelink / At the beginning of your end step, choose one — • Feed — Target opponent loses life equal to the amount of life they lost this turn. • Friends — You gain life equal to the amount of life you gained this turn." The ruling describes Astarion's Thirst: "Exile target creature. Put X +1/+1 counters on a commander creature you control, where X is the power of the creature exiled this way."
- Evidence: the Commander Legends: Battle for Baldur's Gate release notes (Jess Dunks, 2022-06-01) list the ruling under Astarion's Thirst (work item ruling-conflicts-1, case 18), which has no rulings in the data. Astarion's Thirst's flavor text is a quote attributed to Astarion ("—Astarion, the Decadent"), which may be how the two were paired (an inference).
- Engine: Astarion's Thirst uses the exiled creature's power as it last existed on the battlefield (CR 608.2h).
- Recorded: test `tests/rulings/misattached_rulings.rs` (`astarions_thirst_counts_the_exiled_creatures_last_known_power`), citing it under Astarion, the Decadent, which takes part as the commander creature that gets the counters; its earlier exemption (rulings-P194) is withdrawn.

### Clive, Ifrit's Dominant: Clive's Hideaway's rulings
- Kind: misattached
- Status: open for review
- Ruling, Clive, Ifrit's Dominant // Ifrit, Warden of Inferno (2025-06-06): "Clive's Hideaway's last ability only cares if you control four or more legendary creatures at the time it resolves. You don't even have to control four or more legendary creatures when you activate it."
- Ruling, Clive, Ifrit's Dominant // Ifrit, Warden of Inferno (2025-06-06): "If the exiled card is a land card, you may play it only if you have an available land play remaining this turn."
- Ruling, Clive, Ifrit's Dominant // Ifrit, Warden of Inferno (2025-06-06): "You play the card while Clive's Hideaway's last ability is resolving and still on the stack. You can't wait to play it later in the turn."
- Ruling, Clive, Ifrit's Dominant // Ifrit, Warden of Inferno (2025-06-06): ""Hideaway N" means "When this permanent enters, look at the top N cards of your library. Exile one of them face down and put the rest on the bottom of your library in a random order. The exiled card gains ‘The player who controls the permanent that exiled this card may look at this card in the exile zone.'""
- Ruling, Clive, Ifrit's Dominant // Ifrit, Warden of Inferno (2025-06-06): "Any player who has controlled a permanent with a hideaway ability since a card was exiled with it may look at that card."
- Ruling, Clive, Ifrit's Dominant // Ifrit, Warden of Inferno (2025-06-06): "If you cast a spell "without paying its mana cost," you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, those must be paid to cast the spell."
- Ruling, Clive, Ifrit's Dominant // Ifrit, Warden of Inferno (2025-06-06): "If a spell has {X} in its mana cost, you must choose 0 as the value of X when playing it without paying its mana cost."
- Conflicts with: Clive, Ifrit's Dominant // Ifrit, Warden of Inferno has no hideaway ability and no card to play without paying its mana cost (Clive: "When Clive enters, you may discard your hand, then draw cards equal to your devotion to red. ... {4}{R}{R}, {T}: Exile Clive, then return it to the battlefield transformed under its owner's control. ..."). The rulings describe Clive's Hideaway: "Hideaway 4 (...) {T}: Add {C}. {2}, {T}: You may play the exiled card without paying its mana cost if you control four or more legendary creatures."
- Evidence: FINAL FANTASY release-note rulings (2025-06-06) about Clive's Hideaway's last ability: three name Clive's Hideaway, and the four general notes, on hideaway and on playing a card without paying its mana cost, fit only it. Clive's Hideaway itself has only the Town note in the data. Read for Clive's Hideaway, they agree with the CR (602.1b, 608.2g, 608.2h, 305.2a, 305.2b, 305.3): work item ruling-conflicts-2, case 5 (conclusion A, misattached).
- Engine: as the rulings say, for Clive's Hideaway.
- Recorded: test `tests/keywords/k702_075_hideaway.rs` (`clives_hideaway_checks_its_legendary_creatures_and_plays_the_card_as_the_ability_resolves`), citing the three under Clive, which is one of the four legendary creatures there; the four general notes are cited on other cards with the same text.

### Squall, SeeD Mercenary: Starting Town's rulings
- Kind: misattached
- Status: open for review
- Ruling, Squall, SeeD Mercenary (2025-06-06): "If an effect restarts the game, Starting Town will enter tapped in the new game unless it's your first, second, or third turn of that game."
- Ruling, Squall, SeeD Mercenary (2025-06-06): "Starting Town's first ability cares about how many turns you have taken, not necessarily how many turns the game has had, in case you take an extra turn."
- Ruling, Squall, SeeD Mercenary (2025-06-06): "If Starting Town enters during another player's turn, it'll enter tapped no matter how many turns they've taken."
- Conflicts with: Squall counts no turns: "Rough Divide — Whenever a creature you control attacks alone, it gains double strike until end of turn. / Whenever Squall deals combat damage to a player, return target permanent card with mana value 3 or less from your graveyard to the battlefield." The rulings name and describe Starting Town: "This land enters tapped unless it's your first, second, or third turn of the game. / {T}: Add {C}. / {T}, Pay 1 life: Add one mana of any color."
- Evidence: each names Starting Town; Starting Town has only the Town note in the data. Both are FINAL FANTASY cards (rulings dated 2025-06-06).
- Engine: as the rulings say, for Starting Town.
- Recorded: tests `tests/rulings/r_p097_interactions.rs` (`starting_town_enters_tapped_on_other_players_turns`, `starting_town_counts_the_turns_of_a_restarted_game`) and `tests/rulings/r_p098_misc.rs` (`starting_town_counts_the_turns_you_have_taken`, where Squall takes part: its trigger returns a Starting Town from its controller's graveyard on that player's third turn, the game's sixth, and it enters untapped), citing them under Squall. Squall can't make a Starting Town enter on another player's turn, and the restart test has no use for it.

### Gut, True Soul Zealot: Halsin, Emerald Archdruid's rulings
- Kind: misattached
- Status: open for review
- Ruling, Gut, True Soul Zealot (2022-06-10): "Effects that modify a creature's power and/or toughness, such as the one created by Titanic Growth, will apply to the creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch power and toughness."
- Ruling, Gut, True Soul Zealot (2022-06-10): "Halsin, Emerald Archdruid's activated ability overwrites any previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Halsin's ability resolves will overwrite this effect."
- Conflicts with: Gut sets no power or toughness: "Whenever you attack, you may sacrifice another creature or an artifact. If you do, create a 4/1 black Skeleton creature token with menace that's tapped and attacking. (...) / Choose a Background (...)". The rulings describe Halsin, Emerald Archdruid: "{1}: Until end of turn, target token you control becomes a green Bear creature with base power and toughness 4/4 in addition to its other colors and types."
- Evidence: the second names Halsin; the two are the usual pair of rulings for an effect that sets base power and toughness (e.g. Warden of the First Tree, 2014-11-24). Halsin has only the Background notes in the data. Both are Commander Legends: Battle for Baldur's Gate cards (rulings dated 2022-06-10).
- Engine: as the rulings say, for Halsin (layers 7b, 7c and 7d in timestamp order, CR 613.4, 613.7).
- Recorded: tests `tests/rulings/misattached_rulings.rs` (`halsin_sets_base_power_and_toughness_under_effects_that_modify_them`, `halsin_overwrites_earlier_base_power_and_toughness_and_later_ones_overwrite_it`), citing them under Gut, whose Skeleton token is the token Halsin animates.

### Scuttletide: a ruling about a trigger on attackers declared against you
- Kind: misattached
- Status: open for review
- Ruling, Scuttletide (2021-06-18): "The triggered ability triggers when creatures are declared as attackers attacking you or a planeswalker you control. It will not trigger if an effect puts a creature onto the battlefield attacking you or a planeswalker you control."
- Conflicts with: Scuttletide has no triggered ability: "{1}, Discard a card: Create a 0/3 blue Crab creature token. / Delirium — Crabs you control get +1/+1 as long as there are four or more card types among cards in your graveyard."
- Evidence: the card the ruling describes isn't identified: no other card in the data has this ruling, and no Modern Horizons 2 card (the set of Scuttletide and of the ruling's date) has a trigger on creatures attacking you or a planeswalker you control. Scuttletide's other rulings include three about Clue tokens ("The token is named Clue Token ...", 2021-06-18), which fit Hard Evidence ("Create a 0/3 blue Crab creature token. Investigate."), another Modern Horizons 2 card that makes Crabs and has no rulings of its own in the data; so rulings of neighbouring cards were filed under Scuttletide, and this one probably came from another card of the set's release notes (an inference). The only card in the data whose Oracle text both has such a trigger and makes Clues is Search the Premises ("Whenever a creature attacks you or a planeswalker you control, investigate."), which also has no rulings of its own, but its printing in the data is from Murders at Karlov Manor Commander (released 2024-02-09), after the ruling's date, and it isn't a Modern Horizons 2 card, so it isn't likely to be the card the 2021 release notes described.
- Engine: an ability that triggers when a creature attacks you or a planeswalker you control triggers on attackers declared that way (CR 508.3a) and not for a creature put onto the battlefield attacking (CR 508.4).
- Recorded: test `tests/rulings/r_p011_combat.rs` (`creatures_put_onto_the_battlefield_attacking_werent_declared`), with Search the Premises ("Whenever a creature attacks you or a planeswalker you control, investigate.") standing in for the card it describes and Scuttletide on the battlefield, citing it under Scuttletide.

## Outdated rulings

### Aetherplasm: "This allows it to dodge abilities like Deathgazer's."
- Kind: outdated
- Status: open for review
- Ruling, Aetherplasm (2006-02-01): "The creature you put onto the battlefield may be the Aetherplasm you just returned to your hand. This allows it to dodge abilities like Deathgazer's."
- Conflicts with: CR 509.3d: "An ability that reads “Whenever [a creature] becomes blocked by a creature, . . .” triggers once for each creature that blocks the specified creature. ... In addition, it will trigger if a creature is put onto the battlefield blocking that creature. ..." Deathgazer: "Whenever this creature blocks or becomes blocked by a nonblack creature, destroy that creature at end of combat."
- Evidence: the ruling applied 309.4 of the CRs from 2006-05-01 to 2009-05-01: "Abilities that trigger on a creature becoming blocked trigger only at the first point the creature becomes blocked that combat. They will trigger if a creature becomes blocked by a creature declared as a blocker, by a creature that's put into play as a blocker, or by an effect, but only if the attacking creature hadn't yet been blocked that combat. ..." The M10 CR ("effective as of July 11, 2009") deleted 309.4 and gave "Whenever a creature blocks [this creature]" (renamed "Whenever [this creature] becomes blocked by a creature" in the CR effective 2009-09-04) one trigger for each blocking creature; the CR effective 2010-02-01 added its sentence for a creature put onto the battlefield blocking (all from the archived CR texts). Work item ruling-conflicts-1, case 1 (conclusion B); accepted by the user Nov 5. The ruling's first sentence is still right.
- Engine: Deathgazer's ability triggers again for the Aetherplasm put back onto the battlefield blocking it, and destroys it at end of combat.
- Recorded: exemption `docs/rulings-exemptions/rulings-P006.tsv` (its second sentence); register `put-onto-battlefield-blocking-retriggers-becomes-blocked-by`; test `tests/rulings/r_p006_activated_bounce.rs` (`aetherplasm_can_put_itself_back_but_deathgazer_sees_the_new_blocker`).

### Blitz Hellion: "its owner doesn't shuffle their library"
- Kind: outdated
- Status: open for review
- Ruling, Blitz Hellion (2009-05-01): "Blitz Hellion’s last ability triggers at the end of each turn in which it’s on the battlefield. If it’s not on the battlefield by the time this ability resolves, nothing happens. Blitz Hellion stays wherever it is, and its owner doesn’t shuffle their library."
- Conflicts with: CR 701.24c: "If an effect would cause a player to shuffle one or more specific objects into a library, that library is shuffled even if none of those objects are in the zone they’re expected to be in or an effect causes all of those objects to be moved to another zone or remain in their current zone." (since the Magic Origins CR, effective 2015-07-17). Later rulings follow it: Etrata, the Silencer "... it remains in its new zone and Etrata's owner shuffles their library." (2018-10-05); Serra Avatar "If Serra Avatar is no longer in your graveyard when its triggered ability resolves, you still shuffle your library." (2022-12-08).
- Evidence: the ruling stated the rule of its time, written into the CR effective 2010-02-01 (701.15c: "... but none of those objects are in the zone they're expected to be in, that library is not shuffled.") and still in the CR effective 2015-03-27 (701.16c). The Magic Origins CR reversed it, as Wizards' "Magic Origins Update Bulletin—Comprehensive Rules Changes" (Matt Tabak, 2015-07-16; read from a mirror of the Wizards archive) says: "Under old rules, you wouldn't shuffle your library, but that's changing." Work item ruling-conflicts-1, case 3 (conclusion B); accepted by the user Nov 5. "Blitz Hellion stays wherever it is" is still right.
- Engine: the owner's library is shuffled; Blitz Hellion stays where it is.
- Recorded: exemption `docs/rulings-exemptions/rulings-P180.tsv`; register `self-shuffle-gone-library-still-shuffled`; tests `tests/rulings/r_p180_tuck.rs` (`end_step_self_shuffles_leave_a_creature_that_left_where_it_is`, `serra_avatar_gone_from_the_graveyard_still_shuffles_the_library`).

### Cyclopean Tomb: "you must remove a mire counter"
- Kind: outdated
- Status: open for review
- Ruling, Cyclopean Tomb (2008-08-01): "The delayed triggered ability created when Cyclopean Tomb is put into a graveyard from the battlefield triggers during each of your upkeeps for the rest of the game. As it resolves, you must remove a mire counter from a land that had a mire counter put on it by that instance of Cyclopean Tomb, but it doesn't matter where the mire counter you remove came from. For instance, you could remove mire counters that were put on the land by Gilder Bairn."
- Conflicts with: the card's Oracle text since the September 2008 Update Bulletin (Mark L. Gottlieb, 2008-09-24): "When this artifact is put into a graveyard from the battlefield, at the beginning of each of your upkeeps for the rest of the game, remove all mire counters from a land that a mire counter was put onto with this artifact but that a mire counter has not been removed from with this artifact."
- Evidence: the ruling paraphrases the wording of the July 2008 Update Bulletin (Mark L. Gottlieb, 2008-07-09): "remove a mire counter from a land that a mire counter was put onto with Cyclopean Tomb", which the September bulletin replaced so that the card "basically does what it says it does" (both read from a mirror of the Wizards archive). Work item ruling-conflicts-1, case 6 (conclusion B in part); accepted by the user Nov 5. The rest of the ruling (each upkeep for the rest of the game; the counters' source doesn't matter) is still right.
- Engine: each upkeep, all mire counters are removed from one such land (its controller's choice among those that still have mire counters), which then no longer qualifies.
- Recorded: exemption `docs/rulings-exemptions/rulings-P197.tsv`; register `cyclopean-tomb-land-choice`; tests `tests/cards/m_cyclopean_tomb.rs` (`after_it_dies_each_upkeep_one_of_its_lands_loses_its_mire_counters`, `all_mire_counters_go_whatever_their_source_and_the_land_then_no_longer_qualifies`).

### Krosan Adaptation: storm's token copies "considered to have been created"
- Kind: outdated
- Status: open for review
- Ruling, Krosan Adaptation (2019-11-12): "For effects that look for or modify the creation of tokens, the token copies of Krosan Adaptation are considered to have been “created” and those effects will apply."
- Conflicts with: CR 111.13: "A copy of a permanent spell becomes a token as it resolves. The token has the characteristics of the spell that became that token. The token is not “created” for the purposes of any replacement effects or triggered abilities that refer to creating a token." CR 608.3f: "... A token put onto the battlefield this way is no longer a copy of a spell and is not “created” for the purposes of any rules or effects that refer to creating a token." Frontline Heroism (2024-11-08): "The token that a resolving copy of a permanent spell becomes isn't "created" and won't interact with abilities that care about tokens being created, such as the first ability of Doubling Season."
- Evidence: a Mystery Booster playtest card's ruling (2019-11-12), under the Kaladesh-era definition of "create" (put a token onto the battlefield). The Zendikar Rising CR (effective 2020-09-25) added 111.11 and 608.3b, today's 111.13 and 608.3f (archived CR texts: absent from the CR effective 2020-08-07); Wizards' "Comprehensive Rules Changes" for Zendikar Rising (Matt Tabak, 2020-09-22, per work item ruling-conflicts-1, case 7, conclusion B). The card is legal in no format (deferred, docs/DEFERRED.md); the record is kept.
- Engine: the token copies of a permanent spell are not "created".
- Recorded: exemption `docs/rulings-exemptions/rulings-P076.tsv`; test `tests/rulings/r_p076_spells_and_mana.rs` (`krosan_adaptation_storm_copy_token_is_not_created`).

### Jade Statue: Golem "in addition to those creature types"
- Kind: outdated
- Status: open for review
- Ruling, Jade Statue (2013-09-20): "If Jade Statue is animated by some other effect, you get an artifact creature with whatever power, toughness, and creature types (if any) are specified by that effect. If you subsequently use Jade Statue's ability during combat, it will become 3/6 and gain the creature type Golem in addition to those creature types until end of combat."
- Conflicts with: CR 205.1b: "... Some effects state that an object becomes a “[creature type or types] artifact creature”; these effects also allow the object to retain all of its prior card types and subtypes other than creature types, but replace any existing creature types." The card: "{2}: This artifact becomes a 3/6 Golem artifact creature until end of combat. Activate only during combat."
- Evidence: the sentence was added by the Throne of Eldraine CR (effective 2019-10-04; archived CR texts: absent from the CR effective 2019-08-23), announced in "Comprehensive Rules Changes" (Eli Shiffrin, 2019-09-27, per work item ruling-conflicts-1, case 8, conclusion B). Chimeric Staff's ruling (2009-10-01) already reset the creature types the same way. The ruling's other parts (the other effect's power, toughness and types; 3/6 by the later timestamp) are still right.
- Engine: Jade Statue animated by another effect becomes a 3/6 Golem artifact creature, losing the other effect's creature types.
- Recorded: exemption `docs/rulings-exemptions/rulings-P186.tsv`; test `tests/rulings/r_p186_animated.rs` (`jade_statue_animated_by_another_effect_becomes_a_3_6_golem`).

### Deranged Assistant: "it can't be reversed for any reason"
- Kind: outdated
- Status: open for review
- Ruling, Deranged Assistant (2025-01-24): "Once Deranged Assistant’s ability has been activated, it can’t be reversed for any reason. If you activate it while casting a spell and discover you can’t produce enough mana to pay that spell’s costs, the spell is reversed. The spell returns to whatever zone you were casting it from. You may reverse other mana abilities you activated while casting the spell, but Deranged Assistant’s ability can’t be reversed. You’ll still have any mana the ability produced, and the milled card will still be in your graveyard."
- Conflicts with: CR 605.1a: "An activated ability is a mana ability if it meets all of the following criteria: it doesn’t require a target (see rule 115.6), it could add mana to a player’s mana pool when it resolves, it’s not a loyalty ability (see rule 606, “Loyalty Abilities”), and its cost and effect don’t move any card to or from a library. ..." The card: "{T}, Mill a card: Add {C}. (Activate only as an instant. ...)". Millikin, with the same text (2026-08-05): "Millikin’s ability is not a mana ability. Players can respond to it, and it can’t be activated in the middle of paying for a cost. A recent update to rule 605.1a has made any abilities that move cards to or from libraries no longer mana abilities."
- Evidence: the library clause entered 605.1a with the CR revision effective 2026-08-07 (MagicCompRules 20260807, per work item ruling-conflicts-1, case 9, conclusion B; the archived CR effective 2026-04-17 doesn't have it, the CR in this repo, effective 2026-09-25, does). The ruling (2025-01-24; the same content in release notes since 2018-11-29) describes it as a mana ability activated while casting a spell. Its outcome still holds when the ability is activated with priority before a cast that is then reversed (CR 733.1).
- Engine: the ability isn't a mana ability: it uses the stack and can't be activated while a spell is being cast.
- Recorded: exemption `docs/rulings-exemptions/rulings-P219.tsv`; tests `tests/rulings/r_p219_mill_resolution.rs` (`millikin_is_not_a_mana_ability`, `deranged_assistant_mana_survives_a_reversed_cast`).

### Mossfire Egg and the other Eggs: "This is a mana ability"
- Kind: outdated
- Status: open for review
- Ruling, Mossfire Egg (2008-08-01): "This is a mana ability, which means it can be activated as part of the process of casting a spell or activating another ability. If that happens you get the mana right away, but you don’t get to look at the drawn card until you have finished casting that spell or activating that ability."
- Conflicts with: CR 605.1a (as for Deranged Assistant: an ability whose cost or effect moves a card to or from a library isn't a mana ability; a draw does, CR 121.1). The Eggs: "{2}, {T}, Sacrifice this artifact: Add {R}{G}. Draw a card. (Activate only as an instant.)" (Mossfire Egg). Chromatic Sphere, the same shape (2026-08-05): "Chromatic Sphere’s ability is not a mana ability. Players can respond to it, and it can’t be activated in the middle of paying for a cost. A recent update to rule 605.1a has made any abilities that move cards to or from libraries no longer mana abilities."
- Evidence: the ruling (2008-08-01) is on all five Eggs (Mossfire Egg, Skycloud Egg, Sungrass Egg, Shadowblood Egg, Darkwater Egg), written when mana abilities had no library criterion; outdated by the CR revision effective 2026-08-07 (work item ruling-conflicts-1, case 10, conclusion B). The Eggs' Oracle texts now say "(Activate only as an instant.)".
- Engine: the Eggs' abilities use the stack.
- Recorded: exemption `docs/rulings-exemptions/rulings-S22.tsv`; test `tests/rulings/r_s22_mana_abilities.rs` (`an_egg_that_draws_a_card_is_not_a_mana_ability`).

### Charmed Pendant: "The ability is still a mana ability"
- Kind: outdated
- Status: open for review
- Ruling, Charmed Pendant (2004-10-04): "The ability is still a mana ability (and doesn’t use the stack) even though you can only activate it when you could cast an instant."
- Conflicts with: CR 605.1a (as above; milling moves a card from a library, CR 701.17a). The card: "{T}, Mill a card: For each colored mana symbol in the milled card's mana cost, add one mana of that color. (Activate only as an instant. ...)"
- Evidence: the ruling (2004-10-04) predates the library criterion, added by the CR revision effective 2026-08-07 (work item ruling-conflicts-1, case 20, folded into case 10; conclusion B). "(Activate only as an instant.)" is now reminder text.
- Engine: the ability uses the stack.
- Recorded: exemption `docs/rulings-exemptions/rulings-S22.tsv`; test `tests/cards/c4_mana_symbols.rs` (`charmed_pendant_adds_one_mana_for_each_colored_symbol_of_the_milled_card`).

### Kurkesh, Onakke Ancient and Illusionist's Bracers: a mana ability's three criteria
- Kind: outdated
- Status: open for review
- Ruling, Kurkesh, Onakke Ancient (2014-07-18): "A mana ability is an ability that (1) could add mana when it resolves, (2) isn't a loyalty ability, and (3) doesn't require a target."
- Ruling, Illusionist's Bracers (2013-04-15): "A mana ability is an ability that (1) isn't a loyalty ability, (2) doesn't target, and (3) could add mana when it resolves."
- Conflicts with: CR 605.1a: "An activated ability is a mana ability if it meets all of the following criteria: it doesn’t require a target (see rule 115.6), it could add mana to a player’s mana pool when it resolves, it’s not a loyalty ability (see rule 606, “Loyalty Abilities”), and its cost and effect don’t move any card to or from a library. ..." Chromatic Sphere (2026-08-05): "Chromatic Sphere’s ability is not a mana ability. Players can respond to it, and it can’t be activated in the middle of paying for a cost. A recent update to rule 605.1a has made any abilities that move cards to or from libraries no longer mana abilities." Kurkesh: "Whenever you activate an ability of an artifact, if it isn't a mana ability, you may pay {R}. If you do, copy that ability. ..." Illusionist's Bracers: "Whenever an ability of equipped creature is activated, if it isn't a mana ability, copy that ability. ..."
- Evidence: both rulings define a mana ability by the three criteria of the CR of their time (Kurkesh's for Magic 2015, 2014-07-18; the 2013-04-15 text is also on Drain Power, Pygmy Hippo, Imprison, Burning-Tree Shaman and Tsabo's Web); the library criterion entered 605.1a with the CR revision effective 2026-08-07, as for Deranged Assistant above (work item ruling-conflicts-1, case 9). Every criterion they list is still required, and they still hold for every ability that doesn't move a card to or from a library; as definitions, they now also admit abilities that aren't mana abilities, such as Chromatic Sphere's ("{1}, {T}, Sacrifice this artifact: Add one mana of any color. Draw a card. (Activate only as an instant.)") and Deranged Assistant's ("{T}, Mill a card: Add {C}. (Activate only as an instant. ...)"), so Kurkesh and the Bracers copy those. No words of the rulings are wrong, only their completeness, so nothing is exempted: both stay cited for what they say (Kurkesh's in rulings batch P019, by the rulings-P019 reviewer; the Bracers' in S27).
- Engine: activated abilities whose cost or effect moves a card to or from a library aren't mana abilities and use the stack; Kurkesh copies Chromatic Sphere's ability and Illusionist's Bracers copies Deranged Assistant's.
- Recorded: tests `tests/rulings/r_p019_activation_copies.rs` (`kurkesh_doesnt_copy_mana_abilities`, `abilities_that_add_mana_and_draw_or_mill_are_copied_as_non_mana_abilities`), `tests/rulings/r_s27_copy_abilities.rs` (`illusionist_s_bracers_doesn_t_copy_a_mana_ability`).

### Modal double-faced cards: "can't be transformed"
- Kind: outdated
- Status: open for review
- Ruling, Valakut Awakening // Valakut Stoneforge (2020-09-25): "A modal double-faced card can't be transformed or be put onto the battlefield transformed. Ignore any instruction to transform a modal double-faced card or to put one onto the battlefield transformed."
- Ruling, Akoum Warrior // Akoum Teeth (2020-09-25): "A modal double-faced card can’t be transformed or be put onto the battlefield transformed. Ignore any instruction to transform a modal double-faced card or to put one onto the battlefield transformed."
- Conflicts with: CR 701.27a: "To transform a permanent, turn it over so that its other face is up. Only permanents represented by double-faced tokens and double-faced cards can transform." CR 712.14a: "If a spell or ability puts a double-faced card onto the battlefield “transformed” or “converted,” it enters the battlefield with its back face up. ..."
- Evidence: the note is on 73 modal double-faced cards (63 with a straight apostrophe, 10 with a curly one; dated 2020-09-25, 2021-02-05, 2024-06-07 and 2024-11-08), written under the CR that let only transforming double-faced cards and tokens transform; the Marvel's Spider-Man CR (effective 2025-09-19) let modal double-faced cards transform and be put onto the battlefield or cast transformed (archived CR texts: the CR effective 2025-07-25 still said, in 701.27a, "Only transforming tokens and permanents represented by transforming double-faced cards can transform."); Wizards' ruling of 2025-09-19 on the Spider-Man cards (Gwen Stacy // Ghost-Spider and four others): "A modal double-faced card can be transformed or be put onto the battlefield transformed. This is a change from previous rules. ..." (cited in tests/rulings/r_s17_transform_modal.rs). Work item ruling-conflicts-1, with cases 11, 12 and 21 (conclusion B).
- Engine: a modal double-faced permanent transforms if its other face has a permanent type, and one put onto the battlefield transformed enters back face up if that face is a permanent face.
- Recorded: exemption `docs/rulings-exemptions/rulings-P001.tsv`; tests `tests/rulings/r_s17_transform_modal.rs` (`a_modal_double_faced_card_transforms_only_into_a_permanent_face`), `tests/actions/a701_027_transform.rs` (`a_modal_double_faced_permanent_can_transform`).

### Oculus Whelp, Mutagen Connoisseur, Invasion of Pyrulea: modal double-faced permanents "are never transformed permanents"
- Kind: outdated
- Status: open for review
- Ruling, Oculus Whelp (2023-04-14): "A “transformed permanent” is a double-faced permanent with its back face up. Notably, modal double-faced permanents and melded permanents are never transformed permanents, no matter which faces are up."
- Ruling, Mutagen Connoisseur (2023-04-14): "A "transformed permanent" is a double-faced permanent with its back face up. Notably, modal double-faced permanents and melded permanents are never transformed permanents, no matter which faces are up."
- Ruling, Invasion of Pyrulea // Gargantuan Slabhorn (2023-04-14): "A “transformed permanent” is a double-faced permanent with its back face up. Notably, modal double-faced permanents and melded permanents are never transformed permanents, no matter which faces are up."
- Conflicts with: CR 701.27g: "Some spells and abilities refer to a “transformed permanent.” This phrase refers to a double-faced permanent on the battlefield with its back face up. ... Similarly, an object represented by more than one card, such as a melded or merged permanent, is never considered a transformed permanent, even if it has components that are back face up. ..."
- Evidence: the ruling (2023-04-14) followed 701.27g of its day ("a permanent on the battlefield with its back face up that’s also a transforming double-faced card or a transforming token", unchanged through the CR effective 2025-07-25), outdated by the same Marvel's Spider-Man CR update (effective 2025-09-19), dated in the entry for the modal double-faced cards' "can't be transformed" above. Work item ruling-conflicts-1, case 11 (conclusion B). Its melded half is still right.
- Engine: a modal double-faced permanent with its back face up is a transformed permanent; a melded one isn't.
- Recorded: exemption `docs/rulings-exemptions/rulings-P225.tsv`; tests `tests/actions/a701_027_transform.rs` (`a_transformed_permanent_is_a_double_faced_permanent_with_its_back_face_up`), `tests/rulings/r_p225_transform_faces.rs` (`only_a_double_faced_permanent_back_face_up_is_transformed`, `a_modal_double_faced_card_played_or_cast_back_face_up_is_a_transformed_permanent`).

### Corruption of Towashi: modal double-faced permanents "can't enter transformed"
- Kind: outdated
- Status: open for review
- Ruling, Corruption of Towashi (2023-04-14): "Only a transforming double-faced permanent can enter the battlefield "transformed," and only if it enters with its back face up. Notably, melded permanents can't enter transformed, and modal double-faced permanents can't enter transformed, even if they enter with their back faces up."
- Conflicts with: CR 701.27g (as above) and 712.14a; the card: "Whenever a permanent you control transforms or a permanent you control enters transformed, you may draw a card. Do this only once each turn."
- Evidence: the ruling (2023-04-14) followed 701.27g and 712.14a of its day ("transforming double-faced card"), outdated by the same Marvel's Spider-Man CR update (effective 2025-09-19), dated in the entry for the modal double-faced cards' "can't be transformed" above. Work item ruling-conflicts-1, case 12 (conclusion B). Its other points (only if it enters with its back face up; melded permanents can't enter transformed) are still right. That a modal double-faced card cast or played with its back face up "enters transformed" is the register's reading (decided by the user Nov 5).
- Engine: a modal double-faced permanent that enters with its back face up enters transformed and triggers Corruption of Towashi; a melded one doesn't.
- Recorded: exemption `docs/rulings-exemptions/rulings-P225.tsv`; register `mdfc-back-face-up-enters-transformed`; tests `tests/rulings/r_p226_corruption_of_towashi.rs` (`corruption_of_towashi_draws_when_a_modal_double_faced_card_is_cast_or_played_back_face_up`, `corruption_of_towashi_doesnt_draw_for_a_melded_permanent`).

### The Sieges: copies that aren't transforming double-faced cards
- Kind: outdated
- Status: open for review
- Ruling, Invasion of Segovia // Caetus, Sea Tyrant of Segovia (2023-04-14): "If a permanent that is represented by a transforming double-faced card becomes a copy of a Siege, it will be exiled as that Siege's triggered ability resolves, then it will be cast transformed. Note that this applies only to transforming double-faced cards, not to modal double-faced cards that can normally be played using either face."
- Ruling, Invasion of Theros // Ephara, Ever-Sheltering (2023-04-14): "If a permanent that is represented by a transforming double-faced card becomes a copy of a Siege, it will be exiled as that Siege’s triggered ability resolves, then it will be cast transformed. Note that this applies only to transforming double-faced cards, not to modal double-faced cards that can normally be played using either face."
- Ruling, Invasion of Segovia // Caetus, Sea Tyrant of Segovia (2023-04-14): "If a token or a card that isn't represented by a transforming double-faced card becomes a copy of a Siege, it can't be cast as its triggered ability resolves. It will remain in exile. If it's a token, it will cease to exist the next time state-based actions are performed."
- Ruling, Invasion of Theros // Ephara, Ever-Sheltering (2023-04-14): "If a token or a card that isn’t represented by a transforming double-faced card becomes a copy of a Siege, it can’t be cast as its triggered ability resolves. It will remain in exile. If it’s a token, it will cease to exist the next time state-based actions are performed."
- Conflicts with: CR 712.11a: "If a double-faced card or a copy of a double-faced card is cast as a spell “transformed” or “converted,” it’s put on the stack with its back face up."
- Evidence: the rulings (2023-04-14; each text on 6 or 29 Sieges, with a straight or a curly apostrophe) followed 712.11a, 712.8c and 712.11d of their day, which let only a transforming double-faced card be cast transformed (unchanged through the CR effective 2025-07-25), outdated by the same Marvel's Spider-Man CR update (effective 2025-09-19), dated in the entry for the modal double-faced cards' "can't be transformed" above. Work item ruling-conflicts-1, case 21 (conclusion B in part). Each is outdated only for a modal double-faced card; for tokens, single-faced cards and transforming double-faced cards they are still right and cited.
- Engine: a modal double-faced card that became a copy of a Siege is exiled and cast transformed (as its own back face) when the Siege is defeated.
- Recorded: exemption `docs/rulings-exemptions/rulings-P225.tsv`; tests `tests/rulings/r_s17_transform_battles.rs` (`a_modal_double_faced_card_that_became_a_copy_of_a_siege_is_cast_transformed`, `a_copy_of_a_siege_thats_not_a_transforming_double_faced_card_stays_in_exile`).

### Craft: copies that aren't transforming double-faced cards
- Kind: outdated
- Status: open for review
- Ruling, Oteclan Landmark // Oteclan Levitator (2023-11-10): "If a card that isn't a transforming double-faced card becomes a copy of a card with craft, it'll stay in exile if you activate the craft ability. It won't return to the battlefield."
- Conflicts with: CR 712.14a: "If a spell or ability puts a double-faced card onto the battlefield “transformed” or “converted,” it enters the battlefield with its back face up. If a player is instructed to put a card that isn’t a double-faced card onto the battlefield transformed or converted, that card stays in its current zone." Craft's ability ends "Return this card to the battlefield transformed under its owner’s control." (702.167a).
- Evidence: the ruling (2023-11-10, The Lost Caverns of Ixalan; the same text on all 24 cards with craft) restates 712.14a of its day: "If a spell or ability puts a transforming double-faced card onto the battlefield “transformed” or “converted,” it enters the battlefield with its back face up. If a player is instructed to put a card that isn’t a transforming double-faced card onto the battlefield transformed or converted, that card stays in its current zone." (the CR effective 2023-09-01, in force when the ruling was written, and unchanged through the CR effective 2025-07-25; archived CR texts). The Marvel's Spider-Man CR update (effective 2025-09-19) dropped "transforming" from both sentences, as dated in the entry for the modal double-faced cards' "can't be transformed" above; Wizards' ruling of 2025-09-19 on five Spider-Man cards (Gwen Stacy // Ghost-Spider and others): "Similarly, if an effect attempts to put a modal double-faced card onto the battlefield transformed, it will enter transformed if its back face has a permanent type. If it doesn't, it will simply stay in its current zone." So a modal double-faced permanent that became a copy of a card with craft returns to the battlefield as its own back face, unless that face is an instant or sorcery, as the Sieges' copies are now cast transformed (above). The ruling is outdated only for modal double-faced cards: a card that isn't double-faced at all (Phyrexian Metamorph) still stays in exile, and the ruling stays cited for it. Work item ruling-conflicts-3, case 4 (conclusion B in part).
- Engine: craft returns the exiled card with its back face up if it's a double-faced card whose back face is a permanent face, modal or not; a modal double-faced card with an instant or sorcery back face and a card that isn't double-faced stay in exile (712.14a, 304.4, 307.4).
- Recorded: exemption `docs/rulings-exemptions/rulings-P204.tsv`; tests `tests/rulings/r_s03_craft.rs` (`a_modal_double_faced_card_that_became_a_copy_of_a_craft_card_returns_as_its_own_back_face`, `a_copy_of_a_craft_card_that_isnt_double_faced_stays_in_exile`), `tests/keywords/k702_167_craft.rs` (`a_copy_that_isnt_a_double_faced_card_stays_in_exile`).

### Norn's Inquisitor and Corruption of Towashi: "Only transforming double-faced permanents ... can transform"
- Kind: outdated
- Status: open for review
- Ruling, Norn's Inquisitor (2023-04-14): "Only transforming double-faced permanents (including transforming double-faced cards and Incubator tokens) can transform. A face-up permanent turning face down doesn't count as transforming, nor does a face-down permanent turning face up."
- Ruling, Corruption of Towashi (2023-04-14): "Similarly, only transforming double-faced permanents (including transforming double-faced cards and Incubator tokens) can transform. A face-up permanent turning face down doesn't count as transforming, nor does a face-down permanent turning face up."
- Conflicts with: CR 701.27a: "... Only permanents represented by double-faced tokens and double-faced cards can transform."
- Evidence: the rulings' first sentence (2023-04-14) followed 701.27a of its day, outdated by the same Marvel's Spider-Man CR update (effective 2025-09-19), dated in the entry for the modal double-faced cards' "can't be transformed" above. Their second sentence (turning face up or face down isn't transforming) is still right and cited, with a dated comment (work item ruling-conflicts-1, with cases 11, 12 and 21).
- Engine: a modal double-faced permanent can transform (to a permanent face); turning a permanent face up isn't transforming.
- Recorded: no exemption: cited for the second sentence; tests `tests/rulings/r_p225_transform_faces.rs` (`turning_a_permanent_face_up_isnt_transforming`), `tests/actions/a701_027_transform.rs` (`a_modal_double_faced_permanent_can_transform`).

### The Golden-Gear Colossus: "only transforming double-faced cards and transforming tokens can transform"
- Kind: outdated
- Status: open for review
- Ruling, Tetzin, Gnome Champion // The Golden-Gear Colossus (2023-11-10): "You can target any double-faced artifact you control with The Golden-Gear Colossus's triggered ability, but only transforming double-faced cards and transforming tokens can transform. Choosing an artifact represented by a modal double-faced card (Pestilent Cauldron from Strixhaven, for example) won't cause that permanent to transform."
- Conflicts with: CR 701.27a (as above). The Golden-Gear Colossus: "Whenever The Golden-Gear Colossus enters or attacks, transform up to one other target double-faced artifact you control. ..."
- Evidence: the ruling (2023-11-10) gives the old rule as its reason, outdated by the same Marvel's Spider-Man CR update (effective 2025-09-19), dated in the entry for the modal double-faced cards' "can't be transformed" above. Its example still holds for another reason: Pestilent Cauldron's other face, Restorative Burst, is a sorcery, and a permanent doesn't transform into an instant or sorcery face (CR 701.27d). A modal double-faced artifact whose other face is a permanent face (Tergrid's Lantern, the back face of Tergrid, God of Fright) would transform. Cited for its example, with a dated comment (work item ruling-conflicts-1, with cases 11, 12 and 21).
- Engine: the Colossus can target any double-faced artifact you control; a modal double-faced one transforms only if its other face is a permanent face.
- Recorded: no exemption: cited for its example; test `tests/rulings/r_p226_golden_gear_colossus.rs` (`golden_gear_colossus_can_target_pestilent_cauldron_but_it_doesnt_transform`).

### Dance of the Dead: two Dances on one creature
- Kind: outdated
- Status: open for review
- Ruling, Dance of the Dead (2004-10-04): "If more than one Dance of the Dead ends up on a creature, each contributes a +1/+1. But you only have to pay the untap cost once. You may pay for each one, however, and untap the card more than once during upkeep."
- Conflicts with: the card's Oracle text since the Masters Edition Update Bulletin (Mark L. Gottlieb, 2007-08-29, per work item ruling-conflicts-1, case 14): "Enchant creature card in a graveyard / When this Aura enters, if it's on the battlefield, it loses "enchant creature card in a graveyard" and gains "enchant creature put onto the battlefield with this Aura." ..."; its later ruling (2008-04-01): "Once the creature is returned to the battlefield, Dance of the Dead can't be attached to anything other than it (unless Dance of the Dead somehow manages to put a different creature onto the battlefield). Attempting to move Dance of the Dead to another creature won't work."
- Evidence: the ruling (2004-10-04) was written for the older wording, a generic "enchant creature" Aura that another effect (Aura Graft) could move onto a creature another Dance enchanted. The current "enchant creature put onto the battlefield with this Aura" is linked to its own trigger (CR 607.1a, 607.2c), so another Dance can't be moved onto the creature (303.4j) or created attached to it (707.2, 303.4i). Work item ruling-conflicts-1, case 14 (conclusion B).
- Engine: a second Dance of the Dead can't be moved onto or created attached to the first Dance's creature.
- Recorded: exemption `docs/rulings-exemptions/gap-zone-moves.tsv`; test `tests/rulings/animate_dead.rs` (`a_second_dance_of_the_dead_cant_join_the_first_on_its_creature`).

### Lich: "leaves the battlefield"
- Kind: outdated
- Status: open for review
- Ruling, Lich (2004-10-04): "If you have multiple Lich cards on the battlefield, you must sacrifice a permanent for each damage done to you for each Lich. This is because the sacrifice is a triggered ability. But you only draw one card for each life gained regardless of how many Liches you have. This is because the draw is a replacement effect and not a triggered one. You lose if any one of the Liches leaves the battlefield."
- Ruling, Lich (2010-08-15): "The last ability will cause you to lose the game even if you somehow manage to have a life total greater than 0 at the time the Lich leaves the battlefield or if some other effect would prevent you from losing for having 0 life. If, on the other hand, some effect such as that from Platinum Angel says that you can't lose the game then even the last ability of the Lich cannot cause you to do so. The ability will just resolve and the game will continue as normal."
- Conflicts with: the card's Oracle text since the September 2010 Update Bulletin (Matt Tabak, 2010-09-22; the Oracle update was announced for 2010-09-24): "When this enchantment is put into a graveyard from the battlefield, you lose the game." Wizards' rewrite of the second ruling (2011-01-01): "The last ability will cause you to lose the game even if you somehow manage to have a life total greater than 0 at the time the Lich is put into the graveyard ...".
- Evidence: both rulings were written for "When Lich leaves play/the battlefield, you lose the game" (2004-10-04, Gatherer's first batch; 2010-08-15, the Magic 2011 rulings); the September 2010 bulletin narrowed it ("the printed card says you lose when Lich is destroyed, but that was broadened to Lich leaving the battlefield"), and only the second ruling was reworded (2011-01-01) while the 2010-08-15 version stays published. Work item ruling-conflicts-2, case 3 (conclusion B in part). The rest of both rulings is still right and cited.
- Engine: only a Lich put into a graveyard from the battlefield makes you lose; exiled, bounced or under Rest in Peace, it doesn't.
- Recorded: exemption `docs/rulings-exemptions/rulings-P096.tsv`; tests `tests/rulings/r_p096_lich.rs` (`lich_leaving_the_battlefield_except_to_a_graveyard_doesnt_trigger_its_last_ability`, `multiple_liches_sacrifice_for_each_but_draw_only_once`, `lich_last_ability_loses_whatever_the_life_total_unless_you_cant_lose`).

### Sarkhan the Mad: "whether or not to redirect the damage dealt by that Dragon"
- Kind: outdated
- Status: open for review
- Ruling, Sarkhan the Mad (2010-06-15): "If you target an opponent with the third ability, you choose separately for each Dragon creature you control whether or not to redirect the damage dealt by that Dragon to a planeswalker that player controls."
- Conflicts with: CR 306.7: "Previously, planeswalkers were subject to a redirection effect that allowed a player to have noncombat damage that would be dealt to an opponent be dealt to a planeswalker under that opponent’s control instead. This rule has been removed and certain cards have received errata in the Oracle card reference to deal damage directly to planeswalkers." The card's Oracle text: "−4: Each Dragon creature you control deals damage equal to its power to target player or planeswalker."
- Evidence: the ruling applied 306.7 of the CRs from 2010 to early 2018 (CR effective 2010-04-23 and still in the CR effective 2018-01-19: "If noncombat damage would be dealt to a player by a source controlled by an opponent, that opponent may have that source deal that damage to a planeswalker the first player controls instead. This is a redirection effect ..."). The Dominaria CR (effective 2018-04-27) replaced it with today's text (all from the archived CR texts). Work item rulings-P008.
- Engine: the Dragons deal their damage to the target, the player or the planeswalker; no redirection choice is asked.
- Recorded: exemption `docs/rulings-exemptions/rulings-P008.tsv`; test `tests/rulings/r_p008_planeswalkers.rs` (`sarkhan_the_mad_dragons_damage_the_target_with_no_redirection`).

### Manticore of the Gauntlet: "If you can't choose a target opponent"
- Kind: outdated
- Status: open for review
- Ruling, Manticore of the Gauntlet (2017-04-18): "If you can't choose a target opponent when you put Manticore of the Gauntlet's triggered ability on the stack (most likely because your opponents have hexproof), the ability is removed from the stack and you won't put a -1/-1 counter on any creature."
- Conflicts with: the card's Oracle text: "When this creature enters, put a -1/-1 counter on target creature you control. This creature deals 3 damage to target opponent or planeswalker." With its opponents hexproof, a planeswalker can still be targeted (one its controller controls too), and CR 603.3d removes a triggered ability from the stack only "If a choice is required when the triggered ability goes on the stack but no legal choices can be made for it".
- Evidence: the ruling is the Amonkhet release notes' (2017-04-14), written for the printed text they quote: "When Manticore of the Gauntlet enters the battlefield, put a -1/-1 counter on target creature you control. Manticore of the Gauntlet deals 3 damage to target opponent." The Dominaria Oracle Changes (Eli Shiffrin, 2018-04-13; the Oracle update that went with the Comprehensive Rules effective 2018-04-27, which removed planeswalker damage redirection, see the Sarkhan the Mad entry) list Manticore of the Gauntlet among the cards given errata to deal damage to planeswalkers: "Abilities that damage “target opponent” have been changed to “target opponent or planeswalker” with the same exception listed above. These spells and abilities can target a planeswalker you control." (both from the ormos mirror). Found in the review of work item rulings-P008. What the ruling says happens when the ability has no legal target (removed from the stack, no -1/-1 counter) still holds and is cited.
- Engine: with P1 hexproof, the ability targets a planeswalker of P1's, or P0's own when it's the only one, and is put on the stack; with no planeswalker on the battlefield either, it's removed from the stack and no -1/-1 counter is put on a creature.
- Recorded: exemption `docs/rulings-exemptions/rulings-P008.tsv`; tests `tests/rulings/r_p008_targets.rs` (`etb_burn_with_no_legal_opponent_isnt_put_on_the_stack`, `manticore_of_the_gauntlet_targets_a_planeswalker_when_its_opponents_have_hexproof`).

### The Fallen: "only to opponents of its current controller"
- Kind: outdated
- Status: open for review
- Ruling, The Fallen (2007-09-16): "The Fallen’s ability causes it to deal damage only to opponents of its current controller. If The Fallen has dealt damage to you, and then you gain control of it, you won’t be damaged during your upkeep."
- Conflicts with: the card's Oracle text: "At the beginning of your upkeep, this creature deals 1 damage to each opponent and planeswalker it has dealt damage to this game." A planeswalker it has dealt damage to is dealt damage whoever controls it, The Fallen's controller included.
- Evidence: the ruling is older than planeswalkers (Lorwyn, 2007-10-12) and was written for an ability that dealt damage only to opponents; the Dominaria Oracle Changes (Eli Shiffrin, 2018-04-13; the Oracle update that went with the Comprehensive Rules effective 2018-04-27, which removed planeswalker damage redirection) list The Fallen among the cards given errata for that change (ormos mirror), which gave it today's "each opponent and planeswalker". Found in the review of work item rulings-P008. The second sentence still holds and is cited, and so does the first among players.
- Engine: under P1's control, The Fallen deals 1 damage at P1's upkeep to P1's own planeswalker it dealt damage to earlier, and none to P0, whom it has never dealt damage to.
- Recorded: exemption `docs/rulings-exemptions/rulings-P008.tsv`; tests `tests/rulings/r_p008_combat_lki.rs` (`the_fallen_damages_only_its_current_controllers_opponents`, `the_fallen_damages_the_planeswalkers_it_has_dealt_damage_to`).

### Ronom Hulk: "though there are no snow instants or sorceries"
- Kind: outdated
- Status: open for review
- Ruling, Ronom Hulk (2006-07-15): "Ronom Hulk can’t be targeted by snow spells (though there are no snow instants or sorceries) or abilities of snow sources, it can’t be enchanted by snow Auras, it can’t be equipped by snow Equipment (though there are none), it can’t be blocked by snow creatures, and all damage that would be dealt to it by snow sources is prevented."
- Conflicts with: the card pool since Kaldheim (released 2021-02-05), which printed snow instants and sorceries: Frost Bite and Graven Lore ("Snow Instant"), Blizzard Brawl, Blessing of Frost, Blood on the Snow, Search for Glory and Tundra Fumarole ("Snow Sorcery"), all with Kaldheim as their first set in the data.
- Evidence: the ruling was written for Coldsnap (Ronom Hulk's set, 2006), when the snow supertype appeared only on permanents. Only the parenthetical aside is outdated; it states a fact about the cards printed, not a rule. The second aside ("snow Equipment (though there are none)") still holds in the data. Found in work item rulings-P087.
- Engine: protection from snow applies to snow spells like any other quality (CR 702.16b): Frost Bite can't target Ronom Hulk.
- Recorded: exemption `docs/rulings-exemptions/rulings-P087.tsv` (the first aside); test `tests/rulings/r_p087_mechanics.rs` (`ronom_hulk_protection_from_snow`).

### Green Dragon: "before dealing damage to another blocking creature"
- Kind: outdated
- Status: open for review
- Ruling, Green Dragon (2021-07-23): "Green Dragon's Poison Breath ability is different than deathtouch. Notably, creatures still need to deal damage to a blocking creature equal to its toughness before dealing damage to another blocking creature or trampling over to a player or planeswalker."
- Conflicts with: CR 510.1c: "A blocked creature assigns its combat damage to the creatures blocking it. ... If two or more creatures are blocking it, it assigns its combat damage to those creatures divided as its controller chooses among them." (since the CR effective 2024-11-08).
- Evidence: the ruling was published with Adventures in the Forgotten Realms (2021-07-23) under the CR effective 2021-07-23, whose 510.1c had a damage assignment order: "If two or more creatures are blocking it, it assigns its combat damage to those creatures according to the damage assignment order announced for it. This may allow the blocked creature to divide its combat damage. However, it can’t assign combat damage to a creature that’s blocking it unless, when combat damage assignments are complete, each creature that precedes that blocking creature in its order is assigned lethal damage." It is unchanged in the CR effective 2024-09-20; the CR effective 2024-11-08 (Magic Foundations) replaced it with today's text (archived CR texts). Under the old rule a creature couldn't spread its damage over several blockers to have Poison Breath destroy them all, as one with deathtouch could (1 damage being lethal damage then, CR 702.2c); now it can. The rest of the ruling still holds: a creature with trample must assign each blocker lethal damage (its toughness, not 1 as with deathtouch) before the rest tramples over (CR 702.19b). Found in the review of work item rulings-P048 (Nov 6).
- Engine: a creature blocked by several creatures divides its combat damage among them as its controller chooses (CR 510.1c): with Poison Breath, the Grizzly Bears' 1 damage to each of two blocking Hill Giants destroys both; with trample, lethal damage to a blocker is its toughness.
- Recorded: exemption `docs/rulings-exemptions/rulings-P048.tsv`; tests `tests/rulings/r_p048_dnd.rs` (`green_dragons_poison_breath_outlasts_it_and_isnt_deathtouch`, `green_dragons_poison_breath_with_damage_divided_among_blockers`).

## Not yet investigated

Possible further instances found while compiling this log. They haven't been through the
conflict process (an investigator and a skeptic), so they aren't entries yet; the coordinator
decides whether they become a conflict case.

- **Fiery Annihilation** (rulings-P047): its ruling (2024-11-08) that the target Equipment
  isn't exiled whenever the target creature is an illegal target, against Searing Blaze's
  (2010-03-01) and Memory's Journey's (2011-09-22) that a target described by another illegal
  target ("target creature that player ... controls", "target cards from their graveyard") is
  still affected or still legal. The two readings of CR 608.2b's last sentences ("If part of the
  effect requires information about an illegal target, it fails to determine any such
  information. ...") and a lead to check against the archived rules (whether that sentence
  replaced one that said the opposite after 2011, which would make the older rulings
  outdated) are in `docs/agents/engine-gaps.md`, "Apparent conflicts (Nov 5)". Left uncited.
- **Stone Calendar** (rulings-P033): its ruling "You may choose not to apply Stone Calendar’s
  cost reduction effect." (2004-10-04) against its Oracle text "Spells you cast cost {1}
  less to cast." (no "up to"; a reduction is part of the total cost, CR 601.2f, with no
  choice to decline it, and Semblance Anvil's "Semblance Anvil's cost reduction effect is
  mandatory." (2011-01-01) says so for the same wording). Likely origin: the printed text
  "Your spells cost up to 1 less to cast" (as other databases and Scryfall's Italian
  printing still show it), which the English Oracle text no longer has; the date of that
  Oracle change wasn't found. Left uncited; the engine always applies the reduction
  (`tests/rulings/r_p033_generic_only.rs`, `stone_calendar_reduces_what_you_pay_not_the_mana_cost`).
- Work item ruling-conflicts-3 took up the craft copy ruling (now an
  outdated entry above), Run Out of Town and Sanguine Spy (copying errors), Caller of
  the Untamed (the ruling agrees with CR 601.2b; the engine was changed), and Araumi of the
  Dead Tide and Arbiter of Knollridge (inconsistent entries above, set aside by the user Nov 5).
  It also compared the 69 release-notes files in the ormos mirror (2013 to 2022) with the
  data, card by card, for notes the data has with at most four words changed (about 1,100,
  most for grammar such as "they" for "he or she", or later rules updates). Of the 25 or so
  whose changed words are about who, where or what (owner, controller, opponent, graveyard,
  library, can or can't), read by hand, the others are rewordings or corrections that bring
  the note in line with the card (Failure // Comply's "Opponents" for "Players", Choose Your
  Demise's "library" for "graveyard"); only Run Out of Town's and Sanguine Spy's contradict
  it. The rest weren't read one by one.
