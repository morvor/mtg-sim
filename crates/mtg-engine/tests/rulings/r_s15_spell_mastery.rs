//! Rulings batch S15 — spell mastery (an ability word, CR 207.2c): Fiery Impulse,
//! Exquisite Firecraft.

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s07_common::damage_on;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `n` instant cards (Lightning Bolt) into `p`'s graveyard.
fn instants_in_graveyard(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.graveyard(p, "Lightning Bolt");
    }
}

#[test]
fn spell_mastery_counts_the_graveyard_as_the_spell_resolves_without_the_spell() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Fiery Impulse",
        "Check to see if there are two or more instant and/or sorcery cards in your graveyard as the spell resolves to determine whether the spell mastery ability applies. The spell itself won’t count because it’s still on the stack as you make this check."
    );
    supported("Fiery Impulse");
    // "Fiery Impulse deals 2 damage to target creature. Spell mastery — If there are two or
    // more instant and/or sorcery cards in your graveyard, Fiery Impulse deals 3 damage
    // instead." With one instant card in the graveyard (the spell itself doesn't count):
    // 2 damage.
    let mut t = TestGame::new(2);
    instants_in_graveyard(&mut t, P0, 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let fi = in_hand_with_mana(&mut t, P0, "Fiery Impulse");
    t.cast(P0, fi).target(giant).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(damage_on(&t, giant), 2);
    // With two: 3 damage.
    let mut t = TestGame::new(2);
    instants_in_graveyard(&mut t, P0, 2);
    let giant = t.battlefield(P1, "Hill Giant");
    let fi = in_hand_with_mana(&mut t, P0, "Fiery Impulse");
    t.cast(P0, fi).target(giant).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // One when it's cast, a second one by the time it resolves: 3 damage.
    let mut t = TestGame::new(2);
    instants_in_graveyard(&mut t, P0, 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let fi = in_hand_with_mana(&mut t, P0, "Fiery Impulse");
    t.cast(P0, fi).target(giant).go();
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 2);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn exquisite_firecraft_cant_be_countered_only_with_two_other_instants_or_sorceries() {
    cr!("101.2", "701.6a");
    ruling!(
        "Exquisite Firecraft",
        "Check to see if there are two or more instant and/or sorcery cards in your graveyard as the spell resolves to determine whether the spell mastery ability applies. The spell itself won't count because it's still on the stack as you make this check."
    );
    supported("Exquisite Firecraft");
    supported("Cancel");
    // "Exquisite Firecraft deals 4 damage to any target. Spell mastery — If there are two
    // or more instant and/or sorcery cards in your graveyard, this spell can't be
    // countered." P1 counters it with Cancel.
    for (before, added) in [(1, false), (2, false), (1, true)] {
        let mut t = TestGame::new(2);
        instants_in_graveyard(&mut t, P0, before);
        let firecraft = in_hand_with_mana(&mut t, P0, "Exquisite Firecraft");
        let spell = t.cast(P0, firecraft).target(P1).go();
        let cancel = in_hand_with_mana(&mut t, P1, "Cancel");
        t.g.turn.priority = Some(P1);
        t.cast(P1, cancel).target(spell).go();
        if added {
            // A second instant card reaches P0's graveyard before Cancel resolves.
            let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
            t.g.turn.priority = Some(P0);
            t.cast(P0, bolt).target(P1).go();
            t.resolve();
        }
        let mastery = before >= 2 || added;
        t.resolve_all();
        let bolted = if added { 3 } else { 0 };
        assert_eq!(
            t.life(P1),
            20 - bolted - if mastery { 4 } else { 0 },
            "{before} {added}"
        );
    }
}
