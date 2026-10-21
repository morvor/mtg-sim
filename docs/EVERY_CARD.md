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
  `GameConfig::top_of_library`), and the first draws bring the lands up to its mana
  value (or the mana value of its most expensive activated ability, if greater).
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
  orders and card names at random, and answer a yes-or-no question asked over and over
  in a step one way, then the other, for longer and longer stretches (a loop can go on
  by "yes" or by "no").
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

At commit fb94ed0 (pool of 29,534 cards), 2 threads on a 4-CPU machine shared with
another build, in eight chunks of 4,000 cards (`--from N --count 4000`, the last one
1,534):

| | |
|---|---:|
| Cards | 29,534 |
| Games | 59,068 (2 per card) |
| Time | 196 min of chunk wall time (~5 games/s on 2 threads) |
| Panics, hangs | **0** |
| Rules violations | 2 games (the checker's own mistake, fixed; see below) |
| Stopped (slow) games | 3 (two Charismatic Conqueror games, one Earth Kingdom Protectors game; see below) |
| Cast or played (of 29,308 castable cards) | 29,211 (99.7%) |
| Activated and triggered abilities used | 25,470 / 26,890 (94.7%) |
| Cards fully exercised | 28,141 (95.3%) |

Game kinds: 29,007 standard, 301 commander, 99 planechase, 62 archenemy, 29 vanguard,
18 conspiracy, 17 attractions, 1 dungeon.

What it found:

- **Counters that remain on a moving object** (Skullbriar, the Walking Grave as a
  commander; Me, the Immortal cast from its graveyard): "counters put on and removed add
  up to 0". The engine was right (the counters aren't *put* on the new object, so no
  event says so); the counter check now carries the old object's counters over when
  `rule_statics::counters_remain::remain` says the engine did (unit test
  `counters_that_remain_on_a_moving_object_are_accounted_for` in `checks.rs`). Both
  games replay clean.
- **Charismatic Conquerors on both sides** ("whenever an artifact or creature an
  opponent controls enters untapped, they may tap that permanent; if they don't, you
  create a 1/1 Vampire"): each token triggers the other player's Conquerors, and here
  "no" continues the loop where the focus agent's old rule (say "yes" less often the
  more it has this step) assumed "yes" did. The focus agent now answers a question asked
  over and over in a step (same source name and prompt) at random six times, then one
  way 24 times, then the other way 96 times, and so on, so whichever answer ends the
  loop comes up (unit tests in `coverage.rs`). That fixed the Earth Kingdom Protectors
  game (which had Conquerors as synergy cards). The engine's handling is tested in
  `rulings/charismatic_conqueror.rs` (tap or token, the ruling that an already-tapped
  permanent gives a token, the two players' Conquerors triggering on each other's
  tokens).
- **Remaining: Charismatic Conqueror's own games** (card 4080, both games) still run
  past the 60 s watchdog. With two or more Conquerors on each side the loop can't be
  ended by any answer: only one of a player's triggers can tap the entering token, so
  the others each make a token for the other side, which triggers *their* Conquerors in
  turn. Tokens pile up (hundreds) and every action gets slower. Each trigger resolves
  correctly; what's open is how the game ends. CR 104.4b makes a loop of mandatory
  actions "with no way to stop" a draw but says loops containing an optional action
  aren't, and here the optional action (tapping) can't stop it (compare CR 732.6, "[A]
  unless [B]" loops continuing as though [A] were mandatory). The engine recognizes a
  loop only when the game state repeats (`game_end::check_mandatory_loop`), and here it
  never does (more tokens each time). Listed here so the two SLOW reports aren't chased
  again.
- **Found in review: tapping a permanent that had left the battlefield.** The old
  object keeps its zone as last known information, and `Game::tap`/`untap` checked only
  that zone, so "they may tap that permanent" on a creature already gone "tapped" it and
  the Conqueror made no token (its ruling: a permanent no longer on the battlefield
  can't be tapped, so you get a token). They now require the object to still be on the
  battlefield (`rulings/charismatic_conqueror.rs`).

### Not used, by likely reason

| Count | Reason | Typical cause |
|---:|---|---|
| 1,043 | triggered ability never triggered | Its condition needs a board the random decks seldom build. By trigger kind: enters 159 ("whenever a Gate you control enters", "if it was kicked", "if you control three or more artifacts"), cast 138 ("a spell that targets ~", multicolored, Adventure, kicked, mana value 6 or more), dies 108, beginning of a step 88 (Cases' solved abilities, "if you control an Ajani planeswalker", win-the-game upkeeps), "where" 71, attacks 54, batched 41, deals damage 34, ... |
| 342 | activated ability never activatable | Planeswalker ultimates, "Pay eight {E}", tapping five Vampires, sacrificing Foods or a creature of three named colors, specific targets (a Kavu, a face-down creature), graveyard abilities of cards that never got there (embalm, encore, scavenge), cycling lands always played as lands, abilities on the other face of a card that never transformed or flipped |
| 70 | never legal to cast | No legal targets (Aura Graft, Brace for Impact), or too expensive |
| 18 | no mana cost | Melded backs, cards cast only by other permission |
| 19 | activated ability offered, never activated | Activations that failed as their costs or targets were chosen |
| 9 | offered, never cast | Counterspells with nothing to counter; casts whose payment then failed |
| 11 | activated or triggered, never resolved | Countered, or every target gone |

Run with `--report FILE` for the full list (card, reason, ability text). The report also
has one `outcome` line per card (`outcome<TAB>index<TAB>card<TAB>cast | exempt: why |
not cast: why<TAB>failed game kinds`), clean cards included, which `mtg-tools verified`
reads for the fuzz checks of the card tiers (`docs/CARD_COVERAGE.md`). `--cards
verified|round-trip|compiled` limits the pool to cards at that tier.

Deck setup changes after this sweep (`every_card.rs`), from the commonest reasons
above: decks get enough lands for the card's most expensive activated ability, not just
its mana value (Abomination, Terrifying Titan's "Power-up — {5}{R/G}{R/G}", embalm and
encore costs; `coverage::activation_mana_value`); the cards for what its text cares
about ("whenever you cast a spell that targets ~" wants targeting spells) come before
cards for the types and keywords it names, which could crowd them out; and "dies" and
"is put into a graveyard from the battlefield" triggers get removal. Chunk 4,000–7,999
re-run with these changes and the merged newer engine (pool of 29,744): abilities used
94.8% → 95.0%, cards fully exercised 95.2% → 95.5%, and its only failures are the two
Charismatic Conqueror games described above.

### Sweep for the card tiers (Oct 21)

The sweep behind `docs/verified-cards.tsv` (pool of 30,222 cards, 60,444 games, 3 threads,
three chunks of about 46 minutes): 99.7% of the castable cards cast or played, 95.4% of
the activated and triggered abilities used, 95.9% of the cards fully exercised. Its four
failed games keep their cards from being verified (`fuzz-clean`): Charismatic Conqueror's
two slow games (the open loop described above), one slow Shahrazad game (subgames
inside subgames), and one Maelstrom Nexus game with a zone-consistency report ("Sight
Beyond Sight #53 is listed in Library(P0) but has become a new object", from turn 27;
replay with `mtg-sim --every-card --from 15350 --count 1 --game 0 --log`). A card a
failed game's message names as an object is charged too (`fuzz-clean (rules in a game of
Maelstrom Nexus)` for Sight Beyond Sight); the other cards of a failed game's decks
aren't, since the report doesn't list them.

The Maelstrom Nexus violation was an engine bug, since fixed: Sight Beyond Sight resolved
with Skill Borrower out (its controller may look at the top card of their library). Taking
the top card into the hand uncovered the other card, which the top-card effect saw again
and made a new object (CR 401.6), while the dig still held the old id and put that on the
bottom: the stale id stayed listed in the library and the new object stayed on top.
Library rearrangements (`library.rs`: put on top/bottom, the rest of a dig) now follow a
card to the object it is now (`tests/cards/card_flow_sight_beyond_sight.rs`). The tier
file still charges both cards until the next sweep.

### Earlier sweeps

At commit 47b8f9d (pool of 23,116 cards, 46,232 games, 3 threads, 64 min): no panic,
hang, rules violation or slow game; 99.7% cast or played, 95.2% of the abilities used,
95.8% of the cards fully exercised. After merging the newer engine (pool of 23,613
cards), a one-game-per-card sweep found no panic, hang or slow game; its one rules
report was the checker's own mistake (Ancient Adamantoise's damage legitimately isn't
removed during cleanup; the check now asks the engine, `kw::keeps_damage_in_cleanup`).

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
turn, and a yes-or-no question asked over and over in a step gets each answer in
turn for longer and longer stretches (see the latest sweep).
