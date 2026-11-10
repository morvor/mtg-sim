# roundtrip-pairs-6: the next untriaged round-trip mismatches

Same method as roundtrip-pairs-5.md: read it first.

**Baseline:** docs/ROUNDTRIP.md, Nov 5: 28,460 of 32,076 pass and 3,464 mismatch. Regenerate it first from the current branch.

**Scope.** Take about 150 mismatching cards that no earlier roundtrip item's "Taken" or "Remaining" section lists (`grep docs/agents/items/roundtrip-*.md`). Go in the same order as pairs-5:
1. clusters of 3 or more cards;
2. then two-card clusters;
3. then single cards.

Record the chosen cards in a "Taken" section and commit it first.

**Priority within the scope:** a mismatch can mean the engine *behaves* differently from the card, not just that the wording renders differently. Check those first, test them in play, and fix the parser or engine. Only then fix the renderer's wording.

**DONE when:**
- each taken card passes, or is listed under "Remaining" with its reason;
- docs/ROUNDTRIP.md and docs/roundtrip-passing.txt are regenerated;
- `cargo test --workspace` passes.

## Taken

From `roundtrip --clusters 3000` / `--list` on the branch at ad23a7a (28,460 of 32,076 pass). Cards
named in an earlier roundtrip item's brief are left out. The untriaged cards of the 3+-card
clusters, then the untriaged two-card clusters, then single-card clusters in the tool's order
(150; long cluster keys cut short):

- gap: Effect: Kozilek, the Broken Reality; Rootweaver Druid; The Toymaker's Trap
- oracle `of it face down put` vs rendered `{opt:of it} face down put the`: Siphon Insight; Gonti, Lord of Luxury; Black Cat, Cunning Thief
- oracle `each player exile the top` vs rendered `exile the top card of`: Rocco, Street Chef; Timesifter
- oracle `become the` vs rendered `you`: Custodi Lich; Starscream, Power Hungry // Starscream, Seeker Leader
- oracle `it entered under your control` vs rendered `{alt:it is under your control|you control it|you controlled it}`: Hallowed Respite; Phelia, Exuberant Shepherd
- oracle `as 1 other` vs rendered `∅`: Mechanized Production; Chrome Replicator
- oracle `as long as` vs rendered `∅`: Evendo Brushrazer; Cloud, Planet's Champion
- oracle `of it to owner hand` vs rendered `{opt:of it} to owner hand put`: Accumulate Wisdom; See the Truth
- oracle `that player` vs rendered `∅`: Stunted Growth; Vengeful Strangler // Strangling Grasp
- oracle `or black spell this turn` vs rendered `{alt:or|and} black spell this turn`: Veil of Summer
- oracle `that is attached to a` vs rendered `attached to a permanent attach`: Aura Graft
- oracle `thatit dy this way create` vs rendered `{alt:a card in a graveyard is destroyed|a card in a graveyard was des...`: Kalitas, Bloodchief of Ghet
- oracle `in your graveyard when ~` vs rendered `among it in your graveyard`: Toluz, Clever Conductor
- oracle `of each of the sacrificed` vs rendered `that share a color with`: Mind Extraction
- oracle `control no thopter other than` vs rendered `{alt:control no|do not control} other thopter`: Thopter Assembly
- oracle `with no card in hand` vs rendered `ifs it have no card`: Tinybones, Trinket Thief
- oracle `it other cost if thatit` vs rendered `paying it other cost if`: Ogre Battlecaster
- oracle `a number of card from` vs rendered `{alt:a number of cards from the top of your library equal to its power|th...`: End-Blaze Epiphany
- oracle `it library for up to` vs rendered `{alt:that player's|its} library for {opt:up to} x`: Collective Voyage
- oracle `from the command zone in` vs rendered `in the command zone or`: Campfire
- oracle `unless pay {r} when ~` vs rendered `when ~ block {opt:a creature} or`: Goblin Flotilla
- oracle `creature target opponent control do` vs rendered `{alt:and|or} creature target opponent control`: Icebreaker Kraken
- oracle `to the battlefield tapped each` vs rendered `each artifact creature enchantment {alt:and|or}`: Gerrard's Hourglass Pendant
- oracle `each player exile the top` vs rendered `exile the top 7 card`: Memory Vessel
- oracle `from among it` vs rendered `in {alt:that player's|its} hand`: Mindclaw Shaman
- oracle `is a spell with lesser` vs rendered `mana value is less than`: Rashmi, Eternities Crafter
- oracle `become a green bear creature` vs rendered `{alt:{alt:becomes|is} a 4/4 bear creature in addition to its other...`: Halsin, Emerald Archdruid
- oracle `∅` vs rendered `up to`: Doomsday Confluence
- oracle `each friend search it library` vs rendered `if there is a it`: Pir's Whim
- oracle `each color this ability do` vs rendered `{alt:white, blue, black, red, and green|all colors}`: Fallaji Wayfarer
- oracle `card exile it that player` vs rendered `spell exile it {alt:that player|it|the player} copy`: Eye of the Storm
- oracle `each ability your opponent control` vs rendered `{alt:each activated ability or each triggered ability your o...`: Kadena's Silencer
- oracle `gain 4 life if a` vs rendered `if a opponent have more`: Beza, the Bounding Spring
- oracle `power become the toughness of` vs rendered `{alt:has base power and toughness its toughness minus 1/1 plus it...`: Sworn Defender
- oracle `each of up to 1` vs rendered `up to 1 target land`: Omo, Queen of Vesuva
- oracle `vampire` vs rendered `other the new creature type`: New Blood
- oracle `choose a creature card a` vs rendered `put a creature card a`: Green Sun's Twilight
- oracle `it power target creature` vs rendered `~ power target creature where`: Ravenous Gigantotherium
- oracle `it is kicked 2 time` vs rendered `there is 2 or more`: Archangel of Wrath
- oracle `where x is thatit power` vs rendered `{opt:on the battlefield} where x is creature`: Astarion's Thirst
- oracle `ability your opponent activate that` vs rendered `activated ability a opponent activate`: Kopala, Warden of Waves
- oracle `{r} was spent to cast` vs rendered `{u} was spent to cast`: Unnerving Assault
- oracle `tapped attacking it gain indestructible` vs rendered `{alt:tapped and attacking {opt:under your control}|unde...`: Winota, Joiner of Forces
- oracle `your opponent is dealt combat` vs rendered `1 or more opponent is`: Mindblade Render
- oracle `that player or thatit controller` vs rendered `it controller may discard a`: Chain of Plasma
- oracle `thatit mana value is 5` vs rendered `{alt:its mana value is 5 or greater|it has mana value 5 or greater} crea...`: Zaffai, Thunder Conductor
- oracle `in exile to the battlefield` vs rendered `to the battlefield tapped {opt:under your control}`: Torrent Elemental
- oracle `sunburst` vs rendered `∅`: Arcbound Wanderer
- oracle `milled this way to owner` vs rendered `{alt:among it|from among it|from among the milled cards|from among it ...`: Sparring Dummy
- oracle `a permanent is put to` vs rendered `{alt:a permanent {alt:an opponent owns|your opponents own} dies|a permane...`: Patron of the Nezumi
- oracle `it can't be regenerated it` vs rendered `{alt:it|a creature destroyed this way} can't be regenerated {alt:its...`: Polymorph
- oracle `x damage to itself` vs rendered `{alt:x|that much} damage to ~`: Sunflare Shaman
- oracle `flip a coin for each` vs rendered `non-devil non-imp creature {opt:on the battlefield} flip`: Rakdos, the Showstopper
- oracle `spell ability` vs rendered `permanent`: Spider-Punk
- oracle `creature you control is dealt` vs rendered `{alt:or|and} a creature you control`: Heroic Sacrifice
- oracle `if a player have 20` vs rendered `draw a card if a`: Jace, the Perfected Mind
- oracle `2 time that much {g}` vs rendered `a amount of {g} equal`: Fangorn, Tree Shepherd
- oracle `in your hand that share` vs rendered `that share a color in`: Illuminated Folio
- oracle `a spell that is 1` vs rendered `{alt:colored spells that is not a permanent with {x} in its mana cost|a color...`: Titans' Nest
- oracle `activate a ability that is` vs rendered `{alt:activate an ability that is not a mana ability of a permanent|a...`: Verrak, Warped Sengir
- oracle `your library reveal the top` vs rendered `{alt:|your library|its library} reveal the top card`: Unexpected Results
- oracle `if you do not put` vs rendered `create a food token unless`: Courier of Comestibles
- oracle `exactly 2 creature you control` vs rendered `2 creature you control {opt:on the battlefield}`: Spry and Mighty
- oracle `that name exile it that` vs rendered `the same name as it`: Deadly Cover-Up
- oracle `jumblebone a legendary 2/1 black` vs rendered `{alt:jumblebones, a legendary 2/1 black skeleton creature toke...`: Ozox, the Clattering King
- oracle `in your graveyard you surveilled` vs rendered `you surveilled this turn in`: Eye of Duskmantle
- oracle `each player who didn't discard` vs rendered `if there is fewer than`: Strongarm Tactics
- oracle `each card from among it` vs rendered `thatit to owner hand put`: Treasure Hunt
- oracle `permanent` vs rendered `∅`: Bounty Agent
- oracle `is a insect artifact creature` vs rendered `{alt:is a 0/1 insect artifact creature with indestructible and lo...`: Darksteel Mutation
- oracle `become a treasure artifact with` vs rendered `{alt:{alt:becomes|is} a treasure artifact with "{t}, sacrifice ...`: Vraska, Betrayal's Sting
- oracle `or were cast in a` vs rendered `in a graveyard or {alt:cast it in a graveyard|it was cast in a graveyard}`: Twilight Diviner
- oracle `of a opponent choice you` vs rendered `you do not control of`: Volcanic Offering
- oracle `in addition to it other` vs rendered `as long as {alt:there are four or more phyresis counters on ~it|~it has...`: Weatherlight Compleated
- oracle `each of` vs rendered `∅`: Drakuseth, Maw of Flames
- oracle `other` vs rendered `permanent that is not a`: Call Forth the Tempest
- oracle `at the beginning of each` vs rendered `for as long as ~`: Giant Oyster
- oracle `that each have mana value` vs rendered `with mana value x or`: Ecological Appreciation
- oracle `the most creature` vs rendered `at least as many creature`: No Witnesses
- oracle `each other planeswalker you control` vs rendered `other planeswalker you control have`: Kasmina, Enigma Sage
- oracle `it get that many rad` vs rendered `{alt:that player|it|the player} get x rad counter`: Infesting Radroach
- oracle `a equipped creature you control` vs rendered `other equipped creature you control`: Rhuk, Hexgold Nabber
- oracle `whose life total is less` vs rendered `with less than half it`: Anya, Merciless Angel
- oracle `it control` vs rendered `∅`: Drain Power
- oracle `a creature named feral shadow` vs rendered `~ a creature named feral`: Urborg Panther
- oracle `the chosen type get +1/+1` vs rendered `{alt:the chosen type|that type} get +1/+1 creature you`: Radiant Destiny
- oracle `gain x life x is` vs rendered `where x is the number`: Lantern Flare
- oracle `a spell in your hand` vs rendered `spell in your hand without`: One with the Multiverse
- oracle `milled this way to the` vs rendered `{alt:among it|from among it|from among the milled cards|from among it mi...`: The Cyber-Controller
- oracle `or block creature you control` vs rendered `creature you control can't be`: Storm, Windrider
- oracle `it is a artifact it` vs rendered `{alt:it is an artifact {opt:and loses all other card types}|it is an artifa...`: Espers to Magicite
- oracle `create that many 2/2 black` vs rendered `{alt:creates x 2/2 black zombie creature tokens, where x is that man...`: Awaken the Erstwhile
- oracle `that opponent or thatit controller` vs rendered `{alt:that player or thatit's controller|its controller} cont...`: Goblin Lyre
- oracle `if it mana value is` vs rendered `{alt:if {alt:target artifact's mana value is 2 or less|target artifact has ...`: Overload
- oracle `do not untap during target` vs rendered `target player control do not`: Blinding Beam
- oracle `each opponent exile the top` vs rendered `exile the top card of`: Fathom Feeder
- oracle `that player sacrifice a creature` vs rendered `{alt:that player|it} sacrifice a creature`: The Dalek Emperor
- oracle `when sacrifice 1 or more` vs rendered `{opt:you control} when you do tap`: Nyssa of Traken
- oracle `may reveal up to 2` vs rendered `{opt:may} reveal up to 2`: Thunderous Debut
- oracle `target permanent owner shuffle it` vs rendered `{alt:target permanent's owner shuffles it to owner's library|...`: Chaos Warp
- oracle `each player put any number` vs rendered `put any number of card`: Grave Sifter
- oracle `of your choice with shadow` vs rendered `with shadow source of your`: Circle of Protection: Shadow
- oracle `card` vs rendered `graveyard`: Custodi Squire
- oracle `it library for up to` vs rendered `{alt:that player's|its} library for {opt:up to} 2`: Avatar of Growth
- oracle `it have a even number` vs rendered `there is no counter on`: Sab-Sunen, Luxa Embodied
- oracle `card in a graveyard that` vs rendered `permanent such that a player`: Ghost of Ramirez DePietro
- oracle `to it player` vs rendered `{opt:to it}`: Shower of Coals
- oracle `look at the top 6` vs rendered `{opt:with creatures} look at the top`: Cosmic Cube
- oracle `the difference` vs rendered `it power minus it base`: Sovereign Okinec Ahau
- oracle `counter thatit if it have` vs rendered `if it mana value is`: Counterbalance
- oracle `a green aura enchantment token` vs rendered `{alt:settlement, a green aura enchantment token with {alt:enchan...`: Preston Garvey, Minuteman
- oracle `become blue is not a` vs rendered `is not a artifact {alt:becomes|is}`: Neurok Transmuter
- oracle `may cast instant sorcery spell` vs rendered `{opt:may} cast any number of`: Epic Experiment
- oracle `was a nonland creature put` vs rendered `is not a land put`: Princess Yue
- oracle `∅` vs rendered `ability`: Volatile Stormdrake
- oracle `put thatit to the battlefield` vs rendered `{alt:as this resolves, choose one —   put it to the battlefield {...`: Illuna, Apex of Wishes
- oracle `who attacked you during it` vs rendered `for whom {alt:that player|it} attacked you`: Weathered Sentinels
- oracle `it is not cast or` vs rendered `you didn't cast it or`: Freestrider Commando
- oracle `if it target a colorless` vs rendered `equip ability of ~ that`: Ghostfire Blade
- oracle `become a legendary creature named` vs rendered `{alt:{alt:becomes|is} a 5/5 legendary creature named mileva, ...`: Tenth District Hero
- oracle `1 time for each nontoken` vs rendered `for each attacking nontoken`: Sophina, Spearsage Deserter
- oracle `each other creature you control` vs rendered `other creature you control have`: Tam, Mindful First-Year
- oracle `or block unless it controller` vs rendered `unless pay {1} for each`: Whipgrass Entangler
- oracle `each damage that would be` vs rendered `if target attacking creature {opt:source}`: Shimian Night Stalker
- oracle `target face-down permanent if it` vs rendered `it if it is a`: Hauntwoods Shrieker
- oracle `that player gain x life` vs rendered `{alt:that player|it|the player} gain x life where`: Aven Shrine
- oracle `creature card in your graveyard` vs rendered `is put to exile from`: Kaya, Spirits' Justice
- oracle `target player lose each poison` vs rendered `remove x poison counter from`: Leeches
- oracle `become a red dragon with` vs rendered `{alt:{alt:becomes|is} a 4/4 dragon in addition to its other types and ...`: Shadow Puppeteers
- oracle `is a treasure artifact with` vs rendered `{alt:is a treasure artifact with "{t}, sacrifice ~: add one mana of...`: Minimus Containment
- oracle `activate a ability that target` vs rendered `{alt:activate an ability that is not a mana ability of a permane...`: Ertha Jo, Frontier Mentor
- oracle `if it have mutate each` vs rendered `{alt:creature spells with mutate cast|creature spells cast with mutate}`: Pollywog Symbiote
- oracle `target player hand` vs rendered `each card in target player`: Identity Crisis
- oracle `you had no card in` vs rendered `there is no card in`: Mindstorm Crown
- oracle `choose 2 target creature untap` vs rendered `untap 2 target creature put`: Last Night Together
- oracle `that amount of damage to` vs rendered `{alt:that much|that many} damage to target creature`: Fear of Burning Alive
- oracle `a doctor card in your` vs rendered `any doctor card in your`: The Fourteenth Doctor
- oracle `target opponent put a creature` vs rendered `put a creature card in`: Visions of Dread
- oracle `it is a instant or` vs rendered `the top card of your`: Galvanoth

## Result

28,460 of 32,076 fully supported cards passed at the start (ad23a7a); 28,601 of 32,076 after.
88 of the 150 taken cards pass; `roundtrip --check` passes.

Parser and engine: no misreading found (see "Parser bugs found"); every mismatch of the
taken cards was the renderer's wording or a construct it can't word.

Renderer (most in the new `oracle/render/pairs6_terms.rs`, hooks in the existing files):
"Whenever you become the monarch" (the player action named `monarch`); "with the same name
as one another"; "Modular—Sunburst" (the keyword variant, as megamorph); "Spells and
abilities can't be countered" (`CantBeCountered(Any)`); "a mana ability of each land they
control" (CR 605.3a); "As long as it's your turn and X, E" / "During your turn, as long as
X, E" (the effect first, so the comparison can reorder it); "artifacts and creatures target
opponent controls don't untap during that player's next untap step" and "Creatures don't
untap during target player's next untap step"; "opponents whose life total is less than
half their starting life total"; "each player who controls the most creatures"; "you had
no cards in hand at the beginning of this turn"; "a land card milled this way"; "players who
attacked you during their last turn"; "if it wasn't cast or no mana was spent to cast it";
"a creature of your choice with shadow"; "shuffles all other cards revealed this way"; "if
it was kicked twice"; "hexproof from activated and triggered abilities", "hexproof from
each of its colors"; "each player exiles the top card of their library"; per-player searches
("each player searches their library for up to two basic land cards, puts them onto the
battlefield, then shuffles": the searching player's verbs, the subject said again before
each as the comparison reads it); "Choose X" (exactly X modes); "the loyalty abilities of
~"; "if you control no Thopters other than ~"; "3 damage to each of up to two other
targets"; "Whenever a permanent is put into an opponent's graveyard" (a permanent is on the
battlefield, CR 110.1); "If it entered under your control"; "X damage to any target and X
damage to itself"; "Enchanted permanent's controller sacrifices a nonland permanent, then
that player loses 1 life" (a subject the comparison doesn't carry over, said again);
"Whenever an equipped creature you control other than ~ attacks or dies"; "~ deals 10
damage to an opponent chosen at random"; "put all cards exiled this way that weren't cast
into your graveyard"; "then put all cards revealed this way into your hand"; "Target player
loses all poison counters"; "counter all abilities your opponents control"; "Exile all cards
from target player's hand and graveyard"; a named token's abilities name it or say "this
creature" ("Jumblebones can't block"); "that opponent or that planeswalker's controller";
"is every land type", "put an everything counter on each of up to one target land and up to
one target creature"; "creates that many 2/2 black Zombie creature tokens"; "target nonbasic
land of an opponent's choice you don't control"; "discards all cards of each of the
sacrificed creature's colors"; "Reveal two cards from your hand that share a color"; "target
Aura that's attached to a permanent", "attach it to another permanent it can enchant"
(`CanBeAttachedBy`); "the power of the creature exiled this way"; "If ..., put each of those
cards into your hand instead" (the same look, all the cards taken); "[instruction] if
[condition]" in either order for one plain clause; "loses all other card types and
abilities", "and it loses all other abilities"; "That player shuffles, then draws a card"
(an owner named before), "cards with that name"; "Add twice that much {G}"; "a card ... that
was discarded this turn", "put there from a library this turn"; "if target player has fewer
than nine poison counters, they get a number of poison counters equal to the difference";
"you may cast it" after looking at the top card of your library, "if it's an instant or
sorcery spell"; "Target creature becomes a green Bear creature with base power and toughness
4/4 in addition to its other colors and types"; "the rest of the cards"; "unless you pay {R},
whenever ..."; "reveal target face-down permanent" (the target remembered just before);
"You may play the exiled card for as long as
it remains exiled", "on the bottom of that library", "Mana of any type can be spent to cast
spells this way"; "Put all commanders you own from the command zone and from your graveyard
into your hand"; "Target opponent puts a creature card of their choice from their graveyard
onto the battlefield under your control"; "in addition to its other costs"; "Put ~ from
exile onto the battlefield tapped", "Return to the battlefield tapped all ..."; "they get
that many rad counters"; "an opponent's life total is less than half their starting life
total"; "cast a spell from among them with mana value less than or equal to ..."; "Once
during each of your turns, you may cast a spell from your hand or the top of your library"
(one permission compiled per zone, sharing its use); "Shuffle your library, then reveal the
top card"; "If you don't put a card into your hand this way, create a Food token"; "target
legendary permanent that's an artifact, creature, or enchantment"; "Whenever you activate an
ability that isn't a mana ability", "pay that much life again"; "Whenever you activate an
ability that targets a creature or player" (mana abilities have no targets, CR 605.1a);
"equal to the difference" (a creature's power minus its base power); "a spell that's one or
more colors without {X} in its mana cost".

## Parser bugs found

None. Each taken card whose mismatch could have meant a different behavior was checked
against the AST, and in play where the AST alone didn't settle it (tests in
`crates/mtg-engine/tests/cards/roundtrip_pairs_6.rs`, all passing on the old code too, so
wording only): Spider-Punk ("Spells and abilities can't be countered" compiled as a
restriction on every object, rendered "permanents": an ability on the stack can't be
countered either), Aura Graft (no legal place to move the Aura: it stays, and you control
it — its open ruling), Galvanoth (a top card that isn't an instant or sorcery card stays on
top — its open ruling), Blinding Beam ("Creatures don't untap during target player's next
untap step" compiled as the permanents that player controls: the duration is the targeted
player's untap step whoever's turn it's cast on), Tinybones (each opponent with no cards in
hand as the ability resolves — its open ruling; rendered "each opponent ifs they have no
cards"), Chain of Plasma (`ControllerOf` of a player target is that player). Checked by
reading the AST and the engine: Timesifter (each player exiles; the tied players repeat),
Drain Power (only the lands the player controls, `mana_abilities::activate_each`), One with
the Multiverse (the two zones' permissions share one once-each-turn slot,
`kw/once_each_turn_cast.rs`), Kalitas (the token's power and toughness are the destroyed
creature's, as last on the battlefield), Ertha Jo and Verrak (`AbilityActivated` with
`source: Any` matches an ability activated from any zone), Tam ("hexproof from each of its
colors" is judged with the holder as `This`), Unexpected Results (the land and the return
of the spell are one "may"), Arcbound Wanderer ("Modular—Sunburst" is told apart by the
keyword's text in `kw/modular.rs`, as the renderer now does too), Princess Yue (the
intervening "if" keeps "creature"), Fallaji Wayfarer ("This ability doesn't affect its color
identity" is read in `commander_rules::computed_color_identity`; the sentence compiles to
nothing, so the renderer can't say it), Rashmi and Counterbalance (the mana values compared
are the revealed card's and the spell's).

## Remaining

Of the 150 taken cards, these still mismatch; each was checked against the AST and plays as
the text says (wording only) unless noted:

- Custom nodes not worded (`gap:`): Kozilek, the Broken Reality (manifest from hands),
  Rootweaver Druid (opt in), The Toymaker's Trap (secret choices), Pir's Whim (friend or
  foe), Spry and Mighty (`remembering Matching`), Fallaji Wayfarer (no AST for "This
  ability doesn't affect its color identity").
- One printed sentence compiled as several abilities, or two sentences as one: Shower of
  Coals (two sentences, one ability with an "instead"), Storm, Windrider ("can't attack you
  or block creatures you control": two restrictions), Radiant Destiny ("they also have
  vigilance"), Whipgrass Entangler (two granted restrictions, "can't attack or block
  unless"), Kaya, Spirits' Justice (two trigger events in one condition), Veil of Summer
  ("You and permanents you control gain hexproof from blue and from black": a player effect
  per color and one modification), Last Night Together ("Choose two target creatures" first;
  "After this main phase" without the main-phase check the AST makes).
- The comparison's own reading: Unnerving Assault (the normalizer moves the second clause's
  "if {R} was spent" to the front of the sentence), Jace, the Perfected Mind ("Then if ...,
  draw three. Otherwise, draw a card" against "draw a card. If ..., draw three instead"),
  Black Cat, Cunning Thief (the rendering's alternatives cover each phrase — "that library",
  "their library", "Mana of any type can be spent to cast spells this way" — but the line
  still differs somewhere the tool's diff doesn't show).
- Referents and quantities worded differently: Rocco, Street Chef and Memory Vessel ("each
  player may play the card they exiled this way"), Timesifter (the tie loop), Kalitas ("if
  that creature dies this way", "Its power is equal to that creature's power"), Ogre
  Battlecaster ("When you cast that spell": the cast, its replacement and the reflexive
  trigger are rendered by the permission path), End-Blaze Epiphany ("then choose a card
  exiled this way"), Collective Voyage ("where X is" with the cards searched for), Mindclaw
  Shaman ("from among those cards"), Rashmi ("a spell with lesser mana value", "If you
  don't cast it"), Eye of the Storm ("casts an instant or sorcery card"; "For each copy, the
  player may cast the copy"), Sworn Defender, New Blood ("with Vampire" compiled as every
  other type excluded), Green Sun's Twilight, Ravenous Gigantotherium ("Each of those
  creatures deals damage equal to its power"), Mindblade Render ("if any of that damage was
  dealt by a Warrior"), Rakdos, the Showstopper ("whose coin comes up tails"), Eye of
  Duskmantle (word order; "rather than paying"), Strongarm Tactics ("each player who didn't
  discard a creature card this way"), Call Forth the Tempest ("other spells you've cast this
  turn"), Giant Oyster, Ecological Appreciation (the chosen cards shuffled before the rest
  enter; "that each have"), Infesting Radroach ("that many rad counters": the amount is a
  custom value), Urborg Panther (the sacrifice costs' order), Lantern Flare ("X is ..."
  without "where"), Nyssa of Traken ("When you sacrifice one or more artifacts this way",
  "that many"), Thunderous Debut ("the revealed cards"; "If ..., put them onto the
  battlefield. Otherwise, ..."), Chaos Warp ("If it's a permanent card, they put it onto
  the battlefield"), Sab-Sunen ("an even number of counters": the parity arithmetic),
  Ghost of Ramirez DePietro ("card in a graveyard that was discarded or put there from a
  library this turn": the two relative clauses of an `Or` inside the card's filter),
  Counterbalance ("the same mana value as the revealed card"), Preston Garvey (the token's
  "Enchanted land has"), Neurok Transmuter ("becomes blue and isn't an artifact": the order),
  Illuna ("or into your hand"), Ghostfire Blade ("~'s equip ability costs {2} less to
  activate if it targets a colorless creature"), Sophina ("once for each nontoken attacking
  creature"), Shimian Night Stalker ("All damage that would be dealt to you this turn by
  target attacking creature is dealt to ~ instead"), Pollywog Symbiote ("costs {1} less to
  cast if it has mutate"), The Fourteenth Doctor ("a copy of a Doctor card", "If you do, it
  gains haste"), The Dalek Emperor (the villainous choice printed as prose: "That player
  sacrifices a creature of their choice, or you create ..."), Darksteel Mutation and Tenth
  District Hero (the modifications in the card's order), Princess Yue ("She's a land named
  Moon. She gains ..."), The Cyber-Controller ("They're 2/2 Cyberman artifact creatures"; the
  graveyard said after "milled this way"), Heroic Sacrifice ("the chosen creature"), Twilight
  Diviner ("entered or were cast from a graveyard", "one of them"), Weatherlight Compleated
  (the condition's place; "Then draw a card if ... If it doesn't, scry 1").
