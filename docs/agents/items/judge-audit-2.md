# judge-audit-2: a second blind audit, over every hand-written card's test and a fresh sample

WHY: docs/agents/finish-line-plan.md, criterion C3 and risks 10, 11. Judge audit 1 (Oct 14,
docs/agents/reports/judge-audit-1.md) had an agent that saw only the rules predict what 58
sampled tests should assert: 225 of 225 questions agreed, bounding the wrong-test rate at about
5%. Since then the suite has grown to 28,659 tests (the count at audit 1 isn't recorded), and the
153 hand-written cards (`crates/mtg-engine/src/cards/`) are the one place no independent oracle
has looked: a wrong manual definition
with a test written from the same reading stays "verified". The user reversed one tested
behavior of two reviewed so far (Nov 7), so worker readings are worth a blind check.

Run at effort max throughout. The coordinator runs it as a local workflow, as audit 1 was
(the method is in docs/agents/verification-plan.md, "The judge audit (E)", and the report).

## Sample

- Every `tests/cards/m_<card>.rs` (153 files; one test each where a file has several: the one
  that exercises the hand-written block).
- 47 tests drawn at random (seed 20261109) from tests audit 1 did not sample
  (docs/agents/reports/judge-audit-1.json lists its 60) and that were written since it. The
  clones are shallow (history from Nov 5 only; audit 1's commit 9b0af15a isn't reachable), so
  draw from files `git log --diff-filter=A` shows added since Nov 5, else from the rulings test
  files of the batches landed since Nov 5 (docs/agents/reports/finish-line-rulings-pool.md §2
  lists them): 25 from `tests/rulings/` (prefer the ones whose `ruling!` text is an interaction,
  not a restatement), 12 from `tests/cr/` and 10 from the `engine-audit-1` areas' tests once
  they have landed.

## Steps (as audit 1)

1. A describer writes each test's situation and turns each assertion into a question (at most 6
   a test), keeping the asserted values apart.
2. A judge answers from data/ only (CR, Oracle text, rulings); forbidden to read the engine,
   tests, docs or git. Check its transcripts afterwards.
3. An adjudicator classifies every disagreement: equivalent, test wrong, judge wrong, situation
   misdescribed, ambiguous. Two refuters try to overturn each "test wrong"; it counts only if
   neither can.
4. Confirmed wrong tests become a fix item (engine or hand-written definition); ambiguous cases
   become provisional register entries for the user.

## Report

docs/agents/reports/judge-audit-2.md: method, sample, agreement rate with a one-sided 95% bound,
every confirmed test error, the disputed and ambiguous cases, and the judge's low-confidence
answers. Record how many hand-written cards the judge could not evaluate because their text
has no ruling and the CR is silent (those stay on the compile-manual list with a note).

DONE when the report is landed, every confirmed test error has a fix item or a landed fix, and
every ambiguous case is in the register.
