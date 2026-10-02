# Agent protocol

Programs outside the engine — a Python harness around a language model, a search or
learning agent, a game server — can play any seat of a game. At each decision the engine
sends the seat's agent what that player can see of the game, what happened since its
previous decision, and the **complete list of legal options**; the agent answers by
picking among them.

The protocol is implemented by the `mtg-api` crate. There are three ways to use it:

| | |
|---|---|
| Child process | `mtg-sim --agent 1=cmd:'python3 examples/random_client.py'`: the engine starts the program and speaks newline-delimited JSON on its stdin/stdout (one message per line, one answer per line). `ExternalAgent` / `mtg_api::spawn_external` in Rust. |
| Embedded, pull-style | `mtg_api::Session` runs a game on its own thread; `session.next()` returns the next `Request` of an external seat, `session.answer(seat, answer)` answers it. |
| In-process | `mtg_api::ProtocolAgent<T: Transport>` is an engine `Agent`; implement `Transport` to carry requests anywhere. |

`examples/random_client.py` is a minimal dependency-free client that plays random legal
options.

```sh
cargo run --release -p mtg-sim -- --games 5 \
    --agent 0=random --agent 1=cmd:'python3 examples/random_client.py' \
    --transcript /tmp/transcript.txt   # every line sent (>seat) and received (<seat)
cargo run --release -p mtg-sim -- --random-decks --games 20 \
    --agent 0=cmd:'python3 examples/random_client.py' \
    --agent 1=cmd:'python3 examples/random_client.py'
```

`--agent SEAT=KIND` takes `random` (the default), `passive` (lets the engine choose
everything) or `cmd:COMMAND` (run with `sh -c`; the seat number is in the environment
variable `MTG_AGENT_SEAT`). `--auto-pass` answers priority for external agents when they
can do nothing but pass (or activate mana abilities, or concede), which removes most
requests. External agents work with `--deck` and `--random-decks`.

## Messages

Every message is one line of JSON. The engine sends two kinds of messages, told apart
by `type`:

* `"decision"`: a request. The agent must answer with exactly one line.
* `"game_over"`: the game ended. No answer; the engine then closes the agent's stdin, and
  the agent should exit.

Lines the agent writes must each be one answer. Anything the agent wants to log goes to
stderr (which the simulator passes through) or a file.

### Requests

The examples below are taken from a real game (`mtg-sim --seed 5` with the example
client on both seats), shortened where marked `…`.

```json
{
  "type": "decision",
  "version": 1,
  "id": 42,
  "seat": 1,
  "player": 1,
  "kind": "priority",
  "prompt": "You have priority: turn 3, P1's upkeep step; the stack is empty.",
  "answer": {"type": "choose_one"},
  "options": [
    {"index": 0, "text": "Pass priority", "action": {"type": "pass"}},
    {"index": 1, "text": "Cast Giant Growth #125 ({G})",
     "action": {"type": "cast", "card": 125, "method": "normal", "mana_cost": "{G}"}},
    {"index": 2, "text": "Activate mana ability of Forest #134: {T}: Add {G}.",
     "action": {"type": "mana_ability", "source": 134, "ability": 10}},
    {"index": 3, "text": "Activate mana ability of Llanowar Elves #136: {T}: Add {G}.",
     "action": {"type": "mana_ability", "source": 136, "ability": 5}},
    {"index": 4, "text": "Concede the game", "action": {"type": "concede"}}
  ],
  "events": [
    {"kind": "step", "text": "cleanup step of P0's turn", "players": [0]},
    {"kind": "turn", "text": "Turn 3 began (P1's turn)", "players": [1]},
    {"kind": "step", "text": "upkeep step of P1's turn", "players": [1]}
  ],
  "observation": {
    "viewer": 1, "turn": 3, "active_player": 1, "phase": "beginning", "step": "upkeep",
    "priority": 1,
    "players": [
      {"id": 0, "name": "Player 1", "life": 20, "hand_count": 6, "library_count": 52,
       "lands_played_this_turn": 0, "land_plays": 1, "max_hand_size": 7, "team": 0},
      {"id": 1, "name": "Player 2", "life": 20, "hand_count": 5,
       "hand": [
         {"id": 121, "zone": "hand", "owner": 1, "controller": 1, "name": "Lightning Bolt",
          "mana_cost": "{R}", "mana_value": 1, "colors": ["red"], "types": ["instant"],
          "text": "Lightning Bolt deals 3 damage to any target."},
         …
       ],
       "library_count": 53, "lands_played_this_turn": 0, "land_plays": 1,
       "max_hand_size": 7, "team": 1}
    ],
    "battlefield": [
      {"id": 136, "zone": "battlefield", "owner": 1, "controller": 1,
       "name": "Llanowar Elves", "mana_cost": "{G}", "mana_value": 1, "colors": ["green"],
       "types": ["creature"], "subtypes": ["Elf", "Druid"], "power": 1, "toughness": 1,
       "text": "{T}: Add {G}."},
      {"id": 138, "zone": "battlefield", "owner": 0, "controller": 0, "name": "Forest",
       "supertypes": ["basic"], "types": ["land"], "subtypes": ["Forest"],
       "text": "({T}: Add {G}.)", "tapped": true},
      …
    ]
  },
  "attempt": 0
}
```

| Field | Meaning |
|---|---|
| `type` | `"decision"` |
| `version` | Protocol version, currently `1` (see [Versioning](#versioning)). |
| `id` | Identifies the decision within the game (increasing). Re-asks of the same decision keep the id. An answer may echo it. |
| `seat` | The seat whose agent is asked. |
| `player` | The player the decision is for: the seat itself, unless it controls another player (CR 723, e.g. Mindslaver). |
| `kind` | What is being decided (table below). |
| `prompt` | A readable question. |
| `source`, `source_name` | The spell, ability or permanent the decision is about, when there is one. |
| `answer` | The shape of the expected answer (below). |
| `options` | The legal options, each with its `index` (0, 1, …), a readable `text`, and structured fields. |
| `events` | What happened since this seat's previous request (below). |
| `observation` | What this seat can see of the game (below). |
| `error` | Present when the previous answer to this decision was rejected: why. |
| `attempt` | 0 the first time a decision is asked; 1, 2, … for re-asks after rejected answers. |

Fields that would be empty, false, zero or absent are omitted throughout.

### Decision kinds

| `kind` | Answer shape | Options |
|---|---|---|
| `priority` | `choose_one` | Every legal action (CR 117.1): pass (always option 0), each castable spell with each way of casting it, each land play, each activated ability, each mana ability, each special action, and conceding (always last). |
| `mulligan` | `yes_no` | yes = take a mulligan, no = keep (CR 103.5). |
| `put_on_bottom` | `choose_many` (exactly n) | Cards in hand (London mulligan). |
| `choose_modes` | `choose_many` | Only the modes that may be chosen (CR 700.2); with pawprints, each option has a `cost` and the spec a `budget` (CR 700.2i). |
| `choose_x` | `number` | The value of X (CR 107.3). |
| `casting_method` | `choose_one` | Ways to cast a spell. |
| `optional_cost` | `yes_no`, or `number` for a cost that may be paid any number of times (multikicker) | |
| `targets` | `choose_many` | Legal targets for one target slot (CR 115). |
| `divide` | `divide` | Recipients of damage or counters divided as the spell is cast (CR 601.2d). |
| `yes_no` | `yes_no` | "You may …" |
| `choose` | `choose_many` | Players or objects to choose (sacrifice, discard, search, …). |
| `order` | `order` | Items to order (triggers, cards going to a library or graveyard). |
| `choose_option` | `choose_one` | Named choices (a color, a card type, …). |
| `choose_number` | `number` | |
| `name_card` | `text` | Any card name (CR 201.3). |
| `declare_attackers` | `choose_many` | One option per (creature, what it could attack); `group` is the creature, `group_max` 1 (CR 508.1). |
| `declare_blockers` | `choose_many` | One option per (blocker, attacker it could block); `group` is the blocker, `group_max` how many attackers it may block (CR 509.1). |
| `assign_combat_damage` | `assign_damage` | Recipients of a creature's combat damage with the `lethal` damage of each (CR 510.1). |
| `scry` | `split` (top / bottom) | The cards looked at (CR 701.22). |
| `surveil` | `split` (top / graveyard) | The cards looked at (CR 701.25). |
| `replacement` | `choose_one` | Replacement or prevention effects, to choose which applies first (CR 616.1). |

### Options

Each option has `index` and `text`, and, depending on the decision:

* `action` (priority): `{"type": …}` with `type` one of `pass`, `play_land`, `cast`,
  `activate`, `mana_ability`, `special`, `concede`; `card` / `source` object ids;
  `ability` (the ability's id on that object); `method` for casts (`normal`, `free`,
  a keyword such as `flashback` or `evoke`, `face_down:morph`, `half:1` for one half of a
  split card, an Adventure or the back of a modal double-faced card,
  `alternative:<id>` for an alternative cost granted by an ability); `mana_cost`;
  `special` for special actions (`turn_face_up`, `suspend`, `foretell`, `plot`,
  `companion_to_hand`, `static`, `offer`, `roll_planar_die`, `other`).
* `entity`: the player (`{"player": 0}`) or object (`{"object": 42}`) the option is.
* `card`: for a card in a library or hand that the decision lets the player look at (a
  search, a scry, a card to discard): its characteristics, since the observation
  doesn't show it.
* `required` (attackers): choosing this option obeys an attack requirement ("attacks
  each combat if able", goad); a declaration must obey as many requirements as possible.
* `group`, `group_max`, `target` (attackers and blockers), `lethal` (combat damage),
  `cost` (pawprint modes), `value` (yes/no).

Example (`declare_attackers`, from the same game):

```json
{"type": "decision", "version": 1, "id": 97, "seat": 1, "player": 1,
 "kind": "declare_attackers",
 "prompt": "Declare attackers: choose the attacking creatures and what each attacks (none to not attack). The declaration must obey attack requirements and restrictions (CR 508.1c-d).",
 "answer": {"type": "choose_many", "min": 0, "max": 2, "distinct": true},
 "options": [
   {"index": 0, "text": "Llanowar Elves #136 attacks P0", "group": 136, "group_max": 1,
    "target": {"player": 0}},
   {"index": 1, "text": "Llanowar Elves #145 attacks P0", "group": 145, "group_max": 1,
    "target": {"player": 0}}],
 "observation": {…}, "attempt": 0}
```

answered with `{"indices": [0, 1], "id": 97}`. A `targets` request:

```json
{"type": "decision", "version": 1, "id": 43, "seat": 1, "player": 1, "kind": "targets",
 "prompt": "Choose 1 target(s): target creature", "source": 141,
 "source_name": "Giant Growth #141",
 "answer": {"type": "choose_many", "min": 1, "max": 1, "distinct": true},
 "options": [
   {"index": 0, "text": "Llanowar Elves #136", "entity": {"object": 136}},
   {"index": 1, "text": "Llanowar Elves #140", "entity": {"object": 140}}],
 "observation": {…}, "attempt": 0}
```

answered with `{"indices": [1], "id": 43}`. Combat damage:

```json
{"type": "decision", "version": 1, "id": 593, "seat": 1, "player": 1,
 "kind": "assign_combat_damage", "prompt": "Assign 6 combat damage dealt by Craw Wurm #202.",
 "source": 202, "source_name": "Craw Wurm #202",
 "answer": {"type": "assign_damage", "total": 6, "trample": false},
 "options": [
   {"index": 0, "text": "Hill Giant #206", "entity": {"object": 206}, "lethal": 3},
   {"index": 1, "text": "Grizzly Bears #229", "entity": {"object": 229}, "lethal": 2}],
 "observation": {…}, "attempt": 0}
```

answered with `{"numbers": [4, 2], "id": 593}`.

### Answers

An answer is one JSON object with exactly one answer field. `id` is optional; when
present it must be the request's `id`. `"type": "answer"` may be included.

| `answer.type` | Answer | Rules |
|---|---|---|
| `choose_one` | `{"index": 3}` | A valid option index. |
| `choose_many` (`min`, `max`, `distinct`, `budget`?) | `{"indices": [0, 2]}` | Between `min` and `max` indices; each at most once if `distinct`; at most `group_max` options of a group; option `cost`s totalling at most `budget`. |
| `order` | `{"indices": [2, 0, 1]}` | A permutation of all option indices, first first. |
| `number` (`min`, `max`?) | `{"number": 2}` | `min` ≤ n ≤ `max` (no upper bound if `max` is absent). |
| `divide` (`total`, `min_each`) | `{"numbers": [2, 1]}` | One number per option, each ≥ `min_each`, adding up to `total`. |
| `assign_damage` (`total`, `trample`) | `{"numbers": [4, 2]}` | One number per option, ≥ 0, adding up to `total`. With `trample`, nothing to the options after the blocking creatures (those with `lethal`, in order) until each blocking creature has its `lethal` damage, and nothing to a planeswalker's controller until the planeswalker has its `lethal` (CR 702.19b–c). |
| `yes_no` | `{"bool": true}` | `{"index": 0}` (no) and `{"index": 1}` (yes) work too. |
| `split` (`first`, `second`) | `{"split": [[1], [0, 2]]}` | Every option index in exactly one of the two lists; the first list is the top of the library from the top down, the second the bottom (scry) or graveyard (surveil), in order. |
| `text` (`what: "card_name"`) | `{"text": "Lightning Bolt"}` | The exact name of a card. |
| any | `{"default": true}` | Let the engine choose its default (pass priority, keep a hand, no attackers, the first legal targets, …). |

### Exactness

The options are complete and exact: every listed option can be taken and nothing else
is accepted.

* **Priority**: the engine's own legality checks produce the list, and each action is
  then tried on a copy of the game (with the engine's default choices, then the cheapest
  ones: X = 0, no optional costs, as few targets as allowed; then a few random ones) so
  that only actions that can really be completed are listed (CR 733: an action that
  can't be completed would otherwise be reversed). This search is a heuristic: an action
  that can be completed only with an unusual combination of choices may be missed. Mana abilities (CR 117.1d) and
  conceding (CR 104.3a) are listed too. Pass is always option 0.
* Choices made while taking an action (targets, modes, X, costs) are their own requests.
  If an agent's choices make the action impossible to complete (e.g. an X too large to
  pay for), the action is reversed (CR 733.1) and the agent gets priority again: a new
  `priority` request.
* **Attackers and blockers**: any subset of the options obeying the group limits is
  well-formed, but the declaration as a whole must also obey the requirements and
  restrictions (CR 508.1c–d, 509.1b–c: "attacks each combat if able", menace, "can't
  attack alone", …) and its costs must be payable. The engine checks this with its own
  rules code and rejects an illegal declaration with the reason, e.g.
  `"Skittering Precursor #155 can't be blocked except by 2 or more creatures"`.

## Invalid answers

An answer that can't be parsed, has the wrong shape, an index out of range, too many or
too few choices, or breaks a rule is rejected: the same request is sent again (same
`id`) with `error` set and `attempt` increased, for example

```json
{"type": "decision", "version": 1, "id": 100, "seat": 0, "kind": "declare_blockers", …,
 "error": "Skittering Precursor #155 can't be blocked except by 2 or more creatures",
 "attempt": 1}
```

After 3 rejected answers (configurable: `ProtocolOptions::max_retries`) the engine's
default answer is used and the game goes on. If the agent closes its output or exits,
the engine answers every later decision of that seat with its defaults.

## Events

`events` lists what happened since the seat's previous request, oldest first, each with
`kind`, a readable `text`, and the `players` and `objects` involved. Kinds include
`turn`, `step`, `cast`, `activate`, `trigger`, `resolve`, `counter`, `zone_change`,
`draw`, `discard`, `mill`, `land`, `token`, `damage`, `damage_prevented`, `life`,
`counters`, `attack`, `block`, `sacrifice`, `destroy`, `reveal`, `search`, `shuffle`,
`control`, `transform`, `face_up`, `face_down`, `attach`, `phase`, `die`, `coin`,
`day_night`, `designation`, `game` and `other`. For example:

```json
{"kind": "damage", "text": "Craw Wurm #202 dealt 4 combat damage to Hill Giant #206",
 "objects": [202]}
```

The feed is produced from the engine's events (`Game::event_feed`, turned on by
`mtg_api::prepare_game`, which the simulator and `Session` call). Events are described
from the seat's point of view: "P1 drew a card" for the opponent's draw, the card's
name for one's own.

## Observation

`observation` is what the seat's player can see:

* `turn`, `active_player`, `phase` (`beginning`, `precombat_main`, `combat`,
  `postcombat_main`, `ending`), `step` (`untap`, `upkeep`, `draw`, `precombat_main`,
  `beginning_of_combat`, `declare_attackers`, `declare_blockers`,
  `first_strike_damage`, `combat_damage`, `end_of_combat`, `postcombat_main`, `end`,
  `cleanup`), `priority`, `day_night`, `monarch`, `initiative`, `result`.
* `players`: for each player `life`, `poison` and the other player `counters`,
  `mana_pool` (by type: `W`, `U`, `B`, `R`, `G`, `C`), `hand_count` and the `hand` cards
  the viewer may see, `library_count` and `library_known` (pairs of position from the
  top and card: a revealed top card, or the top card the viewer may look at),
  `graveyard` (top first), `exile` and `command` (objects the player owns),
  `lands_played_this_turn`, `land_plays`, `max_hand_size`, `team`, designations
  (`monarch`, `initiative`, `citys_blessing`, `speed`, `ring_tempted`, `ring_bearer`,
  `dungeons_completed`, `venture`), `commander_damage`, `has_lost`, `has_won`.
* `battlefield`: every permanent with its **current** characteristics (after all
  continuous effects, CR 613): `id`, `owner`, `controller`, `name`, `mana_cost`,
  `mana_value`, `colors`, `supertypes`, `types`, `subtypes`, `power`, `toughness`,
  `loyalty`, `defense`, `keywords`, `text` (Oracle text), `granted` (abilities its text
  doesn't show, given by effects), `counters`, `tapped`, `flipped`, `transformed`,
  `phased_out`, `face_down`, `damage`, `summoning_sick`, `attached_to`, `attachments`,
  `token`, and `combat` (`attacking` whom, `blocked`, `blockers`; `blocking` which
  attackers).
* `stack`: top first; each with `id`, `kind` (`spell`, `activated_ability`,
  `triggered_ability`), `controller`, `name`, `source`, `text`, chosen `modes`, `x`,
  `targets` (one list per target slot) and `target_names`, and for spells `card`.

Object ids are the engine's: an object that changes zones becomes a new object with a
new id (CR 400.7).

### Hidden information

The observation, the options and the events only contain what the seat's player may
see:

* Libraries: only their size; no player may look at a library or know its order
  (CR 401.2), except a revealed top card, or the top card its owner may look at
  (CR 401.5).
* Hands: an opponent's hand shows only its size (CR 402.3), unless cards in it are
  revealed or the viewer may look at them (cards revealed by an effect, "play with your
  hand revealed", teammates in some multiplayer variants).
* Face-down permanents and spells: everyone sees a face-down 2/2 creature with no name
  (CR 708.2); their controller also sees `face_down_card`, what it really is (CR 708.5).
* Face-down exiled cards: listed without identity, unless the viewer may look at them
  (CR 406.3).
* A player controlling another player sees what that player can see (CR 723.4).
* Cards hidden from the viewer have no id in options and events ("a hidden card").
  Cards in a library or hand that a decision offers as options (searching, scrying,
  choosing a card to discard) are the ones the player is looking at, and are named.

Prompts and option texts that come from the engine (trigger descriptions to order, the
names of replacement effects) describe objects the player can see.

The **omniscient view** (`mtg_api::observe(&game, None)`) shows everything, including
every library in order and every face-down card, for logging and debugging.

## Game over

```json
{"type": "game_over", "version": 1, "seat": 0, "result": {"win": [1]}, "turns": 29,
 "observation": {…}}
```

`result` is `{"win": [players]}`, `"draw"`, or `null` if the game was stopped. The
observation is the seat's final view.

## Using the Rust API

```rust
use mtg_api::{JsonAnswer, ProtocolOptions, Seat, Session};
use mtg_engine::{GameConfig, PlayerId};

let mut session = Session::start(GameConfig::default(), decks,
    vec![Seat::External, Seat::Agent(Box::new(mtg_engine::agents::RandomAgent::new(1)))],
    ProtocolOptions::default());
while let Some(request) = session.next() {
    // request.options, request.observation, request.events, request.error ...
    session.answer(PlayerId(request.seat), JsonAnswer::index(0)).unwrap();
}
println!("{:?}", session.result());
```

`session.answer_json(seat, line)` takes a line of JSON. `ProtocolOptions` sets the retry
count (`max_retries`), `auto_pass`, whether requests carry the observation and events,
and which extra priority options to list (`present.priority`: `verify`,
`mana_abilities`, `concede`). `mtg_api::prepare(&game, seat, player, &decision, id,
&options)` turns any engine `Decision` into a request and `Prepared::convert` checks an
answer and converts it into the engine's `Answer`.

## Versioning

`version` is the protocol version, currently 1. Adding message fields, decision kinds,
event kinds, action types or answer types doesn't change the version: clients should
ignore fields they don't know and may answer `{"default": true}` to a decision kind or
answer type they don't know. A change that breaks existing clients (renaming or
removing a field, changing a field's meaning) increases the version.
