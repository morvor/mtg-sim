//! Rulings batch P049 — "drain" spells whose target is illegal as they try to resolve
//! (CR 608.2b): with their only target illegal they don't resolve and no life is gained;
//! with the damage prevented they still gain life (the damage was dealt as far as the
//! spell is concerned, CR 615.1); with some of several targets legal they do as much as
//! they can.

use crate::r_p049_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts `name` targeting P1's Grizzly Bears, which is returned to P1's hand in
/// response; the spell resolves (or rather doesn't) and the game is returned.
fn target_leaves(name: &str) -> (TestGame, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let spell = cast_new(&mut t, P0, name, &[obj(bear)]);
    bounce(&mut t, bear);
    t.resolve_all();
    assert!(!resolved(&t, spell), "{name} shouldn't have resolved");
    assert!(t.in_graveyard(P0, name));
    (t, spell)
}

#[test]
fn a_drain_spell_whose_creature_target_is_illegal_gains_no_life() {
    cr!("608.2b");
    ruling!(
        "Pharika's Cure",
        "If the creature is an illegal target as Pharika's Cure tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    assert_eq!(target_leaves("Pharika's Cure").0.life(P0), 20);
    ruling!(
        "Essence Extraction",
        "If the target creature becomes an illegal target, Essence Extraction doesn't resolve and you don't gain life."
    );
    assert_eq!(target_leaves("Essence Extraction").0.life(P0), 20);
    ruling!(
        "Winter's Intervention",
        "If the target creature is an illegal target as Winter's Intervention tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    assert_eq!(target_leaves("Winter's Intervention").0.life(P0), 20);
    ruling!(
        "Sorin's Thirst",
        "If the target creature is an illegal target by the time Sorin’s Thirst tries to resolve, the spell doesn’t resolve. You won’t gain 2 life."
    );
    assert_eq!(target_leaves("Sorin's Thirst").0.life(P0), 20);
    ruling!(
        "Last Kiss",
        "If the targeted creature is an illegal target by the time Last Kiss resolves, the spell doesn’t resolve. You won’t gain life."
    );
    assert_eq!(target_leaves("Last Kiss").0.life(P0), 20);
    // "Any target": the creature chosen leaves all the same.
    ruling!(
        "Agonizing Syphon",
        "If the chosen target becomes an illegal target for Agonizing Syphon, the spell won't resolve. You won't gain life."
    );
    assert_eq!(target_leaves("Agonizing Syphon").0.life(P0), 20);
    ruling!(
        "Warleader's Helix",
        "If the chosen target becomes an illegal target for Warleader’s Helix, the spell doesn’t resolve. You don’t gain life."
    );
    assert_eq!(target_leaves("Warleader's Helix").0.life(P0), 20);
}

#[test]
fn a_drain_spell_whose_damage_is_prevented_still_gains_life() {
    cr!("615.1", "608.2c");
    // Cho-Manno, Revolutionary: "Prevent all damage that would be dealt to Cho-Manno."
    supported("Cho-Manno, Revolutionary");
    let case = |name: &str| {
        supported(name);
        let mut t = TestGame::new(2);
        let cho = t.battlefield(P1, "Cho-Manno, Revolutionary");
        let spell = cast_new(&mut t, P0, name, &[obj(cho)]);
        t.resolve_all();
        assert!(resolved(&t, spell));
        assert!(t.on_battlefield(cho), "{name}: the damage wasn't prevented");
        assert_eq!(t.obj_now(cho).damage, 0);
        t.life(P0)
    };
    ruling!(
        "Pharika's Cure",
        "If, on the other hand, Pharika's Cure does resolve but the damage is prevented, you'll still gain life."
    );
    assert_eq!(case("Pharika's Cure"), 22);
    ruling!("Last Kiss", "You gain 2 life even if the damage is prevented.");
    assert_eq!(case("Last Kiss"), 22);
}

#[test]
fn essence_harvest_targets_only_the_player() {
    cr!("115.1", "608.2b");
    ruling!(
        "Essence Harvest",
        "Essence Harvest targets only the player. It doesn’t target any creatures. If that player is an illegal target when Essence Harvest tries to resolve, it won’t resolve and none of its effects will happen. No life will be lost or gained."
    );
    supported("Essence Harvest");
    supported("Leyline of Sanctity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel");
    t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    let spell = cast_new(&mut t, P0, "Essence Harvest", &[Entity::Player(P1)]);
    // The one target choice offered players only.
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0]
        .iter()
        .all(|e| matches!(e, Entity::Player(_))));
    // P1 gets hexproof in response: the only target is illegal.
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Without that, P1 loses 4 (Serra Angel's power) and P0 gains 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel");
    cast_new(&mut t, P0, "Essence Harvest", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (24, 16));
}

#[test]
fn witherbloom_command_does_as_much_as_it_can_with_a_legal_target_left() {
    cr!("608.2b", "700.2");
    ruling!(
        "Witherbloom Command",
        "If all of Witherbloom Command's targets are illegal as it tries to resolve, it will do nothing."
    );
    ruling!(
        "Witherbloom Command",
        "If at least one target is still legal as Witherbloom Command tries to resolve, it will resolve and do as much as it can."
    );
    supported("Witherbloom Command");
    supported("Leyline of Sanctity");
    // Modes: target player mills three (then you return a land card from your graveyard
    // to your hand), and target creature gets -3/-1. Both targets illegal: nothing.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let land = t.graveyard(P0, "Forest");
    lands_for_cost(&mut t, P0, "Witherbloom Command");
    let c = t.hand(P0, "Witherbloom Command");
    let spell = t
        .cast(P0, c)
        .modes(&[0, 2])
        .target(Entity::Player(P1))
        .target(bear)
        .go();
    bounce(&mut t, bear);
    t.battlefield(P1, "Leyline of Sanctity");
    let lib = t.library_size(P1);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(t.library_size(P1), lib);
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.zone(land), mtg_engine::object::Zone::Graveyard(P0));
    // The creature leaves but the player stays legal: the player mills three and the land
    // comes back.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    lands_for_cost(&mut t, P0, "Witherbloom Command");
    let c = t.hand(P0, "Witherbloom Command");
    let spell = t
        .cast(P0, c)
        .modes(&[0, 2])
        .target(Entity::Player(P1))
        .target(bear)
        .go();
    bounce(&mut t, bear);
    let lib = t.library_size(P1);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(t.library_size(P1), lib - 3);
    assert!(t.in_hand(P0, "Forest"));
}

#[test]
fn inspired_ultimatum_draws_five_unless_both_targets_are_illegal() {
    cr!("608.2b");
    ruling!(
        "Inspired Ultimatum",
        "If either target becomes illegal before Inspired Ultimatum resolves, the other will be affected as appropriate and you'll draw five cards. If both targets become illegal, the spell won't resolve and you won't draw five cards."
    );
    ruling!(
        "Inspired Ultimatum",
        "If either target becomes illegal before Inspired Ultimatum resolves, the other will be affected as appropriate and you’ll draw five cards. If both targets become illegal, the spell won’t resolve and you won’t draw five cards."
    );
    supported("Inspired Ultimatum");
    supported("Leyline of Sanctity");
    // P1 gains 5 life, 5 damage to P1's Serra Angel, then P0 draws five.
    let setup = || {
        let mut t = TestGame::new(2);
        let angel = t.battlefield(P1, "Serra Angel");
        let spell = cast_new(
            &mut t,
            P0,
            "Inspired Ultimatum",
            &[Entity::Player(P1), obj(angel)],
        );
        (t, angel, spell)
    };
    // The creature leaves: P1 still gains 5 and P0 draws five.
    let (mut t, angel, spell) = setup();
    bounce(&mut t, angel);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(t.life(P1), 25);
    assert_eq!(t.hand_size(P0), hand + 5);
    // P1 becomes an illegal target (hexproof): the Angel is still dealt 5 and P0 draws.
    let (mut t, angel, spell) = setup();
    t.battlefield(P1, "Leyline of Sanctity");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Serra Angel"));
    assert!(!t.on_battlefield(angel));
    assert_eq!(t.hand_size(P0), hand + 5);
    // Both illegal: nothing, no cards drawn.
    let (mut t, angel, spell) = setup();
    t.battlefield(P1, "Leyline of Sanctity");
    bounce(&mut t, angel);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), hand);
}
