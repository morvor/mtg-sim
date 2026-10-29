# Replacement and prevention

Judgment calls about replacement and prevention effects (CR 614–616) and the damage and
draw events they modify. Format and kinds: `docs/agents/items/interpretation-register.md`;
index: `docs/INTERPRETATIONS.md`.

## shared-color-source-locked: "A source of your choice that shares a color with X" compares with X's colors as the effect is created
- Kind: interpretation
- Status: decided
- Rules: 609.7a "... The source is chosen when the effect is created. ..." / 609.7b "Some effects from resolved spells and abilities prevent or replace damage from sources with certain properties, such as a creature or a source of a particular color. When the source would deal damage, the “shield” rechecks the source’s properties. If the properties no longer match, the damage isn’t prevented or replaced. ..."
- Question: Protective Sphere ("a source of your choice that shares a color with the mana spent on this activation cost") and Mourner's Shield ("... that shares a color with the exiled card") name a property relative to something else. When the shield rechecks the source's properties (609.7b), which colors does it compare with: those of the mana or exiled card as the effect was created, or as they are then?
- Decision: The colors as the effect is created. The source must have one of them both when it's chosen and when it would deal damage; the exiled card leaving exile, or the activation's mana being long gone, doesn't change them.
- Alternatives: Recheck against the exiled card as it is when the damage would be dealt (impossible for the mana spent, which no longer exists).
- Why: The mana spent on the activation exists only while the ability resolves, so the effect has to remember its colors; treating the exiled card the same way keeps the two wordings consistent, and the source's own colors are still rechecked as 609.7b requires.
- Affects: Protective Sphere, Mourner's Shield.
- Code: `crates/mtg-engine/src/prevention.rs` (`lock_filter`: `SharesColor`, `SHARES_COLOR_WITH_MANA_SPENT`).
- Tests: `tests/cards/c4_mana_provenance.rs` (`protective_sphere_chooses_a_source_sharing_a_color_with_the_mana_spent`).

## per-source-prevention-division: A shared "prevent X of that damage" is divided by its controller only when one source's damage exceeds X
- Kind: policy
- Status: decided
- Rules: 615.7 "... If damage would be dealt to the shielded permanent or player by two or more applicable sources at the same time, the player or the controller of the permanent chooses which damage the shield prevents. ..." / 616.1 "If two or more replacement and/or prevention effects are attempting to modify the way an event affects an object or player, the affected object's controller (or its owner if it has no controller) or the affected player chooses one to apply ..."
- Question: Cover of Winter prevents X of the combat damage each creature would deal to you and/or creatures you control, shared among those recipients; its ruling says "you decide how to divide the prevention effect" when a creature's damage is assigned to several of them (trample). When is the division asked, and what if the answer is invalid?
- Decision: Before the simultaneous damage is replaced, for each creature whose damage to two or more of your recipients totals more than X, Cover of Winter's controller divides X among them (a `Divide` decision, at most each recipient's damage). When the total is at most X there's nothing to choose (all of it is prevented). An invalid answer or a default prevents X in the order the engine replaces the damage (APNAP by recipient, then `prevention::order_for_shields`).
- Alternatives: Always prevent in the engine's order without asking (the player loses a real choice); ask only for an order (can't express splitting X across two partially prevented recipients).
- Why: Follows the ruling; the default only fills in for a missing or invalid answer.
- Affects: Cover of Winter.
- Code: `crates/mtg-engine/src/replacement.rs` (`divide_per_source_prevention`, `PreventPerSource`); `crates/mtg-engine/src/actions.rs` (`deal_damage_batch`).
- Tests: `tests/cards/c4_replacements.rs` (`cover_of_winter_prevents_x_of_each_creatures_combat_damage`, `cover_of_winter_controller_divides_x_among_one_creatures_recipients`).

## words-target-on-activation: A next-draw replacement's "any target" is chosen as the ability is activated; if it's gone, the draw is still replaced and nothing is dealt damage
- Kind: interpretation
- Status: provisional
- Rules: 115.1c "An activated ability is targeted if it identifies something it will affect by using the phrase “target [something],” ... The target(s) are chosen as the ability is activated; see rule 602.2b." / 608.2b "If the spell or ability specifies targets, it checks whether the targets are still legal. ... If all its targets ... are now illegal, the spell or ability doesn't resolve." / 614.11 "Some effects replace card draws." / Mirrorwood Treefolk ruling: "If the target creature is not on the battlefield (or is not a creature) at the time the damage would be redirected, then the damage goes on this card." / Words of War ruling: "If multiple Words have been used prior to drawing a card, then you can choose which one to apply (and use up) each time you draw a card."
- Question: Words of War ("{1}: The next time you would draw a card this turn, this enchantment deals 2 damage to any target instead.") names a target inside a replacement effect that applies later. No rule or ruling says whether the target is chosen as the ability is activated or as the draw is replaced, nor what happens when the target is no longer there when the draw is replaced.
- Decision: The target is the activated ability's (115.1c): chosen as it's activated, checked as it resolves (608.2b: if it's illegal then, the ability doesn't resolve and no shield is created), and locked into the shield. When the draw is replaced, the shield applies and is used up whatever became of the target; if the target object has left its zone (a new object, 400.7) or the player has left the game, the 2 damage is dealt to nothing. Its legality isn't checked again (the replacement effect isn't a spell or ability that targets), so hexproof or protection gained afterwards doesn't stop the damage (damage prevention by protection still applies, 702.16e).
- Alternatives: (a) The target is chosen as the draw is replaced (the replacement effect would choose targets like a triggered ability; nothing in 614 or 616 provides for that). (b) The replacement doesn't apply when its target is gone, and the draw happens (the shield would wait for a later draw or expire); this would make the shield's applicability depend on something 614.1a's "instead" doesn't mention.
- Why: 115.1c is explicit that "target" in an activated ability is chosen on activation, and the same wording on Glarecaster, General's Regalia and Mirrorwood Treefolk (targeted next-time redirections) works that way, with Mirrorwood's ruling: the effect still applies and the missing target simply gets nothing. The draw's replacement is the event the shield watches for; the damage is only what happens instead.
- Affects: Words of War; the other Words (Words of Wind, Waste, Wilding, Worship) have no target.
- Code: `crates/mtg-engine/src/oracle/patterns/replacements_next_draw.rs` (`next_draw_instead`); `crates/mtg-engine/src/prevention.rs` (`lock_def`: `ReplacementAction::Instead` → `lock_targets`).
- Tests: `tests/cards/c5_mana_replacements.rs` (`words_of_war_replaces_the_next_draw_with_damage_to_the_target_chosen_on_activation`, `words_of_war_target_gone_still_replaces_the_draw_without_damage`).

## runesword-damage-this-turn: Runesword's effects cover every creature its target has dealt damage to this turn, from the moment the damage is dealt
- Kind: interpretation
- Status: provisional
- Rules: 701.19c "... Effects that say that a permanent can't be regenerated don't preclude such abilities from being activated or such spells from being cast; rather, they cause regeneration shields to not be applied." / 704.5g "If a creature has toughness greater than 0, it has damage marked on it, and the total damage marked on it is greater than or equal to its toughness, that creature has been dealt lethal damage and is destroyed. ..." / 614.1a "Effects that use the word “instead” are replacement effects. ..."
- Question: Runesword's ability says "If the creature deals damage to a creature this turn, the creature dealt damage can't be regenerated this turn. If a creature dealt damage by the targeted creature would die this turn, exile that creature instead." Is the first sentence a trigger (which would resolve only after state-based actions destroyed, and regenerated, the creature dealt lethal damage), and does damage the target dealt earlier in the turn, before the ability resolved, count?
- Decision: Both sentences are continuous effects created as the ability resolves, lasting until end of turn, about the creatures the targeted creature has dealt damage to this turn (the turn's damage history, as Torch the Tower's "a permanent dealt damage by ~"). A creature it deals damage to can't be regenerated from that moment, so a regeneration shield doesn't save it from lethal damage, and if it would die this turn it's exiled instead. Damage it dealt earlier in the turn counts too. The targeted creature is locked in as the effect begins; damage dealt by other creatures doesn't count.
- Alternatives: (a) Read the first sentence as "Whenever that creature deals damage to a creature this turn, that creature can't be regenerated this turn" (a delayed trigger): the ban would come too late for lethal damage, and the sentence would do nothing in Runesword's usual case. (b) Count only damage dealt after the ability resolves: "this turn" and "dealt damage by" don't say so; with an attacking target, damage before the ability resolves is rare (first strike).
- Why: The Oracle wording is a conditional statement, not a "whenever" trigger, and only a continuous effect gives the sentence its evident purpose. "Dealt damage by the targeted creature ... this turn" reads as the turn's damage history, like every other "dealt damage by ~ this turn" in the engine.
- Affects: Runesword.
- Code: `crates/mtg-engine/src/oracle/patterns/prevention_6.rs`; `crates/mtg-engine/src/prevention.rs` (`lock_damage_history`); `crates/mtg-engine/src/kw/damage_history_by.rs`.
- Tests: `tests/cards/c5_mana_replacements.rs` (`runesword_creatures_dealt_damage_by_the_target_cant_regenerate_and_are_exiled`, `runesword_doesnt_affect_creatures_damaged_by_other_creatures`).

## enter-sacrifice-instead-partial: "Sacrifice two untapped lands instead. If you do, ..." with fewer than two sacrifices those it can, and ~ doesn't enter
- Kind: interpretation
- Status: provisional
- Rules: 609.3 "If an effect attempts to do something impossible, it does only as much as possible." / 614.12 "Some replacement effects modify how a permanent enters the battlefield. ..."
- Question: Lotus Vale and Scorched Ruins say "If ~ would enter, sacrifice two untapped lands instead. If you do, put ~ onto the battlefield. If you don't, put it into its owner's graveyard." The sacrifice is an instruction, not an offer (Mox Diamond's "you may discard a land card instead" is one), so with two untapped lands they're sacrificed. With only one untapped land, is it sacrificed, and does ~ enter?
- Decision: The one land is sacrificed (as much as possible, 609.3), and "if you do" is true only if two were sacrificed, so ~ goes to its owner's graveyard. With no untapped land nothing is sacrificed.
- Alternatives: Treat the instruction like a cost that can't be paid in part (nothing sacrificed with one land); treat it as optional like Mox Diamond (the previous reading, which let a player keep both lands and put ~ into the graveyard).
- Why: The card's text gives no choice and no cost; 609.3 is the general rule for an instruction that can't be done in full, and Lake of the Dead's and Heart of Yavimaya's rulings ("You have to sacrifice a swamp before this card is put onto the battlefield") settle that the sacrifice isn't optional; Lotus Vale's and Scorched Ruins' rulings ("If you don't sacrifice the lands, Lotus Vale never enters") say only that it doesn't enter, not what happens to the one land.
- Affects: Lotus Vale, Scorched Ruins (the one-land versions, Lake of the Dead and the others, have no partial case).
- Code: `crates/mtg-engine/src/oracle/patterns/replacement_bodies.rs` (`enter_only_if_sacrificed`).
- Tests: `tests/cards/roundtrip_pairs_4.rs` (`lotus_vale_with_one_untapped_land_sacrifices_it_and_never_enters`, `lake_of_the_dead_sacrifices_a_swamp_without_asking`); `tests/cards/replacement_bodies_misc.rs` (`lotus_vale_enters_only_if_two_untapped_lands_are_sacrificed`).
