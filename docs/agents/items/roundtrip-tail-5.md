# roundtrip-tail-5: the second half of the 3-card clusters, plus review leftovers

Part of the `roundtrip-followups.md` stream. Baseline (Oct 14, after roundtrip-clusters-4
landed as e82abbe): 25,861 of 28,994 fully supported cards pass.

CLUSTERS TAKEN (from `roundtrip --clusters 90` on fa71495), verbatim:
    3  oracle `lose half your` vs rendered `you lose half it`  [Infernal Contract; Ebonblade Reaper; Death Wish]
    3  oracle `may put it to the` vs rendered `a card may put it`  [Primal Surge; N'Yami-Class Mother Ship; Aid from the Cowl]
    3  oracle `move any number of +1/+1` vs rendered `choose a number from 0`  [Bioshift; Scrounging Bandar; Aetherborn Marauder]
    3  oracle `much` vs rendered `many`  [Kaya, Orzhov Usurper; Foul-Tongue Shriek; Ashling the Pilgrim]
    3  oracle `of a opponent choice it` vs rendered `the chosen player control of`  [Arena; Evangelize; Magus of the Arena]
    3  oracle `on any player turn` vs rendered `{opt:on the battlefield}`  [Teferi, Temporal Archmage; Jace's Machinations; Teferi's Talent]
    3  oracle `other` vs rendered `a`  [Yorvo, Lord of Garenbrig; Imoen, Mystic Trickster; Chlorophant]
    3  oracle `permanent your opponent control lose` vs rendered `each permanent your opponent control`  [The Fire Nation Drill; Shadowspear; Shay Cormac]
    3  oracle `player play with the top` vs rendered `play with the top card`  [Wizened Snitches; Field of Dreams; Lantern of Insight]
    3  oracle `reveal the first card draw` vs rendered `when draw a card {alt:while|if}`  [Keranos, God of Storms; Primitive Etchings; Rowen]
    3  oracle `shuffle it in your hand` vs rendered `you shuffle {alt:that player's|its} hand to`  [Whirlpool Rider; Whirlpool Drake; Whirlpool Warrior]
    3  oracle `spell with the chosen name` vs rendered `player can't cast spell with`  [Nevermore; Voidstone Gargoyle; Meddling Mage]
    3  oracle `tap a permanent` vs rendered `a permanent you control is`  [Forsaken Monument; Nyxbloom Ancient; Mana Reflection]
    3  oracle `tapped as a copy of` vs rendered `as a copy of any`  [Echoing Deeps; Vesuva; Callidus Assassin]
    3  oracle `target creature where x is` vs rendered `it where x is target`  [Thickest in the Thicket; Living Armor; Soul's Might]
    3  oracle `target nonland permanent owner put` vs rendered `as this resolve {alt:target nonland permanent's owner|target nonland permanent's owner} choose`  [Temporal Cleansing; Wan Shi Tong, All-Knowing; Deem Inferior]
    3  oracle `that much` vs rendered `x`  [Geistflame Reservoir; Caustic Bronco; Harnessed Lightning]
    3  oracle `that player or` vs rendered `∅`  [Nicol Bolas, Planeswalker; Rakdos's Return; Blightning]
    3  oracle `that share a creature type` vs rendered `in your graveyard that share`  [Return from Extinction; Raise the Draugr; Unbury]
    3  oracle `thatit with the greatest power` vs rendered `a creature with power greater`  [Summon: Fenrir; Triumph of Cruelty; Triumph of Ferocity]
    3  oracle `the amount of damage it` vs rendered `{alt:that much|that many}`  [Quartzwood Crasher; Mask of the Schemer; Desmond Miles]
    3  oracle `the amount of excess damage` vs rendered `{alt:the excess damage|the amount of excess damage dealt this way|the excess damage dealt this way}`  [Lacerate Flesh; Goblin Negotiation; Hell to Pay]
    3  oracle `the next time a source` vs rendered `choose a permanent source prevent`  [Circle of Despair; Martyr's Cause; Sanctum Guardian]
    3  oracle `the next time a source` vs rendered `choose a permanent source the`  [Dark Sphere; Deflecting Palm; Bone Mask]
    3  oracle `the number of card looked` vs rendered `{alt:that much|that many}`  [Elrond, Master of Healing; Elvish Mariner; Celeborn the Wise]
    3  oracle `the number of creature tapped` vs rendered `x where x is the`  [Harmony of Nature; Jaddi Lifestrider; Marshaling the Troops]
    3  oracle `to a permanent or player` vs rendered `{opt:to a permanent or player} it deal that much`  [The Flame of Keld; Ghosts of the Innocent; Benevolent Unicorn]
    3  oracle `who can't` vs rendered `not who is {alt:the sacrificed creature's|thatit's|the sacrificed permanent's} controller`  [Momentum Breaker; Plaguecrafter; Skull Storm]
    3  oracle `who have 3 or more` vs rendered `ifs there is 3 or`  [Feed the Infection; Ixhel, Scion of Atraxa; Phyrexian Atlas]
    3  oracle `with total mana value 3` vs rendered `in your graveyard with total`  [Patch Up; Scout for Survivors; Call of the Death-Dweller]
    3  oracle `you chose a creature other` vs rendered `it exist it is not`  [Gandalf, Friend of the Shire; Galadriel of Lothlórien; Faramir, Field Commander]

ALSO TAKEN (behavior doubts the roundtrip-clusters-4 reviewer left unfixed):
- Clackbridge Troll, Desecration Demon (and other "any opponent may sacrifice a creature"
  cards): the compiled ability sacrifices each opponent's creature in turn, but the rulings
  say they are sacrificed at the same time. Check the rulings and whether anything observable
  differs (batched "one or more creatures die" triggers, SBA timing, look-back); fix if so.
- Underworld Fires and similar ("a permanent dealt damage this way"): a stale "dealt damage
  this way" record can survive when no damage is dealt. Check and fix if observable.

NOT TAKEN: roundtrip-tail-4 (in progress) has the gap clusters, "other than" and the 3-card
clusters up to and including "life equal to".

For each card: triage — parser bug first (does the AST mean what the text says, in
multiplayer, with control changes, tokens, copies, replacement effects?), with a fix + in-game
test in `tests/cards/roundtrip_tail_5.rs` that fails on the old code + a row in the bug table
(`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else renderer wording (fix, independently of
the parser), or a CR-justified equivalence. Add "Parser bugs found" and "Remaining" sections
(each still-mismatching card with its reason). Regenerate docs/ROUNDTRIP.md and
docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Result

Re-run on the branch (Oct 14): 25,861 of 28,994 passed at the start; 25,983 of 29,179
pass after (merged with origin, whose passing list had 25,861); 26,009 of 29,230 after merging
the later landings. 92 of the 93 cards in the
taken clusters pass.

## Parser bugs found

Each has an in-game test in `crates/mtg-engine/tests/cards/roundtrip_tail_5.rs` that fails
on the old code, and a row in the bug table (`FINDINGS`):

- Vesuva, Echoing Deeps, Callidus Assassin: "You may have ~ enter tapped as a copy of ..."
  entered tapped even when it copied nothing (their rulings: it enters untapped). The
  tapped part was a separate self-replacement; it's now part of
  `ReplacementAction::EnterAsCopy` (`tapped`), applied only when a copy is chosen.
- Primitive Etchings, Rowen, Keranos, God of Storms: "Reveal the first card you draw each
  turn" (Rowen's ruling: a continuous ability) was dropped; only the "whenever you reveal
  ... this way" triggers were compiled, so the card was never revealed. Now a static
  ability (`kw/reveal_first_draw.rs`) reveals it as it's drawn (once, however many apply;
  Keranos only on your turns).

Behavior doubts left by the roundtrip-clusters-4 review, checked in play (both behave as
the text says; tests pass on the old code):
- Clackbridge Troll / Desecration Demon: the opponents' choices are made in turn and the
  creatures are sacrificed together (`simultaneous.rs` performs each instruction of a
  `ForEachPlayer` for all players at once): P1 sacrificing Blood Artist and P2 a Grizzly
  Bears makes Blood Artist see both die (CR 603.10a).
- Underworld Fires: "dealt damage this way" is recorded from the damage actually dealt
  by that instruction (`record_damaged`), so a creature whose damage was prevented
  (Kor Firewalker) isn't exiled when it dies later. No stale record survives: it's set
  for every damage instruction with a source, in the resolving ability's own context.

Also checked in play: Ghosts of the Innocent halves damage rounded down; Plaguecrafter's
"each player who can't" in multiplayer.

Renderer (new files `more_terms.rs`, `you_alone.rs`; small edits elsewhere): "for each
player" over only you; "the cards from your hand" and "each player shuffles ..., then draws
that many cards"; "if it's a permanent card"; "move any number of counters"; "that much"
for a previous amount; "target creature of an opponent's choice they control"; "those
creatures fight each other"; loyalty abilities "on any player's turn" (CR 606.3, 117.1a);
"another" for a repeated counter or card; "loses A, B, and C"; "Players play with the top
card of their libraries revealed"; "Spells with the chosen name can't be cast"; "you tap a
permanent for mana" (CR 602.2); the target before its "where X is"; "the creature with the
greatest power or tied"; "puts it into their library second from the top or on the
bottom"; "that much" after "pay any amount"/"remove any number of counters"; the same life
amount in both branches; "that player or that planeswalker's controller"; "that share a
creature type" / "with total mana value" before the zone; "either of them"; event amounts
in words ("the amount of damage it dealt to that player", "the number of cards looked at
while scrying this way"); excess damage "dealt to that creature"; the chosen-source
prevention forms (any target, half that damage, deflect to that source's controller,
exile cards equal to the damage prevented); damage replacements "this turn" and "half
that damage, rounded down"; "who can't" after a sacrifice (CR 701.21a); "each opponent
who has three or more poison counters"; "if you chose a creature other than ~ as your
Ring-bearer"; "If a land card is revealed this way"; "Reveal the first card ...".

Noticed, not changed: the general damage-replacement rendering ignores a one-use
`uses: Some(1)` ("the next time") when no chosen source is involved; Circle of Despair now
goes through the chosen-source form, which says it.

## Remaining

- Chlorophant: the granted "you may put another +1/+1 counter on ~" repeats its first
  ability; rendered "a +1/+1 counter" (same action; "another" refers to the other
  ability's counter, which the renderer of one ability can't see).
