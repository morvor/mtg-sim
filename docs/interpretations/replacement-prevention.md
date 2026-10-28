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
