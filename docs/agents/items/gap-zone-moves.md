# gap-zone-moves: Zone changes: Auras attached to cards in graveyards, full 'instead' destinations, fabricate fallback

GOAL: Three zone-change gaps: Auras can only be attached to objects on the battlefield (Animate Dead family), 'instead' replacements keep only the destination zone and library position, and fabricate doesn't fall back to Servos when counters are prevented.

GAPS:

[G28] Auras attached to (or targeting) cards outside the battlefield
- Rules: CR 303.4a (an Aura spell's target is defined by its enchant ability, e.g. "Enchant creature card in
  a graveyard"), 303.4b-c, 702.5a, 704.5m; 607.2c / 607.1 ("enchant creature put onto the battlefield with
  ~").
- Engine now: `aura_target_spec` (attach.rs:74-84) lets the Aura spell target a card in a graveyard, but
  `legal_attachment_as` (attach.rs:107-140) returns false for any object not on the battlefield
  (attach.rs:138), so such an Aura can't enter attached and would be put into the graveyard by 704.5m
  before its enters trigger resolves. There's also no "loses enchant A and gains enchant B" handling of
  the enchant keyword's filter beyond generic keyword add/remove, nor a linked "put onto the battlefield
  with ~" enchant quality.
- Cards (UNS): Animate Dead, Dance of the Dead, Necromancy ("becomes an Aura with 'enchant creature put onto
  the battlefield with ~'"), Spellweaver Volute ("Enchant instant card in a graveyard").
- Rulings: Animate Dead "You target a creature card in a graveyard when you cast it. It enters the battlefield
  attached to that card. Then it returns that card to the battlefield, and attaches itself to the card
  again", "If the creature put onto the battlefield has protection from black ... Animate Dead won't be able
  to attach to it. It will be put into the graveyard as a state-based action", "A creature card with shroud
  may be targeted by Animate Dead".
- Approach: allow attachment to objects in the zone the enchant filter names (graveyard), keep
  `attached_to` across that zone, have 704.5m use the enchant filter's zone; support swapping the enchant
  keyword in layer 6 and a linked enchant quality (607.2c).
- Size: M

[G23] "Instead" zone-change replacements drop all destination details
- Rules: CR 614.1a, 614.6 (the modified event happens instead), 701.6 (counter destinations: Hinder,
  Desertion, Delay), 702.62 (suspend granted in exile), 111/708 (face down), 110.2 (under whose control).
- Engine now: `ReplacementAction::MoveInstead(dest)` (replacement.rs:1158-1181) keeps only `dest.zone`
  and `dest.position`; `Destination`'s `tapped`, `controller`, `face_down`, `with_counters`, `attacking`,
  `transformed`, `with_mods`, `attached_to` (ability.rs:504-529) are ignored, and a choice between two
  positions ("top or bottom") can't be expressed.
- Cards (UNS): Delay, Gandalf of the Secret Fire ("exile it with three time counters ... If it doesn't have
  suspend, it gains suspend"; S29 #37: about 12-17 cards share the phrase), Hinder ("your choice of the top
  or bottom"), Desertion ("onto the battlefield under your control").
- Rulings: Delay "If the target spell was cast with flashback, Delay's effect will exile it, not the
  flashback effect. The card will get time counters and gain suspend"; Desertion "The card is put onto the
  battlefield ... not ... cast from your hand"; Hinder "Hinder's controller, not necessarily the controller
  of the countered spell, chooses where the countered spell goes".
- Approach: build the replacement `MoveEv` from the whole `Destination` (as `move_to_destination` does,
  resolve.rs:2195-2240: EtbInfo tapped/controller/counters/face_down/attacking), add a
  `Destination::choice` of positions asked of the replacement's controller, and a "gains suspend" effect for
  the exiled card (a keyword grant that follows the card in exile, CR 400.7 exception like 400.7g).
- Size: M

[G24] Fabricate ignores counters being prevented
- Rules: CR 702.123a + Angel of Invention ruling ("If you can't put +1/+1 counters on the creature for any
  reason as fabricate resolves ..., you just create Servo tokens").
- Engine now: kw/fabricate.rs `derived` (lines 41-70) asks "may put counters" whenever the source is on the
  battlefield and creates Servos only if the player declined (`Not(PrevHappened)`); if a replacement
  effect prevents the counters ("can't have counters put on it", Solemnity), the player gets neither.
- Approach: decide "can't put" by asking the replacement pipeline whether an `AddCounters` event would be
  prevented (as kw/riot.rs `counters_prevented` already does), and treat "put 0" as not putting.
- Size: S

ESTIMATED SIZE: G28 M, G23 M, G24 S.

OVERLAPS: G28 needs enchant-keyword changes in layer 6 (gap-ability-grants edits layer 6 too).

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
