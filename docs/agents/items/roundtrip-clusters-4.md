# roundtrip-clusters-4: the 3- and 4-card wording clusters

Part of the `roundtrip-followups.md` stream. Baseline (Oct 14, after roundtrip-gaps-1 and
roundtrip-gaps-2 landed): about 25,596 of 28,916 fully supported cards pass.

CLUSTERS TAKEN (from `roundtrip --clusters 60` on 0f4fcf1; 4- and 3-card wording clusters):
- 4: `a` vs `each` (Savaen Elves, Miracle Worker, Seedling Charm); `become that type` vs
  `lose each creature type` (Unnatural Selection, Mistform Mutant, Standardize); `by it` vs
  `by enchanted creature` (Sandskin, Heart of Light, Ghostly Possession); `creature card
  milled this way` (Necron Monolith, Gorging Vulture); `creature tapped this way` (Siege
  Striker, Ledev Champion); `damage would be dealt` vs `a source would deal damage`
  (Phytohydra, Force Bubble); `exile the top card of` vs `look at the top card` (Ugin, the
  Ineffable; Mindleecher; Rogue Class); `may cast/put any number of` (Etali, Primal Storm;
  Kamahl's Druidic Vow); `may choose a nonland card` (Reckoner Shakedown, Cracked Skull);
  `or become blocked by a` (Dead-Iron Sledge, Mammoth Harness); `target opponent lose x life`
  vs `that player` (Astarion, Villainous Wrath); `that player` vs `it` (Vision, Price of
  Glory, Tataru Taru); `thatit would be put to` (Vohar, Sins of the Past); `the mana value of
  thatit` (Pyromancy, The Kami War); `the milled card` vs `it` (Heed the Mists, Lluwen);
  `this turn` vs `∅` (Battlegate Mimic and the other Mimics); `who can't` (Refurbished
  Familiar, Cruel Grimnarch); Visions' flashback cost reduction; `∅` vs `or` (Doorman,
  Henrika Domnathi, Glamdring)
- 3: Inscriptions' kicker; `1 mana of the color` (Astral Cornucopia, Selvala); `1 of it
  back` (Gutless Plunderer, Sage of Days); starting-life-total conditions (Chalice of Life,
  Righteous Valkyrie, Leyline of Hope); `a +1/+1 counter` vs `x` (Gev, Gorma); `a permanent
  dealt damage this` (Chandra, Awakened Inferno; Underworld Fires); `ability of creature you
  control` (Thousand-Year Elixir, Tyvar); `any opponent may sacrifice` (Clackbridge Troll,
  Desecration Demon); `any player may activate` (Scandalmonger, Task Mage Assembly);
  `as cast ~ may choose both` (Sakashima's Will, Szat's Will); `at random in your graveyard`
  (Radiant Scrollwielder, Tersa Lightshatter); `but` vs `you` (Edea, The Horus Heresy);
  `choose 2 target creature controlled` (Trial of Agony, Barrin's Spite); `commander you own`
  (Road of Return, Command Beacon, Netherborn Altar)

NOT TAKEN: roundtrip-clusters-3 (in progress) has the leftover `gap:` clusters and every
wording cluster of 5+ cards. Re-run `--clusters 80` first: if a cluster above has changed
shape after the landings, keep to the same cards.

For each card: triage — parser bug (fix + in-game test in `tests/cards/roundtrip_clusters_4.rs`
+ row in the bug table, `FINDINGS` in crates/mtg-tools/src/roundtrip.rs), renderer wording
(fix, independently of the parser), or a CR-justified equivalence. Add a "Remaining"
section listing each still-mismatching card with its reason. Regenerate docs/ROUNDTRIP.md
and docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Result

Re-run on the branch (Oct 14): the clusters kept their shape, with one or two more cards
each (`--clusters 80`); the taken clusters hold 120 cards. 25,597 of 28,918 cards passed
at the start; 25,767 of 28,971 pass after (merged with origin, whose passing list had 25,593 before this item).

Renderer changes (all in `crates/mtg-engine/src/oracle/render/`, new files
`block_pairs.rs`, `chosen_from_targets.rs`, `flashback_reduction.rs`, `joined_lines.rs`):
"attached to a land"; "becomes that type" (CR 205.1a); "dealt to and dealt by" as one
line; "creatures tapped/milled/dealt damage this way" (what-this-way tracking through
sequences, conditions and remembered choices); "if damage would be dealt to ~" with no
source named (CR 120.1); CR 406.3's "look at it, then exile it face down" = "exile it face
down. You may look at it ..."; "you may cast any number of spells from among them" and
"put the rest"; "you may choose a nonland card from it. If you do / If you don't";
"the other creature" of a block and "both creatures"; "~'s base power and toughness become
equal to that creature's"; the target player as the subject before "they" in "where X
is"; "their/that player's turn"; "who can't" after a discard (CR 701.9a);
"at least N life more than your starting life total"; "one mana of the color of your
choice for each" (Selvala's rulings: one color); "one of them back on top"; "any player
may activate this ability but only as a sorcery"; "{opt:activated} abilities" for
activation permissions (CR 602.1); "any opponent may sacrifice a creature. If a player
does / If no one does" (Clackbridge Troll's rulings); "choose two target creatures ...
That player chooses one of them"; "your commander ... from the command zone"; "but don't
own"; "at random from your graveyard"; "choose any number instead" (kicked); a mode's
"where X is"; flashback with its "costs {X} less to cast this way"; per-part "until end of
turn"; "each other creature" after a choice; "the exiled card" in a linked ability
(CR 607.2a). Comparison: "the mana value of it" = "its mana value" (possessive), "a
permanent that's one or more colors" = "colored permanent" (CR 105.2), the
"enters with a counter for each" forms with "additional"/"them"/"plus".

Parser bugs: none found in these clusters. Each card's AST was read against the Oracle
text and its rulings; the doubtful ones were checked in play in
`crates/mtg-engine/tests/cards/roundtrip_clusters_4.rs` (Severance Priest's linked exiled
card, Desecration Demon in multiplayer, Underworld Fires, Mammoth Harness, Refurbished
Familiar, Trial of Agony, Gev, Gorma, Visions of Glory's flashback reduction, Siege
Striker, Phytohydra, Heart of Light, Scandalmonger, Thousand-Year Elixir): all behave as
the text says.

## Remaining

Still mismatching in the taken clusters, each checked against the AST and judged to play
as the text says; what's left is wording:

- Doorman: "creatures with power 2 or less and/or Walls" rendered with "or" (the and/or
  equivalence turns the Oracle text into "and"); same meaning.
- Blitzwing, Cruel Tormentor // Adaptive Assailant: "if no life is lost this way" as "if
  that many is 0"; "choose flying or indestructible at random" as a random number (listed
  by roundtrip-gaps-2 already).
- Fevered Suspicion: "from among those nonland cards" as "from among them" (the found
  nonland cards, `RevealUntil` leaves them in "it").
- Mindleecher: "You may look at and play those cards" as two permissions (look, then play).
- Nexus Mentality: "draw a card for each counter removed this way" as "draw X cards, where
  X is that many".
- Szat's Will: "a creature they control with the greatest power" word order.
- Kamahl's Druidic Vow: "land and/or legendary permanent cards with mana value X or less"
  with the mana value said for each kind.
- Nashi, Searcher in the Dark: "legendary and/or enchantment cards" as "legendary card or
  enchantment cards"; "If you put no cards into your hand this way" as "If you don't".
- Blex, Vexing Pest // Search for Blex: "any number of them" as "any number of cards from
  among them"; "for each card you put into your hand this way" as "that many".
