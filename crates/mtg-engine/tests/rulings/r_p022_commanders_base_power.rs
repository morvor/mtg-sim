//! Rulings batch P022 — being a commander isn't a copiable value (CR 903.3, 707.2), so a
//! copy effect neither makes nor unmakes a commander; abilities lost to a copy effect stop
//! granting (CR 613.1a, 613.1f); and a characteristic-defining ability counts toward a
//! creature's base power (CR 604.3, 208.3, 613.4a).

use crate::r_p022_common::*;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const SAKASHIMA: &str = "Sakashima of a Thousand Faces";
const CHARLATAN: &str = "Shameless Charlatan";
/// The ability Shameless Charlatan grants: "Commander creatures you own have "{2}{U}: This
/// creature becomes a copy of another target creature.""
const GRANTED: &str = "becomes a copy of another target creature";

/// P0's commander `name` (in the command zone) put onto the battlefield.
fn commander_onto_battlefield(t: &mut TestGame, name: &str, copy: Option<ObjectId>) -> ObjectId {
    let c = crate::r_s13_common::commander(t, P0, name);
    if let Some(what) = copy {
        t.answer_choose(P0, &[obj(what)]);
    }
    let id = t
        .g
        .move_object(c, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, None)
        .unwrap();
    t.g.flush_events();
    t.settle();
    assert!(t.obj_now(id).is_commander);
    unsick(t, id);
    id
}

#[test]
fn sakashima_stays_or_stays_not_a_commander_whatever_it_copies() {
    cr!("903.3", "707.2");
    ruling!("Sakashima of a Thousand Faces", "If Sakashima is your commander, it remains your commander even if it's copying another creature. If Sakashima is not your commander, it remains not your commander even if it's copying your commander.");
    supported(SAKASHIMA);
    supported(CHARLATAN);
    // Sakashima is P0's commander and copies Hill Giant: Shameless Charlatan still grants
    // it its ability.
    let mut t = crate::r_s13_common::commander_game();
    t.battlefield(P0, CHARLATAN);
    let giant = t.battlefield(P0, "Hill Giant");
    let s = commander_onto_battlefield(&mut t, SAKASHIMA, Some(giant));
    assert_eq!(name_of(&t, s), "Hill Giant");
    assert_eq!(crate::r_p025_common::activated_with(&t, s, GRANTED), 1);
    assert_eq!(crate::r_p025_common::activated_with(&t, giant, GRANTED), 0);
    // Isamaru is P0's commander; a Sakashima from P0's hand copies it: not a commander.
    supported("Isamaru, Hound of Konda");
    let mut t = crate::r_s13_common::commander_game();
    t.battlefield(P0, CHARLATAN);
    let isamaru = commander_onto_battlefield(&mut t, "Isamaru, Hound of Konda", None);
    assert_eq!(crate::r_p025_common::activated_with(&t, isamaru, GRANTED), 1);
    let s = enter_copying(&mut t, P0, SAKASHIMA, isamaru);
    assert_eq!(name_of(&t, s), "Isamaru, Hound of Konda");
    assert!(t.on_battlefield(s) && t.on_battlefield(isamaru));
    assert_eq!(crate::r_p025_common::activated_with(&t, s, GRANTED), 0);
}

#[test]
fn shameless_charlatan_copying_something_stops_granting_its_ability() {
    cr!("613.1a", "613.1f", "707.2");
    ruling!("Shameless Charlatan", "If Shameless Charlatan is your commander and somehow becomes a creature, and you use the ability it grants itself to make it a copy of another creature, it will no longer have its ability so it would no longer grant the activated ability to commander creatures you own (including itself).");
    supported(CHARLATAN);
    supported("Isamaru, Hound of Konda");
    // P0's commanders: Isamaru and Shameless Charlatan, made a 2/2 creature.
    let mut t = crate::r_s13_common::commander_game();
    let isamaru = commander_onto_battlefield(&mut t, "Isamaru, Hound of Konda", None);
    let charlatan = commander_onto_battlefield(&mut t, CHARLATAN, None);
    modify(
        &mut t,
        charlatan,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(2)), Some(Value::c(2))),
        ],
    );
    assert_eq!(crate::r_p025_common::activated_with(&t, charlatan, GRANTED), 1);
    assert_eq!(crate::r_p025_common::activated_with(&t, isamaru, GRANTED), 1);
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{2}{U}");
    let i = activated_index(&t, charlatan, GRANTED);
    t.activate(P0, charlatan, i, &[obj(giant)]).expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, charlatan), "Hill Giant");
    assert_eq!(crate::r_p025_common::activated_with(&t, charlatan, GRANTED), 0);
    assert_eq!(crate::r_p025_common::activated_with(&t, isamaru, GRANTED), 0);
}

// --- Base power ----------------------------------------------------------------------------------

#[test]
fn a_characteristic_defining_power_counts_as_base_power_for_curie_and_jason_bright() {
    cr!("604.3", "208.3", "613.4a");
    ruling!("Curie, Emergent Intelligence", "If a creature has a characteristic-defining ability that sets its power, indicated with a * or similar in the power and toughness box, that ability is taken into account when determining its base power.");
    ruling!("Jason Bright, Glowing Prophet", "If a creature has a characteristic-defining ability that sets its power, indicated with a * or similar in the power and toughness box, that ability is taken into account when determining its base power.");
    supported("Curie, Emergent Intelligence");
    supported("Jason Bright, Glowing Prophet");
    supported("Master of Etherium");
    // Curie becomes a copy of Master of Etherium ("power and toughness are each equal to
    // the number of artifacts you control"): three artifacts, base power 3, three cards.
    let mut t = TestGame::new(2);
    let curie = t.battlefield(P0, "Curie, Emergent Intelligence");
    let master = t.battlefield(P0, "Master of Etherium");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    lands_for(&mut t, P0, "{1}{U}");
    t.answer_choose(P0, &[obj(master)]);
    let i = activated_index(&t, curie, "becomes a copy");
    t.activate(P0, curie, i, &[]).expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, curie), "Master of Etherium");
    assert_eq!(t.pt(curie), (3, 3));
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(curie, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand + 3);
    // Jason Bright: a Zombie whose power is set by its characteristic-defining ability
    // (Squelching Leeches, made a Zombie) dies with power equal to its base power: no
    // card. With +1/+0 it would draw.
    supported("Squelching Leeches");
    for pumped in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Jason Bright, Glowing Prophet");
        t.lands(P0, "Swamp", 2);
        let leeches = t.battlefield(P0, "Squelching Leeches");
        let mut mods = vec![Modification::AddSubtypes(vec!["Zombie".into()])];
        if pumped {
            mods.push(Modification::ModifyPT(Value::c(1), Value::c(0)));
        }
        modify(&mut t, leeches, mods);
        let hand = t.hand_size(P0);
        kill(&mut t, leeches);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + usize::from(pumped), "pumped: {pumped}");
    }
}

#[test]
fn a_characteristic_defining_power_counts_as_base_power_for_rapid_augmenter_and_zinnia() {
    cr!("604.3", "208.3", "613.4a");
    ruling!("Rapid Augmenter", "If a creature has a characteristic-defining ability that sets its power, indicated with a */* or similar in the power and toughness box, that ability is taken into account when determining its base power.");
    ruling!("Zinnia, Valley's Voice", "If a creature has a characteristic-defining ability that sets its power, indicated with a */* or similar in the power and toughness box, that ability is taken into account when determining its base power.");
    supported("Rapid Augmenter");
    supported("Zinnia, Valley's Voice");
    supported("Swarm of Rats");
    // Swarm of Rats: "power is equal to the number of Rats you control" — base power 1
    // with one Rat.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rapid Augmenter");
    let rats = t.enter(P0, "Swarm of Rats");
    t.resolve_all();
    assert_eq!(t.pt(rats), (1, 1));
    assert!(has_kw(&t, rats, KeywordKind::Haste));
    // Zinnia: "+X/+0, where X is the number of other creatures you control with base
    // power 1".
    let mut t = TestGame::new(2);
    let zinnia = t.battlefield(P0, "Zinnia, Valley's Voice");
    assert_eq!(t.pt(zinnia), (1, 3));
    t.battlefield(P0, "Swarm of Rats");
    assert_eq!(t.pt(zinnia), (2, 3));
    // A second Rat: each Swarm's base power is 2 now.
    t.battlefield(P0, "Swarm of Rats");
    assert_eq!(t.pt(zinnia), (1, 3));
}
