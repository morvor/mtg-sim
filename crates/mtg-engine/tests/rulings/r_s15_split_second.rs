//! Rulings batch S15 — split second (CR 702.61): Siege Smash, Inventory Management.

use crate::r_s01_common::*;
use crate::r_s03_common::{choice_candidates, in_hand_with_mana};
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s08_common::priority_asks_of;
use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether every action is passing, a mana ability, or a special action.
fn only_mana_abilities_and_special_actions(t: &TestGame, actions: &[Action]) -> bool {
    actions.iter().all(|a| match a {
        Action::Pass | Action::Special(_) => true,
        Action::Activate { source, ability } => {
            t.g.obj(*source)
                .chars
                .abilities
                .iter()
                .any(|x| x.uid == *ability && x.is_mana_ability())
        }
        _ => false,
    })
}

#[test]
fn players_get_priority_but_may_only_activate_mana_abilities_and_take_special_actions() {
    cr!("702.61a", "702.61b", "117.3d");
    ruling!(
        "Siege Smash",
        "Players still get priority while a spell with split second is on the stack; their options are just limited to mana abilities and certain special actions."
    );
    supported("Siege Smash");
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1 could cast Lightning Bolt or activate Prodigal Pyromancer ("{T}: This creature
    // deals 1 damage to any target."), and has a Mountain and Llanowar Elves for mana.
    let bolt = t.hand(P1, "Lightning Bolt");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    let mountain = t.battlefield(P1, "Mountain");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.g.turn.priority = Some(P1);
    let before = t.g.legal_actions(P1);
    assert!(before
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == bolt)));
    assert!(!only_mana_abilities_and_special_actions(&t, &before));
    // Siege Smash (split second): "Target creature gets +3/+2 and gains trample until end
    // of turn."
    let smash = in_hand_with_mana(&mut t, P0, "Siege Smash");
    t.cast(P0, smash).modes(&[1]).target(bears).go();
    // P1 gets priority; the only actions offered are passing, mana abilities, and special
    // actions.
    t.g.turn.priority = Some(P1);
    let during = t.g.legal_actions(P1);
    assert!(during.contains(&Action::Pass));
    assert!(only_mana_abilities_and_special_actions(&t, &during));
    assert!(!during
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == pyro)));
    assert!(t.activate(P1, mountain, 0, &[]).is_ok());
    assert!(t.activate(P1, elves, 0, &[]).is_ok());
    assert!(t
        .g
        .cast_spell(P1, bolt, mtg_engine::object::CastMethod::Normal)
        .is_err());
    // Both players are asked for priority before it resolves.
    let from = t.asked().len();
    t.g.turn.priority = Some(P0);
    let ok = t.g.run_until(1000, |g| g.stack.is_empty());
    assert!(ok);
    assert!(priority_asks_of(&t, P0, from) >= 1);
    assert!(priority_asks_of(&t, P1, from) >= 1);
    assert_eq!(t.pt(bears), (5, 4));
}

#[test]
fn split_second_doesnt_stop_triggered_abilities() {
    cr!("702.61b", "603.3");
    ruling!(
        "Inventory Management",
        "Split second doesn’t stop triggered abilities from triggering. If one does, its controller puts it on the stack and chooses targets for it, if any. Those abilities will resolve as normal."
    );
    supported("Inventory Management");
    supported("Monastery Swiftspear");
    // Inventory Management (split second): "For each Aura and Equipment you control, you
    // may attach it to a creature you control." Casting it triggers Monastery Swiftspear's
    // prowess; the trigger goes on the stack above it and resolves first.
    let mut t = TestGame::new(2);
    let swiftspear = t.battlefield(P0, "Monastery Swiftspear");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rancor = attach_new(&mut t, P0, "Rancor", bears);
    let im = in_hand_with_mana(&mut t, P0, "Inventory Management");
    t.cast(P0, im).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(swiftspear)]);
    t.resolve();
    // Prowess resolved: +1/+1 until end of turn; Inventory Management is still on the
    // stack.
    assert_eq!(t.pt(swiftspear), (2, 3));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // Rancor ("Enchanted creature gets +2/+0 and has trample.") moved to the Swiftspear.
    assert_eq!(attached_to(&t, rancor), Some(Entity::Object(swiftspear)));
    assert_eq!(t.pt(swiftspear), (4, 3));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn inventory_management_attaches_only_where_each_can_legally_be_attached() {
    cr!("701.3a", "702.16d");
    supported("Inventory Management");
    supported("Tel-Jilad Chosen");
    // Each Aura and Equipment P0 controls may be attached to a creature P0 controls that
    // it could legally be attached to; P0 may also leave one where it is.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    // Protection from artifacts: it can't be equipped by Bonesplitter.
    let chosen = t.battlefield(P0, "Tel-Jilad Chosen");
    let theirs = t.battlefield(P1, "Savannah Lions");
    let rancor = attach_new(&mut t, P0, "Rancor", bears);
    let bonesplitter = attach_new(&mut t, P0, "Bonesplitter", bears);
    let im = in_hand_with_mana(&mut t, P0, "Inventory Management");
    t.cast(P0, im).go();
    // Rancor stays; Bonesplitter moves to the Giant.
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(attached_to(&t, rancor), Some(Entity::Object(bears)));
    assert_eq!(attached_to(&t, bonesplitter), Some(Entity::Object(giant)));
    // Bonesplitter could go to the creatures P0 controls other than Tel-Jilad Chosen.
    let offered = choice_candidates(&t, from, "");
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&Entity::Object(giant)));
    assert!(!offered[0].contains(&Entity::Object(chosen)));
    assert!(!offered[0].contains(&Entity::Object(theirs)));
}
