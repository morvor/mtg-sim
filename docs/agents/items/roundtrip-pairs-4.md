# roundtrip-pairs-4: the next 150 untriaged mismatches from the long tail

Part of the `roundtrip-followups.md` stream. Baseline (Oct 29, origin 2934da1b, after
roundtrip-pairs-3 landed): 28,198 of 31,972 fully supported cards pass. Almost every
remaining mismatch is a single-card or two-card cluster. roundtrip-pairs-3 took the first 80
two-card clusters in the tool's order. Runs alone.

SCOPE: pick about 150 mismatching cards that no earlier roundtrip item's "Taken" or
"Remaining" section lists (grep docs/agents/items/roundtrip-*.md): first every untriaged
two-card cluster, then single-card clusters in the tool's order, until about 150 cards. Group
them by the kind of difference before working (the same renderer or parser construct across
several cards is the efficient unit). Write the chosen list (card, cluster key) into this
brief's "Taken" section and commit it first.

For each card: triage — parser bug first (does the AST mean what the text says, in
multiplayer, with control changes, tokens, copies, replacement effects, zero-quantity
choices, ordering of instructions where a replacement effect could observe it?) with a fix +
in-game test in `tests/cards/roundtrip_pairs_4.rs` that fails on the old code + a row in the
bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else renderer wording
(independently of the parser), or a CR-justified equivalence. Judgment calls go in the
interpretation register (CLAUDE.md). Add "Parser bugs found" and "Remaining" sections.
Regenerate docs/ROUNDTRIP.md and docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Taken

From `roundtrip --clusters 2000` on origin 2934da1b (28,198 of 31,972 pass). Cards named in an
earlier roundtrip item's brief are left out (e.g. Oblivious Bookworm, Taste of Paradise, Suspend
Aggression, Desperate Research). The untriaged cards of the 3+-card clusters, then the
untriaged two-card clusters in the tool's order up to about 150 cards (151):

- gap: Filter: Shadow the Hedgehog; Protective Sphere
- gap: Value: Boreal Outrider; Vazi, Keen Negotiator
- gap: replacement Discriminant: Island Sanctuary; Rain of Gore; Ojer Axonil, Deepest Might // Temple of Power
- gap: the triggering object outside a triggered ability: Stolen Uniform; Myra the Magnificent; Chancellor of the Annex
- oracle `share a card type with` vs rendered `is a permanent that share`: Cemetery Protector; Amareth, the Lustrous; Cemetery Gatekeeper
- gap: Effect: Helm of Obedience; Extrapolate the Impossible
- gap: TriggerCond: Caged Sun; Tenuous Truce
- gap: who may activate: Personal Incarnation; Martyrdom
- oracle `a third of it life` vs rendered `x life where x is`: Pox; Dire Fleet Ravager
- oracle `for each of it` vs rendered `∅`: Kambal, Profiteering Mayor
- oracle `may cast any number of` vs rendered `{opt:may} cast any number of`: Etali, Primal Conqueror // Etali, Primal Sickness
- oracle `may exile a nonland card` vs rendered `{alt:choose up to one nonland card from it|may choose a nonland card from it} exile it it owner`: Invasion of Gobakhan // Lightshield Array; Elite Spellbinder
- oracle `named ~ you control` vs rendered `you control named ~`: Food Fight; Seven Dwarves
- oracle `no opponent control a white` vs rendered `your opponent control no white`: Kavu Runner; Skittish Kavu
- oracle `number` vs rendered `amount`: Verazol, the Split Current; Search for Glory
- oracle `of a artifact creature or` vs rendered `that is not a mana`: Harsh Mentor; Immolation Shaman
- oracle `of a artifact or creature` vs rendered `that is not a mana`: Crackdown Construct; Ashnod the Uncaring
- oracle `of it to the battlefield` vs rendered `to the battlefield tapped {opt:under your control}`: Viewpoint Synchronization; Flourishing Bloom-Kin
- oracle `of it with that name` vs rendered `card with {alt:the chosen name|that name} {alt:from among it|revealed this way} to`: Wood Sage
- oracle `or ability you control counter` vs rendered `is countered by a spell`: Lullmage Mentor; Baral, Chief of Compliance
- oracle `or activate ability of artifact` vs rendered `during your turn your opponent`: Myrel, Shield of Argive; Grand Abolisher
- oracle `or crew a vehicle during` vs rendered `{alt:or|and when|or when} ~it crew a vehicle`: Reckless Velocitaur; Canyon Vaulter
- oracle `or lost life this turn` vs rendered `life this turn or you`: Starseer Mentor; Star Charter
- oracle `or suspended card put 2` vs rendered `{opt:on the battlefield} or card with suspend`: Shivan Sand-Mage; Timebender
- oracle `or` vs rendered `spell or a`: Firespitter Whelp; Harnesser of Storms
- oracle `other artifact you own dy` vs rendered `{alt:other artifact you own dies|other artifact you own dies} {alt:|and when|or when} a artifact card`: Ultron the Annihilator; Ultron's Auxiliary
- oracle `other card you own exiled` vs rendered `card you own exiled with`: Duplicity; Moonring Mirror
- oracle `other creature when you do` vs rendered `{alt:other creature|a creature other than ~it} when you do ~it`: Bolg of the North; Surtland Flinger
- oracle `other than` vs rendered `not who is`: Fractured Identity; Death by Dragons
- oracle `other` vs rendered `a`: Hymn of the Faller
- oracle `otherwise it get -2/-1` vs rendered `enchanted creature get -2/-1 as`: Serra's Boon; Tahngarth's Rage
- oracle `otherwise it get -2/-2` vs rendered `enchanted creature get -2/-2 as`: Nyx Infusion; Gift of Fangs
- oracle `owner` vs rendered `{alt:its|that player's}`: Patient Rebuilding
- oracle `pay {3} or` vs rendered `{3} equip`: Bloodthorn Flail; Gavel of the Righteous
- oracle `player skip it` vs rendered `skip a player`: Eon Hub; Stasis
- oracle `prevent each damage that would` vs rendered `if a source you do`: Comeuppance; Channel Harm
- oracle `prevent the next 3 damage` vs rendered `choose a permanent source if`: Refraction Trap; Healing Grace
- oracle `put a loyalty counter on` vs rendered `if it is a planeswalker`: Teyo, Lightshield Expert; Teyo, Diamondblade Mage
- oracle `rather than pay ~ mana` vs rendered `may {opt:have} each other player`: Reverent Silence; Skyshroud Cutter
- oracle `red green white` vs rendered `white red green`: Hazezon, Shaper of Sand; Sand Scout
- oracle `room ability of dungeon you` vs rendered `if a triggered ability of`: Dungeon Delver; Hama Pashar, Ruin Seeker
- oracle `sacrifice 2 untapped land instead` vs rendered `may sacrifice 2 untapped land`: Lotus Vale; Scorched Ruins
- oracle `shuffle enchanted creature` vs rendered `the owner of enchanted creature`: Stay Hidden, Stay Silent; Dramatic Accusation
- oracle `source you control would deal` vs rendered `you control would deal damage`: Pyromancer's Swath; Calamity Bearer
- oracle `spell cast cost {1} less` vs rendered `if it is not your`: Naiad of Hidden Coves; Geyser Drake
- oracle `spell cast this way would` vs rendered `card among it would be`: Scholar of the Lost Trove; Bilbo, Thief in the Night
- oracle `tapped for mana plain produce` vs rendered `a plain land is tapped`: Reality Twist; Naked Singularity
- oracle `target creature a opponent control` vs rendered `{alt:target creature's owner {alt:an opponent controls|your opponents control}|target creature {alt:an opponent controls|your opponents control}'s owner}`: Trickster's Stratagem; Happy Hogan, Bodyguard
- oracle `target creature gain protection from` vs rendered `{alt:target creature's controller|that player} choose a color it`: Pale Wayfarer; Wishmonger
- oracle `target player search it library` vs rendered `{alt:that player|it} search it library for`: Fertilid; Fertilid's Favor
- oracle `target spell controller where x` vs rendered `it controller where x is`: Refuse // Cooperate; Parallectric Feedback
- oracle `that do not have` vs rendered `without`: Balduvian Shaman; Coiling Stalker
- oracle `that each have` vs rendered `with`: Catapult Fodder // Catapult Captain; Tiamat
- oracle `that player choose target player` vs rendered `{alt:that player|it} choose target player who`: Oath of Ghouls; Oath of Druids
- oracle `that player control more land` vs rendered `it is not true that`: Rivalry; Greener Pastures
- oracle `that player control that didn't` vs rendered `{alt:that player|it} control that didn't attack`: Total War; Angel's Trumpet
- oracle `that player have more card` vs rendered `the chosen {alt:opponent|player} have more`: Slithermuse; Sandstone Oracle
- oracle `that player may draw a` vs rendered `{alt:that player|it} may draw a card`: Ludevic, Necro-Alchemist; Academy Loremaster
- oracle `that player put it` vs rendered `{alt:the player|that player} put thatit to owner`: Balustrade Spy; Undercity Informer
- oracle `that player reveal the top` vs rendered `{alt:that player|it} reveal the top card`: Gate to the Aether; Shapeshifter's Marrow
- oracle `that player sacrifice a creature` vs rendered `{alt:that player|it} sacrifice a creature if`: Wasitora, Nekoru Queen; Woebringer Demon
- oracle `that player` vs rendered `it controller`: Mogg Infestation; Fall from Favor
- oracle `that player` vs rendered `{alt:thatit's controller|it is controller}`: Thunderbreak Regent; Ashenmoor Liege
- oracle `that was milled this turn` vs rendered `milled this turn to the`: Tato Farmer
- oracle `thatit card power y is` vs rendered `{alt:thatit's|each card exiled with ~it's} power y is {alt:thatit's|each card exiled with ~it's}`: Phyrexian Ingester; Bioplasm
- oracle `thatit deal that damage` vs rendered `that damage is dealt`: Jade Monolith; Aegis of Honor
- oracle `thatit on the bottom of` vs rendered `each card {alt:from among it|revealed this way} {alt:on the bottom of owner's library|on the bottom} in`: Lost in the Woods; Calibrated Blast
- oracle `thatit with the least power` vs rendered `{alt:a creature with the least power|a creature with the least power among creatures|a creature with the least power among creatures on the battlefield} {alt:it|a creature destroyed this way} can't be regenerated`: Drop of Honey; Porphyry Nodes
- oracle `thatit x is thatit` vs rendered `{alt:thatit|each card exiled with ~it} activate only if {alt:thatit|each card exiled with ~it}`: Soul Foundry; Prototype Portal
- oracle `thatit` vs rendered `creature you control`: Deadly Wanderings; Homicidal Seclusion
- oracle `thatit` vs rendered `the exiled {alt:card's|cards'}`: Pit of Offerings; Chrome Mox
- oracle `thatit` vs rendered `the top card of your`: Skill Borrower; Conspicuous Snoop
- oracle `the active player have controlled` vs rendered `that player control {alt:its controller has controlled continuously since the beginning of the turn|that player has controlled continuously since the beginning of the turn} attack`: Norritt; Nettling Imp
- oracle `the amount of life you` vs rendered `{alt:that much|that many}`: Field-Tested Frying Pan; Hurska Sweet-Tooth
- oracle `the base power toughness of` vs rendered `{opt:each} other creature you control`: Sita Varma, Masked Racer; Tanazir Quandrix
- oracle `the color of your choice` vs rendered `that color`: Nykthos, Shrine to Nyx; Nyx Lotus
- oracle `the last time counter is` vs rendered `a time counter is removed`: Veiling Oddity; Riftmarked Knight

## Result

28,198 of 31,972 fully supported cards passed at the start (2934da1b); 28,314 of 31,972 on
the branch before merging origin, 28,348 of 32,023 after merging it (origin's passing list
at that merge had 28,198 cards), and 28,352 of 32,040 after merging again. 92 of the 151
taken cards pass.

Parser (three fixes, see "Parser bugs found"): `phrases::named_this_card` ("named ~" is the
card's name, in `phrases.rs`, `patterns/statics.rs`, `cost_parts.rs`, `etb_choices.rs`,
`conditions_this_way.rs`, `keyword_statics_2.rs`); `patterns/replacement_bodies.rs`
(`enter_only_if_sacrificed`); `patterns/triggers_ability_of_activated.rs` and
`patterns/triggers_activate_ability.rs` (of a permanent).

Renderer (most in the new `oracle/render/pairs_terms.rs`, hooks in the existing files):
color lists in wheel order for three and four colors ("red, green, and white", wedges
"white, black, and green"); "no opponent controls a white or blue creature"; "the number of
{S} spent"; "an ability of an artifact or creature that isn't a mana ability" (either
order) and "activates an ability" of anything; "a spell or ability you control counters a
spell"; "if you gained or lost life this turn"; "suspended card" (CR 702.62b); "a
noncreature or Dragon spell"; "each player other than ..."; "Players skip their upkeep
steps"; "Enchanted creature gets +1/+2 as long as it's white. Otherwise, it gets -2/-1."
(two statics, one line); "As long as you control exactly one creature, that creature ...";
"During your turn, your opponents can't cast spells or activate abilities of ..." (two
restrictions, one line); "Room abilities of dungeons you own trigger an additional time"
(CR 309.4c); "for each land card put into their graveyard this way"; "Put a loyalty counter
on it if it's a planeswalker" (the trailing-condition rewrite now also reaches the second
of two such sentences); "rather than pay ~'s mana cost, you may ..."; "an instant or
sorcery source you control" (damage replacements); "During turns other than yours, spells
you cast cost {1} less"; "gains protection from the color of its controller's choice";
"Choose a color. Add an amount of mana of that color equal to your devotion ..."; "Target
player searches their library ..." (the target named by the subject); "deals damage to
target spell's controller equal to that spell's mana value"; "that doesn't have" for
"without"; "controls more lands than each other player"; "a player other than you lost
life this turn"; "..., then puts those cards into their graveyard" (reveal until);
"If the player can't" after a mandatory sacrifice by another player; "that player" for
the controller of a spell an opponent controls that targeted; "that was milled this turn";
"a source of your choice that shares a color with the mana spent on this activation cost";
"Y is its toughness"; "that source deals that damage to you instead" (CR 614.9); "X is
the mana value of that card" (Prototype Portal's ruling); "the exiled cards' colors";
"the active player has controlled continuously since the beginning of the turn"; "the
amount of life you gained"; "loses a third of their life, rounded up"; "for each of them,
create a tapped token that's a copy of it"; "Shuffle enchanted creature into its owner's
library"; "When the last time counter is removed from ~ while it's exiled" (CR 702.62a);
"if it shares a card type with the exiled card" (any object, not "a permanent that");
"Only ~'s owner may activate this ability"; "if mana from a Treasure was spent to cast it
or activate it", "if {S} of any of that spell's colors was spent to cast it", "Each spell
you cast has split second if mana from an artifact was spent to cast it"; "If a spell or
ability would cause its controller to gain life, ..."; the opening-hand delayed trigger's
"counter that spell"; "put two of them onto the battlefield tapped and the other into your
hand"; "the player" for the event's player on older cards; "another artifact is put into
your graveyard from the battlefield or an artifact card is put into your graveyard from
anywhere other than the battlefield" (one trigger condition per zone, CR 400.1).

## Parser bugs found

Tests in `crates/mtg-engine/tests/cards/roundtrip_pairs_4.rs`; each fails on the old code
(checked by restoring the old parser files and running the tests):

- Seven Dwarves (and every "named ~" that isn't "card named ~": Plague Rats, Relentless
  Rats, Hare Apparent, "not named ~" in Detention Sphere, Tiamat, ...): "for each other
  creature named ~ you control" was compiled as "with the same name as ~". A name in the
  text is that name (CR 201.2; 201.5 is only about the object referring to itself), so a
  Seven Dwarves that Spy Kit gives every nonlegendary creature's name counted Grizzly
  Bears. `phrases::named_this_card` (the card's name) in the places that read "named ~";
  test `seven_dwarves_counts_creatures_named_seven_dwarves_not_ones_sharing_another_name`.
- Lake of the Dead, Heart of Yavimaya, Soldevi Excavations, Balduvian Trading Post,
  Kjeldoran Outpost, Lotus Vale, Scorched Ruins: "If ~ would enter, sacrifice a Swamp
  instead. If you do, put ~ onto the battlefield. If you don't, put it into its owner's
  graveyard." was compiled like Mox Diamond's "you may discard a land card instead", an
  offer: a player could keep the Swamp and put the land into the graveyard. Now the
  sacrifice is done if it can be, then "if you do" checks it was done in full
  (`patterns/replacement_bodies.rs`, `enter_only_if_sacrificed`); with fewer untapped lands
  than Lotus Vale needs, those it can are sacrificed (CR 609.3) and it doesn't enter
  (register entry `enter-sacrifice-instead-partial`, provisional). Tests
  `lake_of_the_dead_sacrifices_a_swamp_without_asking`,
  `lotus_vale_with_one_untapped_land_sacrifices_it_and_never_enters`; the existing
  `lotus_vale_enters_only_if_two_untapped_lands_are_sacrificed`
  (`tests/cards/replacement_bodies_misc.rs`) pinned the offer and now pins the instruction.
- Crackdown Construct, Kurkesh, Onakke Ancient, Ashnod the Uncaring: "Whenever you activate
  an ability of an artifact or creature that isn't a mana ability" (and "... of an
  artifact, if it isn't a mana ability") also triggered for abilities of cards in other
  zones, such as a creature card's cycling from a hand (Crackdown Construct's ruling: it
  doesn't trigger for a card not on the battlefield). An artifact or creature is a
  permanent unless the phrase names a zone (CR 109.2), as the "you activate an ability of
  [permanent]" pattern next to it already had it (`patterns/triggers_ability_of_activated.rs`,
  `patterns/triggers_activate_ability.rs`); tests
  `crackdown_construct_ignores_abilities_of_cards_in_hand`,
  `kurkesh_ignores_an_artifact_cards_cycling`.

Checked in play (same file, passing on the old code too): Rivalry needs more lands than
each other player in a three-player game (the AST says "no other player controls at least
as many", rendered "each player ..." before); Shapeshifter's Marrow becomes a copy of the
creature card put into the graveyard; Kambal, Profiteering Mayor copies each of two tokens
that entered together; Healing Grace's 3 is one shield used up across damage events (CR
615.7); Tanazir Quandrix's base power and toughness are fixed as the ability resolves (CR
608.2h). Also checked by reading the AST or existing tests: Helm of Obedience (Leyline of
the Void, its ruling), Ludevic ("a player other than you", `history_filters_4.rs`), Oath of
Ghouls (the candidate's graveyard against the upkeep player's), Norritt and Nettling Imp
(the active player), Vazi (any ability, in any zone), Immolation Shaman and Harsh Mentor
(of permanents), Bilbo and Scholar of the Lost Trove (the spell cast is exiled, `r_p126`),
Prototype Portal and Soul Foundry (no activation without an exiled card, the Portal's
ruling), Veiling Oddity and Riftmarked Knight (the ability works in exile), Bloodthorn
Flail and Gavel of the Righteous (`cost_choices.rs`), Woebringer Demon, Lost in the Woods,
Duplicity, Mogg Infestation and Fall from Favor (the creature's controller is that
player), Balustrade Spy (both moves in one batch of events).

## Remaining

Of the 151 taken cards, these still mismatch; each was checked against the AST and plays as
the text says (wording only) unless noted:

- Custom nodes or replacement shapes not worded: Island Sanctuary ("during your draw step,
  instead you may skip that draw. If you do, ..."), Ojer Axonil ("an amount of noncombat
  damage less than ~'s power"; "red sources you controlled dealt 4 or more noncombat damage
  this turn"), Stolen Uniform, Myra the Magnificent (the triggering object in a delayed
  trigger's condition), Helm of Obedience (the custom mill-until, tested with Leyline of the
  Void), Extrapolate the Impossible (the custom reveal), Caged Sun, Tenuous Truce (custom
  trigger conditions).
- One printed ability compiled as several: Bloodthorn Flail, Gavel of the Righteous (one
  equip ability for each cost of "Equip—Pay {3} or ...", tested in `cost_choices.rs`),
  Reality Twist, Naked Singularity (one replacement for each land type), Pox (four
  instructions and "Round up each time").
- Order or placement: Reckless Velocitaur, Canyon Vaulter ("during your main phase" after
  both events), Academy Loremaster ("spells they cast this turn"), Comeuppance, Channel
  Harm ("Prevent all damage that would be dealt to you ... this turn by sources you don't
  control"), Refraction Trap, Healing Grace ("Prevent the next 3 damage that a source of
  your choice would deal": the choice, then the shield; Healing Grace tested), Norritt,
  Nettling Imp ("Destroy it at the beginning of the next end step if it didn't attack this
  turn"), Martyrdom ("Only you may activate this ability" after the quoted ability).
- Other wordings: Etali ("the nonland cards exiled this way"), Invasion of Gobakhan, Elite
  Spellbinder ("You may exile a nonland card from it. For as long as that card remains
  exiled"), Verazol ("for each mana spent": the amount of mana; offering "number" broke
  Abby's "a number of ... equal to the amount"), Wood Sage ("all of them with that name"),
  Duplicity, Moonring Mirror ("all other cards you own exiled with ~"), Bolg of the North,
  Surtland Flinger ("deals twice that much damage instead"), Hymn of the Faller ("draw
  another card"), Scholar of the Lost Trove, Bilbo ("an instant or sorcery spell cast this
  way", tested in `r_p126`), Trickster's Stratagem, Happy Hogan ("the owner of target
  creature an opponent controls": the controller's alternatives nested in the owner's),
  Fertilid, Fertilid's Favor ("then shuffles" after the optional "under their control"),
  Catapult Fodder, Tiamat ("that each have"), Oath of Ghouls, Oath of Druids (the target's
  quality; "If the first player does"), Total War ("except for creatures the player hasn't
  controlled continuously"), Slithermuse, Sandstone Oracle ("that player" for the chosen
  opponent), Shapeshifter's Marrow ("the player puts the card into their graveyard"; tested),
  Mogg Infestation, Fall from Favor ("that player" for the creatures' controller),
  Phyrexian Ingester, Bioplasm ("the exiled creature card's power"), Lost in the Woods,
  Calibrated Blast ("the revealed card(s) on the bottom"), Drop of Honey, Porphyry Nodes
  ("If two or more creatures are tied for least power, you choose one of them"), Skill
  Borrower, Conspicuous Snoop ("that card" for the top card), Sita Varma, Tanazir Quandrix
  ("the base power and toughness of ... become equal to"; Tanazir tested).
