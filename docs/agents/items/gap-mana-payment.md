# gap-mana-payment: Mana and payment: X limits and colours, alternative ways to pay symbols, spend triggers for abilities, player mana abilities

GOAL: The payment model lacks several rule-defined variations: minimum X, colour or generic-only restrictions on what mana may pay, paying life instead of a coloured symbol, 'can't spend mana to cast', alternative costs whose X is defined by the cost, mana that triggers when spent on abilities, and player-level mana abilities (Channel).

GAPS:

[G05] Paying costs: X restrictions and alternative ways to pay mana symbols
- Rules: CR 107.3a/107.3f (X chosen as announced, within what the text allows: "X can't be 0"); 107.3a and
  601.2b with an alternative cost whose X is defined by what is exiled (Shoals); 609.4b / 118 payment
  modifications ("Spend only black mana on X"; K'rrik: "For each {B} in a cost, you may pay 2 life rather
  than pay that mana" changes only how the cost is paid).
- Engine now: `Decision::ChooseX { source, max }` (decision.rs:73) has no minimum; every caller
  (casting.rs:1341, 1992; mana_abilities.rs:594; kw/suspend.rs:203; kw/morph_face_up.rs:151) accepts any
  n >= 0. `SpendContext` (mana.rs:423-443) has no way to say which mana may pay the X part. The only
  pay-life-for-mana mechanism is Phyrexian symbols (cost_rules.rs); there's no player modification that
  makes {B} payable with life. The alternative-cost compiler refuses X (costs_casting_alt.rs plain_cost),
  and the casting flow has no "X defined by the cost paid" path.
- Cards (UNS): Hogaak, Arisen Necropolis ("You can't spend mana to cast ~": only convoke/delve may pay);
  Thieving Skydiver, Expansive Reapplication, Marath, Will of the Wild ("X can't be 0");
  Consume Spirit, Drain Life, Soul Burn, Crypt Rats ("Spend only black mana on X"); K'rrik, Son of Yawgmoth;
  Disrupting Shoal, Nourishing Shoal and the other Shoals ("exile a blue card with mana value X from your
  hand rather than pay this spell's mana cost").
- Rulings (UNCOVERED): Disrupting Shoal "include the value chosen for that X when determining the mana value
  of that spell, even if it was cast for an alternative cost and no mana was spent on X"; K'rrik "doesn't
  modify or reduce costs you pay. It changes only how you may pay those costs", "You can't pay 2 life to pay
  for generic mana"; familiars/Helm of Awakening "the generic X cost is still considered generic" (S27 #45/#73).
- Approach: add `min` to `ChooseX` (and validate); add a per-payment restriction "X paid only with mana of
  color C" in `SpendContext` honoured by the solver; a player modification "may pay N life for each {C}
  symbol" handled where Phyrexian symbols are announced (`cost_rules::announce_phyrexian`); an alternative
  cost part "exile a card with MV X" that fixes X from the exiled card.
- Size: M

[G19] Mana: spend triggers for abilities, symbol-type restrictions, player mana abilities
- Rules: CR 106.6 (spending restrictions), 106.6a (mana that triggers when spent), 605.1a/605.3 (a player
  may be granted "any time you could activate a mana ability, you may pay 1 life: add {C}"), 107.4/118.
- Engine now: `Effect::AddManaWithSpentTrigger { add, spell_filter, body }` (ability.rs:3025-3029) only
  fires for spells; `ManaRestriction` (mana.rs) has no "can't be spent to pay generic mana" or "only on
  X" kinds; there's no player-level mana ability usable during payment (mana_abilities.rs only activates
  abilities of objects).
- Cards (UNS): Sunken Palace ("When you spend this mana to cast a spell or activate an ability, copy that
  spell or ability"), Jegantha, the Wellspring and The Rebellious Intelligence ("This mana can't be spent to
  pay generic mana costs"; ruling "You can spend mana from Jegantha's mana ability to pay for a hybrid
  symbol such as {2/W}, but only if you choose to pay the colored mana component"), Channel ("Until end of
  turn, any time you could activate a mana ability, you may pay 1 life. If you do, add {C}"; ruling "Once
  your life total is 0, you can't pay any more life").
- Approach: generalise the spent trigger to `SpendContext.is_ability`; add `ManaRestriction::NotGeneric`
  and an X-only/colour-for-X restriction (see G05) in the solver's per-symbol matching; a
  `PlayerModification::ManaAbility(Cost, ManaProduction)` offered by the payment solver like an
  object's mana ability.
- Size: M

ESTIMATED SIZE: G05 M, G19 M.

OVERLAPS: Shares the payment solver and casting.rs costs with gap-activation and gap-spell-costs.

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
