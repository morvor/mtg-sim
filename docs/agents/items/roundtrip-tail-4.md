# roundtrip-tail-4: the leftover gaps and the first half of the 3-card clusters

Part of the `roundtrip-followups.md` stream. Baseline (Oct 14, origin fa71495, after
roundtrip-clusters-3 landed; roundtrip-clusters-4 still in review): 25,715 of 28,981
fully supported cards pass; 3,236 mismatch.

CLUSTERS TAKEN (from `roundtrip --clusters 90` on fa71495), verbatim:
    8  gap: Filter  [Havengul Lich; Break the Ice; Lord of the Nazgûl]
    4  oracle `other than` vs rendered `that is not a`  [Kjeldoran Pride; Dire Flail // Dire Blunderbuss; Sporogenic Infection]
    3  gap: Condition  [Soul Swindler; The Space Family Goblinson; Medomai the Ageless]
    3  gap: Effect  [Natural Balance; Liege of the Hollows; Plague of Vermin]
    3  oracle `card` vs rendered `∅`  [Runed Halo; Garruk's Harbinger; Sanctum of Shattered Heights]
    3  oracle `creature card put to owner` vs rendered `thatit card among it you`  [Discerning Taste; Song of Blood; Liliana's Indignation]
    3  oracle `damage would be dealt to` vs rendered `a source would deal damage`  [Dralnu, Lich Lord; Wolverine, Fierce Fighter; Crumbling Sanctuary]
    3  oracle `each +1/+1 counter from ~` vs rendered `x +1/+1 counter from ~`  [Vish Kal, Blood Arbiter; Molten Hydra; Catti-brie of Mithral Hall]
    3  oracle `each creature card in your` vs rendered `creature card in your graveyard`  [Ashes of the Fallen; Varolz, the Scar-Striped; Young Deathclaws]
    3  oracle `each creature` vs rendered `for each creature {opt:on the battlefield} it`  [Wave of Reckoning; Solar Blaze; Rakdos Charm]
    3  oracle `each opponent exile a card` vs rendered `for each opponent exile a`  [Lightstall Inquisitor; Yarok's Fenlurker; Mindleech Ghoul]
    3  oracle `each opponent where x is` vs rendered `{alt:that player|it} where x is the`  [Treacherous Terrain; Typhoon; Mjölnir, Storm Hammer]
    3  oracle `each revealed card not cast` vs rendered `the rest {alt:on the bottom of owner's library|on the bottom}`  [Treasure Keeper; Neera, Wild Mage; Plargg, Dean of Chaos // Augusta, Dean of Order]
    3  oracle `each` vs rendered `a`  [King Narfi's Betrayal; Rite of Flame; Ghirapur Orrery]
    3  oracle `for each 1 damage prevented` vs rendered `put x +1/+1 counter on`  [Test of Faith; Temper; Brace for Impact]
    3  oracle `for each token you control` vs rendered `create a token that is`  [Second Harvest; Ocelot Pride; Renewed Solidarity]
    3  oracle `from among card` vs rendered `∅`  [Kaho, Minamo Historian; Jeleva, Nephalia's Scourge; Shell of the Last Kappa]
    3  oracle `from it that player` vs rendered `that player own in hand`  [Mind Warp; Extortion; Abandon Hope]
    3  oracle `from it with mana value` vs rendered `with mana value 3 or`  [Inquisition of Kozilek; Transgress the Mind; Pelakka Predation // Pelakka Caverns]
    3  oracle `greatest mana value among creature` vs rendered `{alt:greatest|highest} mana value among creature`  [Flare of Malice; Soul Shatter; Break Under Pressure]
    3  oracle `greatest mana value of a` vs rendered `greater of the greatest mana`  [Majestic Genesis; Cloudkill; Imposing Grandeur]
    3  oracle `in your hand` vs rendered `∅`  [Urban Retreat; Talon Gates of Madara; Zareth San, the Trickster]
    3  oracle `is` vs rendered `∅`  [Retromancer; Lava Runner; Fractured Loyalty]
    3  oracle `it control or activating a` vs rendered `or activating a ability that`  [Standard Bearer; Coalition Flag; Coalition Honor Guard]
    3  oracle `itself` vs rendered `~`  [Vrondiss, Rage of Ancients; Psionic Entity; Reckless Embermage]
    3  oracle `life equal to` vs rendered `x life where x is`  [Sapling of Colfenor; Phyrexian Fleshgorger; Raubahn, Bull of Ala Mhigo]

NOT TAKEN: every 4-card cluster and the 3-card clusters roundtrip-clusters-4 has (Inscriptions,
"1 mana of the color", "1 of it back", starting-life-total conditions, Gev/Gorma counters,
"a permanent dealt damage this way", "ability of creature you control", "any opponent may
sacrifice", "any player may activate", "as cast ~ may choose", "at random in your
graveyard", "choose 2 target creature", "commander you own", "who can't" after a discard);
the 3-card clusters after "life equal to" in the list (the next item, roundtrip-tail-5).

For each card: triage — parser bug first (does the AST mean what the text says, in
multiplayer, with control changes, tokens, copies, replacement effects?), with a fix + in-game
test in `tests/cards/roundtrip_tail_4.rs` that fails on the old code + a row in the bug table
(`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else renderer wording (fix, independently of
the parser), or a CR-justified equivalence. Add "Parser bugs found" and "Remaining" sections
(each still-mismatching card with its reason). Regenerate docs/ROUNDTRIP.md and
docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Parser bugs found

Tests in `crates/mtg-engine/tests/cards/roundtrip_tail_4.rs`; each fails on the old code
(checked by running them before the fix).

- Liliana's Indignation, Song of Blood, Coerced Confession (and every "[cards] put into
  [a/your/their] graveyard this way" count after a mill): the count included milled cards a
  replacement effect exiled instead (Rest in Peace, Leyline of the Void). Now only those in
  a graveyard (`value_results::this_way_sel`).
- Discerning Taste: "Put one of them into your hand and the rest into your graveyard. You
  gain life equal to the greatest power among creature cards put into your graveyard this
  way" measured the card put into the hand. A dig now records every card it put somewhere
  (`dig_steps::DUG_PLACED`), forgotten as soon as a later instruction of the same
  resolution acts on other objects (`Ctx::set_var`; review: without that, "destroy all
  artifacts ... for each card put into a graveyard this way" after a dig counted the dig's
  cards too, test with a compiled custom card).
- King Narfi's Betrayal: "exile a creature or planeswalker card from each graveyard" exiled
  one card in all. Now one chosen from each player's graveyard (`Cards::per_owner` in
  `hand_graveyard_grammar.rs`); chapter II can cast them (test); a graveyard with no such
  card gives none (3-player test). Summon: Esper Valigarmanda ("Exile an instant or
  sorcery card from each graveyard") had the same bug and is fixed by the same change.
- Verdant Command: "Counter target loyalty ability of a planeswalker" could target the
  loyalty ability of a permanent that isn't a planeswalker (CR 606.2). Now `AbilityFrom` a
  planeswalker (`phrases.rs`).

Review: the comparison's shared-subject rewrite ("each opponent sacrifices ... and
[verb]s") read the plural noun in "among creatures and planeswalkers they control" as a
verb; a plural object noun after "and" no longer starts a clause (`compare.rs`, unit
test), so Flare of Malice, Soul Shatter and Break Under Pressure pass (Soul Shatter
checked in a 3-player game: each opponent sacrifices their own greatest).

Checked in play because the rendering changed (all behave as the text says): Song of Blood
(delayed trigger count), Typhoon in a 3-player game (each opponent's own Islands), Rite of
Flame (every graveyard), Natural Balance (trim to five, search up to five minus lands,
shuffle), Treasure Keeper (the card not cast goes to the bottom too).

## Remaining

Of the 84 cards in the taken clusters, these still mismatch; each was checked against the
AST and plays as the text says (wording only):

- Sapling of Colfenor: "gain life equal to ..., lose life equal to its power" — the
  comparison's "life equal to" rewrite reaches only the first clause of the sentence.
- Havengul Lich: "When you cast it this turn" (the one card the permission is for) rendered
  "a spell this way"; "that card" as "that object".
- Garruk's Harbinger, Sanctum of Shattered Heights: "a creature card or Garruk planeswalker
  card" as "a creature or Garruk planeswalker card" (same filter).
- Psionic Entity, Reckless Embermage: "and 3 damage to itself" — the merged second damage
  instruction still names ~.
- Kaho, Jeleva, Shell of the Last Kappa: "a spell from among cards exiled with ~" as "a
  spell exiled with ~" (the cast choice is rendered through the up-to-one cast path).
  The choice is among the linked exiled cards only (`In(Linked)`, CR 607.2a), so cards
  exiled by something else can't be cast.
- Lightstall Inquisitor: sentence order of the two permission terms and the repeated
  subject of "may play".
- Wolverine, Crumbling Sanctuary: "instead that damage is dealt, but ... is healed";
  "exiles that many cards from the top" as "the top X cards".
- Neera, Wild Mage: "the rest" after the cast permission (the reveal is in an "if you do"
  branch, so the not-cast wording isn't offered).
- Renewed Solidarity: "for each token ... create a token that's a copy of it" with a
  chosen-type filter ("that type" alternative breaks the for-each form).
- Plague of Vermin: `loop totals` payment and "repeat this process until no one pays life"
  wording. Plays as the text says (tests in `tests/cards/loop_totals.rs`: 2 players,
  the process ending only after a full pass with no payment, and Two-Headed Giant).
