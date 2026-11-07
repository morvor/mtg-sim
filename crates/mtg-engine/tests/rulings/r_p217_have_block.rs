//! Rulings batch P217 — "you may have target creature block [it / ~] this turn if able"
//! (a blocking requirement, CR 509.1c): the other cards the "have [subject] block"
//! causative compiles besides Turntimber Basilisk (r_p217_landfall.rs).

use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s05_common::enter;
use crate::r_s06_common::attach_new;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::legal_blocks;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn grappling_hook_makes_the_target_block_the_equipped_creature() {
    cr!("509.1c", "603.2");
    supported("Grappling Hook");
    // "Whenever equipped creature attacks, you may have target creature block it this
    // turn if able."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Grappling Hook", giant);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    attack_with(
        &mut t,
        &[(giant, Entity::Player(P1)), (other, Entity::Player(P1))],
    );
    assert_eq!(triggers_on_stack(&t, "equipped creature attacks"), 1);
    t.resolve_all();
    // The Bears must block the equipped Giant, not the other attacker.
    assert!(legal_blocks(&mut t, P1, &[(bears, giant)]));
    assert!(!legal_blocks(&mut t, P1, &[(bears, other)]));
    assert!(!legal_blocks(&mut t, P1, &[]));
}

#[test]
fn giant_ambush_beetle_makes_the_target_block_it() {
    cr!("509.1c", "603.6a");
    supported("Giant Ambush Beetle");
    // "When this creature enters, you may have target creature block it this turn if
    // able."
    for yes in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.answer_yes(P0, yes);
        let beetle = enter(&mut t, P0, "Giant Ambush Beetle");
        t.resolve_all();
        // (It entered this turn; let it attack anyway.)
        t.g.objects[beetle.0 as usize].summoning_sick = false;
        let other = t.battlefield(P0, "Hill Giant");
        to_beginning_of_combat(&mut t, P0);
        attack_with(
            &mut t,
            &[(beetle, Entity::Player(P1)), (other, Entity::Player(P1))],
        );
        assert!(legal_blocks(&mut t, P1, &[(bears, beetle)]));
        assert_eq!(legal_blocks(&mut t, P1, &[]), !yes);
        assert_eq!(legal_blocks(&mut t, P1, &[(bears, other)]), !yes);
    }
}
