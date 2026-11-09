# roundtrip-pairs-6: the next untriaged round-trip mismatches

Same method as roundtrip-pairs-5.md: read it first.

**Baseline:** docs/ROUNDTRIP.md, Nov 5: 28,460 of 32,076 pass and 3,464 mismatch. Regenerate it first from the current branch.

**Scope.** Take about 150 mismatching cards that no earlier roundtrip item's "Taken" or "Remaining" section lists (`grep docs/agents/items/roundtrip-*.md`). Go in the same order as pairs-5:
1. clusters of 3 or more cards;
2. then two-card clusters;
3. then single cards.

Record the chosen cards in a "Taken" section and commit it first.

**Priority within the scope:** a mismatch can mean the engine *behaves* differently from the card, not just that the wording renders differently. Check those first, test them in play, and fix the parser or engine. Only then fix the renderer's wording.

**DONE when:**
- each taken card passes, or is listed under "Remaining" with its reason;
- docs/ROUNDTRIP.md and docs/roundtrip-passing.txt are regenerated;
- `cargo test --workspace` passes.
