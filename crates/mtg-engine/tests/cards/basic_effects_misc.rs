//! Looking at another player's library, losing keywords, sacrificing groups, and the
//! controller of a destroyed object.

use crate::basic_effects_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The library of `p`, top card first.
fn top(t: &TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    t.g.player(p).library.iter().rev().take(n).copied().collect()
}

#[test]
fn natural_selection_you_reorder_the_targets_library() {
    cr!("401.4");
    assert_supported("Natural Selection");
    let mut t = TestGame::new(2);
    let c = t.library_top(P1, "Forest");
    let b = t.library_top(P1, "Island");
    let a = t.library_top(P1, "Swamp");
    assert_eq!(top(&t, P1, 3), vec![a, b, c]);
    t.lands(P0, "Forest", 1);
    let ns = t.hand(P0, "Natural Selection");
    // P0 (not the library's owner) puts them back: Forest, Swamp, Island.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![2, 0, 1]));
    t.answer_yes(P0, false);
    t.cast(P0, ns).target(P1).go();
    t.resolve();
    assert_eq!(top(&t, P1, 3), vec![c, a, b]);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
}

#[test]
fn thoughtpicker_witch_exiles_one_of_the_top_two() {
    cr!("406.3");
    assert_supported("Thoughtpicker Witch");
    let mut t = TestGame::new(2);
    let second = t.library_top(P1, "Forest");
    let first = t.library_top(P1, "Island");
    let witch = t.battlefield(P0, "Thoughtpicker Witch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, witch, 0, &[Entity::Player(P1)]).unwrap();
    t.answer_choose(P0, &[Entity::Object(second)]);
    t.resolve();
    assert!(t.in_exile("Forest"));
    assert_eq!(top(&t, P1, 1), vec![first]);
}

#[test]
fn scarwood_hag_removes_only_forestwalk() {
    cr!("702.14a", "613.1f");
    assert_supported("Scarwood Hag");
    let mut t = TestGame::new(2);
    let hag = t.battlefield(P0, "Scarwood Hag");
    let walker = t.battlefield(P1, "Shanodin Dryads");
    t.recompute();
    assert!(t.obj_now(walker).has_keyword(mtg_engine::keywords::KeywordKind::Landwalk));
    t.activate(P0, hag, 1, &[Entity::Object(walker)]).unwrap();
    t.resolve();
    t.recompute();
    assert!(!t.obj_now(walker).has_keyword(mtg_engine::keywords::KeywordKind::Landwalk));
}

#[test]
fn canopy_dragon_gains_flying_and_loses_trample() {
    cr!("613.1f");
    assert_supported("Canopy Dragon");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Canopy Dragon");
    t.lands(P0, "Forest", 2);
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve();
    t.recompute();
    use mtg_engine::keywords::KeywordKind as K;
    assert!(t.obj_now(d).has_keyword(K::Flying));
    assert!(!t.obj_now(d).has_keyword(K::Trample));
}

#[test]
fn walking_sponge_removes_the_chosen_keyword() {
    cr!("613.1f");
    assert_supported("Walking Sponge");
    let mut t = TestGame::new(2);
    let sponge = t.battlefield(P0, "Walking Sponge");
    let d = t.battlefield(P1, "Canopy Dragon");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.activate(P0, sponge, 0, &[Entity::Object(d)]).unwrap();
    t.resolve();
    t.recompute();
    assert!(!t
        .obj_now(d)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
}

#[test]
fn sword_of_feast_and_famine_untaps_your_lands() {
    cr!("701.26b");
    assert_supported("Sword of Feast and Famine");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Sword of Feast and Famine");
    assert!(t.g.attach(sword, Entity::Object(bears)));
    let lands = t.lands(P0, "Forest", 2);
    for l in &lands {
        t.g.tap(*l);
    }
    t.hand(P1, "Island");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped));
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn arcum_dagsson_the_controller_sacrifices_it() {
    cr!("701.21a");
    assert_supported("Arcum Dagsson");
    let mut t = TestGame::new(2);
    let arcum = t.battlefield(P0, "Arcum Dagsson");
    let golem = t.battlefield(P1, "Ornithopter");
    t.answer_yes(P1, false);
    t.activate(P0, arcum, 0, &[Entity::Object(golem)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P1, "Ornithopter"));
}

#[test]
fn rakdos_the_defiler_sacrifices_half_rounded_up() {
    cr!("701.21a", "107.1a");
    assert_supported("Rakdos the Defiler");
    let mut t = TestGame::new(2);
    let rakdos = t.battlefield(P0, "Rakdos the Defiler");
    t.lands(P0, "Swamp", 3);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(rakdos, Entity::Player(P1))], &[]);
    // Three non-Demon permanents: two are sacrificed.
    assert_eq!(t.named_on_battlefield("Swamp").len(), 1);
}

#[test]
fn self_inflicted_wound_life_loss_only_if_a_creature_was_sacrificed() {
    cr!("118.12");
    assert_supported("Self-Inflicted Wound");
    for green in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, if green { "Grizzly Bears" } else { "Hill Giant" });
        t.lands(P0, "Swamp", 2);
        let s = t.hand(P0, "Self-Inflicted Wound");
        t.cast(P0, s).target(P1).go();
        t.resolve();
        assert_eq!(t.life(P1), if green { 18 } else { 20 }, "{green}");
    }
}

#[test]
fn kaervek_and_cinder_cloud_damage_the_creatures_controller() {
    cr!("700.4", "608.2h");
    assert_supported("Kaervek's Purge");
    assert_supported("Cinder Cloud");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 5);
    t.lands(P0, "Swamp", 1);
    let kp = t.hand(P0, "Kaervek's Purge");
    t.cast(P0, kp).x(4).target(giant).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);

    for white in [true, false] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, if white { "Savannah Lions" } else { "Grizzly Bears" });
        t.lands(P0, "Mountain", 5);
        let cc = t.hand(P0, "Cinder Cloud");
        t.cast(P0, cc).target(c).go();
        t.resolve();
        assert_eq!(t.life(P1), if white { 18 } else { 20 }, "{white}");
    }
}

#[test]
fn molten_rain_damages_only_for_a_nonbasic_land() {
    cr!("305.6");
    assert_supported("Molten Rain");
    for basic in [true, false] {
        let mut t = TestGame::new(2);
        let land = t.battlefield(P1, if basic { "Forest" } else { "Tropical Island" });
        t.lands(P0, "Mountain", 3);
        let mr = t.hand(P0, "Molten Rain");
        t.cast(P0, mr).target(land).go();
        t.resolve();
        assert!(!t.on_battlefield(land));
        assert_eq!(t.life(P1), if basic { 20 } else { 18 });
    }
}

#[test]
fn word_of_blasting_damages_the_walls_controller() {
    cr!("202.3");
    assert_supported("Word of Blasting");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Mountain", 2);
    let wb = t.hand(P0, "Word of Blasting");
    t.cast(P0, wb).target(wall).go();
    t.resolve();
    assert!(!t.on_battlefield(wall));
    // Wall of Stone's mana value is 3.
    assert_eq!(t.life(P1), 17);
}
