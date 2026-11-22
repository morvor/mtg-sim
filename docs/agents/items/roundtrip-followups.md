# roundtrip-followups: drive the Oracle round-trip mismatches to zero

WHY: the round trip (`mtg-tools roundtrip`, docs/ROUNDTRIP.md) renders every fully supported card's compiled abilities back to Oracle-style text and compares it with the card's text. On Oct 3, 25,245 of 28,584 fully supported cards passed (88.3%); 3,309 mismatched. A mismatch is one of: (a) a parser bug, where the compiled ability doesn't mean what the text says, so the card misbehaves in some situations; (b) a renderer gap; (c) a true equivalence. Parser bugs are the point: each is a card that plays wrong.

This is a stream of items. Each item is named `roundtrip-<topic>-<N>` (for example `roundtrip-clusters-3`, `roundtrip-tail-4`); before starting one, write its brief to `docs/agents/items/<slug>.md` (which clusters it takes, with counts and example cards) and commit it with the item's work.

For each item:
1. On the latest origin PR branch: `cargo run --release -p mtg-tools -- roundtrip --write docs/ROUNDTRIP.md` (and `--clusters N` to see the mismatch clusters). Pick the biggest clusters that no other in-flight item has taken (two items running in parallel must take disjoint clusters; say in the brief which ones).
2. Triage every mismatch in the taken clusters:
   - Parser bug: fix the parser so the AST means what the text says, or make the text unsupported if supporting it faithfully isn't possible yet. Add an in-game test that fails on the old behavior (crates/mtg-engine/tests/cards/oracle_roundtrip*.rs or a topic file), and list the bug in the docs/ROUNDTRIP.md bug table.
   - Renderer gap: fix the renderer (`crates/mtg-engine/src/oracle/render/`), keeping it an independent derivation from the AST: never reuse the parser's phrase tables or print the source text back.
   - Equivalence: only for two wordings that mean the same thing under the CR; add it to the single commented equivalence list with the justification (cite the CR). Never add an equivalence to hide a real difference; reviewers reject vague ones.
3. Regenerate docs/ROUNDTRIP.md and docs/roundtrip-passing.txt. Cards that passed must stay passing (`roundtrip --check`, engine test `oracle_roundtrip`).

DONE per item: every mismatch in the taken clusters is fixed or listed with a reason; the report gives passing counts before and after and the parser bugs fixed. The stream is done when no mismatches remain, or each remaining one is listed with its reason.
