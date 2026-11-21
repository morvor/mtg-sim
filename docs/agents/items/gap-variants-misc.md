# gap-variants-misc: Grand Melee and range-of-influence fidelity, commander eligibility, ante primitives, small fidelity bugs

GOAL: A grab bag of smaller confirmed deviations: Grand Melee's documented simplifications and extra-turn handling, the range-of-influence filter missing for one-shot delayed triggers, commander eligibility for cards that are creatures outside the battlefield, the missing ante primitives (ante a permanent, exchange ownership), and three small fidelity bugs.

GAPS:

[G26] Grand Melee and range-of-influence fidelity
- Rules: CR 807.4 (several turns at once; each turn has its own cleanup step, CR 514.2, and its own
  "that turn" effects), 807.4i/807.4j (extra turns), 807.5b (a player with priority for several stacks
  chooses the stack for a spell or ability), 801.7 (triggers outside the range of influence don't trigger),
  614.10 / "skip that turn" effects for extra turns.
- Engine now (multiplayer/grand_melee.rs documents the first two as "known simplifications", lines 15-17):
  "until end of turn" effects end at the cleanup step of whichever marker's turn reaches it first, and mana
  pools empty whenever any marker's step ends; extra turns re-queued to `extra_before_next` or taken by the
  holder lose their "that turn" actions (grand_melee.rs:382-407 drops them); `begin_marker_turn`
  (grand_melee.rs:243-) never calls `skip::extra_turn_skipped` (only turn.rs:609 does), so Stranglehold /
  Gerrard's Hourglass Pendant don't apply on that path; spells and abilities cast with priority for several
  stacks always go on the stack of the turn being played (no 807.5b choice); one-shot delayed triggers
  (`once_matches`, triggers.rs:589-598) skip the 801.7 `trigger_in_range` filter that ordinary and
  repeating delayed triggers get.
- Approach: key "until end of turn"/"this turn" durations and mana-pool emptying to the marker whose turn
  created them; keep extra-turn actions with queue entries by id; call `extra_turn_skipped` when a marker
  begins an extra turn; ask for the stack per 807.5b; apply `trigger_in_range` to once-delayed matches.
- Size: M

[G27] Commander eligibility ignores abilities that work outside the game
- Rules: CR 903.3 (the commander must be a legendary creature card, or have text letting it be one),
  113.6c (an ability that says which zones it doesn't function in works everywhere else, "even outside the
  game and before the game begins").
- Engine now: `kw::partner::can_be_commander` (kw/partner.rs:210-224) looks only at the front face's printed
  characteristics, so a card that is a creature outside the battlefield through its own static ability
  is rejected; that static itself is also unsupported.
- Cards: Grist, the Hunger Tide ("As long as Grist isn't on the battlefield, it's a 1/1 Insect creature in
  addition to its other types", UNS). Ruling: "Grist, the Hunger Tide can be your commander as its first
  ability works before the game begins during deck construction."
- Approach: compute the card's characteristics with its own abilities that function outside the game
  (layers on a hypothetical object in the command zone) before testing eligibility; support the static.
- Size: S

[G33] Ante: anteing a permanent and exchanging ownership
- Rules: CR 407.3 (ante cards may add or remove cards from the ante zone or change a card's owner), 407.4
  (to ante an object is to put it into the ante zone from whichever zone it's in; only its owner can),
  108.3 (owner).
- Engine now: ante.rs provides the pregame ante (407.2), `ANTE_TOP` (ante the top card of a library),
  `GAIN_OWNERSHIP` ("You own target card in the ante") and `EXCHANGE_WITH_TOP`; there is no effect that antes
  a permanent or other object from its current zone, and none that exchanges the ownership of two objects
  (no hits for "exchange ownership" in src/).
- Cards (legal nowhere, so `mtg-tools unsupported` skips them; implementer reports listed them unsupported and
  no test uses them): Jeweled Bird ("Ante this artifact. If you do, put all other cards you own from the ante
  into your graveyard"), Tempest Efreet, Timmerian Fiends, Bronze Tablet ("Exchange ownership of the revealed
  card and ~"), Rebirth, Amulet of Quoz ("may ante the top card of their library. If they don't, ...").
- Rulings: Jeweled Bird "The card is exchanged for your entire contribution to the ante".
- Approach: an `ante` move cause usable on any object (owner check per 407.4) and an "exchange ownership"
  effect for two objects (owners swap; controllers follow from the move instructions on the cards); add tests
  with `GameConfig { ante: true }` (tests/cr/r407_ante.rs has the harness).
- Size: S

[G32] Small rules-fidelity bugs
- Graveyard "cards" counts include tokens: `Value::GraveyardSize` and `PlayerFilter::GraveyardSize`
  (eval.rs:1017-1019, 249) count every object in the graveyard zone; a token there until the next
  state-based action check isn't a card (CR 108.2b, 111.7). Affects threshold-style "seven or more cards
  in your graveyard" checks made mid-resolution (e.g. "Sacrifice a creature token. Then if you have seven or
  more cards in your graveyard ..."). Fix: count only cards. Size S.
- Several spell copies created at once are stacked in a fixed order: `Effect::CopySpell` (resolve.rs:854-865)
  loops `copy_spell` per object and per count without letting the controller order them (CR 405.3; with new
  targets the order matters). Cards: Display of Power, Thousand-Year Storm (copies with different targets),
  Gogo, Mister Fantastic. Fix: create all copies, then ask for their relative order (the copy-for-each-target
  path already does). Size S.
- A meld trigger on a copy of a meld card does nothing: `MELD_PAIR_CONDITION` (merge.rs:806-809) is false
  unless a real meld pair resolves, but the instruction "exile them, then meld them" should still exile both
  and leave them in exile (CR 701.42b-c: objects that can't be melded stay where they are, i.e. exiled).
  Size S.

ESTIMATED SIZE: G26 M, G27 S, G33 S, G32 S (several small fixes).

OVERLAPS: Independent pieces; can be split across sessions if needed.

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
