# Metamorphic properties

Tests that check one situation at a time can share the implementer's misunderstanding.
The properties here relate two runs of a whole game that must agree, so they catch whole
classes of bugs on every card without anyone predicting outcomes. (The in-game invariants
of the every-card fuzz — no state-based action pending at priority, consistent zones,
damage, pools, life and counter accounting — are in `crates/mtg-sim/src/checks.rs`.)

Code: `crates/mtg-engine/src/properties/` (the checks, a seat-independent agent, the
canonical renaming of players and objects), `crates/mtg-api/src/replay.rs` (transcripts
and replays). Tests: `crates/mtg-engine/tests/properties/` (a fixed set of games),
`crates/mtg-api/tests/replay.rs`.

## Running

```sh
cargo test -p mtg-engine --test properties        # the fixed set (2, 3 and 4 players)
cargo test -p mtg-api --test replay               # protocol replays
cargo run --release -p mtg-sim -- --properties --games 2000 --jobs 3
cargo run --release -p mtg-sim -- --properties --property seat --games 500 --seed 7
```

`--property` is `seat`, `snapshot`, `rollback`, `determinism`, `replay` or `all` (the
default; repeatable). Games get new random decks (`mtg-sim`'s random 60-card decks of fully
supported cards) and cycle through 2, 2, 3 and 4 players unless `--players N` is given; they
end in a draw after `--max-turns` (40) turns or 4,000 decisions. A violation or a panic is
printed with the command that reproduces it (`--only G`), and the run exits with status 1.
For a difference in the hash of a whole game, the report shows the first differing text;
`MTG_PROPERTY_CONTEXT=N` widens the context shown around it.

The agents (`properties::PropertyAgent`) play at random, a little like `RandomAgent`, but
they choose among options sorted by canonical keys, so they make the same choices in games
that differ only in seats; they take a mulligan now and then, and at priority they
sometimes try an action that isn't legal (see property 3). Every decision is recorded:
who made it, the decision with its options in canonical order, and the answer; at every
priority decision also the canonical state (turn, step, priority, each player's life,
counters, zones, mana pool, and every object on the battlefield, on the stack, in exile and
in the command zone with its status, counters, damage, timestamp and characteristics).

## 1. Seat symmetry

**Rules basis.** CR 101.4 (APNAP order: the active player, then the others in turn order),
102.1, 103.1 (turn order begins with the starting player), 500.1, 613.7m (simultaneous
timestamps in APNAP order). Nothing in the rules depends on a player's seat number, only on
turn order and the players' choices.

**The property.** Two games differ only in which seat each deck sits in: deck *i* sits in
seat *(i + r) mod n* (a rotation, so turn order is unchanged), and deck 0 goes first. With
`GameConfig::player_rng_keys` each player's randomness (library shuffles, random discards,
random cards revealed from a hand) comes from their own stream keyed by their deck, not
their seat, so each library starts in the same order in both games. The two games must be
identical up to renaming the players and objects: their transcripts are equal.

**Renaming** (`properties::Canon`). A player is named by their deck. Objects are numbered in
creation order; the cards a game starts with are renumbered by their owner's deck, and
objects created later keep their ids, which agree between two games that stay alike.
Ability ids are named by their place among their object's abilities. Lists whose order
the rules don't give (option lists, the attackers in combat) are sorted.

**Scope.** The fuzz mode checks one rotation per random game (2, 3 and 4 players); the
tests check every rotation of a fixed set of 2- and 3-player games and one of a 4-player
game.

## 2. Snapshot equivalence

**The property.** The engine reverses illegal actions by restoring a clone of the whole
`Game` (`Game::roll_back`). At a random point of a random game (after `seed mod 1200`
units of play), the game and its agents are cloned; the original and the copy are played
to the end with identical agents, and their transcripts after the copy, a hash of the
whole game's debug output at the beginning of every turn, and the final game must be
equal. A difference means state a clone doesn't copy faithfully or shares with the
original (shared mutable state behind `Rc`/`Arc`, registries, caches, per-action
trackers).

## 3. Rollback identity

**Rules basis.** CR 733.1: if a player starts an action that can't legally be completed,
the entire action is reversed and payments are canceled; no abilities trigger and no
effects apply as a result. CR 733.2: the player who had priority retains it.

**The property.** Whenever the engine reverses an action a player began at priority
(casting a spell, activating an ability, playing a land, a special action), the game when
that player is next asked equals the game when they chose the action. The agents try
actions that aren't legal (at least one priority decision in five): casting any card in any
hand, activating any activated ability of any permanent, playing any land card in their
hand.

**What may differ.** CR 733.1 lets these stay: moving cards to a library, moving cards from
a library to a zone other than the stack, shuffling a library, revealing cards from a
library. The engine reverses even those (it restores the whole game), which the property
allows. Besides, the comparison ignores the simulation's bookkeeping, which isn't game
state: the decision counter, the count of reversed attempts at this priority, the log,
the count of state-based-action checks, and the loop watch (CR 732.3, 104.4b), which
records the choices players made at priority, attempts included.

## 4. Determinism and replay

**The property.** The same seed and the same decisions give the same game:

* `determinism`: the same game played twice in one process gives equal transcripts, equal
  hashes of the whole game at the beginning of every turn, and an equal final game.
* `replay`: a game played through the agent protocol (an `mtg_api::Session` with every seat
  external, answered at random) is recorded as a transcript in the `--transcript` format;
  replayed in a new game from the transcript's answers (`mtg_api::replay`), every request
  and each seat's game-over message must equal the recorded one. The tests also replay a
  game played by child processes (`ExternalAgent`, `examples/random_client.py`) from its
  transcript file, and check that a different game is noticed. On the command line,
  `mtg-sim --agent 1=replay:FILE` replays a seat from a transcript written by
  `--transcript FILE` (with the same seed and decks), reporting any difference.

**Ability ids.** Ability ids come from a counter shared by the whole process, and many
abilities are created as a game goes (a delayed trigger's ability as it triggers, a Clue's
ability as a player investigates, abilities acquired from other objects), so two runs of
the same game in one process, or a run and its replay, number them differently. They are
compared up to a consistent renaming, like objects: in debug output by order of first
appearance (`properties::agent::rename_uids`), in requests by pairing the ids as they are
met (`replay::AbilityRenaming`).

## Bugs found

Each was fixed, with a regression test citing the rules:

| Property | Bug | Rule | Fix | Test |
|---|---|---|---|---|
| seat | A player's opponents were listed in seat order, so defaults among them (the defender of an attack the engine declares when a declaration breaks a requirement) depended on seats | 101.4, 508.1d | `Game::opponents` lists them in APNAP order | `tests/cr/r101_004_opponents_in_apnap_order.rs` |
| seat | Choices made at random ("target opponent chosen at random", random picks among objects) shuffled candidates listed in seat order | 101.4 | `Game::sort_apnap` before shuffling | same file |
| seat | Targets that must go together ("two target cards from an opponent's graveyard") were replaced, when they didn't, by the first group among the candidates in seat order | 101.4, 601.2c | candidates in APNAP order for the replacement group | same file |
| seat | When several players lost at once, only the first in seat order had their objects leave the game (for the next ones the game was already over) | 104.4a, 800.4a | players losing together leave together, once all have lost (`Game::finish_leaving`) | `tests/cr/r800_004a_players_leaving_together.rs` |
| seat | Players losing at once were handled in seat order, so what happened as the first of them lost (effects ending, characteristics recomputed) depended on seats | 101.4, 104.4a | in APNAP order | same file |
| seat | A player whose cards outside a subgame left it with them (they lost the subgame) had their main-game hand and permanents put into their library as the subgame ended, as though brought into it | 729.4, 729.5, 800.4a | such cards are left in the main game | same file |
| seat | Who goes first in a subgame was drawn among the players in seat order; cards returning from a subgame were created in seat order | 729.2, 101.4 | APNAP order | `tests/properties/seat_symmetry.rs` (`a_subgame_is_seat_symmetric`) |

Not bugs in the rules, but found and handled by the properties:

* Ability ids, some links (CR 607.5: links of abilities an effect gives come from the
  ability-id counter), mana riders' ids and replacement keys are drawn from a counter shared
  by the whole process, so they differ between two runs of the same game in one process,
  and between a protocol game and its replay (a Clue's ability as a player investigates,
  a delayed trigger's ability as it triggers). The comparisons rename them consistently.
* Prompts name players by seat ("attacking P2") and objects by id ("Forest #53"); the
  comparison renames those.

## Remaining gaps

* **CR 733.1's exceptions aren't kept.** The engine reverses an illegal action by restoring
  a copy of the whole game, so it also reverses what CR 733.1 says players may not reverse
  (moving cards to or from a library, shuffling, revealing library cards). A scan of the
  compiled costs and mana abilities of every fully supported card found none that can
  reach it (a cost's library actions are paid after everything that can make the action
  illegal, and no supported mana ability touches a library), so it's registered as the
  provisional deviation `illegal-action-reversal-restores-libraries`
  (docs/interpretations/stack-casting-costs.md, with the cards found). The scan also found
  that Thought Lash's cumulative upkeep could be paid in part; that's fixed.
* **Ability ids depend on the process.** They come from a global counter (`ability.rs`)
  and are assigned as abilities are first needed, so the ids in protocol requests
  (`action.ability`) differ between processes unless both play the same games in the same
  order. Answers refer to options by index, so replays work; making the ids deterministic
  would mean allocating them per game, which the caches of derived abilities (shared by all
  games) don't allow without reworking them. Registered as the policy
  `ability-ids-depend-on-the-process` (docs/interpretations/decisions-policies.md).
* **Seat symmetry compares what the canonical state shows**: the turn, players, zones and
  public objects with their status and characteristics, the stack, combat and every
  decision. Internal bookkeeping that never reaches those (and hidden-information views)
  is only compared through its effects on later decisions; snapshot equivalence and
  determinism compare the whole game.
* **Rollback checks are sampled in long games**: the first 30 actions a game's players
  start, then every fifth (each check copies and prints the whole game).

## Results

Latest runs (release build, 4-core machine), `--seed 10 --games 2000`: games cycle through
2, 2, 3 and 4 players with new random 60-card decks, up to 40 turns or 4,000 decisions.

| Property | Games | Held | Decisions | Violations | Time (CPU) |
|---|---|---|---|---|---|
| seat | 2,000 | 2,000 | 4,561,047 | 0 | 2,648 s |
| snapshot | 2,000 | 2,000 | 4,625,498 | 0 | 3,585 s |
| rollback | 2,000 | 2,000 | 2,431,653 (120,750 reversed actions checked) | 0 | 2,469 s |
| determinism | 2,000 | 2,000 | 4,625,498 | 0 | 4,126 s |
| replay | 2,000 | 1,990 + 10 slow | 1,455,369 requests | 0 | 14,799 s |

The replay run allowed 8,000 requests per game (now 4,000) and stopped ten games at the
two-minute limit (replays answer every request through JSON with whole observations, and
the random answerer, unlike the property agents, doesn't hold back from repeating
actions, so some games pile up effects and slow down). Checked again one by one with a
longer limit, nine of them held (five with up to 8,000 requests, four with up to 4,000); game 1178
was still going after 30 minutes, its requests ever slower — a slow game, not a
difference (`mtg-sim --properties --property replay --seed 10 --only 1178`). So 1,999 of
2,000 replays held and none differed. On the command line, a game against
`examples/random_client.py` recorded with `--transcript` replays with `--agent
1=replay:FILE` in a new process without a difference, and with another seed the first
request is reported as differing.

Earlier runs found the bugs above: seat symmetry 12 violations in 300 games (`--seed 3`)
and 4 in 2,000 (`--seed 10`), all fixed; snapshot and determinism flagged ability ids,
links, mana riders' ids and replacement keys drawn from the process-wide counter (now
compared up to renaming; the links of abilities resolving effects give are now numbered
per game in a range of their own and skip links still in use, so they can't equal another
link on the object, CR 607.5); rollback identity found the loop watch recording reversed
attempts, which it now ignores as bookkeeping. Rollback identity, snapshot equivalence
and determinism found no engine bug.

After merging the card-tier work (sessions and `mtg-sim` decks now default to verified
cards; the property games keep using every fully supported card), 100 more games of each
property (`--seed 20`) held.
