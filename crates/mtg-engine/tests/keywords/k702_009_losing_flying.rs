//! CR 702.9 Flying: effects that make creatures lose it, together with other changes in
//! the same effect ("get -2/-2 and lose flying until end of turn").

use crate::common_k702_011_017::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn attacking_flyers_get_minus_two_and_lose_flying_until_end_of_turn() {
    cr!("702.9b");
    ruling!(
        "Wind Shear",
        "The -2/-2 and loss of Flying both last until end of turn. The -2/-2 is not permanent."
    );
    assert_supported("Wind Shear");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(angel, Entity::Player(P0)), (bears, Entity::Player(P0))],
    );
    assert!(!t.g.can_block(giant, angel));
    // "Attacking creatures with flying get -2/-2 and lose flying until end of turn."
    t.lands(P0, "Forest", 3);
    let shear = t.hand(P0, "Wind Shear");
    t.cast(P0, shear).go();
    t.resolve_all();
    assert_eq!(t.pt(angel), (2, 2));
    assert!(!t.obj_now(angel).has_keyword(KeywordKind::Flying));
    // The attacking creature without flying isn't affected.
    assert_eq!(t.pt(bears), (2, 2));
    // Without flying, a creature without flying or reach can block it.
    assert!(t.g.can_block(giant, angel));
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 16);
    // Both parts end at end of turn.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.pt(angel), (4, 4));
    assert!(t.obj_now(angel).has_keyword(KeywordKind::Flying));
}

#[test]
fn a_creature_that_gets_a_bonus_and_loses_flying() {
    cr!("702.9b");
    assert_supported("Leering Gargoyle");
    let mut t = TestGame::new(2);
    // "{T}: This creature gets -2/+2 and loses flying until end of turn."
    let gargoyle = t.battlefield(P0, "Leering Gargoyle");
    assert_eq!(t.pt(gargoyle), (2, 2));
    t.activate(P0, gargoyle, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(gargoyle), (0, 4));
    assert!(!t.obj_now(gargoyle).has_keyword(KeywordKind::Flying));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(gargoyle), (2, 2));
    assert!(t.obj_now(gargoyle).has_keyword(KeywordKind::Flying));
}
