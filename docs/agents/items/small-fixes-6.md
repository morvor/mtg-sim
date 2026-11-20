# small-fixes-6: self-granted abilities in every zone, and two seat-symmetry violations

Implementation item (effort high; review at max). Quote every CR rule and ruling you rely on verbatim; check every ruling of the cards involved before calling anything provisional.

1. **Bloodbraid Marauder's "has cascade as long as ..." works in every zone** (ruling-conflicts-13's remaining gap; that item fixed the same thing for "has flash as long as ..." under CR 113.6e, only on the stack, with would_have_flash_as_it_begins). Survey every self-granted conditional keyword ("~ has [keyword] as long as ...") and make each function only in the zones CR 113.6 allows for it (cascade is a cast trigger: CR 702.85a, 113.6e/113.6k as relevant), with tests.
2. **Seat-symmetry violation, game 175** (gaps-residual-2; docs/agents/engine-gaps.md "Found, not fixed"): before the first turn the nameless face-down conspiracies in the command zone have different owners in the rotated game. Repro: `cargo run --release -p mtg-sim -- --properties --property seat --seed 1 --only 175 --players 4 --max-turns 40`. Find whether it's the property's canonical renaming or the engine's placement, and fix it.
3. **Seat-symmetry violation, game 405**: an "Order objects entering the battlefield" decision lists Watchful Giant and Deepway Navigator in a different order by seat (the order of the proposed moves, likely object ids; actions.rs order_simultaneous_entries, from ruling-conflicts-12). Repro: `cargo run --release -p mtg-sim -- --properties --property seat --seed 1 --only 405 --players 2 --max-turns 40`. Make the listing seat-independent (e.g. APNAP then a seat-independent key), with a test.
Then run `--properties --games 500 --jobs 3` once and report.

`cargo test --workspace` passes; yield row; follow worker.md's one-item-per-session rule and notify the coordinator.

## Result (implementer, Nov 20)

1. **Self-granted keywords, by zone.** Survey of every Oracle line that reads "[this object] has [keyword] ... as long as ..." (167 cards, a few of them "for as long as" durations of activated abilities; a scratch test printed the zone of every compiled static that adds a keyword): 158 of those statics function on the battlefield (evasion, combat keywords, haste, hexproof/shroud, protection, persist, wither, infect, banding, firebending, ...: CR 113.6's default for a permanent's abilities, and where those keywords do anything; persist is read by look-back, CR 603.10a), 6 are "has flash" (stack only since ruling-conflicts-13, CR 113.6e) and 1 is Bloodbraid Marauder's "This spell has cascade". No instant or sorcery gives itself any other keyword; 8 lines are unsupported (un-cards and Alchemy). Fix: `conditional_flash_zone` gives the self-granted cascade the stack as its zone too (CR 113.6b with 112.1: "this spell" names its zone; 702.85a), register `self-granted-cascade-functions-on-the-stack`; test `tests/cr/r113_6_self_granted_cascade.rs`.
2. **Game 175** wasn't conspiracies, nor the property's renaming: the nameless face-down objects were supplementary-deck cards (Attraction or scheme cards, three copies of a random deck's spell each). `shuffle_attraction_decks` and `shuffle_scheme_decks` (and `shuffle_planar_decks`) iterated players in seat order, shuffling with the game's randomness and making the cards new objects (CR 400.7) as they went, so the new ids followed seats. Now in APNAP order, each deck with its owner's randomness (`Game::rng_for`, as libraries are). Found alongside: `variants::begin_game` gave vanguards (and conspiracies) their start-of-game timestamps in the command zone's order (vanguards in seat order) instead of APNAP order (CR 613.7i, 613.7j, 613.7m). Tests: `tests/properties/seat_symmetry.rs` (`supplementary_decks_are_shuffled_and_renumbered_whatever_the_seats`), `tests/cr/r609_613_variant_timestamps.rs` (`vanguards_get_their_timestamps_in_apnap_order`).
3. **Game 405:** `order_simultaneous_entries` now lists each player's objects by owner in APNAP order, then creation order (`sort_objects_apnap`), not in the order the moves were proposed; register `simultaneous-entries-listed-by-owner-then-creation`; test `tests/cr/r613_7m_entry_order_listing.rs`.

Both games now hold (`--only 175 --players 4`, `--only 405 --players 2`, seed 1).

Properties run (release build, `--properties --games 500 --jobs 3`, seed 1, 2,484 s): seat 500/500 held
(1,134,649 decisions), snapshot 500/500, rollback 500/500 (29,488 reversed actions checked), determinism
500/500, replay 498 held and 2 stopped at the time limit (games 83 and 469), no violations, no panics.
Re-checked one by one with `--slow 300` (replays get three times that): game 83 held (718 s, 4,001
requests); game 469 was still going at 900 s, and is slow the same way with the engine before this item
(360 s limit) — a slow game, not a difference.

## Review (Nov 20)

- **Both repro games hold** with the final code (`--properties --property seat --seed 1`: `--only 175
  --players 4`, 3,555 decisions; `--only 405 --players 2`, 1,470 decisions). The item's tests pass and
  fail as they should: the deck shuffles iterated in seat order again fail
  `supplementary_decks_are_shuffled_and_renumbered_whatever_the_seats`; without the sort in
  `order_simultaneous_entries` both listing tests fail; without the APNAP sort in `variants::begin_game`
  `vanguards_get_their_timestamps_in_apnap_order` fails (deliberate breaks, debug builds).
- **Untested: the per-player stream.** Shuffling the supplementary decks with the game's randomness in
  APNAP order (a deliberate break) passed `supplementary_decks_are_shuffled_and_renumbered_whatever_the_seats`:
  that is seat symmetric too. New test
  `a_players_supplementary_decks_are_shuffled_with_their_own_randomness` (a player's Attraction deck is
  in the same order whether or not the player before them has supplementary decks; fails with that break).
- **Test error:** `supplementary_decks_are_shuffled_and_renumbered_whatever_the_seats` cited CR 400.7, which
  it doesn't exercise (a shuffle isn't a zone change; the renumbering is register
  `deck-order-ids-before-shuffle`). It now cites 101.4e ("If multiple players would make choices or take
  actions while starting the game, the starting player is considered the active player and each other
  player is considered a nonactive player."), with 103.3a and 717.2; the same in docs/PROPERTIES.md.
- **The survey** covered only the trailing form "[this object] has [keyword] ... as long as ...". The review
  added the other forms: "As long as ..., [this object] has ..." (147 cards), "During ...", "If ...". They
  give their objects only keywords that function on the battlefield, apart from two spells, so "No instant
  or sorcery gives itself any other keyword" above was wrong: Viral Spawning ("As long as an opponent has
  three or more poison counters and this card is in your graveyard, it has flashback {2}{G}.", compiled with
  the graveyard as its zone, CR 113.6b; `tests/cards/cards3_statics.rs`) and Molten Disaster ("If this spell
  was kicked, it has split second.", compiled on the stack by `spell_has_keyword_if`, CR 702.61a; its ruling
  of 2007-05-01: "If Molten Disaster was kicked, Molten Disaster has split second as long as it's on the
  stack."). Both were already right.
- **Compiler fix (latent; no card's compilation changed).** The "k702: trailing as long as" pattern compiled a
  card's own grant of any other keyword that doesn't function on the battlefield ("This spell has storm as
  long as you control a Wizard." on a creature card; split second, replicate, casualty, convoke, cycling), or
  of flash or cascade along with another keyword ("has flash and haste"), to an ability of the permanent,
  where it does nothing, and reported the text supported. It now refuses them
  (`self_granted_keyword_zone`, formerly `conditional_flash_zone`, using `statics::functions_on_battlefield`).
  The fallback pattern "statics: has flash as long as" (`conditional_flash`, which no card reaches: the k702
  pattern comes first) still gave the ability every zone, against CR 113.6e and ruling-conflicts-13; now the
  stack. Test `a_cards_own_keyword_is_compiled_only_where_it_can_function`; register
  `self-granted-cascade-functions-on-the-stack` extended (and Molten Disaster's ruling added to its Why).
  The full `unsupported` listing and the card-coverage numbers are the same before and after.
- **Judgment calls**: `self-granted-cascade-functions-on-the-stack` (interpretation) and
  `simultaneous-entries-listed-by-owner-then-creation` (policy) kept decided: every ruling of Bloodbraid
  Marauder (eleven, all 2021-06-18) was read, none speaks of a zone other than the stack, and no card reads
  cascade outside the stack (Aurora Phoenix and The First Doctor read "a spell with cascade"); the listing
  order is a policy for a player's choice. `seeded-randomness` extended (decided).
- **Control and ownership** (CR 613.7m): `order_simultaneous_entries` still groups the entering objects by the
  player who will control them; only the listing within a player's group changed.
- **Remaining gaps:** the planar decks' shuffle (APNAP order, each owner's randomness) has no seat test (the
  property harness plays no Planechase games); replay game 469 is slow (before this item too); Bloodbraid
  Marauder's tier is refreshed at the next sweep.
