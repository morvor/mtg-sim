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

## Results

(See below.)
