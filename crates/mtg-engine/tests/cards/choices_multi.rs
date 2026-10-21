//! Choices among listed values (`oracle/patterns/choice_grammar_multi.rs`): "gains your
//! choice of A, B, or C".

use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
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
fn argivian_avenger_gains_the_chosen_ability_until_end_of_turn() {
    cr!("608.2d");
    compiles("Argivian Avenger");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let avenger = t.battlefield(P0, "Argivian Avenger");
    t.lands(P0, "Plains", 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(2)); // deathtouch
    let (p, th) = t.pt(avenger);
    t.activate(P0, avenger, 0, &[]).unwrap();
    t.resolve_all();
    let o = t.obj_now(avenger);
    assert!(o.chars.has_keyword(KeywordKind::Deathtouch));
    assert!(!o.chars.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(avenger), (p - 1, th - 1));
}

#[test]
fn jodahs_avenger_your_choice_includes_protection_from_red() {
    cr!("608.2d", "702.16a");
    compiles("Jodah's Avenger");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let avenger = t.battlefield(P0, "Jodah's Avenger");
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // protection from red
    t.activate(P0, avenger, 0, &[]).unwrap();
    t.resolve_all();
    let o = t.obj_now(avenger);
    assert!(o.chars.has_keyword(KeywordKind::Protection));
    assert!(!o.chars.has_keyword(KeywordKind::DoubleStrike));
}

#[test]
fn ballroom_brawlers_both_gain_the_same_chosen_ability() {
    cr!("608.2d");
    compiles("Ballroom Brawlers");
    let mut t = TestGame::new(2);
    let brawlers = t.battlefield(P0, "Ballroom Brawlers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // lifelink
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(brawlers, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    for id in [brawlers, bears] {
        assert!(t.obj_now(id).chars.has_keyword(KeywordKind::Lifelink));
        assert!(!t.obj_now(id).chars.has_keyword(KeywordKind::FirstStrike));
    }
}
