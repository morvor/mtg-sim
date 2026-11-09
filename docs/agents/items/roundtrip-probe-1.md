# roundtrip-probe-1: find the round-trip mismatches that are behavioral before the wording ones

WHY: docs/agents/finish-line-plan.md, risk 1 and item A5. 3,464 fully supported cards mismatch
the round trip (Nov 5); about 3,100 of them are single-card differences nobody has read. Of the
69 parser bugs the round trip has found, about 20 were referents bound to the wrong antecedent
("it", "that player", "that card", "another") and about 15 were dropped qualifiers or clauses
("other", "unless ...", "reveal", "you own", "your graveyard"). Those are detectable from the
AST and the text without a human: this item builds that probe so the `roundtrip-pairs-N` items
triage the likely-behavioral mismatches first.

Run it at effort max for the probe's design; the sweep itself is routine.

## The probe (`mtg-tools roundtrip --probe`, or a new subcommand)

For every mismatching card, compare the compiled AST (`roundtrip --ast`, add it if missing) with
the normalized Oracle text and flag:
1. **Referents**: a pronoun or "that <noun>" in the text whose AST `Sel`/`PlayerRef` resolves to
   something other than the text's nearest matching antecedent (object vs player, source vs
   target vs triggering object, "you" vs "that player").
2. **Dropped words with rules weight**: "other", "another", "each", "up to", "exactly", "you
   own", "you control", "your", a zone word, "unless", "instead", "reveal", "may", "can't",
   "this way", "that aren't", a duration ("until end of turn", "this turn", "next"), "target"
   counts, number words, and keyword names that appear in the text but not in the rendered text.
3. **Quantities**: a number, X or "that many" in the text with no corresponding `Value` in the
   AST of that ability.
4. **Scope**: a condition or "for each" in the text attached to a different instruction than in
   the AST (compare the clause order).

Print one line per hit: card, ability index, kind, the text span, the AST element. Rank cards by
the number and kind of hits (referents and dropped "unless/instead/other" first). Write the
ranked list to `docs/agents/items/roundtrip-probe-1.hits.tsv` and commit it; the next
`roundtrip-pairs-N` items take cards from its top.

## Checks on the probe itself

- Run it on the cards the earlier roundtrip items fixed (the bug table in docs/ROUNDTRIP.md names
  them; check out the commit before each fix or reproduce the old AST from the item notes): the
  probe must flag most of them. Report the recall on that set.
- Run it on 100 random passing cards: hits there are false positives; report the rate and tune.

DONE when the probe runs from `mtg-tools`, its recall on the known bugs and false-positive rate
on passing cards are reported in docs/ROUNDTRIP.md (a new section), and the ranked hits file is
committed. Fix any parser bug you confirm on the way (with an in-game test and a bug-table row),
but the item is the tool, not the triage.
