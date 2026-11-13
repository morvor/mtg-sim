# roundtrip-tail-2: Oracle round-trip follow-up, the long tail, part 2 (cards E–L)

Background: `mtg-tools roundtrip` renders every fully supported card's compiled abilities (AST) back to
Oracle-style English (`crates/mtg-engine/src/oracle/render/`, written independently of the parser) and
compares with the Oracle text (`render/compare.rs`). Read `oracle-roundtrip.md`,
`roundtrip-parser-bugs-1.md`, `roundtrip-renderer-gaps.md` and `roundtrip-tail-1.md` for the method.
GOAL: drive round-trip mismatches toward zero so every supported card is verified. After roundtrip-tail-1,
about 3,200 cards mismatch overall (72 gap, ~300 in clusters of 3+, ~560 in pairs, ~2,260 singletons).

SCOPE: mismatching cards whose name starts with E through L whose difference is a singleton or shared by
a pair (roundtrip-clusters-3 handles clusters of 3+ cards and gap clusters in parallel). Regenerate the
report first and get the list from `roundtrip --list` (it prints each card's cluster). Look for shared
root causes and fix those first (fixes helping cards outside E–L are fine). Also:
- `Effect::ChooseOne` (a choice made as the effect resolves) renders as "choose one —", the same as a
  modal spell's cast-time choice, so a parser that wrongly compiled a cast-time "Choose one" into it would
  pass. Make the two render distinguishably (CR 700.2 vs 608.2d) and fix any card that was wrongly
  compiled.
- The Rune cycle (Rune of Flight, Might, Mortality, Speed, Sustenance) mismatches until the renderer can
  name the creature an Equipment is attached to inside a granted ability; fix it.

For each card triage:
- (a) PARSER BUG: the AST doesn't mean what the text says. Fix the parser generally or make the text
  unsupported. Every parser bug fixed needs an in-game test (TestGame, real card) showing the corrected
  behavior, failing on the old code.
- (b) RENDERER: fix wording, keeping the renderer independent of the parser (never reuse parser phrase
  tables or print stored source text; no catch-alls printing plausible text; no dropping AST parts unless
  provably redundant under the CR for every matched card).
- (c) TRUE EQUIVALENCE under the CR: add to the one EQUIVALENCES list with a CR-cited justification only
  if genuinely identical for every matched card. Never add an equivalence or loosen the comparison to
  hide a real difference.

DONE when the E–L tail is handled (or, if too big, the biggest root causes are done and what's left is
reported by cause), `docs/ROUNDTRIP.md` and `docs/roundtrip-passing.txt` are regenerated with `--write`,
and `roundtrip --check` passes (no previously passing card fails unless it was wrongly passing, each
justified).
