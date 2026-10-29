# roundtrip-tail-3: Oracle round-trip follow-up, the long tail, part 3 (cards M–R)

Background: `mtg-tools roundtrip` renders every fully supported card's compiled abilities (AST) back to
Oracle-style English (`crates/mtg-engine/src/oracle/render/`, written independently of the parser) and
compares with the Oracle text (`render/compare.rs`). Read `oracle-roundtrip.md` and the earlier
`roundtrip-*.md` briefs for the method. GOAL: drive round-trip mismatches toward zero so every supported
card is verified. After roundtrip-tail-2, 24,495 of 27,739 cards pass; 3,214 mismatch (158 gap, 332 in
clusters of 3+, 524 in pairs, 2,200 singletons).

SCOPE: mismatching cards whose name starts with M through R whose difference is a singleton or shared by a
pair (roundtrip-clusters-3 handles clusters of 3+ cards and gap clusters in parallel). Regenerate the
report first and get the list from `roundtrip --list`. Look for shared root causes and fix those first
(fixes helping cards outside M–R are fine). Cross-cutting causes left by roundtrip-tail-2 are in scope:
trailing "otherwise" and conditional structures, wording of abilities quoted on tokens, copy "except …"
forms, "that player" vs "its controller", and how "reveal until" is rendered. Also: an older comparison
rule drops "you" (Chain of Smog's "You may choose new targets" passes only because of it); make sure no
rule drops a word that carries meaning (who chooses, who may).

For each card triage:
- (a) PARSER BUG: the AST doesn't mean what the text says. Fix the parser generally or make the text
  unsupported. Every parser bug fixed needs an in-game test (TestGame, real card) showing the corrected
  behavior, failing on the old code.
- (b) RENDERER: fix wording, keeping the renderer independent of the parser (never reuse parser phrase
  tables or print stored source text; no catch-alls printing plausible text; no dropping AST parts unless
  provably redundant under the CR for every matched card).
- (c) TRUE EQUIVALENCE under the CR: add to the one EQUIVALENCES list with a CR-cited justification only
  if genuinely identical for every matched card. Never add an equivalence or loosen the comparison to
  hide a real difference (for example, "When X, Y if C" is not "When X, if C, Y": CR 603.4).

DONE when the M–R tail is handled (or, if too big, the biggest root causes are done and what's left is
reported by cause), `docs/ROUNDTRIP.md` and `docs/roundtrip-passing.txt` are regenerated with `--write`,
and `roundtrip --check` passes (no previously passing card fails unless it was wrongly passing, each
justified).
