# roundtrip-clusters-2: Oracle round-trip follow-up, part 2 (all remaining non-gap clusters)

Background: `mtg-tools roundtrip` renders every fully supported card's compiled abilities (AST) back to
Oracle-style English (`crates/mtg-engine/src/oracle/render/`, written independently of the parser) and
compares with the Oracle text (`render/compare.rs`). `docs/ROUNDTRIP.md` lists mismatches in clusters. Read
`oracle-roundtrip.md` and `roundtrip-parser-bugs-1.md` (part 1) for the method. GOAL: drive round-trip
mismatches toward zero so every supported card is verified.

SCOPE: all non-"gap:" clusters in the "Mismatch clusters" table of the current report, any size (e.g.
"exiled with ~" vs "among each card exiled with", "it" vs "~", "owner graveyard" vs "a graveyard from
anywhere", "a" vs "your", "each damage that would be" vs "if a source would deal", "is" vs "become",
"mana value was", "named ~ in each", "target creature's owner puts it", "the exiled card", "this turn do
so", "choose 1 of" vs "for each", "half", "∅ vs under your control", ...). "gap:" clusters are out of scope.
Regenerate the report first and work from `roundtrip --list` / `--card NAME --ast` / `--all`.

For each cluster triage every card:
- (a) PARSER BUG: the AST doesn't mean what the text says. Fix the parser generally or make the text
  unsupported. Every parser bug fixed needs an in-game test (TestGame, real card) showing the corrected
  behavior, failing on the old code.
- (b) RENDERER: fix wording, keeping the renderer independent of the parser (never reuse parser phrase
  tables or print stored source text; no catch-alls printing plausible text; no dropping AST parts unless
  provably redundant under the CR).
- (c) TRUE EQUIVALENCE under the CR: add to the one EQUIVALENCES list with a CR-cited justification only
  if genuinely identical. Never add an equivalence or loosen the comparison to hide a real difference.

Clusters where one side has a word the other lacks ("a" vs "your", "is" vs "become", "choose one of" vs
"for each", "half") are prime parser-bug suspects. Known latent bug to fix if a supported card hits it:
`library::dig` makes the library owner choose and order when "you look at the top N cards of target
player's library".

DONE when every non-gap cluster is triaged and handled, `docs/ROUNDTRIP.md` and
`docs/roundtrip-passing.txt` are regenerated with `--write`, and `roundtrip --check` passes (no previously
passing card fails unless it was wrongly passing, each justified).
