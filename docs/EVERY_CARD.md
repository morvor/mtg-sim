# Every-card fuzzing

`mtg-sim --every-card` plays games built around every fully supported card, with the
rules checks running, so that a crash, hang or rules violation involving any card is
found — and reports which cards were never cast and which of their abilities were never
used.

```sh
cargo run --release -p mtg-sim -- --every-card --threads 4                  # everything
cargo run --release -p mtg-sim -- --every-card --from 4000 --count 4000      # a chunk
cargo run --release -p mtg-sim -- --every-card --filter "goblin" --report r.tsv
cargo run --release -p mtg-sim -- --every-card --kind planechase             # by game kind
cargo run --release -p mtg-sim -- --every-card --list                        # the pool
cargo run --release -p mtg-sim -- --every-card --from 6437 --count 1 --game 0 --log  # replay
```

## How it works

- **The pool**: every playable paper card (one per name) whose every ability the
  compiler understood, sorted by name. Card N is the Nth card of the pool; the seeds of
  its games are derived from N, so every failure is printed with the one command that
  replays it (`--from N --count 1 --game G --log`). The pool grows as card support does,
  so indexes shift between versions: replay with the same build.
- **The games** (`--games-per-card K`, default 2; `--max-turns`, default 30): both
  players' decks hold eight copies of the card, 22–24 basic lands of its colors (snow
  basics and Wastes when it asks for them), cards that make its abilities matter (cards
  with the types and keywords its text names — Goblins for a Goblin lord, creatures with
  flying for "target creature with flying" — and cards that do what it cares about:
  discard for "whenever you discard", mill for graveyard abilities, energy, poison,
  lifegain, tokens, targeting spells for heroic, ...), and random cheap supported spells.
  The opening hand holds two copies and enough lands (a stacked start,
  `GameConfig::top_of_library`), and the first draws bring the lands up to its mana value.
  Cards that need more get the right game (`--kind`):
  - `commander`: a Commander game with the card as commander, or with a commander of its
    colors (a Background goes with a commander that can choose one);
  - `planechase`, `archenemy` (Supervillain Rumble), `vanguard`: the card on top of a
    legal supplementary deck (or as the vanguard);
  - `attractions`, `dungeon`: an Attraction deck or the dungeon card, with cards that
    open Attractions or venture into the dungeon (in any colors);
  - `conspiracy`: the card in the sideboard; ante cards play for ante; cards that learn or
    wish get Lessons and other cards in the sideboard.
- **The players** (`coverage.rs`, `FocusAgent`): random play, but when a player can use
  the card — cast or play it, activate its abilities (mana abilities too, which the
  engine doesn't list as priority actions), take a special action with it — they
  usually do, preferring what they haven't done yet; they build loyalty for a
  planeswalker's ultimate, don't use the card up from their hand (cycling, say) before
  casting it, pick optional costs, casting methods, numbers, scry and surveil splits,
  orders and card names at random, and say "yes" less and less often within a step.
- **What's counted**, through the engine's events (`Game::observer`): spells cast and
  lands played, abilities activated (every activation, mana abilities included, is in the
  turn history), triggered abilities put on the stack, abilities that resolved, and
  triggered mana abilities that resolved without the stack (CR 605.4a). A card is
  *fully exercised* when it was cast or played (unless it's a supplementary-deck or
  conspiracy card) and each activated and triggered ability on any face — keyword
  abilities included — resolved at least once.
- **The checks** (`checks.rs`, every 4th priority decision unless `--check N` says
  otherwise, and on every event): no state-based action pending when a player gets
  priority (CR 117.5); every object in exactly the zone that lists it (CR 400.7);
  characteristics computable and up to date (a forced recompute on a copy must agree);
  damage marked only on creatures, battles, or permanents dealt damage this turn
  (CR 120.3); the stack empty and mana pools emptied as each step ends (CR 500.2, 500.5;
  mana an effect keeps is fine); no damage left as a turn begins (CR 514.2); life totals
  and counters on players and permanents equal to what the life and counter events add
  up to (reversed illegal actions excepted, CR 733.1). Each check has unit tests showing
  it reports a violation.

## Latest full sweep

At commit 47b8f9d (the build whose pool had 23,116 cards), 3 threads on a shared 4-CPU
machine, in six chunks of 4,000 cards:

| | |
|---|---:|
| Cards | 23,116 |
| Games | 46,232 (2 per card) |
| Time | 64 min of chunk wall time (~12 games/s) |
| Panics, hangs, rules violations, stopped (slow) games | **0** |
| Cast or played (of 22,959 castable cards) | 22,893 (99.7%) |
| Activated and triggered abilities used | 19,361 / 20,336 (95.2%) |
| Cards fully exercised | 22,153 (95.8%) |

Game kinds: 22,796 standard, 163 commander, 59 planechase, 40 archenemy, 27 vanguard,
16 conspiracy, 14 attractions, 1 dungeon.

### Not used, by likely reason

| Count | Reason | Typical cause |
|---:|---|---|
| 659 | triggered ability never triggered | Its condition needs a board the random decks seldom build: another planeswalker type ("if you control an Ajani planeswalker"), a spell that targets the card, an opponent's discard spell, a nonbasic land to destroy, a face-down creature turned up, a chosen name. By trigger kind: enters 128, cast 104, dies 75, beginning of a step 73, "where" (first spell each opponent's turn, ...) 41, attacks 29, deals damage 28, batched 27, ... |
| 238 | activated ability never activatable | Costs and targets the game didn't offer: planeswalker ultimates (−7 to −11), "Pay eight {E}", tapping five Goblins, sacrificing Foods, specific targets (a Kavu, a Wall, a red permanent), eight or more mana, graveyard abilities of cards that never reached the graveyard (embalm, encore, scavenge, unearth), cycling lands that are always played as lands |
| 48 | ability on another face | A transforming card that never transformed (day/night, conditions) |
| 42 | never legal to cast | No legal targets (Aerial Volley needs fliers, Coral Net a green or white creature), or too expensive |
| 13 | no mana cost | Melded backs, cards cast only by suspend or other permission (Evermind) |
| 16 | activated ability offered, never activated | Activations that failed as their costs or targets were chosen (copy target ability you control) |
| 11 | offered, never cast | Counterspells with nothing to counter; casts whose payment then failed |
| 14 | activated or triggered, never resolved | Countered, or every target gone (Aetherflux Reservoir's 50 life) |

Run with `--report FILE` for the full list (card, reason, ability text).

After merging the newer engine (pool of 23,613 cards), a one-game-per-card sweep
(`--games-per-card 1`, 23,613 games, 33 min) found no panic, hang or slow game; its one
rules report was the checker's own mistake (Ancient Adamantoise's damage legitimately
isn't removed during cleanup; the check now asks the engine,
`kw::keeps_damage_in_cleanup`). With one game per card: 99.5% cast or played, 93.7% of
the abilities used, 94.7% of the cards fully exercised.

## Engine problems the sweeps found and fixed

Each with a regression test in `crates/mtg-engine/tests`:

- **Stale characteristics after events** (Brightspear Zealot: "+2/+0 as long as you've
  cast two or more spells this turn" didn't apply until something else changed):
  characteristics are recomputed after every batch of events
  (`cr/r611_conditions_on_this_turn.rs`).
- **Speed from "start your engines!" didn't update characteristics** (Samut, the
  Driving Force's "+X/+0, where X is your speed"): the state-based action now marks the
  game for recomputation (`keywords/k702_179_start_your_engines.rs`).
- **A loop of targeted mandatory triggers never ended** (three Faceless Devourers
  exiling and returning each other): choosing the targets the looping ability calls for
  no longer counts as an optional action breaking the loop, so the game is a draw
  (CR 104.4b, 732.5; `cr/r104_loop_with_targets.rs`).
- **Layer dependency analysis was cubic** (a game with dozens of pump effects took a
  minute): the earliest effect applies at once when it depends on nothing, the usual
  case, before the full analysis (CR 613.8b).
- **Scheme decks weren't shuffled** before the game (CR 103.3a;
  `cr/r103_stacked_starts.rs`).
- **The back face of a modal double-faced land couldn't be played** (Pathways): the
  player chooses the face (CR 712.12; `cr/r712_land_faces.rs`).
- **Triggered mana abilities' resolutions weren't counted** like other abilities'
  (CR 605.4a; `cr/r605_triggered_mana_resolution.rs`).

Hooks added for the fuzzer, usable elsewhere: `Game::observer` (`EventObserver`, sees
every event and every reversal of an illegal action, `Game::roll_back`),
`Event::StepEnded`, and `GameConfig::top_of_library` (a stacked start, also for a
supplementary deck's top card).

Slow games found by earlier sweeps (two Enduring Scalelords answering "yes" to each
other forever; Opal Acrolith's and the Lemures' "{0}:" abilities adding an effect each
time) were the random players' doing: the focus agent now caps its uses of a card per
turn and says "yes" less often the more it has this step.
