# Metamorphic properties

Tests that check one situation at a time can share the implementer's misunderstanding.
The properties here relate two runs of a whole game that must agree, so they catch whole
classes of bugs on every card without anyone predicting outcomes. (The in-game invariants
of the every-card fuzz — no state-based action pending at priority, consistent zones,
damage, pools, life and counter accounting — are in `crates/mtg-sim/src/checks.rs`.)

Properties 1–4 relate whole random games. Properties 5–8 are checked exhaustively: over
every fully supported card, or over many small generated cases against a brute-force
oracle written from the rules (not from the engine code it checks).

Code: `crates/mtg-engine/src/properties/` (the checks, a seat-independent agent, the
canonical renaming of players and objects; `copies.rs`, `payment.rs`, `damage.rs`),
`crates/mtg-api/src/replay.rs` (transcripts and replays), `crates/mtg-api/src/hidden.rs`
(hidden information). Tests: `crates/mtg-engine/tests/properties/` (a fixed set of games,
and samples of properties 5, 7 and 8), `crates/mtg-api/tests/replay.rs`,
`crates/mtg-api/tests/hidden_information.rs`. `mtg-sim --properties --property NAME` runs
each property at scale (`crates/mtg-sim/src/properties.rs`, `exhaustive.rs`).

## Running

```sh
cargo test -p mtg-engine --test properties        # the fixed set (2, 3 and 4 players)
cargo test -p mtg-api --test replay               # protocol replays
cargo run --release -p mtg-sim -- --properties --games 2000 --jobs 3
cargo run --release -p mtg-sim -- --properties --property seat --games 500 --seed 7
```

`--property` is `seat`, `snapshot`, `rollback`, `determinism`, `replay` or `all` (the
default; repeatable) for properties 1–4; properties 5–8 have their own runner:

```sh
cargo run --release -p mtg-sim -- --properties --property copies --jobs 3           # every card (about a minute)
cargo run --release -p mtg-sim -- --properties --property copies --card "Clone"     # one card
cargo run --release -p mtg-sim -- --properties --property copies-dynamic --stride 1 --jobs 3
cargo run --release -p mtg-sim -- --properties --property hidden --games 400 --jobs 3
cargo run --release -p mtg-sim -- --properties --property payment --games 200000 --jobs 3
cargo run --release -p mtg-sim -- --properties --property damage --games 20000 --jobs 3
```

`copies` takes `--from N --count M` (cards of the pool sorted by name) or `--card NAME`;
`copies-dynamic` checks every `--stride`th card (default 100) from `--from`; `hidden`,
`payment` and `damage` check `--games` cases from `--seed` (`--only G` replays case G).
Each violation is printed with the command that reproduces it.

For properties 1–4: Games get new random decks (`mtg-sim`'s random 60-card decks of fully
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

## 5. Copies have the original's copiable values

**Rules basis.** CR 707.2 (a copy acquires the copiable values: name, mana cost, color
indicator, types, rules text, power, toughness, loyalty — and defense — as modified by
other copy effects, face-down status and "as ... enters" abilities that set them), 707.3,
707.4 (becoming a copy), 707.5 (entering as a copy: its own "as this enters" abilities
apply), 707.7 (linked abilities stay linked on the copy), 707.8–707.8a (double-faced
permanents: the face that's up; a token copy is double-faced with the same face up),
707.9 (exceptions), 707.10 (a copy of a spell: its characteristics and every choice made
for it), 707.10f (a copy of a permanent spell becomes a token).

**The static check** (`properties::copies::check_card`), for every fully supported card
(the pool `mtg-sim --every-card` uses: real paper playable cards, one per name; 31,181).
For a permanent card, with each face that can be up (the front, the back of a
double-faced card, a flip card flipped):

* the card put onto the battlefield directly, and entered through a real zone change,
  has the printed copiable values (unless an ability of its sets them as it enters);
* a token created as a copy of it (under the opponent's control, so the original's
  abilities about its controller's tokens don't apply; with the back face up for a
  double-faced card, CR 707.8a), a vanilla creature that becomes a copy of it, and a
  permanent that enters as a copy of it (Clone, Sculpting Steel, Copy Enchantment or Vesuva,
  whichever can copy what the original is now) have the original's copiable values;
* cast with plenty of mana on a board with something to target, a copy of the spell
  (`copy::copy_spell`) has the spell's copiable values and every choice — modes and
  targets, X, times kicked, costs paid, the casting method, the objects that paid its
  costs, chosen values and choices — and is a spell copy its controller owns (CR 707.10);
  a copy of a permanent spell resolves as a token with the card's copiable values.

Copiable values are compared field by field: name, mana cost, color indicator, colors,
supertypes, card types, subtypes, "every creature type", power, toughness, loyalty, defense,
the card the values come from (its other faces and halves, CR 709.5b, 715.2b), and the
abilities by their structural fingerprints (`structure::fingerprint`), in order — with
which abilities share a link compared as a partition, since a copy's links are its own.

**Allowed differences.** A token copy is a token. A copy effect's own exceptions (CR
707.9): the cards used here have none. A copy that may itself enter as a copy of
something (a token copy of a Clone) doesn't. A copy of a card whose "as this enters"
ability sets characteristics ("it becomes your choice of a 3/3 creature with flying,
...") makes its own choice as it enters, on top of the copied values ("there will be two
copy effects", Corrupted Shapeshifter's rulings; CR 707.2): for those cards the power,
toughness, abilities, types, subtypes and colors aren't compared. Cards that can't be
copied this way in the fixture are counted (743: spells that can't be cast there, Auras
with nothing they could enchant).

**The dynamic check** (`properties::copies::check_dynamic`): the same script with the
card and with a copy of it must give the same outcome. The copy is a card entering as a
copy of the card (a custom Clone for any permanent), or for a double-faced card a token
copy (only double-faced permanents transform, CR 701.27c). The script: it enters (its
triggered abilities resolve with the default choices), each of its first four activated
abilities is activated with plenty of mana and resolves, it attacks if it's a creature,
and the game goes on to its controller's next turn. After each step the two games'
outcomes are compared: each player's life, counters, hand, library and graveyard sizes,
the names in graveyards and exile, mana pools; every permanent's name, controller,
status, counters, power and toughness, damage and attachment; the stack. Once the card
leaves the battlefield in both games the comparison stops (the copy was a different card,
or a token, from then on); if it leaves in only one, that's a difference.

**Scope.** `cargo test` checks every 20th card of the pool statically (1,558 cards,
11 s in a debug build) and the dynamic check on every 300th (104 cards); `mtg-sim`
checks the whole pool, both ways.

## 6. Hidden information stays hidden

**Rules basis.** CR 400.2 (libraries and hands are hidden zones), 401.2 (library order),
402.3 (a player may look at only their own hand), 406.3 (face-down exiled cards), 708.5
(face-down spells and permanents), 103.3a and 717.2 (face-down supplementary decks),
702.106a (hidden agenda); the visibility rules of `mtg_api::visibility`.

**The property.** What a player is shown — the observation, every legal option of the
decision with its description, and the events since their previous decision, as the JSON
request of `docs/AGENT_PROTOCOL.md` — doesn't depend on information hidden from them.

**The check** (`mtg_api::hidden::check_game`). A random game is played with random
agents. At each decision that isn't a priority decision, and every 4th priority decision
(3rd in the tests), a copy of the game is made that differs only in what the deciding
player can't see, and the request is prepared for that player in both; the two must be
identical, object ids included:

* another player's hand: the identities of hidden hand cards are exchanged with those of
  hidden library cards (the hand keeps its size, each object its id);
* the order of every library, the player's own too, and of every face-down deck in the
  command zone (Attraction, planar, scheme): the hidden cards change places, each keeping
  its id;
* the identity of a face-down card the player can't look at (a face-down permanent or
  spell, a card exiled face down): exchanged with that of a hidden library card of the
  same owner; and which card is which among the hidden cards of a command-zone deck.

Only objects hidden from the player since they were put into their zone are changed (a
card returned to a hand from a graveyard was seen), and the cards a decision shows from
a library (a search) are left alone.

**Ids** (`check_start`): two games whose decks hold the same cards in different orders
(each library then holds them in another order, and the objects get their ids in the
deck's order) must look the same to each player at their first decision once the opening
hands are drawn. Before that, the decks are in the order they were given, which nobody
sees but which ids follow; the comparison starts after the shuffle.

**Scope.** The tests play two games of decks full of secrets (morphs, manifests, cloaks,
disguises, foretell, hands looked at and revealed, library searches; 2 players) and two
random games (2 and 3 players), and check ids at the start with random decks and with
Attraction, planar and scheme decks and hidden agenda conspiracies; `mtg-sim` plays
random games of 2, 2, 3 and 4 players.

## 7. Mana payment

**Rules basis.** CR 601.2g–h (mana abilities are activated, then the total cost is paid),
605.3a, 106.6 (spending restrictions), 107.4 (mana symbols: hybrid, monocolored hybrid,
Phyrexian, snow, X, generic and colorless), 107.4f (2 life for a Phyrexian symbol), 118.3,
119.4 (paying life only with that much life).

**The check** (`properties::payment::check`). A random board: up to five sources among
basic, snow, dual and pain lands, City of Brass, Llanowar Elves, Birds of Paradise,
Treasures, Sol Ring, and sources with spending restrictions (Ancient Ziggurat: creature
spells; Pillar of the Paruns: multicolored spells; Mishra's Workshop: artifact spells);
sometimes mana already in the pool; life 1, 2, 3, 4 or 20. A spell with one to four
random symbols (colored, colorless, generic, hybrid, monocolored hybrid, Phyrexian, snow,
X with a value of 0–2), a creature, artifact or sorcery. The oracle models each source by
hand from its Oracle text (what each of its mana abilities adds, the damage it deals, the
restriction on its mana) and searches every way of activating them (each at most once,
each choice of color) and every assignment of the mana to the symbols, with life for
Phyrexian symbols — only life left after the mana abilities' damage.

* Whenever the oracle finds a payment, the spell must be a legal action (checked when X
  is 0, since the list doesn't depend on the X chosen later) and casting it must succeed.
* Every payment the engine makes must be legal: some activation of the sources it used
  must make exactly the mana spent plus what's left in the pool, and the mana spent (with
  its snow status and restrictions) and the life it lost beyond the sources' damage must
  pay the cost.

**Scope.** 300 boards in `cargo test` (two seconds); `mtg-sim` runs any number.

## 8. Combat damage assignment

**Rules basis.** CR 510.1a–e (each creature assigns its damage; a blocked creature
divides it as its controller chooses among the creatures blocking it — damage assignment
order no longer exists; the total assignment is checked), 510.4 (first strike and double
strike steps), 702.2c (deathtouch: any damage is lethal), 702.19b–e (trample and trample
over planeswalkers: lethal damage counts damage already marked and the damage the other
creatures assign in the step), 702.22h–k (banding: the defending player divides the
damage of an attacker its creature with banding blocks; a band is blocked together).

**The check** (`properties::damage::check`). A random combat: one to three attackers
(power 0–4, toughness 1–4; trample, trample over planeswalkers, deathtouch, first strike,
double strike and banding at random; attacking bands) attacking the defending player, a
planeswalker (loyalty 1–4) or a battle (Invasion of Segovia), and up to three blockers
(banding, deathtouch, first strike, damage already marked) blocking at random; one case in
eight has two tramplers in a band blocked together. Before each combat damage step, the
oracle computes from the rules alone which creatures assign combat damage, to what, and
which joint assignments are legal. Then for every joint assignment in a superset (every
division of each creature's damage among the recipients the rules or the engine name),
the step is played on a copy of the game with agents answering that assignment (and, when
the engine asks in which order tramplers sharing a blocker assign, every order): the damage
dealt must be the assignment exactly when the rules allow it, and otherwise the engine's
default. Cases whose declarations the engine changed (an illegal block) or with more than
600 joint assignments are skipped.

**Scope.** 60 combats in `cargo test` (about 3 seconds); `mtg-sim` runs any number.

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
* **Links of abilities continuous effects give are hashes.** Abilities a static or
  other continuously reevaluated effect gives (`layers::acquired_ability`,
  `ability_grants::gained_link`) get links hashed from the ability, the granter and the
  source into 0x8000–0xbfff, recomputed each time characteristics are; two such grants to
  one object could, very rarely, hash to the same link (CR 607.5). Links resolving
  effects give are numbered per game (0xc000 on) and can't collide.
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
