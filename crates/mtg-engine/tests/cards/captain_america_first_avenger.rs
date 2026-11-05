//! Captain America, First Avenger: "Throw ... — {3}, Unattach an Equipment from Captain
//! America: He deals damage equal to that Equipment's mana value divided as you choose
//! among one, two, or three targets."

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Captain America with `equipment` attached, and {3}. Returns (Captain America, the
/// Equipment, the lands).
fn setup(t: &mut TestGame, equipment: &[&str]) -> (ObjectId, Vec<ObjectId>, Vec<ObjectId>) {
    t.set_step(P0, Step::PrecombatMain);
    let cap = t.battlefield(P0, "Captain America, First Avenger");
    let attached = equipment
        .iter()
        .map(|name| {
            let e = t.battlefield(P0, name);
            assert!(t.g.attach(e, Entity::Object(cap)));
            e
        })
        .collect();
    let lands = t.lands(P0, "Plains", 3);
    (cap, attached, lands)
}

/// The amounts the Divide decisions asked P0 to divide.
fn divided_totals(t: &TestGame) -> Vec<u32> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide { total, .. } => Some(total),
            _ => None,
        })
        .collect()
}

#[test]
fn the_unattached_equipments_mana_value_is_divided_among_the_targets() {
    cr!("601.2d", "601.2h");
    interp!("division-after-cost-amount");
    let mut t = TestGame::new(2);
    // Bonesplitter (mana value 1) and Loxodon Warhammer (mana value 3): the Warhammer
    // is unattached, so 3 damage is divided.
    let (cap, eq, _) = setup(&mut t, &["Bonesplitter", "Loxodon Warhammer"]);
    let (splitter, hammer) = (eq[0], eq[1]);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer_choose(P0, &[Entity::Object(hammer)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    t.activate(P0, cap, 0, &[]).expect("activates");
    // The cost was paid: the Warhammer is unattached and stays on the battlefield.
    assert!(t.on_battlefield(hammer));
    assert_eq!(t.obj_now(hammer).attached_to, None);
    assert_eq!(t.obj_now(splitter).attached_to, Some(Entity::Object(cap)));
    // The division was announced with the Warhammer's mana value, once.
    assert_eq!(divided_totals(&t), vec![3]);
    t.resolve_all();
    assert!(t.on_battlefield(a), "1 damage");
    assert_eq!(t.obj_now(a).damage, 1);
    assert!(!t.on_battlefield(b), "2 damage");
}

#[test]
fn a_player_and_a_creature_share_the_damage() {
    cr!("601.2d");
    let mut t = TestGame::new(2);
    let (cap, _, _) = setup(&mut t, &["Loxodon Warhammer"]);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1), Entity::Object(a)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    t.activate(P0, cap, 0, &[]).expect("activates");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(!t.on_battlefield(a));
}

#[test]
fn an_amount_smaller_than_the_number_of_targets_cant_be_divided() {
    cr!("601.2d", "601.2", "733.1");
    interp!("division-after-cost-amount");
    let mut t = TestGame::new(2);
    // Bonesplitter's mana value is 1: two targets can't each be dealt at least 1, so the
    // activation is illegal and the game returns to the moment before it (the
    // Equipment stays attached, the mana isn't spent).
    let (cap, eq, lands) = setup(&mut t, &["Bonesplitter"]);
    let splitter = eq[0];
    let a = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1), Entity::Object(a)]);
    assert!(t.activate(P0, cap, 0, &[]).is_err());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.obj_now(splitter).attached_to, Some(Entity::Object(cap)));
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped));
    assert!(divided_totals(&t).is_empty(), "no division was announced");
    // With one target, the 1 damage is dealt to it.
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.activate(P0, cap, 0, &[]).expect("one target");
    assert_eq!(t.obj_now(splitter).attached_to, None);
    assert_eq!(divided_totals(&t), vec![1]);
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn an_equipment_with_mana_value_0_leaves_nothing_to_divide() {
    cr!("601.2d", "601.2");
    interp!("division-after-cost-amount");
    let mut t = TestGame::new(2);
    // Bone Saw's mana value is 0: no target can be dealt at least 1.
    let (cap, eq, _) = setup(&mut t, &["Bone Saw"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(t.activate(P0, cap, 0, &[]).is_err());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.obj_now(eq[0]).attached_to, Some(Entity::Object(cap)));
    assert_eq!(t.life(P1), 20);
}
