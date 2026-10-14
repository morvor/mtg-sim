# cr-examples-N: turn the Comprehensive Rules' worked examples into tests

WHY: the Comprehensive Rules have 277 paragraphs that start with "Example:". The rules team wrote them, and they are often exactly the tricky cases. Each one says independently what must happen, so a test built from it checks the engine against the rules' own authors, not against our reading of the rules. The user chose this on Oct 14.

ITEMS. An example belongs to the rule paragraph it follows. Counts are from the CR effective 2026-09-25.

| Item | Rules | Examples |
|---|---|---:|
| cr-examples-1 | the tooling below, plus 100–199 | 34 |
| cr-examples-2 | 200–599 | 33 |
| cr-examples-3 | 600–611 | 47 |
| cr-examples-4 | 612–699, 700, 701, 703–706 | 46 |
| cr-examples-5 | 702, 707 | 49 |
| cr-examples-6 | 708–799 (707 excluded), 900–999 | 33 |
| cr-examples-7 | 800–899 | 35 |

Items 2–7 use item 1's tooling. Start them only after cr-examples-1 has landed.

TOOLING (cr-examples-1):
- **Citation macro.** Add `cr_example!("613.10", "distinctive substring")` to mtg-data, next to `cr!` and `ruling!`.
  - It fails the test unless the rule exists and has an example containing the substring. The parser already attaches examples to their rule (`Rule.examples` in `crates/mtg-data/src/rules.rs`).
  - It also counts as citing that rule, like `cr!`.
  - It cites by substring rather than by index, like `ruling!`, so citations survive the CR adding or reordering examples.
- **Coverage report.** Extend `mtg-tools cr-coverage`:
  - Report example coverage (covered, exempt and open, out of 277), and add it to `docs/CR_COVERAGE.md`.
  - Add a listing mode for a rule range that prints each example with its rule and status, e.g. `cr-coverage --examples --range 600-611`.
- **Exemptions.** Add `docs/cr-exemptions/examples.tsv` with lines `rule<TAB>substring<TAB>reason`, checked like the other exemption files (the substring must match one of that rule's examples).
  - Be conservative. Exempt an example only when no game can show it: it explains a term, or it is about something tested elsewhere (then cite it there instead).

PER EXAMPLE:
1. Read the example and its rule together with the rules around them (`grep -n` in `data/comprehensive-rules.txt`). The CR text is newer than training data.
2. Write a test that sets up exactly the example's situation:
   - Use the real cards it names, or custom `CardDef`s when it is abstract ("a 2/2 creature").
   - Play it out through the engine's normal actions, and assert every outcome the example states.
   - Cite `cr_example!(rule, substring)`, plus `cr!` for any other rules the test exercises.
   - Put the tests in `tests/cr/r<NNN>_examples.rs`, one file per three-digit rule.
3. If the engine disagrees with the example, the engine is wrong. Fix it, with the test as the regression test (CLAUDE.md: prefer hooks and registry files, and keep core edits minimal).
   - If the example names a card the engine doesn't support, prefer making it compile.
   - Otherwise use a custom `CardDef` that behaves the same, and say so in a comment.
4. An existing test may already cover the example. If it really sets up the example's situation, add the `cr_example!` citation to it instead of writing a duplicate.

DONE per item: every example in the range is cited by a passing test or exempted with a reason, and the engine bugs found are fixed. Report how many examples are covered, each bug found (with its rule and the fix), and the exemptions.
