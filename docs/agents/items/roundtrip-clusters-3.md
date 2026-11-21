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

## Parser bugs found

Tests in `crates/mtg-engine/tests/cards/roundtrip_clusters_3.rs`; each fails on the old code.

- Adventure Awaits, Contagious Vorrac, Blossom Prancer, Pulsar Squadron Ace, Rosecot Knight:
  "You may reveal a [card] from among them and put it into your hand. ... If you didn't put
  a card into your hand this way, ...": `library::dig` never recorded whether it took a card,
  so the condition read an earlier instruction's result (you took the card and also drew).
  Fixed in `library::dig`.
- Sorcerous Spyglass, Anointed Peacekeeper, Arachne, Psionic Weaver: "look at an opponent's
  hand" revealed the hand to every player (and emitted reveal events). Now `LookAtHand`
  (`patterns/etb_choices.rs`).
- Akoum Hellkite: "~ deals 1 damage to any target. If that land is a Mountain, ~ deals 2
  damage instead": "that land" was the target (so a player target never got 2). Found
  because the new "that permanent or player" rendering of an any-target pronoun no longer
  matched "that land". Fixed in `patterns/conditions_referents.rs`.

- Darksteel Colossus, Blightsteel Colossus, Progenitus, Nexus of Fate, Legacy Weapon (found
  in review): "If ~ would be put into a graveyard from anywhere, reveal ~ and shuffle it into
  its owner's library instead" dropped the reveal. Now `ReplacementAction::RevealAndMoveInstead`
  (the modified move of Nephalia Academy's fix, so sacrifice costs such as Viscera Seer's are
  still paid); `patterns/r400_zones.rs`, `patterns/control_exile_graveyard.rs`,
  `patterns/replacement_grammar_zones.rs` (`move_instead`). Tests: sacrificed as a cost,
  dying, and a resolving spell.

Also a renderer error, not a parser bug: a move to the battlefield with no controller named
was rendered "under its owner's control"; it enters under the control of the player putting
it there (CR 110.2a), which the engine does (Purgatory's ruling). Cauldron's Gift, Tomb
Tyrant, Moldgraf Monstrosity, Hedge Shredder, Purgatory now pass.

Checked and kept: Arashin War Beast triggers once for two blockers (test, its ruling).

## Remaining

Mismatches left in the taken clusters, each checked against the AST:

- Garruk's Harbinger: "a creature card or Garruk planeswalker card" as "a creature or Garruk
  planeswalker card" (same filter).
- Arachne, Psionic Weaver: "a card type other than creature" as the list of the other
  card types (same choice).
- Anointed Peacekeeper: the cost-increase statics' word order ("spells your opponents cast
  with the chosen name"; "unless they're mana abilities").
- The Nipton Lottery: "all other creatures" as "each creature that isn't among them" (the
  chosen one).
- Allure of the Unknown, Fervent Mastery, Ingenious Mastery: the chosen opponent is named
  twice ("an opponent ..., then draws", "That opponent may cast"); rendered "choose an
  opponent. The chosen opponent ..." (the rulings: you choose the opponent).
- Devastating Mastery: "returns them" said by the opponent; rendered as an imperative inside
  the opponent's instruction (same player performs it).
- Animal Magnetism: "An opponent chooses a creature card from among them" rendered "puts";
  the dig step keeps the card among them for "Put that card onto the battlefield" (under
  your control; checked in play by a test).
- Slumbering Trudge: one paragraph compiled as two abilities ("If X is 2 or less, it enters
  tapped" as its own replacement).
- Stinging Vitriol, Pumpkin Bombs, The Fall of Kroog, Cerebral Vortex, Curse Artifact: "they"
  for the target player rendered "that player" where the subject isn't rendered through
  `with_subject` (a pronoun-only difference).
- Draco: the upkeep cost "{10}, reduced by {2} for each ..." as "{X}, where X is the greater
  of 0 and 10 minus ...". Licia, Bloodsoaked Insight: "{1} less for each 1 life" as "{X}
  less, where X is the amount of life" (cards with the "{X}, where X" form exist, e.g.
  Ezzaroot Channeler).
- Death-Rattle Oni: "creatures that were dealt damage" vs "each creature that was" (number).
- Kheru Lich Lord: "instead of putting it anywhere else" as "instead".
- Optimus Prime: "converted" vs "transformed" (CR 701.28 convert is transform's name for
  these cards); "the chosen creature" vs "it".
- Lumbering Megasloth: "for each counter among players and permanents" as a sum of two
  counts.
- Squirrel Squatters: "a token ... for each Attraction you've visited" as "X tokens, where X
  is the number of Attractions you've visited".
- Myth Unbound: "your commander" as "commander spells you own"; "your command zone".
- Frodo, Sauron's Bane: the "Otherwise" order inside the granted ability's quotes.
- Glissa Sunseeker: where the target is first named ("if its mana value ... destroy target
  artifact").
- Leyline Tyrant: "that much damage" as "X damage, where X is that many".
- Cinderheart Giant: "to a creature an opponent controls chosen at random" as "choose a
  creature ... at random. ~ deals 7 damage to it" (same).
- Path of Mettle // Metzali: face 1 now renders ("choose a creature at random that
  attacked this turn"); face 0's "that doesn't have" vs "without" and the attack trigger's
  quality list are outside the taken clusters.
- Aetherworks Marvel: the second ability's "from among them" wording (outside the cluster).
- Natural Balance: still `Effect::Custom(look: opt in)`.
