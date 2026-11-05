# register-decisions-nov5b: record the user's evening decisions of Nov 5

On Nov 5 (evening) the user accepted every recommendation below: "I accept all of these as recommended". Record each decision the way register-decisions-nov5 did:
- `Status: decided`;
- the Decision starts with "Decided by the user, Nov 5.";
- the evidence and the `interp!` citations stay.

Then regenerate INTERPRETATIONS.md, run `rulings-coverage --check`, and run `cargo test --workspace`.

1. **Arbiter of Knollridge** (ruling-conflicts-3 case 7): set its ruling aside as inconsistent with CR 801.10–801.11 and the Imperial Mask and Benediction of Moons rulings, as Gnoll War Band's was. Add an exemption line citing the decision, and set the RULING_DISCREPANCIES.md entry to the decided state.
2. **Araumi of the Dead Tide** (case 6): set aside its "you can choose any value for X" clause, which is inconsistent with CR 107.3h and the Varolz and Young Deathclaws rulings. `x-in-a-cost-equal-to-a-cards-mana-cost-is-0` becomes decided. The clause gets an exemption line citing it; the ruling's second sentence stays cited. Update the log entry.
3. **A new log kind, "copying error"**, in docs/RULING_DISCREPANCIES.md and the policy entry: a ruling whose first publication in the release notes agrees with the card while the data's text doesn't (Run Out of Town, Sanguine Spy). It falls under `ruling-contradicting-oracle-gives-way`. Re-label those two entries.
4. **`x-defined-by-target`**: confirm its corrected Affects line (Caller of the Untamed's card is chosen as X is announced, CR 601.2b).
5. Mark these entries decided:
   - `limited-free-basics-have-a-basic-land-type` (as reconciled);
   - `number-guess-range` (keep: any number from 1 to 5);
   - `attack-requirements-once-a-cost-is-paid`;
   - `next-time-damage-batch`;
   - `target-player-reference-rechecked`;
   - `full-text-undefined-mana-cost`;
   - `copy-target-legality-original-or-copy`;
   - `hybrid-half-for-a-reduction`;
   - `reflexive-per-counters-put`.

Leave any other provisional entry as it is.

DONE when the entries are decided, the exemptions and log are updated, and the checks pass.
