# register-sweep-1: every behavioral deviation in the register, the equivalence list read, one rulings number

WHY: docs/agents/finish-line-plan.md, criterion C8 and risks 10, 11. The interpretation register
(docs/INTERPRETATIONS.md, 128 entries, 0 provisional) is the user's check on the judgment calls
workers made. Behavioral differences can live only in an item's "Remaining" section instead:
Inquisitor Eisenhorn's (roundtrip-tail-5: a card drawn while casting is revealed only after the
cast, so its "this way" ability never triggers) turned out, on reading CR 121.8, to be a bug
rather than a reading (the card is revealed when the spell becomes cast, and the trigger should
fire then; gaps-residual-2 fixes it); others may be readings that belong in the register. The
renderer's equivalence list (`crates/mtg-engine/src/oracle/render/compare.rs`, `EQUIVALENCES`,
about 150 rules plus named approximations such as "cycle or discard" read as discard under
702.29d) is where a real difference could be normalized away, and nobody has read it as a whole
against the CR. And the committed docs/RULINGS_COVERAGE.md (1,558 unique texts, 4.6%) was never regenerated:
`mtg-tools rulings-coverage --check` agrees with `scripts/rulings_batches.py` on the cited count
(25,777) but counts 34,000 texts and 166 exempt where the script counts 33,955 in-scope texts
and 189 exempt, so the standard update has two sets of numbers.

Run the register work at effort max; the rest is routine. This item changes no engine behavior.

## Work

1. **Sweep the items for unregistered deviations.** Read every "Remaining", "Not done",
   "unless noted", "deviation", "approximation" and "we chose" passage in
   docs/agents/items/*.md, docs/agents/round*/*.md, docs/ROUNDTRIP.md, docs/PROPERTIES.md
   ("Remaining gaps"), docs/EVERY_CARD.md and docs/MUTATION.md (survivors called "equivalent for
   today's cards"). For each behavior the engine shows that a reasonable judge could read the CR
   differently on, or that knowingly deviates from the CR: add a register entry (provisional,
   with the CR text quoted and the alternatives) or cite the existing one, and pin it with a
   test citing `interp!("<id>")`. A difference the CR settles plainly is a bug, not a reading:
   hand it to `gaps-residual-2` (or fix it when small) rather than register it. Also the two
   medium-confidence judge calls from audit 1 (cr-04, rulings-18) if they are not registered
   yet.
2. **Read EQUIVALENCES against the CR.** For each rule and each named approximation: cite the
   CR rule that makes the two wordings the same (in the list's comment); or, where no rules
   content differs (a grammatical or number-word variant), mark it "wording only", which the
   reviewer confirms; or, if it hides a difference, remove it, let the affected cards mismatch,
   and list them for the next `roundtrip-pairs-N` item. Never cite a rule that doesn't make the
   wordings equivalent just to fill the slot. Report every rule removed with the cards it
   affected.
3. **One rulings number.** Regenerate docs/RULINGS_COVERAGE.md; make `mtg-tools
   rulings-coverage` and the batches script agree on the denominator (which texts are in scope:
   the tool counts 34,000, the script 33,955 after the Contraption and host/augment texts) and
   on the exempt count (166 against 189: find which exemption lines one side skips), with
   DEFERRED and out-of-scope texts reported separately; add a test that compares the two on the
   committed data so they stay in step.

## Report

The register entries added (each with its quotes; the provisional ones go to the user), the
equivalence rules cited or removed, and the reconciled rulings numbers.

DONE when the sweep finds nothing unregistered, every EQUIVALENCES rule carries a CR citation or
a confirmed "wording only" note, and the two rulings counts agree.
