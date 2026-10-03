//! "As this creature enters, choose another creature you control." and the abilities
//! that refer to "the chosen creature" (CR 607.2d, 614.12).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn tyrannical_pitlord_the_chosen_creature_gets_bigger_and_is_sacrificed() {
    cr!("614.12", "607.2d", "603.6c");
    ruling!(
        "Tyrannical Pitlord",
        "If another player gains control of the chosen creature while Tyrannical Pitlord is still on the battlefield, that creature will continue to have flying and get +3/+3."
    );
    compiles("Tyrannical Pitlord");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let pit = t.enter(P0, "Tyrannical Pitlord");
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(other), (3, 3));
    // Another player gains control: still +3/+3.
    t.g.obj_mut(bears).base_controller = P1;
    t.g.dirty = true;
    assert_eq!(t.pt(bears), (5, 5));
    t.g.obj_mut(bears).base_controller = P0;
    t.g.dirty = true;
    // When the Pitlord leaves, you sacrifice the chosen creature.
    t.lands(P0, "Swamp", 2);
    let murder = t.hand(P0, "Murder");
    t.lands(P0, "Swamp", 1);
    t.cast(P0, murder).target(pit).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "{}", t.dump_log());
    assert!(t.on_battlefield(other));
}

#[test]
fn tyrannical_pitlord_chosen_creature_controlled_by_another_player_isnt_sacrificed() {
    cr!("701.21a");
    ruling!(
        "Tyrannical Pitlord",
        "If Tyrannical Pitlord’s leaves-the-battlefield triggered ability resolves while its controller is not the same as the controller of the chosen creature, that creature will not be sacrificed."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let pit = t.enter(P0, "Tyrannical Pitlord");
    t.resolve_all();
    t.g.obj_mut(bears).base_controller = P1;
    t.g.dirty = true;
    t.lands(P0, "Swamp", 3);
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(pit).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
}

#[test]
fn dauntless_bodyguard_gives_the_chosen_creature_indestructible() {
    cr!("614.12", "607.2d");
    ruling!(
        "Dauntless Bodyguard",
        "If the chosen creature leaves the battlefield, you can’t choose a new creature for Dauntless Bodyguard to protect."
    );
    compiles("Dauntless Bodyguard");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let guard = t.enter(P0, "Dauntless Bodyguard");
    t.resolve_all();
    t.activate(P0, guard, 0, &[]).expect("activates");
    t.resolve_all();
    let dbg = format!("{:?}", t.obj_now(bears).chars.abilities);
    assert!(dbg.contains("Indestructible"), "{dbg}");
    let dbg = format!("{:?}", t.obj_now(other).chars.abilities);
    assert!(!dbg.contains("Indestructible"));
    // The chosen creature left the battlefield: no new creature is chosen, and no
    // creature gains indestructible.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let guard = t.enter(P0, "Dauntless Bodyguard");
    t.resolve_all();
    t.g.destroy(bears, None);
    t.settle();
    let back = t.enter(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.activate(P0, guard, 0, &[]).expect("activates");
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. })));
    for id in [other, back] {
        let dbg = format!("{:?}", t.obj_now(id).chars.abilities);
        assert!(!dbg.contains("Indestructible"), "{dbg}");
    }
}
