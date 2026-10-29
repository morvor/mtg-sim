//! CR 701.2: activate; CR 701.3d: unattach.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn uid_of(t: &TestGame, id: ObjectId, i: usize) -> u64 {
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .nth(i)
        .map(|a| a.uid)
        .unwrap()
}

#[test]
fn activating_puts_an_ability_on_the_stack_and_pays_its_costs() {
    cr!("701.2", "701.2a");
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    // "{T}: This creature deals 1 damage to any target."
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let ability = t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap().unwrap();
    assert!(t.obj(sorcerer).tapped);
    assert_eq!(t.stack_len(), 1);
    assert!(matches!(
        t.obj(ability).stack.as_deref().unwrap().kind,
        StackKind::Activated { source, .. } if source == sorcerer
    ));
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn only_the_controller_may_activate_and_only_with_priority() {
    cr!("701.2a");
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    // Another player can't activate it.
    assert!(t.activate(P1, sorcerer, 0, &[Entity::Player(P0)]).is_err());
    assert!(!t.obj(sorcerer).tapped);
    // Its controller can't without priority.
    let uid = uid_of(&t, sorcerer, 0);
    t.g.turn.priority = Some(P1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(t.g.activate_ability(P0, sorcerer, uid).is_err());
    t.clear_answers();
    // After a control change, the new controller can.
    run(
        &mut t,
        P1,
        None,
        Effect::GainControl {
            what: Sel::All(Filter::Objects(vec![sorcerer])),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
    );
    t.g.objects[sorcerer.0 as usize].summoning_sick = false;
    assert!(t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).is_err());
    assert!(t.activate(P1, sorcerer, 0, &[Entity::Player(P0)]).is_ok());
}

#[test]
fn a_card_without_a_controller_is_activated_by_its_owner() {
    cr!("701.2a");
    supported("Windcaller Aven");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P1, "Island", 1);
    // Cycling {U} from P0's hand.
    let aven = t.hand(P0, "Windcaller Aven");
    let i = t
        .g
        .obj(aven)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text == "Cycling")
        .unwrap();
    assert!(t.activate(P1, aven, i, &[]).is_err());
    let hand = t.hand_size(P0);
    t.activate(P0, aven, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Windcaller Aven"));
}

#[test]
fn unattaching_leaves_the_equipment_on_the_battlefield_unattached() {
    cr!("701.3", "701.3d");
    supported("Disarm");
    supported("Grafted Wargear");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // "Whenever this Equipment becomes unattached from a permanent, sacrifice that
    // permanent."
    let wargear = t.battlefield(P1, "Grafted Wargear");
    t.g.attach(wargear, Entity::Object(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 4));
    t.lands(P0, "Island", 1);
    let disarm = t.hand(P0, "Disarm");
    t.cast(P0, disarm).target(bears).go();
    t.resolve_all();
    // The Equipment stays on the battlefield, attached to nothing; it became unattached
    // from the Bears, so they were sacrificed.
    assert!(t.on_battlefield(wargear));
    assert_eq!(t.obj(wargear).attached_to, None);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn leaving_the_battlefield_counts_as_becoming_unattached() {
    cr!("701.3d");
    supported("Grafted Wargear");
    // The Equipment leaves the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wargear = t.battlefield(P0, "Grafted Wargear");
    t.g.attach(wargear, Entity::Object(bears));
    t.g.recompute();
    t.g.destroy(wargear, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // It's moved to another creature with its equip ability.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wargear = t.battlefield(P0, "Grafted Wargear");
    t.g.attach(wargear, Entity::Object(bears));
    t.g.recompute();
    t.activate(P0, wargear, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj(wargear).attached_to, Some(Entity::Object(giant)));
    let _ = Zone::Battlefield;
}
