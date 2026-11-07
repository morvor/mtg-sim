//! cards5-referents-statics, family `chosen-object-statics`: the permanent chosen as a
//! permanent entered is the object of its static abilities (CR 607.2d, 614.12; a new
//! object after a zone change is no longer the chosen one, CR 400.7).

use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
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

fn activated(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
}

#[test]
fn scheming_fence_takes_the_activated_abilities_of_the_chosen_permanent() {
    cr!("607.2d", "614.12", "613.1f", "609.4b");
    ruling!(
        "Scheming Fence",
        "Scheming Fence gains only activated abilities"
    );
    ruling!("Scheming Fence", "refers to Scheming Fence instead");
    supported("Scheming Fence");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P1, "Shivan Dragon");
    t.lands(P0, "Island", 2);
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    let fence = t.enter(P0, "Scheming Fence");
    t.g.recompute();
    // The Dragon's controller can't activate its firebreathing.
    assert!(t.activate(P1, dragon, 0, &[]).is_err());
    // Scheming Fence has it (not flying), and pays {R} with blue mana; the ability names
    // the Fence.
    assert!(!t.obj_now(fence).chars.has_keyword(KeywordKind::Flying));
    let before = t.pt(fence);
    t.activate(P0, fence, 0, &[]).expect("firebreathing via the Fence");
    t.resolve_all();
    assert_eq!(t.pt(fence), (before.0 + 1, before.1));
    assert_eq!(t.pt(dragon), (5, 5));
}

#[test]
fn scheming_fence_choice_is_optional_and_lost_when_the_permanent_leaves() {
    cr!("607.2d", "400.7");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P1, "Shivan Dragon");
    // Nothing chosen: no abilities, nothing restricted.
    t.answer_choose(P0, &[]);
    let fence = t.enter(P0, "Scheming Fence");
    t.g.recompute();
    assert_eq!(activated(&t, fence), 0);
    t.lands(P1, "Mountain", 1);
    assert!(t.activate(P1, dragon, 0, &[]).is_ok());
    t.resolve_all();
    // Chosen, then the Dragon blinks: the new object isn't the chosen one.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P1, "Shivan Dragon");
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    let fence = t.enter(P0, "Scheming Fence");
    t.g.recompute();
    assert_eq!(activated(&t, fence), 1);
    let id = t.g.move_object(dragon, Zone::Exile, events::MoveCause::Effect, None).expect("exiled");
    let back = t.g.move_object(id, Zone::Battlefield, events::MoveCause::Effect, Some(P1)).expect("returned");
    t.g.recompute();
    assert_eq!(activated(&t, fence), 0);
    t.lands(P1, "Mountain", 1);
    assert!(t.activate(P1, back, 0, &[]).is_ok());
}

#[test]
fn scheming_fence_gains_no_loyalty_abilities() {
    cr!("607.2d");
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P1, "Ajani Goldmane");
    t.answer_choose(P0, &[Entity::Object(ajani)]);
    let fence = t.enter(P0, "Scheming Fence");
    t.g.recompute();
    assert_eq!(activated(&t, fence), 0);
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.activate(P1, ajani, 0, &[]).is_err());
}
