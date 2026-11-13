# finish-line-plan: define "done" for this engine, and the shortest sound path to it

WHY: the user, Nov 9: "This model is extremely powerful, so we should find a way to leverage the free usage we have with it to maximally accelerate and improve this project. We are getting close to the end, so using our Fable usage to push us over the finish line with good judgement would be ideal."

The project's aim (the user, from the start): simulate Magic **precisely** by the Comprehensive Rules, be as sure of that as possible, and make adding cards simple.

This item plans; it changes no engine code. Run it at effort max.

## Inputs
- CLAUDE.md, docs/agents/worker.md and docs/agents/verification-plan.md: the history, the user's decisions and the current priorities.
- The coverage reports: docs/CARD_COVERAGE.md, docs/CR_COVERAGE.md, docs/RULINGS_COVERAGE.md (or `python3 scripts/rulings_batches.py summary`), docs/ROUNDTRIP.md, docs/STRUCTURE_COVERAGE.md and docs/verified-cards.tsv.
- The other evidence: docs/PROPERTIES.md and the mutation-testing results (`scripts/mutants.sh` and the items that used it), docs/agents/engine-gaps.md, docs/RULING_DISCREPANCIES.md, docs/INTERPRETATIONS.md and docs/DEFERRED.md.

## Questions to answer
1. **Done criteria.** What would let us say with justified confidence that the engine plays every supported card by the CR? Propose measurable criteria and a target for each. For example:
   - rulings: every text cited, exempt with a reason, or uncovered for a named, deferred reason;
   - the round-trip pass rate;
   - verified cards;
   - structure coverage;
   - mutation score on the core files;
   - property-test runs;
   - fuzz runs with no panics;
   - an independent audit of the core rules areas.
   Say which criteria actually measure correctness and which only measure bookkeeping.
2. **Where the remaining risk is.** Name the places the engine is most likely still wrong, with evidence:
   - interactions the rulings don't cover;
   - areas with weak tests or surviving mutants;
   - engine-gaps.md entries;
   - hand-written abilities (153 cards);
   - the round-trip mismatches that may hide wrong behavior rather than wording.
3. **The plan.** A short ordered list of work items to reach the criteria, each with:
   - its brief name;
   - what it fixes or proves;
   - the effort (sessions, roughly);
   - whether it needs the strongest model's judgment (an audit, a hard rules question) or is routine (a rulings batch).
   Include the user's standing "later" items: compile every hand-written card, then support new cards after the user refreshes the data.
4. **Human review.** Summarize docs/RULING_DISCREPANCIES.md for the user to check and report to Wizards: counts by kind, and the ten most consequential entries.

## Output
Write docs/agents/finish-line-plan.md (the plan) and, where routine items are new, their briefs in docs/agents/items/. Land them with scripts/land.sh as a docs-only commit. End with a report the coordinator can relay:
- the done criteria with today's values;
- the top risks;
- the ordered item list;
- any decision only the user can make, each with the CR rules and rulings it turns on, quoted verbatim with dates.
