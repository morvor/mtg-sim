# roundtrip-tail-1: Oracle round-trip follow-up, the long tail, part 1 (cards A–D)

Background: `mtg-tools roundtrip` renders every fully supported card's compiled abilities (AST) back to
Oracle-style English (`crates/mtg-engine/src/oracle/render/`, written independently of the parser) and
compares with the Oracle text (`render/compare.rs`). Read `oracle-roundtrip.md`,
`roundtrip-parser-bugs-1.md` and `roundtrip-renderer-gaps.md` for the method. GOAL: drive round-trip
mismatches toward zero so every supported card is verified.

SCOPE: mismatching cards whose name starts with A, B, C or D, EXCLUDING cards in a cluster of the "Mismatch
clusters" table (roundtrip-clusters-2 handles all non-gap clusters in parallel; remaining "gap:" cards are
in scope if named A–D). Regenerate the report first and get the list from `roundtrip --list` (it prints
each card's cluster). These are mostly one-off differences: find shared root causes and fix those first
(fixes that help cards outside A–D are fine).

For each card triage:
- (a) PARSER BUG: the AST doesn't mean what the text says. Fix the parser generally or make the text
  unsupported. Every parser bug fixed needs an in-game test (TestGame, real card) showing the corrected
  behavior, failing on the old code.
- (b) RENDERER: fix wording, keeping the renderer independent of the parser (never reuse parser phrase
  tables or print stored source text; no catch-alls printing plausible text; no dropping AST parts unless
  provably redundant under the CR for every matched card).
- (c) TRUE EQUIVALENCE under the CR: add to the one EQUIVALENCES list with a CR-cited justification only
  if genuinely identical. Never add an equivalence or loosen the comparison to hide a real difference.

DONE when the A–D tail is handled (or, if too big, the biggest root causes are done and what's left is
reported by cause), `docs/ROUNDTRIP.md` and `docs/roundtrip-passing.txt` are regenerated with `--write`,
and `roundtrip --check` passes (no previously passing card fails unless it was wrongly passing, each
justified).
