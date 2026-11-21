//! Rulings batch S26 — abilities that copy an ability as it's activated (CR 707.10,
//! 602.2): Kurkesh, Onakke Ancient ("Whenever you activate an ability of an artifact, if
//! it isn't a mana ability, you may pay {R}. If you do, copy that ability.") and
//! Illusionist's Bracers ("Whenever an ability of equipped creature is activated, if it
//! isn't a mana ability, copy that ability."). A copy of a modal ability has its mode
//! (CR 700.2g).

use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s25_common::abilities_from;
use crate::r_s26_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn kurkeshs_copy_of_a_modal_ability_has_the_same_mode() {
    cr!("707.10", "700.2g", "603.4");
    ruling!(
        "Kurkesh, Onakke Ancient",
        "If the ability is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. You can't choose a different one."
    );
    supported("Kurkesh, Onakke Ancient");
    supported("Bow of Nylea");
    // Bow of Nylea: "{1}{G}, {T}: Choose one — ... • You gain 3 life. ..."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kurkesh, Onakke Ancient");
    // "If it isn't a mana ability": an artifact's mana ability doesn't trigger it.
    let ring = t.battlefield(P0, "Sol Ring");
    activate_containing(&mut t, P0, ring, "Add {C}{C}").expect("Sol Ring");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    mtg_engine::mana_abilities::empty_pool(&mut t.g, P0);
    let bow = t.battlefield(P0, "Bow of Nylea");
    t.lands(P0, "Taiga", 3);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    let ability = activate_containing(&mut t, P0, bow, "Choose one")
        .unwrap()
        .unwrap();
    t.settle();
    // Kurkesh: pay {R} and copy it.
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    let from = t.asked().len();
    t.resolve();
    let copies = abilities_from(&t, bow);
    assert_eq!(copies.len(), 2);
    assert!(copies
        .iter()
        .all(|c| modes_on_stack(&t, *c) == modes_on_stack(&t, ability)));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn illusionists_bracers_copy_of_a_modal_ability_has_the_same_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Illusionist's Bracers",
        "If the ability is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. You can't choose a different one."
    );
    supported("Illusionist's Bracers");
    supported("Kargan Intimidator");
    // Kargan Intimidator: "{1}: Choose one that hasn't been chosen this turn — • This
    // creature gets +1/+1 until end of turn. • ..."
    let mut t = TestGame::new(2);
    let kargan = t.battlefield(P0, "Kargan Intimidator");
    attach_new(&mut t, P0, "Illusionist's Bracers", kargan);
    t.lands(P0, "Mountain", 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    activate_containing(&mut t, P0, kargan, "Choose one")
        .unwrap()
        .unwrap();
    t.settle();
    t.resolve();
    // The copy (+1/+1, the mode chosen for the original) is on the stack too.
    let copies = abilities_from(&t, kargan);
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().all(|c| modes_on_stack(&t, *c) == vec![0]));
    t.resolve_all();
    // A 3/1 that got +1/+1 twice.
    assert_eq!(t.pt(kargan), (5, 3));
}
