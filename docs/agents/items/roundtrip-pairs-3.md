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

## Result

28,046 of 31,866 fully supported cards passed at the start (711b6545); 28,165 of 31,866 pass on the branch before the merge, and 28,186 of 31,951 after
merging the later landings (origin's passing list at the merge has 28,046 cards). 87 of the
152 taken cards pass.

Parser (one fix, one fidelity change):

- `oracle/costs.rs`: "Tap enchanted land: ..." / "Tap enchanted creature: ..." is the
  permanent the Aura enchants (`Filter::AttachedToSource`), not any enchanted land or
  creature you control (the bug below).
- `oracle/statics.rs` (`restrict_enchanted_creature`, called once per face from
  `oracle::compile`): on an Aura that can enchant a permanent that isn't a creature
  ("Enchant creature or Vehicle": Aether Meltdown, Mists of Littjara, Aerial Modification,
  Siege Modification), "Enchanted creature gets ..." affects the enchanted permanent only
  while it's a creature, as the text says (CR 303.4m names the attachment; the noun is the
  kind). Not observable through the rules (a noncreature Vehicle has no power or
  toughness, CR 301.7a), so not listed as a bug; the test pins the creature case through
  the crew ability's type change (layer 4 before 7c).

Renderer (small hooks in the existing files): "At the beginning of each of your main
phases" (CR 505.1, 505.1a); "protection from each color among permanents you control";
"if it has greater power or toughness than ~"; dig steps that exile "face down" and "with
a hatching counter on it" (the face-down and counter qualities used to be dropped, and a
battlefield destination's counters were named `"hatching" counter counter`); "When they
do" / "If they do" after "Target opponent gains control of ..."; "each of that player's
opponents"; "Each player shuffles their graveyard into their library" (CR 400.3); "Kavu"
as its own plural; a permission's "until the end of their next turn" for another player;
"any time you could activate a mana ability, you may pay 1 life" (`mana_timing`); "When
you next activate an ability that isn't a mana ability this turn by spending four or more
mana to activate it"; "~ is the chosen color" in a static ability (not a choice on
resolution); "If ~ would be destroyed, regenerate it"; "If you control a creature, damage
that would ..." (a conditional replacement's other wording); "if that spell was kicked";
"if that spell has mana value 5 or greater" / "if it has power 7 or greater"; "for each
graveyard with seven or more cards in it" (CR 404.1); "Whenever you exert a creature";
"When a player doesn't pay ~'s cumulative upkeep"; "until the end of defending player's
next turn, whenever ..."; "if it wasn't sacrificed", "if it wasn't the first land you
played this turn"; "creatures it was blocking that had become blocked by only ~";
"exile a land you control" / "exile any number of other nonland permanents you control
until ~ leaves the battlefield" (a choice of permanents, then the exile); "For each
color, return up to one target creature card of that color ..." (CR 105.1); "from the
battlefield" for a permanent returned to a hand or the command zone (CR 109.2); "gets
-5/-0 and loses all abilities" in either order (CR 613.1f, 613.4c); "~ can't attack if
defending player controls an untapped land" (CR 506.2); "your third spell in a turn";
"becomes the target of an instant or sorcery spell"; "is all creature types";
"Equipped creature gets +2/+0" in an ability granted to objects made Equipment, the
granted abilities in their listed order; "creatures you control have total toughness 10
or greater"; "into an opponent's graveyard from their library"; "one of your opponents or
a planeswalker they control"; "target creature blocking or blocked by it"; "Choose target
artifact or enchantment. Its owner shuffles it into their library."; "Reveal the top card
of your library. If it's a creature card, put it onto the battlefield. Otherwise, put it
into your graveyard."; "the cards in your hand"; "you may play lands and cast spells from
that player's hand"; "all combat damage that would be dealt to you by unblocked creatures
is dealt to ~ instead"; "is enchanted or equipped"; "until it leaves the battlefield";
"each creature that had a +1/+1 counter put on it this way"; "Each player's life total
becomes the number of creatures they control"; "All cards that aren't on the battlefield,
spells, and permanents"; "each other creature you control enters with a number of
additional +1/+1 counters on it equal to ~'s toughness"; "dies while ~ is in your
graveyard" (CR 113.6k). The equivalence "in addition to its other creature types" now also
covers "other land types" (CR 305.7).

## Parser bugs found

- Tourach's Gate, Earthlore, Krovikan Plague, Nature's Chosen, Veteran's Voice: "Tap
  enchanted land: ..." / "Tap enchanted creature: ..." was compiled as "tap an untapped
  enchanted land (creature) you control", so another land with an Aura on it (Wild
  Growth's) could pay the cost, leaving the enchanted one untapped for another
  activation. Fixed in `oracle/costs.rs`; test
  `tap_enchanted_land_taps_the_land_this_aura_enchants` in
  `crates/mtg-engine/tests/cards/roundtrip_pairs_3.rs` (fails on the old code: checked by
  restoring the old `costs.rs`).

Checked in play (same file, passing on the old code too): Ydwen Efreet's lost flip
unblocks the creature it blocked; Abdel Adrian makes a Soldier for an exiled token (its
ruling) and nothing when it left before its ability resolved (its ruling, CR 610.3b);
Boneyard Scourge and Furious Forebear don't trigger for creatures dying with them (their
rulings); two Goblin Researchers each let you play only their own card (its ruling);
Weathered Bodyguards takes the unblocked creature's damage but not a blocked trampler's
(its ruling). Also checked by reading the AST or existing tests: Master Biomancer (the
Mutant type goes to the entering creature), Suspend Aggression (the permission lasts
through each owner's own next turn, `PlayGrant::duration_player`), Lost in Thought (the
payer exiles from their own graveyard, `tests/cr/r116_special_actions.rs`), Mnemonic
Nexus, Truce, Ill-Gotten Gains, Duskmantle Seer, Living Death, Fight for the Throne (the
opponent's creature), Rhino's Rampage (excess dealt to the second creature only), Yes Man
and Bill Ferny (the draw and the Treasures need the control change), Fangs of Kalonia and
Spectacular Showdown (only creatures that got a counter), Discover the Impossible (the
spell's characteristics, CR 601.3e), Muse Vortex, Undead Alchemist (each trigger makes a
Zombie, its ruling), Dynaheir (one activation, mana abilities excluded).

## Remaining

Of the 152 taken cards, these still mismatch; each was checked against the AST and plays as
the text says (wording only) unless noted:

- Several sentences or abilities printed as one: Hand to Hand, Sen Triplets ("This turn,
  that player can't cast spells or activate abilities and plays with their hand
  revealed"), Lost in Thought, Volrath's Curse ("for that player to ignore this effect";
  the payer's "your graveyard" is theirs), Master Biomancer ("and as a Mutant"), Frontier
  Siege (the modes after "choose Khans or Dragons"), Aerial Modification, Siege
  Modification ("it's a creature" after naming the enchanted permanent), Serra the
  Benevolent (the emblem's condition placement), Living End, Living Death, Magus of the
  Jar, Memory Jar, Ill-Gotten Gains, Duskmantle Seer ("each player ..., then ..." with the
  subject repeated), Nuka-Nuke Launcher ("that player gets two rad counters whenever they
  cast a spell": the trigger after the effect).
- "Choose target ..." first: Kitesail Larcenist, Galion, Surgical Extraction, Extirpate.
- Order only: Gamble, Wild Research (the shuffle compiled as part of the search, before the
  random discard: no replacement can put your own discarded card into your library at a
  position the shuffle would change), Katara, Water Tribe's Hope ("X can't be 0" before
  the timing), Bohn ("in your hand without foretell"), Lazav, Likeness Looter ("in your
  graveyard with mana value X"), Dragon-Kami's Egg ("from among cards you own in exile
  with hatching counters"), Kjeldoran Royal Guard, The Sea Devils, Harsh Justice ("this
  turn" placement).
- Other wordings: Coram (permission for "cards in graveyards that were put there from
  libraries": `any owner` term), Candlekeep Inspiration (X), Truce, Temporary Truce ("for
  each card less than two a player draws this way"), Boros Strike-Captain, Goblin
  Researcher ("During any turn you attacked with ...": tested), Carpet of Flowers ("if you
  haven't added mana with this ability this turn", compiled as once each turn), Discover
  the Impossible, Abdel Adrian ("for each permanent exiled this way"), Suspend Aggression
  ("For each of those cards"), Aurelia's Vindicator, Angel of Serenity ("from the
  battlefield and/or creature cards from graveyards"), Huatli, Nissa, Katara, Seeking
  Revenge, Agent Maria Hill ("her", as for Michelangelo in earlier items), Parallel
  Thoughts, Mangara's Tome, Ghastly Conscription, Expose the Culprit ("in a face-down pile,
  shuffle that pile": the controller may look at face-down permanents anyway, CR 708.5),
  Head Games, Jester's Mask ("that many"), Goblin War Cry, Eunuchs' Intrigues ("Other
  creatures they control"), Rhino's Rampage ("When excess damage is dealt ... this way"),
  Fight for the Throne ("the creature an opponent controls"), Fa'adiyah Seer, Sindbad
  ("reveal it", "discard it"), Muse Vortex, Talent of the Telepath, Collected Conjuring,
  Doom Reigns Supreme ("cast up to two ... spells from among them"), Shelinda ("its power"
  after "that creature"), Dream-Thief's Bandana, Klaw ("mana of any type can be spent").
