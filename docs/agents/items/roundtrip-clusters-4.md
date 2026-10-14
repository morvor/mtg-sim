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
