//! Linked abilities where one refers to the players or objects the other affected or
//! chose (CR 607.1, 607.2e): "When ~ enters, target player loses 6 life. When ~ leaves the
//! battlefield, that player gains 6 life." (Laquatus's Champion), "Pay 1 life: Choose a
//! creature card exiled with ~. ~ has all activated and triggered abilities of the last
//! chosen card." (Koh, the Face Stealer).

use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts Laquatus's Champion for P0, its enters trigger targeting `target`; returns the
/// permanent with the trigger still on the stack.
fn champion_entering(t: &mut TestGame, target: PlayerId) -> ObjectId {
    t.lands(P0, "Swamp", 6);
    let c = t.hand(P0, "Laquatus's Champion");
    t.cast(P0, c).go();
    t.answer_targets(P0, &[Entity::Player(target)]);
    t.resolve(); // the creature spell; its trigger goes on the stack
    assert_eq!(t.stack_len(), 1);
    t.g.current(c)
}

#[test]
fn the_linked_ability_refers_to_the_player_the_first_affected() {
    cr!("607.1", "603.6c", "603.10a");
    let mut t = TestGame::new(2);
    let champion = champion_entering(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // Another player is irrelevant: only the targeted player gains the life.
    t.g.destroy(champion, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn while_the_first_ability_is_on_the_stack_that_player_is_its_target() {
    cr!("607.1", "603.6c");
    ruling!(
        "Laquatus's Champion",
        "if that ability is still on the stack, the player who is its target"
    );
    let mut t = TestGame::new(2);
    let champion = champion_entering(&mut t, P1);
    // In response, the Champion leaves the battlefield: its leaves trigger resolves first.
    t.g.destroy(champion, None);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 26, "that player is the pending ability's target");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_first_ability_that_affected_no_one_leaves_no_player() {
    cr!("607.1", "608.2b");
    let mut t = TestGame::new(2);
    let champion = champion_entering(&mut t, P1);
    // The target becomes illegal: the ability doesn't resolve, so it affects no player.
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    t.g.destroy(champion, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_new_object_has_no_notes() {
    cr!("607.1", "400.7");
    let mut t = TestGame::new(2);
    let champion = champion_entering(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // Flickered: the creature that returns is a new object; its enters trigger targets P0.
    let gone = t
        .g
        .move_object(champion, Zone::Exile, MoveCause::Effect, None)
        .unwrap();
    t.resolve_all(); // the old object's leaves trigger: P1 gains 6
    assert_eq!(t.life(P1), 20);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    let back = t
        .g
        .move_object(gone, Zone::Battlefield, MoveCause::Effect, Some(P0))
        .unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
    t.g.destroy(back, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn the_last_chosen_card_replaces_earlier_choices() {
    cr!("607.2d", "607.2e", "613.1f");
    let mut t = TestGame::new(2);
    let koh = t.battlefield(P0, "Koh, the Face Stealer");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    let husk = t.battlefield(P1, "Nantuko Husk");
    // Both die and Koh exiles them.
    for c in [sorcerer, husk] {
        t.g.destroy(c, None);
        t.answer_yes(P0, true);
        t.resolve_all();
    }
    assert!(t.in_exile("Prodigal Sorcerer") && t.in_exile("Nantuko Husk"));
    let exiled = |t: &TestGame, name: &str| t.g.find_in_zone(Zone::Exile, name)[0];
    let activated = |t: &mut TestGame| -> Vec<String> {
        t.g.recompute();
        t.g.obj(koh)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
            .map(|a| a.text.clone())
            .collect()
    };
    // No card chosen yet: Koh has only its own activated ability.
    assert_eq!(activated(&mut t).len(), 1);
    let s = exiled(&t, "Prodigal Sorcerer");
    t.answer_choose(P0, &[Entity::Object(s)]);
    t.activate(P0, koh, 0, &[]).unwrap();
    t.resolve_all();
    assert!(activated(&mut t).iter().any(|a| a.contains("deals 1 damage")));
    // Choosing again replaces the choice.
    let h = exiled(&t, "Nantuko Husk");
    t.answer_choose(P0, &[Entity::Object(h)]);
    t.activate(P0, koh, 0, &[]).unwrap();
    t.resolve_all();
    let now = activated(&mut t);
    assert!(now.iter().any(|a| a.contains("Sacrifice a creature")));
    assert!(!now.iter().any(|a| a.contains("deals 1 damage")));
    assert_eq!(t.life(P0), 18);
}
