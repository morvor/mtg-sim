# user-decisions-nov19: the user's answers on Chaotic Transformation, "exactly N" searches and "an opponent chooses"

Implementation item (effort high; its review at max). The user answered on Nov 19. Record each answer in the register (status decided, the user's words quoted), implement it generally with tests citing cr!/ruling!/interp!, and regenerate docs/INTERPRETATIONS.md.

## 1. Chaotic Transformation and CR 608.2f (entry `per-object-actions-in-given-order`, item ruling-conflicts-10)

The user: "I agree with the conclusion on chaotic transformation, but I would mark this as a legitimate inconsistency between the CR and the rulings. Based on the wording of the ruling, it seems clear the intention of the card is for the controllers to pick the order, but I see the CR as generally disagreeing with that."

- **Behavior:** reading (2). The player who performs a per-object process orders their own repetitions of it: for Chaotic Transformation, each exiled permanent's controller chooses the order of their own reveal-until processes (as the ruling says). The controller of the resolving spell or ability orders everything else 608.2f covers (Kaboom!: the caster now chooses the order of one player's targets as it resolves, rather than the order chosen at casting; Soulfire Eruption's hand-written exception may then become the general path; check `mtg-tools manual-check`). Implement in `simultaneous.rs` (`for_each_object`): ask the chooser for each next object within a player's group, defaulting to the order given (`default-order-as-listed`); reuse `kwa::next_in_apnap_order`'s allocation where it fits.
- **Record it as a genuine inconsistency, not a reconciliation:** the register entry becomes decided, Kind: deviation (from 608.2f's plain words), saying the user judges the CR and the ruling to disagree and follows the ruling as the card's intent. Do not cite the reconciling reading as the reason. In docs/RULING_DISCREPANCIES.md, keep Chaotic Transformation's entry as "inconsistent" and add the user's judgment (Nov 19) to it: a CR/ruling inconsistency to report to Wizards. Cite the ruling with ruling!() in the test that pins it.
- Update the two tests in `tests/rulings/per_object_order.rs` and add tests: two of an opponent's permanents exiled, the opponent chooses the order (a Grafdigger's Cage case where the order is observable); Kaboom! with the caster choosing at resolution.
- A mutation run of simultaneous.rs is in progress in another session (mutation-2); keep the change to `for_each_object` minimal, and say in the report which functions changed so that session can re-run its survivors.

## 2. "Exactly N" searches (entry `exactly-n-cards-are-n-or-none`, item ruling-conflicts-9)

The user: "I agree with the recommendation on burning rune demon as well." Mark the entry decided (as recommended: N cards or none). No behavior change.

## 3. Who picks the opponent for "an opponent chooses" (new policy)

The user: "when "an opponent chooses" the controller should always get to pick the opponent unless the text, a ruling, or the CR specifically says otherwise."

- Add a decided register entry (e.g. `controller-picks-the-choosing-opponent`), quoting CR 601.7a ("If there is more than one opponent who could make such a choice, the spell's controller decides which of those opponents will make the choice.") and 602.3a (the same for abilities), which cover choices while casting or activating; the user's policy extends this to choices made as an effect resolves.
- Survey every place the engine picks an opponent to make a choice (`grep -rn "opponent" crates/mtg-engine/src` for chooser selection; known: Burning-Rune Demon's "An opponent chooses one of them" uses the first opponent in APNAP order, Turtles Forever lets its controller pick; also Fact or Fiction-style "an opponent separates", Gifts Ungiven, Fatespinner, face-down piles, "target opponent" is excluded since targeting already picks). Make one shared helper that asks the controller which opponent chooses (default: the next opponent in turn order), and route every such case through it, except where the text, a ruling or the CR says otherwise (list those exceptions in the entry with their sources quoted).
- Tests: a 3-player game where the controller picks the second opponent, for Burning-Rune Demon and at least two other wordings.

## Done

All three entries decided, docs/INTERPRETATIONS.md regenerated, docs/RULING_DISCREPANCIES.md updated, `cargo test --workspace` passes, a yield row. Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.
