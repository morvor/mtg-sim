//! CR 702.31 Horsemanship.

use crate::common_k702_011_017::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn horsemanship_is_an_evasion_ability() {
    cr!("702.31", "702.31a");
    assert!(KeywordKind::Horsemanship.is_evasion());
    assert_supported("Wu Light Cavalry");
    let mut t = TestGame::new(2);
    let cavalry = t.battlefield(P0, "Wu Light Cavalry");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(cavalry, Entity::Player(P1))]);
    assert!(!t.g.can_block(bears, cavalry));
    block_and_finish(&mut t, P1, &[(bears, cavalry)]);
    assert!(!is_blocking(&t, bears));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn only_creatures_with_horsemanship_can_block_it_but_it_can_block_anything() {
    cr!("702.31b");
    assert_supported("Wei Scout");
    // A creature with horsemanship can be blocked by one with horsemanship.
    let mut t = TestGame::new(2);
    let cavalry = t.battlefield(P0, "Wu Light Cavalry");
    let scout = t.battlefield(P1, "Wei Scout");
    attack_with(&mut t, &[(cavalry, Entity::Player(P1))]);
    assert!(t.g.can_block(scout, cavalry));
    block_and_finish(&mut t, P1, &[(scout, cavalry)]);
    assert_eq!(t.life(P1), 20);
    // A creature with horsemanship can block a creature without it (unlike shadow).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let scout = t.battlefield(P1, "Wei Scout");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.g.can_block(scout, bears));
    block_and_finish(&mut t, P1, &[(scout, bears)]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Wei Scout"));
}

#[test]
fn horsemanship_doesnt_interact_with_flying_or_reach() {
    cr!("702.31b");
    ruling!(
        "Wu Light Cavalry",
        "Despite the similarities between horsemanship and flying, horsemanship doesn’t interact with flying or reach."
    );
    assert_supported("Serra Angel");
    assert_supported("Giant Spider");
    // Flying or reach doesn't let a creature block one with horsemanship.
    let mut t = TestGame::new(2);
    let cavalry = t.battlefield(P0, "Wu Light Cavalry");
    let angel = t.battlefield(P1, "Serra Angel");
    let spider = t.battlefield(P1, "Giant Spider");
    attack_with(&mut t, &[(cavalry, Entity::Player(P1))]);
    assert!(!t.g.can_block(angel, cavalry));
    assert!(!t.g.can_block(spider, cavalry));
    // Horsemanship doesn't let a creature block one with flying.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let scout = t.battlefield(P1, "Wei Scout");
    attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    assert!(!t.g.can_block(scout, angel));
}

#[test]
fn multiple_instances_of_horsemanship_are_redundant() {
    cr!("702.31c");
    let mut t = TestGame::new(2);
    let rider = t.custom(
        P0,
        custom_card(
            "Twice-Mounted Rider",
            "Creature — Human Soldier",
            Some((2, 2)),
            "Horsemanship\nHorsemanship",
        ),
        Zone::Battlefield,
    );
    assert_eq!(keyword_count(&t, rider, KeywordKind::Horsemanship), 2);
    let scout = t.battlefield(P1, "Wei Scout");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(rider, Entity::Player(P1))]);
    assert!(t.g.can_block(scout, rider));
    assert!(!t.g.can_block(bears, rider));
    block_and_finish(&mut t, P1, &[(scout, rider)]);
    assert_eq!(t.life(P1), 20);
}
