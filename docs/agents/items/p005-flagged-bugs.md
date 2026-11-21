# p005-flagged-bugs: fix the three engine bugs rulings-P005 flagged

The rulings-P005 worker found three engine bugs and wrote them up, each with a repro, the cards involved and the rule, in docs/agents/engine-gaps.md, section "Flagged by rulings-P005 (Nov 5)":
1. A card not yet on the stack is judged castable without the continuous effects that would apply to the spell.
2. The default X ignores costs and target counts that depend on X.
3. Flashback, and other casts from the graveyard, are offered when the only target is the card itself.

For each bug:
- reproduce it with a failing test that cites the rule (`cr!`), and a ruling where one applies;
- fix it generally;
- check the legal-actions sweep, or similar cards, for the same construct.

If a fix turns out to need a judgment call, add a provisional register entry. When a bug is fixed, remove it from engine-gaps.md.

DONE when the three bugs are fixed and tested (or a bug is shown not to be one, with the evidence), and `cargo test --workspace` passes.
