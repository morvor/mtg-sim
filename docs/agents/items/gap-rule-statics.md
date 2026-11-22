# gap-rule-statics: Missing rule-modifying statics (can't be copied, counters persist, damage not removed, sacrifice/payment restrictions, face-up restrictions, turns taken, as-though infect)

GOAL: Eight small rule-modifying effects with no engine representation. Each is a focused hook plus a restriction/static kind and an oracle pattern.

GAPS:

[G14] Rule-modifying statics the engine has no representation for
- Rules / engine now (none of these exist anywhere outside oracle/, checked by grep):
  - "[This] can't be copied" (CR 113.6g, 707.10): `copy::copy_spell` (copy.rs) never checks it.
    Cards (UNS): See Double, Display of Power, Choreographed Sparks, Gogo, Master of Mimicry, Ulalek.
  - "Counters remain on ~ as it moves to any zone other than a player's hand or library" (exception to
    CR 122.2 / 400.7): Skullbriar, the Walking Grave, Me, the Immortal. Rulings: Skullbriar "The counters
    that remain ... aren't 'placed'", "Counters that adjust power and/or toughness affect Skullbriar's power
    and/or toughness in zones other than the battlefield".
  - "Damage isn't removed from ~ during cleanup steps" (modifies CR 514.2): Ancient Adamantoise, Uthgardt
    Fury, Switchgrass Grazer, Patient Zero, Victory of the Pyrohammer, Case of the Market Melee. Ruling:
    Ancient Adamantoise "Effects that remove all damage from a permanent (such as regeneration) will still
    remove damage".
  - "Spells and abilities your opponents control can't cause you to sacrifice permanents" (CR 701.21):
    `Restriction::CantBeSacrificed(Filter)` (ability.rs:2019, actions.rs:1168) has no notion of what
    causes the sacrifice. Cards: Sigarda, Host of Herons, Tajuru Preserver, The Master, Multiplied. Rulings:
    Sigarda "if it would force you to sacrifice a permanent, you just don't", "It won't stop ... the legend
    rule", "you can't choose to sacrifice a permanent" for "unless you sacrifice".
  - "can't be turned face up" restrictions (CR 708, 702.37e): only merged permanents are checked
    (merge.rs:217, facedown.rs:271). Cards: Unable to Scream, Karlov Watchdog (ruling: "opponents can't
    attempt to turn face-down creatures face up either by paying a disguise or morph cost or by paying the
    mana cost of a cloaked or manifested creature").
  - Turns a player has taken ("your first, second, or third turns of the game"): `Player` has no turn
    counter (only `turn.number`, game-wide). Cards: Serra Avenger, Jace Reawakened, Spider-Man 2099, The
    Keeper of the Yellow Hat, Captivating Crossroads. Rulings: Serra Avenger "cares about how many turns
    you have taken, not ... how many turns the game has had", "If the game is restarted ... you can't cast".
  - "Players can't pay life [or sacrifice X] to cast spells or activate abilities [that aren't mana
    abilities]" (CR 118.3, 119.4): `can_pay_life` (actions.rs:1386-1388) only checks "can't lose life".
    Cards (UNS): Karn's Sylex, Yasharn, Implacable Earth, Angel of Jubilation.
  - "All damage is dealt to you as though its source had infect" (CR 702.90, 120.3b): only the wither
    variant exists (kw/wither.rs:16); damage to players checks the source's own infect (actions.rs:1547-1556).
    Card (UNS): Phyrexian Unlife.
- Approach: one small hook each — a `Restriction::CantBeCopied(Filter)` checked in `copy_spell` and token
  copy creation; a zone-change hook keeping counters (counters not "put"); a cleanup-step check in
  turn.rs; a `cause_controller` on sacrifice (pass the resolving object's controller to
  `Game::sacrifice`); `Restriction::CantTurnFaceUp(Filter, PlayerRel)` checked by facedown/morph/disguise
  turn-up paths; `Player::turns_taken` incremented in turn begin and a `Value`; a cost-payment restriction checked in
  `can_pay_cost`/`pay_cost_part` for life and sacrifices; an "as though infect" player modification.
- Size: M (eight S pieces)

ESTIMATED SIZE: G14 M (eight S pieces).

OVERLAPS: The cost-payment restriction overlaps gap-activation/gap-mana-payment payment code; the face-up restriction touches facedown.rs and kw/morph*.rs.

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
