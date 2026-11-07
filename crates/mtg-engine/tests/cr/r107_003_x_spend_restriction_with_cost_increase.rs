//! CR 107.3a, 601.2f, 602.2b: "Spend only black mana on X" restricts the mana spent on the
//! X of an activation cost, not the generic mana a cost increase adds to it.

use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn only_the_x_of_crypt_rats_needs_black_mana_under_suppression_field() {
    cr!("107.3a", "601.2f", "602.2b");
    // "{X}: This creature deals X damage to each creature and each player. Spend only
    // black mana on X." Suppression Field: "Activated abilities cost {2} more ...".
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Crypt Rats");
    t.battlefield(P1, "Suppression Field");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // X = 1 costs {1}{2}: one black mana for X and two red for the increase.
    t.g.players[0].mana_pool.add_type(ManaType::B, 1);
    t.g.players[0].mana_pool.add_type(ManaType::R, 2);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, rats, 0, &[]).expect("activate Crypt Rats");
    // All three mana paid the increased cost.
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn suppression_field_doesnt_increase_the_cost_of_a_mana_ability() {
    cr!("605.1a");
    ruling!(
        "Suppression Field",
        "Suppression Field’s ability doesn’t affect static abilities, triggered abilities, or mana abilities."
    );
    // Llanowar Elves ("{T}: Add {G}.") with no other mana available.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Suppression Field");
    t.activate(P0, elves, 0, &[]).expect("activate Llanowar Elves");
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::G), 1);
}
