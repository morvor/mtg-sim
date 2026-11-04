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

## Result

28,358 of 32,059 fully supported cards passed at the start (ae329d28); 28,443 of 32,059 on the branch after merging origin. 65 of
the 151 taken cards pass.

Parser (one fix, see "Parser bugs found"): `patterns/r104_game_end.rs` ("its owner" /
"its controller loses the game" names the owner of the object "it" is).

Renderer (most in the new `oracle/render/pairs5_terms.rs`, hooks in the existing files):
"~ can't have counters put on it"; "If you would get one or more {E}, you get that many
plus one {E} (twice that many {E}) instead"; "The new target must be a creature (a
player)"; "X can't be greater than ..." after a "you may pay {X}" (the cap the payment
reads, `value_results::X_MAX`, was silently dropped); "if there is no monarch"; "if you
didn't attack with a creature this turn" (intervening "if"); "if you created a token this
turn"; "if you sacrificed three or more Clues this turn"; "you had another creature enter
the battlefield under your control this turn"; "Equip {3}. This ability costs {1} less to
activate for each ..." (any own cost change of a keyword's abilities, with a condition
too); "the power of target creature you control"; "where X is the amount of {E} paid this
way" for an X/X token; "[instruction] unless you control another colorless creature";
"When ~ enters and whenever it ..." (an equivalence: one triggered ability with two
trigger conditions, CR 603.1b, 603.2); "Counter target spell unless its controller pays
{3}. If that spell is countered this way, exile it instead ..." (the replacement after the
payment); "~ costs {1} less ... It also costs {1} less ..."; "During combat, players can't
cast spells or activate abilities that aren't mana abilities"; "You and Humans you control
have hexproof", "You, planeswalkers you control, and other creatures you control have
hexproof"; "Activate only if you control three or more lands with the same name"; "~ can't
attack (block) unless you control more creatures than defending (attacking) player"; "You
can't cast ~ unless ..."; "Artifact spells you cast this turn cost {1} less to cast"; "It
gets an additional -1/-1 until end of turn for each ..." (also after "If ~ was kicked");
"-X/-Y" (a power and toughness change by two different negative amounts was "+X/-X" with
one X, a wrong rendering); "-4/-4 for each ..." (negative multiples); "+2/+0 for every seven
cards in your graveyard"; "goad up to X target creatures" after "becomes monstrous" (CR
701.37c); "reveals X cards from their hand, where X is ..."; "that targets you or a
creature you control" (either order); "if you own a card in exile"; "If you would create
one or more Treasure tokens, instead create those tokens plus an additional Treasure
token"; "the total number of cards in all players' hands"; "cards in your graveyard with
four or more card types among them"; "the loyalty abilities of planeswalkers you control"
and "twice this turn rather than only once"; "Put one into your hand and the other ...";
"if {W}{W} was spent to cast it"; "+X/+X or -X/-X"; "Creatures and nonbasic lands your
opponents control enter tapped"; "You and permanents you control gain hexproof until end
of turn".

## Parser bugs found

Test in `crates/mtg-engine/tests/cards/roundtrip_pairs_5.rs`; it fails on the old code
(checked by running it before the fix):

- The Deck of Many Things: "20 | Put a creature card from any graveyard onto the
  battlefield under your control. When that creature dies, its owner loses the game." "Its
  owner" in "loses the game" was always read as the owner of target 0 (`phrases::
  parse_player`), and the delayed ability has no targets, so when the creature died nobody
  lost. `patterns/r104_game_end.rs` reads "its owner"/"its controller" through
  `effects::player_ref` (the object "it" is: the creature that died); test
  `the_deck_of_many_things_creature_owner_loses_when_it_dies`.

Checked in play (same file, passing on the old code too): Pact Weapon uses the drawn
card's mana value, and gives no bonus and costs no life when Thought Reflection replaces
the draw (its ruling); Chain of Vapor asks the bounced permanent's controller as it last
existed (Control Magic), not its owner. Also checked by reading the AST or existing tests:
Shanna and Mortarion (the X cap is applied, `value_results_payments.rs`), Tatterkite and
Melira's Keepers (a cost putting a counter on them can't be paid, `r_p123_no_counters.rs`),
Wheel of Potential (`c5_costs_sets.rs`), Stronghold Gambit (ties enter under each owner's
control, `cards5_choices_players.rs`), Cryptic Spires (`r207_text_box.rs`), The Ring Goes
South and Skyserpent Seeker (only the land cards found go onto the battlefield,
`DUG_CHOSEN`), Johtull Wurm and Dark Matter Manipulator (the AST had the right numbers;
only the rendering was wrong), Write into Being (the other card is on top after the
manifest, so "on the top or bottom" is the choice offered), Kaito, Dancing Shadow (one of
the creatures), Parallax Wave (each player returns the cards they own), Turtles in Time
(only the players who shuffle draw), Tainted Pact, Struggle for Sanity, Band Together
(another target), Reject (the payment, then the replacement), Eye for an Eye (any source,
CR 609.7a, `prevention::source_candidates`).

## Remaining

Of the 151 taken cards, these still mismatch; each was checked against the AST and plays as
the text says (wording only) unless noted:

- Custom nodes or replacement shapes not worded (`gap:`): Eye of Ojer Taq, Zevlor, Struggle
  for Sanity, Stronghold Gambit, Wheel of Potential (opt-in), Alania, Dance of Many, Rain
  of Riches, Unpredictable Cyclone, Land Equilibrium, Freyalise's Winds, Tainted Pact,
  Stand or Fall, Chaos Dragon, Psychic Paper, Paleontologist's Pick-Axe, Rasputin
  Dreamweaver, Primitive Justice, Cemetery Prowler, Foraging Wickermaw, False Dawn,
  Evolving Door, Knights of the Black Rose, Forcefield, Flickerform.
- "The next time a source of your choice would ...": Eye for an Eye, New Way Forward (the
  choice, then the shield, as Refraction Trap and Healing Grace in roundtrip-pairs-4).
- One printed sentence compiled as several, or several abilities as one line: The Deck of
  Many Things ("If the result is 0 or less, discard your hand" after the table: no row
  applies then, so the order isn't observable), Reveka, Ellie ("and doesn't untap", "and
  gains indestructible": the subject repeated), Collective Defiance, Steal the Show ("then
  draws that many": the subject repeated), Dawn's Truce ("also gain indestructible"),
  Elenda, Cosmos Elixir ("your life total is greater than your starting life total": the
  predicate is worded for the player), Dragon's Approach, Sphinx's Approach (the "four
  cards named ~" condition said before the choice).
- Referents and quantities worded differently: Crown of Doom ("gains control of it"),
  Mindshrieker ("the milled card's mana value"), Pact Weapon ("The creature" for the
  equipped attacker), The Ring Goes South, Skyserpent Seeker ("those land cards"),
  Worldpurge ("shuffles the rest"), Turtles in Time ("each player who does"), Chain of
  Vapor, String of Disappearances ("If the player does, they may copy ~ and may choose a
  new target for that copy"), Torch the Witness ("to that creature"), Lashknife Barrier
  ("to that creature instead"), Parallax Wave, Parallax Tide ("each player returns ... all
  cards they own": "you" inside the per-player instruction), Last-Ditch Effort, Landslide
  ("that much damage" for the number sacrificed), Stench of Evil, Pandemonium ("they"),
  Dakra Mystic ("the revealed cards"), Vesuvan Duplimancy, Dack Fayden ("those
  permanents"), Kaito, Dancing Shadow ("one of them"), Experiment Twelve, Lotleth Giant
  ("for each"), Blademane Baku ("For each counter removed"), Johtull Wurm ("beyond the
  first"), Dimir Charm ("Put one back"), Grime Gorger ("up to one card of each card
  type"), Bend or Break ("one of their opponents of their choice"), Write into Being,
  Advice from the Fae ("Otherwise, put one of them"), You Find Some Prisoners ("Until the
  end of your next turn" first), Taster of Wares ("If an instant or sorcery card is exiled
  this way"), Arcane Savant, Caller of the Untamed ("a card you exiled with cards named
  ~"), Cryptic Spires (the circled colors are written into the card's text when circled,
  `deck::circle_colors`; the printed card's ability adds nothing).
- Word order: Winter, Cynical Opportunist ("a permanent card from among them"), Reach the
  Horizon ("with different names" once for both kinds), Journey of Discovery ("up to two
  additional lands"), Band Together, Allies at Last ("each deal damage equal to their
  power"), Treeguard Duo ("until end of turn," first), Ral's Staticaster ("for each card in
  your hand until end of turn"), Grim Giganotosaurus, Hidden Dragonslayer ("with power 4 or
  greater an opponent controls"), Thrasher Brute, Decorated Champion ("your team
  controls"), Shield of Kaldra ("Sword of Kaldra, ~, and Helm of Kaldra"), Mister Hyde,
  Sanctuary Warden ("with a counter on it" stated), Sigarda's Summons ("have base power and
  toughness 4/4, have flying, and are Angels").
