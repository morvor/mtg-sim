//! Modal triggered abilities with extra structure: "whenever you cast a spell with power,
//! toughness, or mana value 4, choose one that hasn't been chosen this turn" (CR 700.2).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
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
