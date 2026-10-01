# gap-trigger-timing: Trigger detection at event time; resolution-time keyword values; draw-step draw tracking

GOAL: Triggered abilities are matched against the game state after a whole resolution instead of right after each event; keyword-derived trigger values are frozen too early; and the engine can't tell the first card a player draws in their draw step from later draws.

GAPS:

[G09] Trigger conditions are checked after the whole resolution, not immediately after each event
- Rules: CR 603.2 (an ability triggers when its trigger event occurs), 603.10 ("objects that exist
  immediately after an event are checked ... and continuous effects that exist at that time are used to
  determine ... what the objects involved in the event look like"; leaves-the-battlefield looks back),
  603.6a (enters triggers checked each time an event puts permanents onto the battlefield), 608.2c
  (later instructions of the same resolution happen after that check).
- Engine now: events are queued in `self.events` and trigger detection runs in `flush_events`
  (triggers.rs:124-161), which recomputes the *current* state first (triggers.rs:130-132). A resolving
  spell or ability flushes only at its end (stack.rs:802) or before creating a delayed trigger
  (resolve.rs:1522, 1566). So a later instruction of the same resolution changes whether earlier events
  match: an enters trigger with a power/type/controller condition sees the token after "Put two +1/+1
  counters on it"; `EnteredPermanent::as_entered` (game.rs:383-385) is likewise taken at flush time
  (reviewer note on wf_b2f642ec). The amass action works around it locally; living weapon, job select and
  For Mirrodin! needed their own "check triggers between creating and attaching" fixes.
- Cards: any "Create a token ... Put N +1/+1 counters on it"/"It gains ..." effect combined with
  conditional enters triggers (e.g. Garruk's Uprising "Whenever a creature with power 4 or greater enters",
  Elemental Bond, Guardian Project), "The token enters with N counters" built as create-then-put (reviewer
  note wf_a01a7158: anything checking the token's P/T as it enters sees a 0/0).
- Approach: snapshot what triggers need at event time — evaluate `detect_triggers` per event right after
  each primitive action (or record each event with the characteristics of the objects involved, as
  `as_entered` does, at emission rather than flush), keeping batch boundaries for "one or more" triggers.
  Then remove the local workarounds.
- Size: L

[G13] Keyword parameters that vary ("bushido X") are frozen when the trigger triggers
- Rules: CR 107.3c/608.2h (X defined by text is evaluated as the ability resolves), Fumiko ruling.
- Engine now: keyword-derived triggered abilities embed the keyword's current `n` as a constant
  (kw/bushido.rs: `Value::c(n)` in `derived`), and `AddKeywordX` (layers.rs:1060, 1630) evaluates X when
  characteristics are computed, so the bonus is fixed at trigger time.
- Cards: Fumiko the Lowblood (supported; wrong amount if attackers change before resolution).
- Rulings (UNCOVERED): Fumiko "The bushido bonus is calculated each time Fumiko's bushido trigger
  resolves, based on the number of attackers at that time."
- Approach: when a keyword's N comes from `AddKeywordX`, give the derived ability a `Value` that
  re-reads the keyword's value from the source on resolution (or carry the `Value` in `Keyword`).
- Size: S

[G30] "Except the first one they draw in each of their draw steps"
- Rules: CR 504.1 (draw step draw), 121.2; the cards' wording counts draws made during that player's draw
  step.
- Engine now: `Event::Drew { player, card, nth }` (events.rs:121-126) counts draws per turn; nothing records
  which draws happened in the player's current draw step, and the draw replacement events
  (`ReplacementEvent::Draw`, ability.rs:1730) can't be qualified by it.
- Cards (UNS, 10): Orcish Bowmasters, Notion Thief, Hullbreacher, Alhammarret's Archive, Teferi's Ageless
  Insight, Chains of Mephistopheles, Magus of the Chains, Xyris, the Writhing Storm, Leela, Sevateem
  Warrior, Bard, King of Dale.
- Rulings: Notion Thief "If two or more players each control a Notion Thief and a player would draw a card
  other than the first one in their draw step, that player chooses one of the applicable Notion Thief effects
  to apply"; "If an opponent is instructed to draw a card then discard a card, and Notion Thief causes you to
  draw a card instead, that opponent still discards a card".
- Approach: track draws per (player, step) in `TurnHistory`, expose a filter/condition "not the first card
  drawn in their draw step" usable by draw triggers and draw replacements.
- Size: S

ESTIMATED SIZE: G09 L, G13 S, G30 S.

OVERLAPS: gap-simultaneity changes event batching for 'each player' effects and gap-event-causes adds fields to `Event` variants; keep `flush_events`/batch-boundary changes compatible.

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
