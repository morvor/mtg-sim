# register-sweep-1: every behavioral deviation in the register, the equivalence list read, one rulings number

WHY: docs/agents/finish-line-plan.md, criterion C8 and risks 10, 11. The interpretation register
(docs/INTERPRETATIONS.md, 128 entries, 0 provisional) is the user's check on the judgment calls
workers made. It is incomplete: at least one deviation from the CR lives only in an item's
"Remaining" section (Inquisitor Eisenhorn: a card drawn while casting is revealed only after
the cast, CR 121.8, so its "drawn this way" ability doesn't trigger then; roundtrip-tail-5). The
renderer's equivalence list (`crates/mtg-engine/src/oracle/render/compare.rs`, `EQUIVALENCES`,
about 150 rules plus named approximations such as "cycle or discard" read as discard under
702.29d) is where a real difference could be normalized away, and nobody has read it as a whole
against the CR. And docs/RULINGS_COVERAGE.md disagrees with `scripts/rulings_batches.py` on
what a cited ruling is (1,558 unique texts, 4.6%, against 25,777, 75.9%), so the standard
update has two numbers.

Run the register work at effort max; the rest is routine. This item changes no engine behavior.

## Work

1. **Sweep the items for unregistered deviations.** Read every "Remaining", "Not done",
   "unless noted", "deviation", "approximation" and "we chose" passage in
   docs/agents/items/*.md, docs/agents/round*/*.md, docs/ROUNDTRIP.md, docs/PROPERTIES.md
   ("Remaining gaps"), docs/EVERY_CARD.md and docs/MUTATION.md (survivors called "equivalent for
   today's cards"). For each behavior the engine shows that a reasonable judge could read the CR
   differently on, or that knowingly deviates from the CR: add a register entry (provisional,
   with the CR text quoted and the alternatives) or cite the existing one, and pin it with a
   test citing `interp!("<id>")`. Eisenhorn first. Also the two medium-confidence judge calls
   from audit 1 (cr-04, rulings-18) if they are not registered yet.
2. **Read EQUIVALENCES against the CR.** For each rule and each named approximation: either
   cite the CR rule that makes the two wordings the same (in the list's comment) or, if it hides
   a difference, remove it, let the affected cards mismatch, and list them for the next
   `roundtrip-pairs-N` item. Report every rule removed with the cards it affected.
3. **One rulings number.** Make `mtg-tools rulings-coverage --write` count what the batches
   script counts (a ruling text is cited when any test cites it under any card it is attached
   to; DEFERRED and out-of-scope texts reported separately), regenerate docs/RULINGS_COVERAGE.md,
   and make `rulings-coverage --check` and the script agree on the totals (add a unit test that
   compares them on the committed data).

## Report

The register entries added (each with its quotes; the provisional ones go to the user), the
equivalence rules cited or removed, and the reconciled rulings numbers.

DONE when the sweep finds nothing unregistered, EQUIVALENCES carries a CR citation on every
rule, and the two rulings counts agree.
