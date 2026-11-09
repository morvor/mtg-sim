# data-refresh: after the user refreshes data/, re-check the CR and support the new cards

WHY: docs/agents/finish-line-plan.md, phase C item C3 and decision D6. The data files are the
source of truth (CLAUDE.md): Scryfall bulk data of 2026-09-24 and the CR effective 2026-09-25.
Only the user updates them (Nov 4: "No new card data for now"). When they do, two things
follow: the CR may have changed under the engine, and new cards need support (189 cards of sets
unreleased on 2026-09-24 are in the data already, deferred as legal in no format; the refresh
makes them legal and adds the sets since). Start this item only when the user says the data is
refreshed.

## Part 1: `cr-diff-1` (judgment, effort max)

1. Diff the old and new `data/comprehensive-rules.txt` (keep the old one in the commit's
   parent). List every rule whose number or text changed, every new rule, every removed rule
   and every renumbered rule (the diff of the glossary too).
2. `cargo test --workspace`: `cr!` and `cr_example!` fail on numbers that no longer exist;
   re-point each citation to the rule that now carries the behavior (never cite a rule the
   test doesn't exercise). `mtg-tools cr-coverage` must report 0 uncovered again; new rules get
   tests or `docs/cr-exemptions/` entries by the usual standard.
3. For every rule whose *text* changed: read the engine's behavior against the new text, fix
   deviations with tests, and record any judgment call in the register. Rulings that the
   change outdates go to docs/RULING_DISCREPANCIES.md as "outdated" with the change dated.
4. Report the changes and what each one did to the engine.

## Part 2: card-support round 7 (routine, planned like rounds 1–6)

1. `mtg-tools unsupported` lists the texts the compiler doesn't take; `card-coverage` the new
   counts. Group the unsupported texts by construct, as docs/agents/card-support-plan-6.md did,
   into items `cards7-<topic>` with `.texts.tsv` files, and run them through the usual
   implementer and reviewer pipeline. Anything that fits a deferred category (docs/DEFERRED.md)
   stays out; note new mechanics that would need a deferral decision for the user.
2. New rulings: `python3 scripts/rulings_batches.py summary` shows the new OPEN texts; the
   script regroups them into batches, which join the rulings pool (worker.md's rulings brief).
3. When the round has landed: `verified-refresh-final` again (an incremental refresh with
   `--names` for the new cards is enough if the engine didn't change otherwise).

DONE when `cr-coverage` is back at 0 uncovered on the new CR, every text change was read against
the engine, the new cards are at the same tiers as the rest (round 7's items landed, each
with its "Remaining" list), and the coverage reports are regenerated.
