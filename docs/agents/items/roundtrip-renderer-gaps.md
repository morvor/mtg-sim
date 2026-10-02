# roundtrip-renderer-gaps: Oracle round trip — put every AST node into words

Follow-up of `oracle-roundtrip.md`.

Background: `mtg-tools roundtrip` renders every fully supported card's compiled abilities
(AST) back to Oracle-style English (`crates/mtg-engine/src/oracle/render/`, written
independently of the parser) and compares with the Oracle text (`render/compare.rs`).
`docs/ROUNDTRIP.md` lists the mismatches in clusters. GOAL: drive round-trip mismatches
toward zero so every supported card is verified.

SCOPE: every "gap: ..." cluster — cards where the renderer reports it cannot put some AST
node into words (gap: Effect, Condition, Value, Filter, StaticEffect, "a remembered group
no instruction names", "counters put by a player", "permission to play with terms", and
any other "gap:" rows). Regenerate the report first (`cargo run --release -q -p mtg-tools
-- roundtrip --write docs/ROUNDTRIP.md`) and work from the current list (`roundtrip
--list`, `--card NAME --ast`).

For each gap: teach the renderer to word that AST node from its meaning, written
INDEPENDENTLY of the parser (never reuse the parser's phrase tables or print stored source
text; no catch-all that prints plausible text). Once a card renders, if it still
mismatches, triage: (a) PARSER BUG — the AST doesn't mean what the text says: fix the
parser generally or make the text unsupported, and every parser bug fixed needs an in-game
test (TestGame, real card) showing the corrected behavior that fails on the old code;
(b) remaining renderer wording issue — fix; (c) TRUE EQUIVALENCE under the CR — add to the
one EQUIVALENCES list with a CR-cited justification only if genuinely the same meaning;
never add an equivalence or loosen the comparison to hide a real difference. Where the AST
can't distinguish a node's meaning well enough to render it (e.g. a "remembered group no
instruction names"), that is often itself a parser bug — investigate.

Another item (roundtrip-parser-bugs-1) works in parallel on the non-gap clusters with 10+
cards: merge origin often, keep edits localized, don't take on its clusters.

When done, regenerate `docs/ROUNDTRIP.md` and `docs/roundtrip-passing.txt` with the tool
(`--write`). `roundtrip --check` must pass (no previously passing card may start failing
unless it was wrongly passing — justify each).
