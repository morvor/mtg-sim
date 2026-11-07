//! Targets chosen by a player other than the controller
//! (`oracle/patterns/choice_grammar_targets_others.rs`; CR 601.2c, 601.7).

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

/// Who was asked to choose targets.
fn target_choosers(t: &TestGame) -> Vec<PlayerId> {
    t.asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(p, _)| p)
        .collect()
}

#[test]
fn erithizon_the_defending_player_chooses_the_target() {
    cr!("601.2c", "603.3d");
    ruling!(
        "Erithizon",
        "controller controls the triggered ability, but their opponent chooses the target"
    );
    compiles("Erithizon");
    let mut t = TestGame::new(2);
    let erithizon = t.battlefield(P0, "Erithizon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(erithizon, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(target_choosers(&t), vec![P1]);
}

#[test]
fn volcanic_offering_an_opponent_chooses_the_second_target() {
    cr!("601.7", "601.7a");
    ruling!(
        "Volcanic Offering",
        "may choose the same target nonbasic land or creature you don’t control that you did"
    );
    compiles("Volcanic Offering");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let a = t.battlefield(P1, "Mutavault");
    let b = t.battlefield(P1, "Tectonic Edge");
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let spell = t.hand(P0, "Volcanic Offering");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P1, &[Entity::Object(b)]);
    t.answer_targets(P1, &[Entity::Object(wurm)]);
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(giant)])
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert!(!t.on_battlefield(giant) && !t.on_battlefield(wurm));
    assert_eq!(
        target_choosers(&t),
        vec![P0, P0, P1, P1],
        "you first, then the opponent"
    );
}

#[test]
fn necrotic_plague_returns_attached_to_a_creature_its_old_controllers_opponent_controls() {
    cr!("601.2c", "303.4f");
    compiles("Necrotic Plague");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Savannah Lions");
    let plague = t.hand(P0, "Necrotic Plague");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, plague).target(mine).go();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.g.destroy(mine, None);
    t.settle();
    t.resolve_all();
    let p = t.named_on_battlefield("Necrotic Plague");
    assert_eq!(p.len(), 1);
    assert_eq!(t.g.obj(p[0]).attached_to, Some(Entity::Object(theirs)));
}

#[test]
fn echo_chamber_an_opponent_chooses_one_of_their_creatures_to_copy() {
    cr!("601.7", "707.2");
    compiles("Echo Chamber");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let chamber = t.battlefield(P0, "Echo Chamber");
    t.lands(P0, "Plains", 4);
    t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_targets(P1, &[Entity::Object(angel)]);
    t.activate(P0, chamber, 0, &[]).unwrap();
    t.resolve_all();
    let angels = t.named_on_battlefield("Serra Angel");
    assert_eq!(angels.len(), 2);
    assert!(angels.iter().any(|a| t.g.obj(*a).controller == P0));
    // P1 could only choose among their own creatures.
    let cands = t.asked().into_iter().find_map(|(p, d)| match d {
        Decision::ChooseTargets { candidates, .. } if p == P1 => Some(candidates),
        _ => None,
    });
    assert_eq!(cands, Some(vec![Entity::Object(angel)]));
}
