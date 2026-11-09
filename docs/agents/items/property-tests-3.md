# property-tests-3: properties for the areas the first eight can't see

WHY: docs/agents/finish-line-plan.md, criterion C4 and risks 2, 7, 8, 9, 13. Properties 1–8
(docs/PROPERTIES.md) have held on every run since their first, and their first runs found 22
bugs. Each new property below targets an area where rulings batches keep finding bugs and no
property looks: the legal-action list, layer ordering, copies of abilities and spells,
multiplayer variants the fuzz never plays, and library cards shown to a player. Design each
oracle from the rules, never from the engine code it checks (as `payment.rs` and `damage.rs`
were). Every violation is a bug: fix it generally with a regression test citing the rules.

Run the design at effort max; the runs are routine. Same code homes as property-tests-1/2:
`crates/mtg-engine/src/properties/`, `crates/mtg-sim/src/properties.rs` and `exhaustive.rs`,
tests in `crates/mtg-engine/tests/properties/`, results in docs/PROPERTIES.md.

## Properties

9. **Legal-action exactness** (CR 601.2, 602.2, 305, 118). Inside the every-card fuzz and the
   properties fuzz, at every priority decision:
   - every offered cast, activation, land play and special action must succeed when taken with
     the engine's own payment (no "can't pay after all", no reversal under 733.1);
   - for a sample of actions *not* offered (a card in hand, an ability of a permanent), taking
     it anyway through the engine's `cast`/`activate` entry points must fail for a reason the
     rules give (timing, permission, cost unpayable, no legal target, restriction); report the
     reason and check it against the card's text and the CR. A sample is enough: the point is to
     catch under-offering, which nothing else sees.
10. **Layer order-independence** (CR 613.8 and 613.8a–c). For every object on a fuzzed board,
    recompute its characteristics with the list of active continuous effects shuffled before
    dependency analysis and timestamp ordering; the result must equal the unshuffled one. Also
    recompute after reversing the order statics are collected in. Any difference is an ordering
    bug (or a dependency the analysis misses).
11. **Copies of abilities and of non-permanent spells** (CR 707.10, 707.10a–c, 603.6 for
    copied triggers). Extend `properties::copies`: for every card with an activated or
    triggered ability, copy the ability on the stack (Strionic Resonator, Lithoform Engine,
    Rings of Brighthearth) and resolve both on cloned games; the outcomes must agree except where
    707.10 says otherwise (new targets chosen, "if you do" costs not paid for the copy). For
    every instant and sorcery, resolve a copy of the spell alongside the original (Twincast) and
    compare. Copy planeswalkers and battles with a Clone-like effect as well as with tokens.
12. **Multiplayer variants** (CR 805, 806, 807, 809, 811). Add `--kind 2hg|emperor|grand-melee|
    range` to the properties fuzz, so seat symmetry, snapshot equivalence, rollback identity and
    determinism run over those games; a few hundred games each. Include players leaving the game
    and shared team turns (805.10c).
13. **Library cards shown are the ones the effect entitles** (CR 701.20 look, 701.23 search,
    401.5, 400.2). In `hidden::check_game`, for every decision that shows a player cards from a
    library, check the count and the zone against the effect that caused it (the effect's
    `Sel`/count in the AST): a search shows the whole library, "look at the top N" shows N, a
    reveal shows what it reveals; anything else is a leak.
14. **Payment oracle, extended** (CR 601.2f, 118.7–118.8). Add one cost reduction (a generic
    reducer with a floor), one alternative cost and one "spend as though any color" source to
    `payment.rs`'s catalog and brute force, and run 200,000 boards.

## Runs and results

Each property gets a `mtg-sim --properties --property NAME` runner and a sampled test in
`tests/properties/`. Run each at scale (property 9 inside one every-card chunk of 4,000 cards
and 500 properties-fuzz games; 10 over the same; 11 over every card; 12 at 300 games a kind;
13 at 400 games; 14 at 200,000 boards) and record the sizes, violations and bugs in
docs/PROPERTIES.md.

DONE when the six properties run, their first-run violations are fixed with regression tests
(or registered as interpretations where the CR is silent), and docs/PROPERTIES.md has the
results. Report the bugs found, each with its rule and fix.
