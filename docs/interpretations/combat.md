# Combat

Judgment calls about attacking, blocking and combat damage (CR 506–511). Format and kinds:
`docs/agents/items/interpretation-register.md`; index: `docs/INTERPRETATIONS.md`.

## combat-tax-written-zero-is-a-cost: A stated {0} cost to attack or block is still a cost
- Kind: interpretation
- Status: decided
- Rules: 118.5 "Some costs are represented by {0}, or are reduced to {0}. The action necessary for a player to pay such a cost is the player’s acknowledgment that they are paying it. Even though such a cost requires no resources, it’s not automatically paid." / 508.1d "... If a creature can’t attack unless a player pays a cost, that player is not required to pay that cost, even if attacking with that creature would increase the number of requirements being obeyed. ..." / 509.1c "... If a creature can’t block unless a player pays a cost, that player is not required to pay that cost, even if blocking with that creature would increase the number of requirements being obeyed. ..." / War Tax ruling: "You can choose X=0. This doesn’t have an effect most of the time, but it does mean that creatures can’t attack unless the controller chooses to pay the cost (a cost of zero is not automatically paid, so the controller can choose to not pay)." / War Cadence ruling: "You can choose X=0. This doesn’t have an effect most of the time, but it does mean that creatures can’t block unless the controller chooses to pay the cost (a cost of zero is not automatically paid). You can use this to make your creatures unable to block an attacker."
- Question: War Tax or War Cadence with X = 0 locks in a cost of {0} to attack or block. Is that a cost (so attack and block requirements don't force the creature), or no cost at all?
- Decision: A cost. An effect whose locked cost is an explicit {0} still makes attacking or blocking optional for a creature that must otherwise attack or block.
- Alternatives: Treat {0} as no cost: requirements would force the attack or block, contradicting both rulings.
- Why: 118.5 and the two rulings say so directly; recorded because the engine has to tell a written {0} apart from a scaled cost that comes to nothing (see `combat-tax-scaled-to-nothing`).
- Affects: War Tax, War Cadence (X = 0).
- Code: `crates/mtg-engine/src/combat.rs` (`is_written_zero`, `required_attack_cost`, `required_block_cost`, `block_cost_for`).
- Tests: `tests/cards/combat2_referents.rs` (`war_tax_with_x_zero_makes_attacking_optional`, `war_cadence_with_x_zero_makes_blocking_optional`).

## combat-tax-scaled-to-nothing: A scaled tax to attack or block that comes to nothing isn't a cost
- Kind: interpretation
- Status: provisional
- Rules: 118.5 "Some costs are represented by {0}, or are reduced to {0}. The action necessary for a player to pay such a cost is the player’s acknowledgment that they are paying it. Even though such a cost requires no resources, it’s not automatically paid." / 508.1d (above) / 508.1h "If any of the chosen creatures require paying costs to attack, or if any optional costs to attack were chosen, the active player determines the total cost to attack. ... Once the total cost is determined, it becomes “locked in.” ..." / 509.1d "If any of the chosen creatures require paying costs to block, the defending player determines the total cost to block. ..."
- Question: A tax scaled by a count ("pays {X} for each creature they control that's attacking you, where X is the number of basic land types among lands you control", "{1} for each counter on it") can come to nothing. Is a creature under such a tax one that "can't attack unless a player pays a cost" (so requirements don't force it), or is there no cost?
- Decision: No cost. When every applicable tax is a scaled amount that comes to nothing, the creature has no cost to attack or block, so "attacks each combat if able" and similar requirements force it.
- Alternatives: Treat any applicable tax as a cost, even when it comes to {0} (118.5's "reduced to {0}"): the creature would never be forced to attack or block while the tax applies, and the attacking player would acknowledge paying {0}.
- Why: A scaled tax that comes to nothing describes no payment, unlike War Tax's chosen X = 0 that the rulings single out; the engine keeps such creatures under their requirements. Provisional: 118.5 also covers costs "reduced to {0}", and a judge could read a scaled cost of {0} the same way as a written one.
- Affects: Collective Restraint (no basic land types), and other per-creature or per-counter taxes whose multiplier is 0 (count not measured).
- Code: `crates/mtg-engine/src/combat.rs` (`is_written_zero`: only a stated {0} counts; `required_attack_cost`, `required_block_cost` drop a total that is free otherwise).
- Tests: `tests/cr/interp_engine_policies.rs` (`a_scaled_attack_tax_that_comes_to_nothing_isnt_a_cost`).
