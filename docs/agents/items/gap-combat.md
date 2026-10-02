# gap-combat: Combat: who declares attackers, as-though-haste attacks and tapped blockers, per-creature attack targets, trample assignment order

GOAL: Combat declarations are hard-wired to the active player, as-though permissions for attacking and blocking don't exist, creatures moved onto the battlefield attacking share one attack target, and trample's lethal-damage check ignores later-declared attackers' assignments.

GAPS:

[G17] Combat declarations and as-though permissions
- Rules: CR 508.1 (active player declares attackers), 509.1; effects that make another player choose
  attackers (Master Warcraft); 508.4 (each creature put onto the battlefield attacking: its controller
  chooses what it attacks); 302.6 / 508.1c ("can attack as though it had haste"); 509.1a ("can block as
  though untapped"); 702.19b (lethal damage counts damage from *other creatures being assigned in the same
  step*, in whatever order the player assigns).
- Engine now:
  - `declare_attackers_step` always asks the active player (combat.rs:964-990); only blocks have a chooser
    override (`block_choice::block_decider`, combat.rs:1616-1618). Master Warcraft (UNS) needs "you choose
    which creatures attack", the active player then choosing what each attacks (ruling).
  - Attack legality checks `summoning_sick && !Haste` only (combat.rs:267); blockers must be untapped
    (`can_block_at_all`, combat.rs:337-345).
    No "as though it had haste" for attacking (Instill Energy, Frenzied Saddlebrute: UNS) or "tapped
    creatures can block as though untapped" (Masako the Humorless: UNS; ruling "allows tapped creatures to
    block only if they could otherwise block").
  - `move_to_destination` picks one attack target for all moved objects, chosen by the effect's controller
    (resolve.rs:2212-2216 -> `attack_target_for_new_attacker`, resolve.rs:2302-2306), not per object by
    each object's controller (tokens are per token, resolve.rs:2275-2296). No current card moves several
    nontoken cards attacking at once, but owner-control returns are affected.
  - Trample: attackers' assignments are made in declaration order and each sees only earlier ones
    (`combat_damage_step`, combat.rs:1818-1825; `kw::trample::lethal_damage` uses `pending`), so a
    trampler declared before another attacker blocked by the same creature can't count that damage.
- Rulings: Master Warcraft "the person who cast Master Warcraft first chooses the complete group of
  creatures that are going to attack. Then, for each of those creatures, the active player chooses who or
  what it's going to attack"; "You choose attackers ... regardless of whether it's your turn".
- Approach: an attack-chooser hook mirroring `block_decider`; a `Restriction`/static permission "may attack
  as though it had haste" and "may block while tapped" consulted in `attack_options`/`block_options`;
  per-object attack target in `move_to_destination`; assign trample damage after all non-trample
  assignments to shared blockers (or let the player order assignments).
- Size: M

ESTIMATED SIZE: G17 M.

OVERLAPS: The attack-side 'as though it had haste' should share its permission type with gap-activation's activation-side one.

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
