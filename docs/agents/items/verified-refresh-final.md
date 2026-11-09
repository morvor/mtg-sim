# verified-refresh-final: the final measurement on the finished code

WHY: docs/agents/finish-line-plan.md, criteria C4, C6 and C11 and phase C item C2. The
committed coverage reports were measured on code from October and early November; several
engine areas have changed since. Done is measured on the final code, with nothing else pending.

Run it only when the coordinator says no engine change is in flight (every phase A and B item
landed). Routine. It needs the machine to itself for about four hours: tell the coordinator the
hours, and run nothing else heavy alongside (CLAUDE.md: 4 cores, 15 GB; an out-of-memory
restart kills every agent).

## Steps (CLAUDE.md "Regenerate docs/verified-cards.tsv", in full)

```sh
export CARGO_BUILD_JOBS=2
rm -rf target/structlog && MTG_STRUCTURE_LOG=$PWD/target/structlog cargo test --workspace
cargo run --release -p mtg-sim -- --every-card --threads 3 --from 0 --count 8500 --report target/every-card-0.tsv
cargo run --release -p mtg-sim -- --every-card --threads 3 --from 8500 --count 8500 --report target/every-card-1.tsv
cargo run --release -p mtg-sim -- --every-card --threads 3 --from 17000 --count 8500 --report target/every-card-2.tsv
cargo run --release -p mtg-sim -- --every-card --threads 3 --from 25500 --count 8500 --report target/every-card-3.tsv
cat target/every-card-[0123].tsv > target/every-card.tsv
cargo run --release -p mtg-tools -- verified --logs target/structlog --report target/every-card.tsv --write
cargo run --release -p mtg-tools -- card-coverage --write docs/CARD_COVERAGE.md
cargo run --release -p mtg-tools -- structure-coverage --write docs/STRUCTURE_COVERAGE.md
cargo run --release -p mtg-tools -- roundtrip --write docs/ROUNDTRIP.md
cargo run --release -p mtg-tools -- cr-coverage --write docs/CR_COVERAGE.md
cargo run --release -p mtg-tools -- rulings-coverage --write docs/RULINGS_COVERAGE.md
```

Then the properties and the fuzz, at the sizes docs/PROPERTIES.md records as the latest runs
(properties 1–4 at `--games 2000`; copies and copies-dynamic over every card; hidden 400 games;
payment 200,000 boards; damage 20,000 combats; the property-tests-3 properties at their stated
sizes), and `--random-decks --games 10000`. Every failure the sweep or a property reports is a
bug: fix it (a regression test citing the rule), then re-run the affected chunk or property.

Record the results in docs/EVERY_CARD.md ("Latest full sweep": pool, games, failures, cards cast,
abilities used, and the `--random-decks` result, which no document records today; the Nov 5
sweep of 32,656 cards was never written up there, so this one must be), docs/PROPERTIES.md
("Results") and the standard update, with the commit measured and the date.

DONE when the five reports and the tier file are regenerated from one commit, every property
holds, the sweep and the random-deck fuzz report no panic, hang or violation, and the
finish-line criteria table (docs/agents/finish-line-plan.md §1.2) has a "final" column filled
in from these runs.
