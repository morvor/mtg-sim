# structure-tests-2: raise the verified card count with tests for the structures that block the most cards

WHY: verified-cards (docs/CARD_COVERAGE.md, Oct 21) made "verified" the headline: 14,297 of 31,051 cards (46.0%). The biggest blocker by far is structure coverage. 10,113 cards pass the Oracle round trip and the fuzz sweep but have exactly one ability whose structure no test exercises: 4,769 with "1 of 1 unexercised", 3,810 with "1 of 2" and 1,534 with "1 of 3". Cards that share a structure run the same engine code, so one test covers all of them. A test for a widely shared structure verifies many cards at once.

This is a stream of items, `structure-tests-2`, `-3` and so on. Follow docs/agents/items/structure-tests.md (measure, choose, test, re-measure, report), with one change: choose the structures by **verified gain**. Verified gain is the number of cards that would become verified if that one structure were exercised, i.e. cards whose only failed check is that structure (`docs/verified-cards.tsv`, failed = "structure (1 of N unexercised)"). `mtg-tools verified --card NAME` and `structure-coverage --card NAME --show-fingerprints` show which structure a card is missing.

For each item:
1. **Measure.** Regenerate the structure log and `docs/verified-cards.tsv` on the latest origin (the recipe is in docs/CARD_COVERAGE.md; `CARGO_BUILD_JOBS=2`).
2. **Choose.** Take the ~60 structures with the highest verified gain.
3. **Test.** Write one test per structure with a real card that has it, through `TestGame`. The test asserts the exact in-game outcome the card's text says, as structure-tests.md requires: the right objects and players affected and no others, amounts, durations ending, conditions both true and false, and target legality.
   - If the engine disagrees with the card text, that is an engine bug. Fix it, with the test as the regression test.
   - Cite `cr!` (and `ruling!` when a ruling applies).
4. **Re-measure.** Regenerate docs/STRUCTURE_COVERAGE.md, `docs/verified-cards.tsv` and docs/CARD_COVERAGE.md. Report the verified count before and after, and the engine bugs found.

DONE per item: ~60 structures tested, the verified count re-measured and reported. The stream goes on while verified gain per test stays high; stop and report when the top structures each gain fewer than about 5 cards.
