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
