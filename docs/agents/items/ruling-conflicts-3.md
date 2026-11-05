# ruling-conflicts-3: follow-up conflict cases (Nov 5, second set)

Same method, conclusion types and actions as `ruling-conflicts-1.md` and `ruling-conflicts-2.md` (an independent investigator and an adversarial skeptic for each case, at effort max; quote the sources with dates; at most two subagents at a time). Add a STATUS section at the end when done.

**Cases.** Both come from the "Apparent conflicts (Nov 5)" list in docs/agents/engine-gaps.md.
1. **Run Out of Town** (rulings-P140). The ruling has the permanent's controller choose top or bottom; the Oracle text has its owner choose.
2. **Caller of the Untamed** (rulings-P025). The ruling chooses the card to copy during activation. The decided register entry `x-defined-by-target` chooses it as the ability resolves.
3. **Wastes in limited** (rulings-P192, register entry `limited-free-basics-have-a-basic-land-type`). CR 100.2b lets a limited deck add "basic land cards" beyond the product. The Wastes ruling (2021-03-19) says Wastes must come from your card pool. The engine follows the ruling. Look into tournament rules (MTR) and release notes (Oath of the Gatewatch, 2016) as sources, and whether 100.2b's "basic land cards" is meant as the five basic land types.
4. Any further entries added to that list after ruling-conflicts-2 landed, other than Homicide Investigator, Lich and Rimescale Dragon.

The user has not yet answered ruling-conflicts-2's questions about rulings that contradict the Oracle text, so a case concluded D stays as it is: engine on the Oracle text, ruling OPEN.

DONE when every case has a conclusion as in ruling-conflicts-1, the engine and tests follow A/B/C, the register and exemptions say why, and `cargo test --workspace` passes.
