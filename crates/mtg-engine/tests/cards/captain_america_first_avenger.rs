//! Captain America, First Avenger: "Throw ... — {3}, Unattach an Equipment from Captain
//! America: He deals damage equal to that Equipment's mana value divided as you choose
//! among one, two, or three targets."

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Captain America with Loxodon Warhammer (mana value 3) attached, and {3}.
fn setup(t: &mut TestGame) -> (ObjectId, ObjectId) {
    t.set_step(P0, Step::PrecombatMain);
    let cap = t.battlefield(P0, "Captain America, First Avenger");
    let hammer = t.battlefield(P0, "Loxodon Warhammer");
    t.g.attach(hammer, Entity::Object(cap));
    t.lands(P0, "Plains", 3);
    (cap, hammer)
}

#[test]
fn the_unattached_equipments_mana_value_is_divided_among_the_targets() {
    cr!("601.2d", "601.2h");
    interp!("division-after-cost-amount");
    let mut t = TestGame::new(2);
    let (cap, hammer) = setup(&mut t);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer_choose(P0, &[Entity::Object(hammer)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    t.activate(P0, cap, 0, &[]).expect("activates");
    // The cost was paid: the Equipment is unattached and stays on the battlefield.
    assert!(t.on_battlefield(hammer));
    assert_eq!(t.obj_now(hammer).attached_to, None);
    t.resolve_all();
    assert!(t.on_battlefield(a), "1 damage");
    assert!(!t.on_battlefield(b), "2 damage");
}

#[test]
fn a_player_and_a_creature_share_the_damage() {
    cr!("601.2d");
    let mut t = TestGame::new(2);
    let (cap, _hammer) = setup(&mut t);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1), Entity::Object(a)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    t.activate(P0, cap, 0, &[]).expect("activates");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(!t.on_battlefield(a));
}
