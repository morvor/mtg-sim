# engine-audit-1: an independent, adversarial audit of the core rules areas

WHY: the rulings batches find about one real engine bug per 100 rulings, almost always in interactions. A ruling only tests what someone thought to write down. This audit looks for wrong behavior directly: read the CR, read the engine, and try to break the engine with interactions the rulings don't mention.

Run it at effort max. Use the strongest judgment you have: assume the engine is wrong somewhere in every area until you've tried hard to show otherwise.

## Areas
Do one area at a time, in this order. Each area is its own landed commit series, with subject prefix `engine-audit-1/<area>:`.
1. **layers**: CR 613, `layers.rs`. Dependencies, timestamps, copy effects interacting with type and ability changes, characteristic-defining abilities, counters against setting effects, and 613.7 timestamps (including the decided `room-door-unlock-timestamp`).
2. **replacement**: CR 614–616, `replacement.rs`, `prevention.rs`. Order of application and the affected player's or controller's choice (616.1), self-replacement, "instead" chains, prevention shields against "can't be prevented", and ETB replacements with copies.
3. **triggers**: CR 603, `triggers.rs`. Leaves-the-battlefield look-back (603.10), trigger conditions on simultaneous events, APNAP ordering with control changes, delayed and reflexive triggers, intervening "if" clauses, and state triggers.
4. **zones-lki**: CR 400.7, 608.2h, 113.7. New objects, last known information, an effect finding a moved object (400.7 subrules) and phasing.
5. **combat**: CR 506–511. Requirements and restrictions with costs, removal from combat, damage division (510.1c/d), first strike and double strike changes mid-combat, and banding.
6. **casting-costs**: CR 601–602, 118. The order of 601.2 steps, cost increases and reductions against locked-in costs, X, alternative and additional costs, and mana abilities during casting.
7. **sba-copy**: CR 704 and 707. SBA simultaneity, the legend rule with copies, copiable values, and copies of spells (707.10).

## Method for each area
1. Read the CR section and its subrules in data/comprehensive-rules.txt.
2. Read the engine code for the area.
3. List the interactions most likely to be wrong. For each one, write a focused test using real cards in TestGame: cite the rules with `cr!`, and the rulings with `ruling!` where one exists.
4. Run the tests:
   - **A failure is an engine bug.** Fix it generally, with no card-name special cases, and keep the test.
   - **A case where the CR is silent or ambiguous** goes to the interpretation register as a provisional entry, with the CR text quoted, for the user.
5. For each area, have an independent reviewer subagent (effort max) check the tests: do they really exercise the claim, and would they fail if the engine were wrong? Temporarily break the behavior to confirm. The reviewer lands the area with scripts/land.sh.
6. Use the mutation tool (`scripts/mutants.sh`) on the area's main file where it's practical, and kill the surviving mutants that matter with tests.

## Report
For each area:
- the number of tests added;
- the bugs found and fixed, one line each with the rule;
- any provisional register entries (with quotes);
- the mutation results;
- your confidence in the area afterwards, and why.
