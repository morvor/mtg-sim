# gap-event-causes: Event attribution: who put counters (and whether as a cost), what destroyed or countered something

GOAL: Several events don't carry who or what caused them, which blocks 'whenever you put counters', tribute's putter, 'If an effect would put counters' (Doubling Season) vs costs, Karmic Justice-style 'a spell or ability an opponent controls destroys', trap conditions, and umbra armor's destruction credit.

GAPS:

[G08] Who put counters, and counters put as a cost vs. by an effect
- Rules: CR 122.6, 122.6a (the player who puts counters: the controller of the spell/ability, or the player
  the effect names, e.g. tribute's opponent); 118.3 / 602.2 (counters put as a cost aren't put by an
  effect: Doubling Season "loyalty ... put on as a cost, not as an effect").
- Engine now: `Event::CountersAdded { target, kind, n }` (events.rs:140-144) records no player and no
  source; `TriggerCond::CountersPut { filter, kind }` (ability.rs:2404) can't say "you put";
  `TurnHistory::counters_put` is a bare count (game.rs:431). `counter_rules::who_puts_counters`
  (counter_rules.rs:314-320) infers the putter from the source's controller, which is wrong for tribute
  (the chosen opponent puts them, kw/tribute.rs) and has no notion of a cost. Loyalty and "put a counter
  on ~" costs go through the same `add_counters` as effects (casting.rs:2595-2598, 2761-2764), so an
  "If an effect would put one or more counters" replacement can't exclude costs.
- Cards (UNS): All Will Be One, Stocking the Pantry and other "Whenever you put one or more counters on"
  triggers; Doubling Season and Selesnya Loft Gardens ("If an effect would put one or more counters ...";
  Doubling Season's counter half is UNS) need the cost/effect split, while "If you would put" replacements
  (Vorinclex, Monstrous Raider: supported) correctly apply to costs too.
- Rulings: tribute (11 cards, e.g. Nessian Wilds Ravager) "For effects that check which player put counters
  on the entering creature, the player chosen to pay tribute puts those counters on it" (UNCOVERED); Doubling
  Season "if you activate an ability whose cost has you put loyalty counters on a planeswalker, the number you
  put on isn't doubled"; All Will Be One "toxic ... those counters are placed as a single event, and the
  ability triggers one time".
- Approach: add `by: Option<PlayerId>` and `as_cost: bool` to `ReplEvent::AddCounters`/`Event::CountersAdded`
  (and to enters-with-counters), set by every caller (costs pass `as_cost`, tribute passes the opponent);
  add a `by: PlayerRel` to `TriggerCond::CountersPut` and `ReplacementEvent::PutCounters`; an
  `effect_only` flag for "If an effect would put".
- Size: M

[G16] Events don't record what caused a destruction or a counter
- Rules: CR 701.8 (destroy), 701.6 (counter); trigger and condition wording "destroyed/countered by a
  spell or ability an opponent controlled" (Cobra Trap ruling: "A spell or ability destroys a permanent
  only if that spell or ability specifically contains the word 'destroy'"), umbra armor (702.89a: the
  spell or ability that tried to destroy the creature is what destroys the Aura).
- Engine now: `Event::Destroyed { obj }` (events.rs:244-246) and `Event::Countered { what }`
  (events.rs:89-91) carry no cause; `ReplEvent::Destroy` has a `source` (replacement.rs:129-132) but it's
  dropped from the event. `ZoneChange.by` is a player only. Umbra armor's replacement runs
  `Effect::Destroy { what: Sel::This }` in the Aura's own context (kw/umbra_armor.rs:34-42), so the Aura is
  destroyed by itself rather than by the original spell or ability.
- Cards (UNS): Karmic Justice ("Whenever a spell or ability an opponent controls destroys a noncreature
  permanent you control"), Cobra Trap, Summoning Trap (history conditions on that cause).
- Rulings: Karmic Justice "Spells and abilities that cause you to sacrifice permanents will not cause
  Karmic Justice's ability to trigger"; Cobra Trap "destroys a permanent only if that spell or ability
  specifically contains the word 'destroy'"; Hyena Umbra "that spell or ability is what causes the Aura to
  be destroyed instead ... if a spell or ability deals lethal damage ..., the game rules regarding lethal
  damage cause the Aura to be destroyed".
- Approach: add `cause: Option<ObjectId>` (the resolving spell/ability, None for SBAs) to `Destroyed` and
  `Countered`, record it in `TurnHistory`, add trigger/condition forms keyed on its controller, and make
  `ReplacementAction::Instead` effects inherit the replaced event's cause.
- Size: S-M

ESTIMATED SIZE: G08 M, G16 S-M.

OVERLAPS: Adds fields to `Event`/`ReplEvent` (counters, destroy, counter) that gap-trigger-timing's per-event detection consumes.

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
