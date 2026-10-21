# Stack, casting and costs

Judgment calls about casting spells, activating abilities and paying costs (CR 117, 118,
601–602, 605, 733). Format and kinds: `docs/agents/items/interpretation-register.md`;
index: `docs/INTERPRETATIONS.md`.

## automatic-mana-payment: The engine chooses which mana abilities pay a cost
- Kind: policy
- Status: decided
- Rules: 601.2g "If the total cost includes a mana payment, the player then has a chance to activate mana abilities (see rule 605, “Mana Abilities”). Mana abilities must be activated before costs are paid." / 601.2h "The player pays the total cost. ..." / 605.3a "A player may activate an activated mana ability whenever they have priority, whenever they are casting a spell or activating an ability that requires a mana payment, or whenever a rule or effect asks for a mana payment, ..."
- Question: Which mana abilities a player activates to pay a cost is the player's choice; there is no decision for it in the agent interface.
- Decision: The engine plans the payment: mana already in the pool first, then a backtracking search over the player's mana sources, preferring cheaper sources (lands before creatures, tapping before sacrificing, e.g. a land before a Treasure) and, among equals, those that make fewer kinds of mana (a Mountain before a Volcanic Island for {R}). An agent that wants a particular payment activates mana abilities with priority first (mtg-api lists them) and the pool is spent before anything else.
- Alternatives: Ask the player for each mana ability (a `ChoosePayment` decision): exact, but far more decisions per game and an interface every agent would have to handle.
- Why: The choice rarely matters and the planner keeps the most flexible sources untapped; agents keep full control through priority-time mana abilities.
- Affects: every mana payment.
- Code: `crates/mtg-engine/src/mana_abilities.rs` (`mana_sources_with` ranking, `plan_payment`, `pay_mana`).
- Tests: `tests/cr/interp_engine_policies.rs` (`mana_is_paid_from_the_cheapest_least_flexible_sources`).

## castability-is-optimistic: The engine's list of castable spells assumes favorable choices
- Kind: policy
- Status: decided
- Rules: 601.2f "The player determines the total cost of the spell. ... Some effects may increase or reduce the cost to pay, ..." / 733.1 "If a player takes an illegal action or starts to take an action but can’t legally complete it, the entire action is reversed and any payments already made are canceled. ..."
- Question: Whether a spell can be cast can depend on choices made while casting it (a reduction "if it targets ..."). What should the list of legal actions offered with priority contain?
- Decision: `Game::legal_actions` is optimistic: a cost change that depends on the spell's targets or other choices is assumed to apply if it's a reduction and not to apply if it's an increase, so a spell is listed if some choices might make it affordable. Trying it with choices that don't is reversed (733.1) and the player keeps priority. mtg-api's `legal` module tries each listed action on a copy of the game, so external agents get only actions that can really be taken.
- Alternatives: Search every combination of targets and choices before listing a spell: exact but expensive on every priority check.
- Why: An action listed but impossible is harmless (it is reversed as the CR prescribes); an action wrongly left out would be a missing legal play.
- Affects: spells and abilities with target- or choice-dependent cost changes (Roadside Blowout, Depower, ...).
- Code: `crates/mtg-engine/src/spell_costs.rs` (`own_change_applies`, `assume_targets`); `crates/mtg-engine/src/activation_costs.rs`; `crates/mtg-api/src/legal.rs`.
- Tests: `tests/cr/interp_engine_policies.rs` (`a_spell_whose_reduction_depends_on_its_targets_is_listed_as_castable`).
