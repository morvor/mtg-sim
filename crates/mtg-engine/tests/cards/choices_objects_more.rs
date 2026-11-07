//! Objects chosen by several players or for each value
//! (`oracle/patterns/choice_grammar_objects_more.rs`).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn wicked_guardian_damage_another_creature_to_draw() {
    cr!("603.5");
    ruling!(
        "Wicked Guardian",
        "you can't have it deal 2 damage multiple times or to multiple creatures"
    );
    compiles("Wicked Guardian");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Indomitable Ancients");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let hand = t.hand_size(P0);
    t.enter(P0, "Wicked Guardian");
    t.resolve_all();
    assert_eq!(t.g.obj(giant).damage, 2);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn wicked_guardian_declined_no_card() {
    cr!("603.5");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Indomitable Ancients");
    t.answer_yes(P0, false);
    let hand = t.hand_size(P0);
    t.enter(P0, "Wicked Guardian");
    t.resolve_all();
    assert_eq!(t.g.obj(giant).damage, 0);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn wormfang_crab_an_opponent_chooses_what_is_exiled_until_it_leaves() {
    cr!("607.2a", "601.7a");
    compiles("Wormfang Crab");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.answer_choose(P1, &[Entity::Object(lions)]);
    let crab = t.enter(P0, "Wormfang Crab");
    t.resolve_all();
    assert!(t.on_battlefield(bears) && !t.on_battlefield(lions));
    assert!(t.in_exile("Savannah Lions"));
    // P1 chose; the Crab wasn't a candidate.
    let cands = t.asked().into_iter().find_map(|(p, d)| match d {
        Decision::ChooseEntities { candidates, .. } if p == P1 => Some(candidates),
        _ => None,
    });
    assert!(cands.is_some_and(|c| !c.contains(&Entity::Object(crab))));
    t.g.destroy(crab, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Savannah Lions").len(), 1, "returned");
}

#[test]
fn fortunate_few_each_player_saves_a_different_permanent_they_dont_control() {
    cr!("101.4", "101.4b");
    ruling!(
        "Fortunate Few",
        "Lands can’t be chosen and won’t be destroyed"
    );
    compiles("Fortunate Few");
    let mut t = TestGame::new(3);
    t.lands(P0, "Plains", 5);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let p1 = t.battlefield(P1, "Savannah Lions");
    let p2 = t.battlefield(P2, "Hill Giant");
    let other = t.battlefield(P2, "Craw Wurm");
    let land = t.battlefield(P1, "Forest");
    let spell = t.hand(P0, "Fortunate Few");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(p2)]);
    t.answer_choose(P1, &[Entity::Object(mine)]);
    t.answer_choose(P2, &[Entity::Object(p1)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(mine) && t.on_battlefield(p1) && t.on_battlefield(p2));
    assert!(!t.on_battlefield(other));
    assert!(t.on_battlefield(land));
    // P2 couldn't choose what P0 chose.
    let p2_cands = t.asked().into_iter().find_map(|(p, d)| match d {
        Decision::ChooseEntities { candidates, .. } if p == P2 => Some(candidates),
        _ => None,
    });
    assert!(p2_cands.is_some_and(|c| !c.contains(&Entity::Object(p2))));
}

#[test]
fn sculpted_sunburst_opponents_keep_a_creature_no_more_powerful_than_yours() {
    cr!("101.4");
    ruling!(
        "Sculpted Sunburst",
        "If you do not choose a creature, most likely because you don't control any, no creatures will be exiled"
    );
    compiles("Sculpted Sunburst");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let mine = t.battlefield(P0, "Hill Giant"); // 3
    let mine2 = t.battlefield(P0, "Grizzly Bears");
    let big = t.battlefield(P1, "Craw Wurm"); // 6
    let small = t.battlefield(P1, "Savannah Lions"); // 2
    let spell = t.hand(P0, "Sculpted Sunburst");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.answer_choose(P1, &[Entity::Object(small)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(mine) && t.on_battlefield(small));
    assert!(!t.on_battlefield(big) && !t.on_battlefield(mine2));
    let p1_cands = t.asked().into_iter().find_map(|(p, d)| match d {
        Decision::ChooseEntities { candidates, .. } if p == P1 => Some(candidates),
        _ => None,
    });
    assert_eq!(p1_cands, Some(vec![Entity::Object(small)]), "power 3 or less");
}

#[test]
fn sculpted_sunburst_without_your_creature_exiles_nothing() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let big = t.battlefield(P1, "Craw Wurm");
    let spell = t.hand(P0, "Sculpted Sunburst");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(big));
}

#[test]
fn celestial_judgment_one_creature_survives_for_each_power() {
    cr!("608.2d");
    ruling!(
        "Celestial Judgment",
        "All creatures that aren't chosen are destroyed simultaneously"
    );
    compiles("Celestial Judgment");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let a = t.battlefield(P0, "Grizzly Bears"); // 2
    let b = t.battlefield(P1, "Grizzly Bears"); // 2
    let c = t.battlefield(P1, "Hill Giant"); // 3
    let spell = t.hand(P0, "Celestial Judgment");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(a) && t.on_battlefield(c));
    assert!(!t.on_battlefield(b));
}

#[test]
fn rite_of_ruin_sacrifices_one_two_three_in_the_chosen_order() {
    cr!("101.4");
    ruling!(
        "Rite of Ruin",
        "First you choose the order for the card types"
    );
    compiles("Rite of Ruin");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, "Forest");
        t.battlefield(P1, "Mind Stone");
    }
    let spell = t.hand(P0, "Rite of Ruin");
    t.advance_to(P0, Step::PrecombatMain);
    // Order: creatures, lands, artifacts.
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // creature (of artifact, creature, land)
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // land (of artifact, land)
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2, "one creature");
    assert_eq!(t.named_on_battlefield("Forest").len(), 1, "two lands");
    assert_eq!(t.named_on_battlefield("Mind Stone").len(), 0, "three artifacts");
    assert_eq!(t.named_on_battlefield("Mountain").len(), 5, "P0 sacrifices two lands too");
}
