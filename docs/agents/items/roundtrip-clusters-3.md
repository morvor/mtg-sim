# roundtrip-clusters-3: the biggest wording clusters and the leftover gaps

Part of the `roundtrip-followups.md` stream. Baseline (Oct 14, origin 0f4fcf1, after
roundtrip-gaps-2 landed): 25,440 of 28,910 fully supported cards pass; 3,440 mismatch.
(An earlier, interrupted roundtrip-clusters-3 never landed; this one starts fresh.)

CLUSTERS TAKEN (from `roundtrip --clusters 60`):
- gaps not taken by roundtrip-gaps-1: `gap: Value` (5: Black Hole, Squirrel Squatters, Myth
  Unbound), `gap: PlayerModification` (4: Electro, Assaulting Battery; Yurlok of Scorch
  Thrash; Hall of Echoes), `gap: EventsThisTurn` (3: Estinien Varlineau, Cruel
  Calculations), `gap: remembering AtRandom` (3: Cinderheart Giant, The Nipton Lottery),
  and any `gap: Effect`/`Sel`/`StaticEffect` stragglers (Natural Balance's "look: opt in")
- `that many card from the` vs `the top that many card of` (7 + 5: Garruk's Harbinger, Dream Pillager, Thor, Guardian of Midgard)
- `1 or more` vs `a` (6: Staff of the Storyteller, Akim, Forth Eorlingas!)
- `a opponent` vs `choose a opponent the chosen` (6: Wishclaw Talisman, Akroan Horse, Wall of Shards)
- `it` vs `~` (6: Repeat Offender, Deep-Sea Kraken, Profane Procession)
- `didn't put a card to` vs `do not` (5: Contagious Vorrac, Blossom Prancer)
- Genju `is put to a graveyard` (5)
- `it` vs `that player` (5: Stinging Vitriol, Pumpkin Bombs, The Fall of Kroog)
- `more target creature become` vs `99 target creature` (5: Dwarven Song, Touch of Darkness)
- `or player` vs `∅` (5: Guardian Angel, Urza's Rage, Arrow Storm)
- `to a graveyard from anywhere` (5: Darksteel Colossus, Progenitus, Nexus of Fate)
- `{2} less to cast for` vs `{x} less to cast where` (5: Draco, Eris)
- `∅` vs `under it owner control` (5: Tomb Tyrant, Cauldron's Gift, Purgatory)
- `∅` vs `you` (5: Nazgûl, Wyll, Barbarian Class)

NOT TAKEN: the Filter/Condition/replacement Discriminant/TriggerCond/target-slot/triggering
object gaps (roundtrip-gaps-1, in review) and the clusters of 4 or fewer wording
mismatches (the next item).

For each card: triage — parser bug (fix + in-game test in `tests/cards/roundtrip_clusters_3.rs`
+ row in the bug table, `FINDINGS` in crates/mtg-tools/src/roundtrip.rs), renderer wording
(fix, independently of the parser), or a CR-justified equivalence. Add a "Remaining"
section listing each still-mismatching card with its reason. Regenerate docs/ROUNDTRIP.md
and docs/roundtrip-passing.txt; `roundtrip --check` must pass.
