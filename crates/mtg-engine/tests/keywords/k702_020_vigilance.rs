//! CR 702.20 Vigilance.

use crate::common_k702_011_017::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn attacking_doesnt_tap_a_creature_with_vigilance() {
    cr!("702.20", "702.20a", "702.20b");
    assert_supported("Serra Angel");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(angel, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    // As attackers are declared, only the creature without vigilance becomes tapped.
    assert!(t.g.is_attacking(angel));
    assert!(!t.obj_now(angel).tapped);
    assert!(t.obj_now(bears).tapped);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - 4 - 2);
    assert!(!t.obj_now(angel).tapped);
}

#[test]
fn vigilance_doesnt_let_a_tapped_creature_attack() {
    cr!("702.20b", "508.1a");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    t.g.tap(angel);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!t.g.can_attack(angel));
    t.g.untap(angel);
    assert!(t.g.can_attack(angel));
}

#[test]
fn gaining_vigilance_after_attacking_doesnt_untap() {
    cr!("702.20a", "702.20b");
    ruling!(
        "Radiant Destiny",
        "Gaining vigilance any time after the moment you choose to attack with a creature won't cause that creature to become untapped"
    );
    assert_supported("Sunmane Pegasus");
    let mut t = TestGame::new(2);
    let pegasus = t.battlefield(P0, "Sunmane Pegasus");
    attack_with(&mut t, &[(pegasus, Entity::Player(P1))]);
    assert!(t.obj_now(pegasus).tapped);
    // "{1}{W}: This creature gains vigilance and lifelink until end of turn."
    t.lands(P0, "Plains", 2);
    t.activate(P0, pegasus, 0, &[]).unwrap();
    t.resolve();
    assert!(t.obj_now(pegasus).has_keyword(KeywordKind::Vigilance));
    assert!(t.obj_now(pegasus).tapped);
    block_and_finish(&mut t, P1, &[]);
    assert!(t.obj_now(pegasus).tapped);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn losing_vigilance_after_attacking_doesnt_tap() {
    cr!("702.20a", "702.20b");
    ruling!(
        "Kiyomaro, First to Stand",
        "Losing vigilance after attackers are declared doesn't cause Kiyomaro to tap."
    );
    ruling!(
        "Radiant Destiny",
        "losing vigilance after that time won't cause it to become tapped"
    );
    assert_supported("Kiyomaro, First to Stand");
    let mut t = TestGame::new(2);
    let kiyomaro = t.battlefield(P0, "Kiyomaro, First to Stand");
    let growth = t.hand(P0, "Giant Growth");
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    t.g.recompute();
    assert!(t.obj_now(kiyomaro).has_keyword(KeywordKind::Vigilance));
    attack_with(&mut t, &[(kiyomaro, Entity::Player(P1))]);
    assert!(!t.obj_now(kiyomaro).tapped);
    // Casting Giant Growth leaves three cards in hand: Kiyomaro loses vigilance.
    t.lands(P0, "Forest", 1);
    t.cast(P0, growth).target(kiyomaro).go();
    t.resolve();
    assert!(!t.obj_now(kiyomaro).has_keyword(KeywordKind::Vigilance));
    assert!(!t.obj_now(kiyomaro).tapped);
    assert!(t.g.is_attacking(kiyomaro));
    block_and_finish(&mut t, P1, &[]);
    assert!(!t.obj_now(kiyomaro).tapped);
    assert_eq!(t.life(P1), 20 - (3 + 3));
}

#[test]
fn multiple_instances_of_vigilance_are_redundant() {
    cr!("702.20c");
    ruling!(
        "Sunmane Pegasus",
        "Multiple instances of vigilance or lifelink on the same creature are redundant."
    );
    let mut t = TestGame::new(2);
    let pegasus = t.battlefield(P0, "Sunmane Pegasus");
    t.lands(P0, "Plains", 4);
    t.activate(P0, pegasus, 0, &[]).unwrap();
    t.resolve();
    t.activate(P0, pegasus, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(keyword_count(&t, pegasus, KeywordKind::Vigilance), 2);
    assert_eq!(keyword_count(&t, pegasus, KeywordKind::Lifelink), 2);
    let angel = t.battlefield(P0, "Serra Angel");
    t.battlefield(P0, "Always Watching");
    t.g.recompute();
    assert_eq!(keyword_count(&t, angel, KeywordKind::Vigilance), 2);
    attack_with(
        &mut t,
        &[(pegasus, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    assert!(!t.obj_now(pegasus).tapped);
    assert!(!t.obj_now(angel).tapped);
    block_and_finish(&mut t, P1, &[]);
    // Two instances of lifelink gain life once for the damage (3 from the pumped Pegasus).
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 20 - 3 - 5);
}
