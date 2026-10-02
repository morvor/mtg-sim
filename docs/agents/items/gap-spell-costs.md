# gap-spell-costs: Spell costs from other objects: external alternative costs, optional additional costs, minimum costs, any-graveyard costs

GOAL: Only a spell's own alternative and optional costs are offered; cost-changing statics of other permanents can only increase or reduce. This item makes the casting flow accept alternative costs, optional/choice additional costs and flash-for-cost from other objects, a minimum total cost (Trinisphere), and exile costs that may use any graveyard.

GAPS:

[G01] Alternative costs offered by other objects ("rather than pay the mana cost for spells you cast")
- Rules: CR 118.9, 118.9a (only one alternative cost), 601.2b, 601.2f; jump-start (702.133) is not an
  alternative cost, so it combines with one (Radical Idea ruling).
- Engine now: `Game::cast_options` (crates/mtg-engine/src/casting.rs:610-637) builds alternative-cost
  options only from the card's own `CostTarget::ThisSpell` statics (plus Omniscience-style "free from hand",
  casting.rs:638-650). `CostModifier { applies_to: CostTarget::Spells(_), change: AlternativeCost(_) }` from
  other permanents is explicitly ignored when totalling costs (casting.rs:1634-1637) and never produces a
  cast option. A cast action is identified by card + `CastMethod`, so "jump-start/permission X *and* an
  external alternative cost" can't be expressed.
- Cards (UNS): Fist of Suns, Jodah, Archmage Eternal, Rooftop Storm, Runeforge Champion, Darksteel
  Monolith, As Foretold, Grenzo, Crooked Jailer, Charred Foyer, Tlincalli Hunter, Demon of Fate's Design,
  Cramped Vents ("once each turn/once during each of your turns" variants need per-source use tracking,
  cf. `TurnHistory::once_permissions_used`, game.rs:446). (Alternative costs attached to a resolved
  effect's permission are G18; static `PlayPermission`s already carry a `cost`, ability.rs:2278-2279.)
- Rulings to cite once fixed: Radical Idea "you may pay that alternative cost when you jump-start a spell";
  Fist of Suns "You can't combine this with other alternative costs, such as flashback", "the only legal
  choice for X is 0".
- Approach: in `cast_options`, also scan `self.statics.cost_modifiers` for `Spells(f)` +
  `AlternativeCost` matching the would-be spell (reuse `spells_change_applies_as`), producing an option
  per (face, method) that has no alternative cost yet (normal cast, jump-start, permission casts, not
  flashback/escape/other alt costs). Add an optional "once each turn" use key.
- Size: M

[G02] Optional/choice additional costs and flash-for-cost granted to classes of spells by other objects
- Rules: CR 601.2b, 601.2f, 118.8; 601.3c.
- Engine now: from other permanents' statics only `AdditionalCost`, increases and reductions apply;
  `OptionalAdditionalCost`, `AdditionalCostChoice` and `FlashForAdditionalCost` with
  `CostTarget::Spells(_)` are dropped (casting.rs:1634-1637); `cost_choices.rs` only reads the spell's own.
- Cards (UNS): Defiler of Vigor/Faith/Dreams/Flesh/Instinct ("As an additional cost to cast green permanent
  spells, you may pay 2 life. Those spells cost {G} less ... if you paid life this way"), Chorus of the
  Conclave ("you may pay any amount of mana").
- Rulings: Defiler of Vigor "You may only pay the additional cost once per permanent spell."; "this ability
  does not cause the spells to have Phyrexian mana symbols".
- Also missing: a minimum total cost applied after all increases and reductions ("each spell that would
  cost less than three mana to cast costs three mana", Trinisphere, UNS; `CostChange` has no such kind,
  ability.rs:2092-2119). Ruling: Trinisphere "start with the mana cost or alternative cost you're paying,
  add any cost increases, then apply any cost reductions. Finally, apply Trinisphere's effect".
- Approach: collect optional additional costs from matching `Spells(f)` modifiers in `cost_choices.rs`
  (announced at 601.2b, recorded under a per-source name in `CastInfo::paid`); allow a cost modifier to
  be conditional on that name; add `CostChange::MinimumMana(n)` applied last.
- Size: S-M

[G20] Cost parts that look only at your own zones
- Rules: CR 118 / 602.2b: "Exile a Fungus card from a graveyard" may use any graveyard.
- Engine now: `cost_zone_cards` (casting.rs:2494-2508) returns only `p`'s graveyard/hand/library for
  `CostPart::Exile { zone, .. }`; "from a single graveyard" needs G06's group rule.
- Cards: Thelon of Havenwood ("{B}{G}, Exile a Fungus card from a graveyard": compiles, but can only use
  your own graveyard), Night Soil (UNS: "Exile two creature cards from a single graveyard").
- Approach: carry an owner qualifier on `CostPart::Exile` (yours / any / single graveyard) and choose
  across all graveyards.
- Size: S

ESTIMATED SIZE: G01 M, G02 S-M, G20 S.

OVERLAPS: Shares casting.rs cost code with gap-activation and gap-mana-payment and `cast_options` with gap-cast-permissions.

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
