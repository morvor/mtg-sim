//! CR 702.22j: when a creature is blocked by a creature with banding, the defending player
//! divides its combat damage among the blocking creatures. An answer that doesn't give an
//! amount for each blocker is illegal and gets the default assignment (lethal damage to
//! each blocker in order).

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_banding_division_with_too_few_amounts_gets_the_default() {
    cr!("702.22j");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let hero = t.battlefield(P1, "Benalish Hero");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // One amount for two blockers, though it adds up to the Giant's 3 damage.
    assign_damage(&mut t, P1, &[3]);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(hero, giant), (bears, giant)]);
    // The default: 1 to the 1/1 Hero and 2 to the 2/2 Bears.
    assert!(!t.on_battlefield(hero));
    assert!(!t.on_battlefield(bears));
}
