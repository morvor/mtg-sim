# roundtrip-pairs-5: the next 150 untriaged mismatches (after the Nov 4 resume)

Part of the `roundtrip-followups.md` stream. Baseline (Nov 4, origin ae329d28, after round 5,
16 rulings batches and register decisions landed): 28,358 of 32,059 fully supported cards
pass; 3,551 mismatch, in 1,362 single-card clusters, 129 two-card clusters and 9 larger ones
(mostly `gap:` clusters). Runs alone.

SCOPE: pick about 150 mismatching cards that no earlier roundtrip item's "Taken" or
"Remaining" section lists (grep docs/agents/items/roundtrip-*.md): first the untriaged cards
of the 9 clusters of 3+ cards (gap: Effect, Filter, replacement Discriminant, Condition,
StaticEffect, Value, remembering Before, ...), then untriaged two-card clusters, then
single-card clusters in the tool's order, until about 150 cards. Group them by the kind of
difference before working. Write the chosen list (card, cluster key) into this brief's
"Taken" section and commit it first.

For each card: triage — parser bug first (does the AST, or a custom node's implementation,
mean what the text says, in multiplayer, with control changes, tokens, copies, replacement
effects, zero-quantity choices, instruction order a replacement could observe?) with a fix +
in-game test in `tests/cards/roundtrip_pairs_5.rs` that fails on the old code + a row in the
bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else renderer wording
(independently of the parser), or a CR-justified equivalence. Judgment calls go in the
interpretation register (CLAUDE.md). Add "Parser bugs found" and "Remaining" sections.
Regenerate docs/ROUNDTRIP.md and docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Taken

From `roundtrip --clusters 3000` / `--list` on origin ae329d28 (28,358 of 32,059 pass). Cards named
in an earlier roundtrip item's brief are left out. The untriaged cards of the 3+-card clusters,
then the untriaged two-card clusters, then single-card clusters in the tool's order (151):

- gap: Effect: Eye of Ojer Taq // Apex Observatory; Zevlor, Elturel Exile; Struggle for Sanity; Stronghold Gambit; Wheel of Potential
- gap: Filter: Alania, Divergent Storm; Dance of Many; Rain of Riches
- gap: replacement Discriminant: Unpredictable Cyclone; Land Equilibrium; Freyalise's Winds
- gap: Condition: Tainted Pact; Stand or Fall; Chaos Dragon
- gap: StaticEffect: Psychic Paper; Paleontologist's Pick-Axe // Dinosaur Headdress; Rasputin Dreamweaver
- gap: Value: Primitive Justice; Cemetery Prowler
- oracle `it` vs rendered `~`: Geistlight Snare; Crown of Doom
- oracle `during combat player can't cast` vs rendered `player can't activate ability that`: Yuriko, Blade of the Mighty
- oracle `in your graveyard with 4` vs rendered `with 4 or more card`: Nethergoyf; Winter, Cynical Opportunist
- oracle `the loyalty ability of planeswalker` vs rendered `loyalty ability of planeswalker you`: Urza Assembles the Titans; Oath of Teferi
- oracle `the milled card` vs rendered `it`: Mindshrieker
- oracle `the new target must be` vs rendered `to each`: Silver Wyvern; Rebound
- oracle `the next time a source` vs rendered `choose a permanent source the`: Eye for an Eye; New Way Forward
- oracle `the power of target creature` vs rendered `target creature you control power`: Wall of Reverence; Soul's Majesty
- oracle `there is no` vs rendered `it is not true that`: M'Baku, Jabari Chieftain; The Spear of Bashenga
- oracle `this ability` vs rendered `equip ability of ~`: Plate Armor; Arm-Mounted Anchor
- oracle `this turn cost {1} less` vs rendered `cost {1} less to cast`: Armor Wars; Goblin Maskmaker
- oracle `this turn it get -1/-1` vs rendered `get -x/-x this turn where`: Desert's Due
- oracle `those land card to the` vs rendered `each card from among it`: The Ring Goes South; Skyserpent Seeker
- oracle `to owner hand each player` vs rendered `{opt:from the battlefield} to owner hand each`: Worldpurge; Turtles in Time
- oracle `to owner hand thatit controller` vs rendered `{opt:from the battlefield} to owner hand {alt:its controller|that player}`: Chain of Vapor; String of Disappearances
- oracle `to thatit` vs rendered `∅`: Torch the Witness; Lashknife Barrier
- oracle `to the battlefield each card` vs rendered `each card you own exiled`: Parallax Wave; Parallax Tide
- oracle `total number of card in` vs rendered `number of card in`: Sage of Ancient Lore // Werewolf of Ancient Hunger; Multani, Maro-Sorcerer
- oracle `unless it controller pay {3}` vs rendered `if it would be put`: Reject; No More Lies
- oracle `unless you control` vs rendered `if you {alt:control no|do not control}`: Reaver Drone; Fathom Fleet Boarder
- oracle `up to 2 basic land` vs rendered `{opt:up to} 2 basic land card`: Reach the Horizon; Journey of Discovery
- oracle `up to 2 target creature` vs rendered `for each of up to`: Band Together; Allies at Last
- oracle `when it` vs rendered `or`: Ivora, Insatiable Heir; Autarch Mammoth
- oracle `where x is the number` vs rendered `this turn where x is`: Treeguard Duo; Ral's Staticaster
- oracle `with power 4 or greater` vs rendered `{alt:an opponent controls|your opponents control} with power 4 or`: Grim Giganotosaurus; Hidden Dragonslayer
- oracle `x can't be greater than` vs rendered `∅`: Mortarion, Daemon Primarch; Shanna, Purifying Blade
- oracle `x card in it hand` vs rendered `a number of card in`: Taster of Wares; Thieving Sprite
- oracle `x` vs rendered `{alt:that much|that many}`: Death Kiss; Vitality Hunter
- oracle `you can't cast ~ unless` vs rendered `cast ~ only if`: Proft, Sinister Mastermind
- oracle `you control 3 or more` vs rendered `there is 3 or more`: Endless Atlas; Sceptre of Eternal Glory
- oracle `you control more creature than` vs rendered `defending player control fewer creature`: Mogg Toady; Goblin Goon
- oracle `you created a token` vs rendered `there is a token you`: Bennie Bracks, Zoologist; Idol of Oblivion
- oracle `you didn't attack with a` vs rendered `it is not true that`: Curious Obsession; See Red
- oracle `you exiled with card named` vs rendered `among a card you exiled`: Arcane Savant; Caller of the Untamed
- oracle `you had other creature enter` vs rendered `a other creature entered`: Saddled Rimestag; Bellowing Elk
- oracle `you or a creature you` vs rendered `a creature you control or`: Reparations; Siren Stormtamer
- oracle `you own a card` vs rendered `there is a card you`: Dreadlight Monstrosity; Howling Galefang
- oracle `you permanent you control gain` vs rendered `gain hexproof {opt:this turn} each permanent`: Dawn's Truce; Surge of Salvation
- oracle `you sacrificed 3 or more` vs rendered `there is 3 or more`: Briarbridge Patrol
- oracle `you would create 1 or` vs rendered `1 or more treasure would`: Xorn; Jolene, the Plunder Queen
- oracle `you would get 1 or` vs rendered `1 or more energy counter`: Izzet Generatorium; Aether Refinery
- oracle `your life total is greater` vs rendered `you have more than your`: Elenda, Saint of Dusk; Cosmos Elixir
- oracle `your team` vs rendered `you control or other warrior`: Thrasher Brute; Decorated Champion
- oracle `{w} {w} was spent to` vs rendered `2 or more white mana`: Catharsis; Emptiness
- oracle `~ can't have counter put` vs rendered `if 1 or more counter`: Melira's Keepers; Tatterkite
- oracle `~ deal that much damage` vs rendered `{opt:you control} ~ deal x damage`: Last-Ditch Effort; Landslide
- oracle `∅` vs rendered `controller`: Pandemonium; Stench of Evil
- oracle `∅` vs rendered `each card from among`: Dakra Mystic; Pact Weapon
- oracle `∅` vs rendered `each permanent targeted by`: Vesuvan Duplimancy; Dack Fayden
- oracle `∅` vs rendered `have hexproof`: Sigarda, Heron's Grace; Shalai, Voice of Plenty
- oracle `∅` vs rendered `if there is 4 or`: Dragon's Approach; Sphinx's Approach
- oracle `∅` vs rendered `of it`: Codecracker Hound; Faerie Snoop
- oracle `∅` vs rendered `or`: Shield of Kaldra
- oracle `∅` vs rendered `that player`: Collective Defiance; Steal the Show
- oracle `∅` vs rendered `with a counter on it`: Mister Hyde, Monster Within; Sanctuary Warden
- oracle `∅` vs rendered `your opponent control`: Thalia, Heretic Cathar; Thalia and The Gitrog Monster
- oracle `∅` vs rendered `~`: Reveka, Wizard Savant; Ellie, Vengeful Hunter
- gap: Modification: Foraging Wickermaw
- gap: PlayerModification: False Dawn
- gap: a comparison of values of the object: Evolving Door
- gap: monarch as the turn began: Knights of the Black Rose
- gap: one-use replacement effect: Forcefield
- gap: remembering Matching: Flickerform
- gap: target slot 0 out of range: The Deck of Many Things
- oracle `+1/+1 counter on it have` vs rendered `a +1/+1 counter on it`: Sigarda's Summons
- oracle `+1/+1 counter on thatit equal` vs rendered `x +1/+1 counter on it`: Experiment Twelve
- oracle `+2/+0 for every 7 card` vs rendered `+x/+0 where x is 2`: Dark Matter Manipulator
- oracle `+x/+0 this turn where x` vs rendered `+2/+0 this turn x time`: Blademane Baku
- oracle `-1/-1 this turn for each` vs rendered `-x/-x this turn where x`: Tribute to Urborg
- oracle `-2/-1 this turn for each` vs rendered `+x/-x this turn where x`: Johtull Wurm
- oracle `-x/-x where x is 4` vs rendered `+x/+x where x is -4`: Dread Slag
- oracle `-x/-x` vs rendered `+0 minus x/+0 minus x`: Liliana of the Dark Realms
- oracle `1 back` vs rendered `{alt:a card from among it|one of it back} on top of owner`: Dimir Charm
- oracle `1 card of each card` vs rendered `15 card defending player own`: Grime Gorger
- oracle `1 damage to target opponent` vs rendered `x damage to target opponent`: Lotleth Giant
- oracle `1 mana of either of` vs rendered `∅`: Cryptic Spires
- oracle `1 of it may play` vs rendered `{alt:one|either} of it may play`: You Find Some Prisoners
- oracle `1 of it opponent of` vs rendered `each player who is a`: Bend or Break
- oracle `1 of it put the` vs rendered `a card among it in`: Write into Being
- oracle `1 of it to owner` vs rendered `it to owner hand if`: Kaito, Dancing Shadow
- oracle `1 of it to owner` vs rendered `{alt:one|either} of it to owner`: Advice from the Fae
