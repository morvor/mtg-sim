# roundtrip-pairs-3: 150 untriaged mismatches from the long tail

Part of the `roundtrip-followups.md` stream. Baseline (Oct 29, origin 711b6545, after
roundtrip-gaps-5 landed): 28,046 of 31,866 fully supported cards pass; 3,674 mismatch, in
1,074 single-card clusters, 223 two-card clusters and 4 larger ones. Clustering by the first
difference no longer groups much, and many of the remaining pairs were already triaged as
"Remaining (wording only)" by earlier items. Runs alone.

SCOPE: pick about 150 mismatching cards that no earlier roundtrip item's "Remaining" section
lists (grep docs/agents/items/roundtrip-*.md): first the 4 clusters of 3+ cards and every
untriaged two-card cluster, then single-card clusters in the tool's order until about 150
cards. Group them by the kind of difference before working (the same renderer or parser
construct across several singletons is the efficient unit). Write the chosen list (card,
cluster key) into this brief's "Taken" section and commit it first.

For each card: triage — parser bug first (does the AST mean what the text says, in
multiplayer, with control changes, tokens, copies, replacement effects, zero-quantity
choices?) with a fix + in-game test in `tests/cards/roundtrip_pairs_3.rs` that fails on the
old code + a row in the bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else
renderer wording (independently of the parser), or a CR-justified equivalence. Judgment calls
go in the interpretation register (CLAUDE.md). Add "Parser bugs found" and "Remaining"
sections. Regenerate docs/ROUNDTRIP.md and docs/roundtrip-passing.txt; `roundtrip --check`
must pass.

## Taken

From `roundtrip --clusters 1400` / `--list` on origin 711b6545 (28,046 of 31,866 pass).
Cards listed in an earlier item's "Remaining" section are left out (e.g. the rest of
`gap: Filter`, `remembering Before`, Jetfire, Pia Nalaar ...). Every untriaged two-card
cluster has 382 cards, more than the item takes, so it takes the 3+-card clusters' untriaged
cards and then the two-card clusters in the tool's order up to the 80th (152 cards):

- gap: Filter: Urza's Miter; Fastbond
- gap: Condition: Nuka-Nuke Launcher; Dynaheir, Invoker Adept
- gap: static permission term: Coram, the Undertaker; Sen Triplets
- oracle `choose up to 1 other` vs rendered `up to 1 other target`: Kitesail Larcenist; Galion, Elvenking's Butler
- oracle `creature it was blocking` vs rendered `each attacking creature`: Ydwen Efreet
- oracle `creature you control have base` vs rendered `{opt:each} creature you control have`: Katara, Water Tribe's Hope; Candlekeep Inspiration
- oracle `creature you control have total` vs rendered `the total toughness of creature`: Betor, Kin to All; Orysa, Tide Choreographer
- oracle `creature` vs rendered `permanent`: Aether Meltdown; Mists of Littjara
- oracle `discard a card at random` vs rendered `shuffle discard a card at`: Gamble; Wild Research
- oracle `do not pay ~ cumulative` vs rendered `cumulative upkeep not paid {alt:while|if}`: Heart of Bogardan; Thought Lash
- oracle `draw up to 2 card` vs rendered `choose {alt:a number from 0 to 2|0, 1, or 2} draw x card`: Truce; Temporary Truce
- oracle `during any turn you attacked` vs rendered `may play it {alt:for as long as it remains exiled|for as long as it remain exiled} {alt:if|as long as|during any turn}`: Boros Strike-Captain; Goblin Researcher
- oracle `during combat player can't cast` vs rendered `player can't activate ability that`: Hand to Hand
- oracle `dy while ~ is in` vs rendered `{alt:dies|is put to a graveyard} while in your graveyard`: Furious Forebear; Boneyard Scourge
- oracle `each card that is not` vs rendered `card in hand card in`: Mycosynth Lattice; Painter's Servant
- oracle `each color among` vs rendered `permanent that share a color`: Empty-Shrine Kannushi; Pledge of Loyalty
- oracle `each combat damage that would` vs rendered `if a unblocked creature would`: Kjeldoran Royal Guard; Weathered Bodyguards
- oracle `each creature that had a` vs rendered `it among it`: Fangs of Kalonia; Spectacular Showdown
- oracle `each land is a swamp` vs rendered `land is a swamp in`: Blanket of Night
- oracle `each nonland card in your` vs rendered `nonland card without foretell in`: Bohn, Beguiling Balladeer
- oracle `each of that player` vs rendered `{alt:that player|it} each`: Heartwood Storyteller; Standstill
- oracle `each of your main phase` vs rendered `your first main phase {alt:or|and at|or at}`: Carpet of Flowers; Frontier Siege
- oracle `each other creature you control` vs rendered `other creature you control enter`: Master Biomancer; Arwen, Weaver of Hope
- oracle `each player discard it hand` vs rendered `{alt:at the beginning of the next end step, each player discards its hand. put each card among it in exile to owner's hand|each player discards its hand at the beginning of the next end step. put each card among it in exile to owner's hand}`: Magus of the Jar; Memory Jar
- oracle `each player exile each creature` vs rendered `exile each creature card {alt:in a graveyard|in all graveyards|in graveyards}`: Living End; Living Death
- oracle `each player life total become` vs rendered `{alt:that player's|its} life total become {alt:x, where x is the number of creatures {alt:that player controls|it control}|equal to the number of creatures {alt:that player controls|it control}}`: Biorhythm; Shaman of Forgotten Ways
- oracle `each player` vs rendered `∅`: Ill-Gotten Gains; Duskmantle Seer
- oracle `enchanted land` vs rendered `a untapped enchanted land you`: Tourach's Gate; Earthlore
- oracle `enchanted or equipped` vs rendered `a enchanted permanent or a`: Novice Knight; Warchanter Skald
- oracle `exert` vs rendered `exerted {alt:while|if} it is`: Resolute Survivors; Battlefield Scavenger
- oracle `exile 1 face down put` vs rendered `{alt:put a card from among it to exile|exile a card from among it} put the rest {alt:on the bottom of owner's library|on the bottom}`: Clone Shell
- oracle `exile 1 of it face` vs rendered `{alt:put a card from among it to exile|exile a card from among it} put the rest {alt:on the bottom of owner's library|on the bottom}`: Discover the Impossible; The Dragon-Kami Reborn // Dragon-Kami's Egg
- oracle `exile a land you control` vs rendered `choose a land you control`: Wormfang Turtle; Wormfang Newt
- oracle `exile any number of other` vs rendered `choose any number of other`: Abdel Adrian, Gorion's Ward; Lumbering Battlement
- oracle `for each color put up` vs rendered `put up to 1 target`: Rogues' Gallery; All Suns' Dawn
- oracle `for each of it it` vs rendered `it owner may play it`: Suspend Aggression
- oracle `for the first time during` vs rendered `during your turn for the`: Cat Collector; Tale of Katara and Toph
- oracle `from the battlefield creature card` vs rendered `{opt:on the battlefield} or creature card in`: Aurelia's Vindicator; Angel of Serenity
- oracle `from the battlefield` vs rendered `∅`: Leadership Vacuum; Sanctum of Eternity
- oracle `get -5/-0 lose each ability` vs rendered `lose each ability get -5/-0`: Quantum Reduction; Fresh Start
- oracle `graveyard with 7 or more` vs rendered `player with 7 or greater`: The Master of Lake-town; Master's Councillors
- oracle `have greater power or toughness` vs rendered `power is greater than ~`: Hulkling, Burgeoning Bruiser; Evolving Adaptive
- oracle `have mana value 5 or` vs rendered `mana value is 5 or`: Prismari Apprentice; Herald of Ilharg
- oracle `have mana value` vs rendered `mana value is`: Consuming Ashes; Serum Snare
- oracle `have power 7 or greater` vs rendered `power is 7 or {alt:greater|more}`: Yavimaya Bloomsage // Channel
- oracle `her to the battlefield transformed` vs rendered `it to the battlefield transformed`: Huatli, Poet of Unity // Roar of the Fifth People; Nissa, Vastwood Seer // Nissa, Sage Animist
- oracle `her` vs rendered `~`: Katara, Seeking Revenge; Agent Maria Hill
- oracle `if defending player control a` vs rendered `~ can't attack a player`: Veteran Brawlers; Branded Brawlers
- oracle `if you control a creature` vs rendered `damage that would reduce your`: Serra the Benevolent; Worship
- oracle `if ~ would be destroyed` vs rendered `regenerate ~`: Knight of the Holy Nimbus; Clergy of the Holy Nimbus
- oracle `in a face-down pile shuffle` vs rendered `face down shuffle it shuffle`: Parallel Thoughts; Mangara's Tome
- oracle `in a face-down pile shuffle` vs rendered `face down`: Ghastly Conscription; Expose the Culprit
- oracle `in a` vs rendered `each`: Vance's Blasting Cannons // Spitfire Bastion; Emerald Collector
- oracle `in it hand on top` vs rendered `hand on top of owner`: Head Games; Jester's Mask
- oracle `in your graveyard with mana` vs rendered `with mana value x in`: Lazav, the Multifarious; Likeness Looter
- oracle `instant or sorcery` vs rendered `spell or ability {alt:while|if} it`: Wild Defiance; Silverfur Partisan
- oracle `is each creature type` vs rendered `{alt:is every creature type|gains all creature types|all creature types}`: Stalactite Dagger; Undercover Skrull
- oracle `is equipment in addition to` vs rendered `{alt:is an equipment with equip {2} and "enchanted permanent gets +2/+0." in addition to its other types|is an equipment in addition to its other types and have equip {2} and "enchanted permanent gets +2/+0."}`: Armed with Proof; Arterial Alchemy
- oracle `is the chosen color` vs rendered `become the color of your`: Alloy Golem; Shimmerwilds Growth
- oracle `it activated ability can't be` vs rendered `{alt:enchanted creature's|its} activated ability can't be`: Lost in Thought; Volrath's Curse
- oracle `it control other creature it` vs rendered `{alt:that player controls|it control} {opt:on the battlefield} each creature that`: Goblin War Cry; Eunuchs' Intrigues
- oracle `it deal that much damage` vs rendered `this turn it deal {alt:that much|that many}`: The Sea Devils; Harsh Justice
- oracle `it fight target creature a` vs rendered `{alt:it|thatit you control} fight target creature {alt:an opponent controls|your opponents control}`: Rhino's Rampage; Fight for the Throne
- oracle `it if it is not` vs rendered `each card from among it`: Fa'adiyah Seer; Sindbad
- oracle `it in your hand on` vs rendered `your hand on the bottom`: Mindmoil; Arjun, the Shifting Flame
- oracle `it is a creature in` vs rendered `enchanted permanent is a creature`: Aerial Modification; Siege Modification
- oracle `it owner graveyard hand library` vs rendered `target card in a graveyard`: Surgical Extraction; Extirpate
- oracle `it owner shuffle target artifact` vs rendered `target artifact owner or enchantment`: Deglamer; Unravel the Aether
- oracle `it to owner graveyard if` vs rendered `a creature card from among`: Zoologist; Call of the Wild
- oracle `it to owner hand if` vs rendered `a land card from among`: Nadu, Winged Wisdom; Skyward Eye Prophets
- oracle `it without paying it mana` vs rendered `{alt:it|it|thatit} without paying it mana`: Muse Vortex; Talent of the Telepath
- oracle `it` vs rendered `a opponent`: Undead Alchemist; Frontier Warmonger
- oracle `it` vs rendered `each`: Struggle // Survive; Mnemonic Nexus
- oracle `it` vs rendered `that-object`: Sharp-Eyed Rookie; Shelinda, Yevon Acolyte
- oracle `it` vs rendered `you`: Yes Man, Personal Securitron; Bill Ferny, Bree Swindler
- oracle `it` vs rendered `~`: Steadfast Armasaur
- oracle `kavu` vs rendered `kavus`: Kavu Mauler; Shoreline Raider
- oracle `kicked` vs rendered `a kicked permanent`: Hallar, the Firefletcher; Bloodstone Goblin
- oracle `look at the top card` vs rendered `{alt:look at the top card of {alt:that player's|its} library, then exile it face down|exile the top card of {alt:that player's|its} library face down and look at it|exile the top card of {alt:that player's|its} library face down. may look at {alt:thatit|it} for as long as it remains exiled} may play it {alt:for as long as it remains exiled|for as long as it remain exiled}`: Dream-Thief's Bandana; Klaw, Master of Sound
- oracle `may cast up to 2` vs rendered `cast up to 2 nonland`: Collected Conjuring; Doom Reigns Supreme
