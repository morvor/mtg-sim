# small-fixes-6: self-granted abilities in every zone, and two seat-symmetry violations

Implementation item (effort high; review at max). Quote every CR rule and ruling you rely on verbatim; check every ruling of the cards involved before calling anything provisional.

1. **Bloodbraid Marauder's "has cascade as long as ..." works in every zone** (ruling-conflicts-13's remaining gap; that item fixed the same thing for "has flash as long as ..." under CR 113.6e, only on the stack, with would_have_flash_as_it_begins). Survey every self-granted conditional keyword ("~ has [keyword] as long as ...") and make each function only in the zones CR 113.6 allows for it (cascade is a cast trigger: CR 702.85a, 113.6e/113.6k as relevant), with tests.
2. **Seat-symmetry violation, game 175** (gaps-residual-2; docs/agents/engine-gaps.md "Found, not fixed"): before the first turn the nameless face-down conspiracies in the command zone have different owners in the rotated game. Repro: `cargo run --release -p mtg-sim -- --properties --property seat --seed 1 --only 175 --players 4 --max-turns 40`. Find whether it's the property's canonical renaming or the engine's placement, and fix it.
3. **Seat-symmetry violation, game 405**: an "Order objects entering the battlefield" decision lists Watchful Giant and Deepway Navigator in a different order by seat (the order of the proposed moves, likely object ids; actions.rs order_simultaneous_entries, from ruling-conflicts-12). Repro: `cargo run --release -p mtg-sim -- --properties --property seat --seed 1 --only 405 --players 2 --max-turns 40`. Make the listing seat-independent (e.g. APNAP then a seat-independent key), with a test.
Then run `--properties --games 500 --jobs 3` once and report.

`cargo test --workspace` passes; yield row; follow worker.md's one-item-per-session rule and notify the coordinator.
