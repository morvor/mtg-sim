# roundtrip-gaps-3: the new gap clusters

Part of the `roundtrip-followups.md` stream. Baseline (Oct 14, origin ae5c399, after
roundtrip-tail-5 landed): 26,145 of 29,374 fully supported cards pass; 3,199 mismatch.
New cards became fully supported since the earlier gap items, and roundtrip-tail-5 made
one-use replacement effects a gap when nothing words their limit; so new gap clusters appeared.
This item runs alone (one item at a time).

CLUSTERS TAKEN (every `gap:` cluster from `roundtrip --clusters 400` on ae5c399):
   51  gap: EventsThisTurn  [Vizier of Deferment; Macabre Reconstruction; Mad Dog]
   29  gap: Value  [Nalathni Dragon; Void Mirror; Initiates of the Ebon Hand]
   15  gap: Filter  [Kotis, Sibsig Champion; Rapid Augmenter; Grist, Voracious Larva // Grist, the Plague Swarm]
   14  gap: Effect  [Tempt with Immortality; The Death of Gwen Stacy; Osseous Sticktwister]
   10  gap: one-use replacement effect  [Circle of Solace; Impulsive Maneuvers; Dazzling Reflection]
    9  gap: EventsDuring  [Tangle Kelp; Command the Stage; Shape of the Wiitigo]
    8  gap: Condition  [Karlach, Fury of Avernus; Finest Hour; Tifa, Martial Artist]
    7  gap: Sel  [Avengers Assemble!; Neyali, Suns' Vanguard; Robber of the Rich]
    2  gap: TriggerCond  [Critical Hit; Resolute Veggiesaur]
    2  gap: a remembered group no instruction names  [Spelunking; Breaking Wave]
    1  gap: Modification  [Blood Sun]
    1  gap: Restriction  [Invasion Plans]
    1  gap: keyword action undo  [Serene Sleuth]
    1  gap: several remembered numbers  [Rottenmouth Viper]
    1  gap: target slot 0 out of range  [Behemoth of Vault 0]

For each card: teach the renderer (`crates/mtg-engine/src/oracle/render/`) to word the AST
node from its meaning, independently of the parser. Once it renders, triage any remaining
mismatch: parser bug first (does the AST mean what the text says, in multiplayer, with
control changes, tokens, copies, replacement effects, zero-quantity choices?) with a fix +
in-game test in `tests/cards/roundtrip_gaps_3.rs` that fails on the old code + a row in the
bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else renderer wording, or a
CR-justified equivalence. A gap where the AST can't express the text's meaning is usually a
parser bug. For the one-use replacement gap: a card whose text has no "the next time" (or
equivalent) limit but whose AST has one is a parser bug. Add "Parser bugs found" and
"Remaining" sections (each still-mismatching card with its reason). Regenerate
docs/ROUNDTRIP.md and docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Result

26,145 of 29,374 fully supported cards passed at the start (ae5c399); 26,355 of 29,555 pass after merging the later landings (origin's passing list at that merge has 26,145 cards).
141 of the 152 cards in the taken clusters pass.

Renderer (new files `events_happened.rs`, `cast_history_terms.rs`, `opt_in.rs`,
`next_time.rs`, `remembered_groups.rs`; small hooks elsewhere): comparisons of this
turn's (or last turn's, this combat's, since your last upkeep's) events as past-tense
clauses ("if ~ didn't attack or come under your control this turn", "if three or more
cards were put into your graveyard from anywhere other than the battlefield this turn",
"if an opponent lost 3 or more life this turn", "if you attacked with three or more Merfolk
this turn", "if a source you controlled dealt 5 or more damage this turn", "during any turn
you attacked with a token"), "unless it came under your control this turn"; how a spell was
cast and paid for ("if it wasn't cast", "if one or more of them entered from a graveyard or
was cast from a graveyard", "if mana from a Treasure was spent to cast it", "if this ability
has been activated four or more times this turn", "if it's the second creature spell you
cast this turn"); opt-in choices and their follow-ups ("each player who doesn't loses 3
life", "for each opponent who does, ..."); one-use damage replacements ("the next time ...
would deal damage this turn, prevent that damage. You gain life equal to the damage
prevented this way"); "it's the first combat phase of the turn"; die triggers ("a natural
20", "your third die each turn"); "simultaneously untap all tapped creatures and tap all
untapped creatures"; "if you put a Cave onto the battlefield this way"; Rottenmouth Viper's
"for each blight counter on it, ... unless that player sacrifices ... or discards a card";
a reflexive trigger's "that permanent" (the creating ability's target, checked in play);
"the first player controls the second player during the second player's next turn"; "no
longer goaded"; Blood Sun's and Invasion Plans' rules. A condition about a target the
instruction names is offered in both orders ("Counter target spell if it was kicked").
Speculative renderings are wrapped in `attempt`, which undoes target mentions and gaps when
a wording doesn't apply. The comparison's trailing-"if" rewrite now treats a rendering's
`{alt:...}` as one piece.

## Parser bugs found

Each has an in-game test in `crates/mtg-engine/tests/cards/roundtrip_gaps_3.rs` that fails
on the old code (checked by reverting the parser files) and a row in the bug table:

- Shape of the Wiitigo: "put a +1/+1 counter on enchanted creature if it attacked or
  blocked since your last upkeep": "it" was the Aura (which never attacks), so the counter
  was always removed. `conditional_followups::trailing_if` no longer reads "it" as the
  source after an instruction naming the enchanted or equipped creature.
- Regna, the Redeemer: "if your team gained life this turn" looked only at your own life
  gain (CR 102.4). `history_conditions::team_condition` (you or a teammate; tested in
  Two-Headed Giant, both a teammate's and an opponent's gain).
- Prized Amalgam (and Grist, Voracious Larva, Archfiend's Vessel, Fblthp, the Lost):
  "if it entered from your graveyard or you cast it from your graveyard" accepted any
  graveyard, so a creature an opponent reanimated from their own graveyard returned
  Prized Amalgam. "your [zone]" is now owned by you (CR 400.3; Breathless Knight's "a
  graveyard" stays any).
- Tempt with Glory: "For each opponent who does" didn't count an opponent who accepted but
  controlled no creatures (CR 118.12, the tempting offer ruling: the number of opponents
  who accepted). `followup_each_player::done_cond`: putting counters on each of a group can
  always be done.

- (review) Rakdos, Lord of Riots, and every "You can't cast ~ unless/if [condition]" and
  "Cast ~ only during your turn" card: the condition was checked for the card's owner
  (its controller in hand or exile), not the player who would cast it (CR 109.5). Another
  player casting your Rakdos (Hostage Taker) after you lost life couldn't. A spell put on
  the stack was re-checked for its caster, so only this wrongful refusal showed (Rock
  Jockey's case was already right). `layers::collect_statics` checks such a condition for
  each player and restricts those it holds for. Tests:
  `rakdos_checks_the_opponents_of_the_player_casting_it` (fails on the old code),
  `rock_jockey_checks_the_land_plays_of_the_player_casting_it` (regression).

Checked in play (tests pass on the old code too): Mog, Moogle Warrior (the cards every
player discarded), Behemoth of Vault 0 (the reflexive trigger destroys the dies trigger's
target), Fyndhorn Druid and Wave of Rats (a dies trigger remembers what the creature did),
Samut, Vizier of Naktamun ("that creature entered this turn"), Barrin, Tolarian Archmage
("a permanent was put into your hand from the battlefield").

## Remaining

- Tangle Kelp, Shape of the Wiitigo (after the fix): pronoun placement. The condition is
  rendered before the instruction ("as long as enchanted creature attacked ..., enchanted
  creature doesn't untap"; "X. If C, Y instead" with the names swapped), so the card's "it"
  lands on the other mention. Same meaning.
- Rock Jockey: "You can't cast ~ if you've played a land this turn" is compiled as a
  restriction on any player (`CantCast { who: Any }`) conditioned on you; the engine now
  reads "you" as the player who would cast it (see the review fix above). Rendered "players
  can't cast" (renderer only).
- Grist, the Plague Swarm: "For each creature card in your graveyard, create a token that's
  a copy of it" wording (the −6, outside the taken gap).
- Malamet Battle Glyph: "Choose target creature you control and target creature you don't
  control. If the creature you control entered this turn, ..." — the targets are named in
  the condition and the fight rather than chosen up front.
- Robber of the Rich: "the top card of their library" rendered "defending player's" (the AST is right: `TopOfLibrary(DefendingPlayer)`, "their" being the defending player of the intervening if).
- Imprison: the second ability's "creatures it was blocking that had become blocked by only
  that creature" rendered "each attacking creature that had become blocked by only it"
  (the same creatures).
- Extraordinary Journey: "For each of those cards, its owner may play it" wording (first
  ability).
- Wolverine, Best There Is: "Double all damage ~ would deal" rendered as the replacement
  ("if ~ would deal damage, it deals double that damage instead").
- Professor Onyx: "If they don't, they lose 3 life. Repeat this process six more times."
  rendered "each opponent who doesn't loses 3 life", and the repetition as the sequence
  again "six times".
- Plague of Vermin: the `loop totals` payment (as in roundtrip-tail-4).
