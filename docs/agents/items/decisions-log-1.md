# decisions-log-1: a generated docs/DECISIONS.md, for reviewers

Tooling item (effort high; review at max). The user wants a log of every decision to run by other people (Nov 20: "Yes, make the decisions.md").

Extend `mtg-tools interpretations --write` (crates/mtg-tools; the register parser in crates/mtg-data) to also write docs/DECISIONS.md, and make `--check` (run by `cargo test`) fail if it's stale, as it does for docs/INTERPRETATIONS.md. Contents, generated from the register and the other sources, never hand-edited:
1. **Decided by the user**, newest first, grouped by date: each entry's id, title, the user's words where the Decision quotes them, the reading chosen, the alternatives not chosen, low-confidence marks, and a link to the full entry. Parse "Decided by the user, <date>" (normalize the date formats used).
2. **Decided by the sources** (a ruling, a rule, or a dated change: the entry says "decided by the ruling"/"by the rule" etc.) and **decided by the coordinator or a worker** (everything else decided), each with its basis in one line, so a reviewer can see who decided what.
3. **Provisional** (awaiting the user).
4. **Revisit later**: the bullets of docs/agents/finish-line-plan.md §5 "Revisit later".
5. **Ruling discrepancies**: a table of docs/RULING_DISCREPANCIES.md entries by kind (outdated, template error, misattached, misleading, inconsistent) and status, with the card and date.
6. **Policies the user set** that aren't single entries (e.g. the standing rulings policies: rulings contradicting Oracle text give way; pre-battle and pre-Dominaria rulings; controller picks the choosing opponent), drawn from their register entries.
Where an entry's "who decided" can't be parsed, list it under a "needs a source line" heading rather than guessing, and fix the entries the parser can't read if the fix is mechanical. Add a short header saying how the file is generated and what each section means, in plain language for a reader with moderate context.

Another session (user-decisions-nov20b) is editing many register entries at the same time: pull before landing and regenerate. `cargo test --workspace` passes; yield row; follow worker.md's one-item-per-session rule and notify the coordinator.
