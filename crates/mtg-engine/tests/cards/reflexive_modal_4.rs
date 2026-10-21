//! Modal triggered abilities with extra structure: "whenever you cast a spell with power,
//! toughness, or mana value 4, choose one that hasn't been chosen this turn" (CR 700.2).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn fantastic_four_triggers_on_power_four_and_offers_only_unchosen_modes() {
    cr!("700.2", "208.1");
    supported("The Fantastic Four");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Fantastic Four");
    t.lands(P0, "Plains", 4);
    t.lands(P0, "Forest", 2);
    // Power 4, mana value 2: triggers.
    let a = t.hand(P0, "Loxodon Peacekeeper");
    let b = t.hand(P0, "Loxodon Peacekeeper");
    // Power 2, mana value 2: doesn't.
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![3]));
    let hand = t.hand_size(P0);
    t.cast(P0, a).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.cast(P0, b).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The second time, "Draw a card" wasn't available.
    let offered: Vec<Vec<usize>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 2);
    assert!(offered[0].contains(&3));
    assert!(!offered[1].contains(&3));
    let stack_before = t.stack_len();
    t.cast(P0, bears).go();
    assert_eq!(t.stack_len(), stack_before + 1);
}

#[test]
fn glorfindel_chosen_mode_then_gets_plus_one() {
    cr!("700.2", "608.2c", "509.1b");
    supported("Glorfindel, Dauntless Rescuer");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Glorfindel, Dauntless Rescuer");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    assert_eq!(t.g.max_blocked_by(g), None);
    t.cast(P0, opt).go();
    t.resolve_all();
    assert_eq!(t.pt(g), (4, 3));
    assert_eq!(t.g.max_blocked_by(g), Some(1));
}

#[test]
fn zuko_unchosen_mode_and_you_lose_two_life() {
    cr!("700.2", "608.2c");
    supported("Zuko, Conflicted");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zuko, Conflicted");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 18);
    // Next time, "Draw a card" has been chosen.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    let offered: Vec<Vec<usize>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![vec![0, 1, 2, 3], vec![1, 2, 3]]);
}

#[test]
fn tranquil_frillback_pays_twice_and_chooses_up_to_two_modes() {
    cr!("603.12", "603.12a", "700.2");
    ruling!(
        "Tranquil Frillback",
        "you may pay {G} up to three times. That is, you may pay {G}, {G}{G}, {G}{G}{G}, or choose not to pay"
    );
    supported("Tranquil Frillback");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    t.graveyard(P1, "Grizzly Bears");
    let f = t.hand(P0, "Tranquil Frillback");
    // Pay {G} twice, then stop.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1, 2]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, f).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.graveyard_size(P1), 0);
    let maxes: Vec<u32> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { max, .. } => Some(max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![2]);
}

#[test]
fn tranquil_frillback_without_paying_no_reflexive_ability() {
    cr!("603.12");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let f = t.hand(P0, "Tranquil Frillback");
    t.answer_yes(P0, false);
    t.cast(P0, f).go();
    t.resolve_all();
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
}

#[test]
fn resplendent_marshal_dies_exiles_another_creature_card() {
    cr!("603.12", "400.7");
    supported("Resplendent Marshal");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Resplendent Marshal");
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.graveyard(P0, "Baneslayer Angel");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.g.destroy(m, None);
    t.resolve_all();
    assert!(t.in_exile("Baneslayer Angel"));
    assert!(t.in_graveyard(P0, "Resplendent Marshal"));
    assert_eq!(t.counters(angel, "+1/+1"), 1);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    // The Marshal itself (now a card in the graveyard) wasn't a choice.
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(cands, vec![vec![Entity::Object(other)]]);
}

#[test]
fn cemetery_desecrator_exiles_another_card_and_shrinks_by_its_mana_value() {
    cr!("603.12", "700.2");
    supported("Cemetery Desecrator");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Cemetery Desecrator");
    let giant = t.graveyard(P1, "Hill Giant");
    let target = t.battlefield(P1, "Serra Angel");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.g.destroy(d, None);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.on_battlefield(target));
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(cands, vec![vec![Entity::Object(giant)]]);
}
