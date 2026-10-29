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

## enter-sacrifice-instead-partial: "Sacrifice two untapped lands instead. If you do, ..." with fewer than two sacrifices those it can, and ~ doesn't enter
- Kind: interpretation
- Status: provisional
- Rules: 609.3 "If an effect attempts to do something impossible, it does only as much as possible." / 614.12 "Some replacement effects modify how a permanent enters the battlefield. ..."
- Question: Lotus Vale and Scorched Ruins say "If ~ would enter, sacrifice two untapped lands instead. If you do, put ~ onto the battlefield. If you don't, put it into its owner's graveyard." The sacrifice is an instruction, not an offer (Mox Diamond's "you may discard a land card instead" is one), so with two untapped lands they're sacrificed. With only one untapped land, is it sacrificed, and does ~ enter?
- Decision: The one land is sacrificed (as much as possible, 609.3), and "if you do" is true only if two were sacrificed, so ~ goes to its owner's graveyard. With no untapped land nothing is sacrificed.
- Alternatives: Treat the instruction like a cost that can't be paid in part (nothing sacrificed with one land); treat it as optional like Mox Diamond (the previous reading, which let a player keep both lands and put ~ into the graveyard).
- Why: The card's text gives no choice and no cost; 609.3 is the general rule for an instruction that can't be done in full, and the rulings ("If you don't sacrifice the lands, Lotus Vale never enters") say only that it doesn't enter.
- Affects: Lotus Vale, Scorched Ruins (the one-land versions, Lake of the Dead and the others, have no partial case).
- Code: `crates/mtg-engine/src/oracle/patterns/replacement_bodies.rs` (`enter_only_if_sacrificed`).
- Tests: `tests/cards/roundtrip_pairs_4.rs` (`lotus_vale_with_one_untapped_land_sacrifices_it_and_never_enters`, `lake_of_the_dead_sacrifices_a_swamp_without_asking`).
