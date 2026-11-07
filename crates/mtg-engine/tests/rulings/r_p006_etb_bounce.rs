//! Rulings batch P006 — "when this enters, return a permanent you control to its owner's
//! hand": the permanent is chosen as the ability resolves, not targeted (CR 608.2c,
//! 115.1), so a creature with shroud that arrived after the trigger can be chosen, nobody
//! gets priority between the choice and the return (CR 608.2), and the permanent itself
//! may (or, alone, must) be chosen; abilities that must target their own source when it's
//! the only legal target (CR 603.3d); Stormfront Riders' leaves-the-battlefield trigger
//! (CR 603.10a, 111.7).

use crate::r_p006_common::*;
use crate::r_s01_common::supported;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The ETB bounce trigger of `name` is chosen on resolution: it has no targets; a shroud
/// creature put onto the battlefield after the trigger went on the stack is a candidate and
/// is returned; and no one is asked for priority while the ability resolves.
fn chosen_on_resolution(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let me = t.enter(P0, name);
    t.settle();
    assert_eq!(triggers_from(&t, me), 1, "{name}: trigger");
    let trig = top(&t);
    assert!(targets_of(&t, trig).is_empty(), "{name}: no targets");
    // Arrives after the trigger was put on the stack; it has shroud (can't be targeted).
    let shroud = t.battlefield(P0, "Argothian Enchantress");
    let from = mark(&t);
    t.answer_choose(P0, &[obj(shroud)]);
    t.g.resolve_top();
    let choices = choices_since(&t, P0, from);
    assert_eq!(choices.len(), 1, "{name}: one choice");
    assert!(choices[0].0.contains(&obj(shroud)), "{name}: shroud candidate");
    assert!(priority_since(&t, from).is_empty(), "{name}: no priority");
    assert!(in_hand(&t, shroud), "{name}: shroud creature returned");
    assert!(t.on_battlefield(me), "{name}: stays");
}

#[test]
fn etb_bounce_chooses_on_resolution_without_targeting() {
    cr!("608.2c", "115.1", "608.2");
    ruling!(
        "Dream Stalker",
        "Dream Stalker's ability doesn't target any permanent. Because of this, no player may take action between the time you choose the permanent to return and the time you return it."
    );
    ruling!(
        "Kor Skyfisher",
        "Kor Skyfisher’s triggered ability doesn’t target a permanent. You choose which one to return to its owner’s hand as the ability resolves. No one can respond to the choice."
    );
    ruling!(
        "Shrieking Drake",
        "Shrieking Drake's ability doesn't target any creature. Therefore, no player may take actions between the time you choose the creature to return and the time you return it."
    );
    ruling!(
        "Whitemane Lion",
        "Whitemane Lion's ability doesn't target any creature, therefore no player may take actions between the time you choose the creature to return and the time you return it."
    );
    ruling!(
        "Ambush Krotiq",
        "The triggered ability doesn’t target any permanent. You choose which one to return as the ability resolves. No player can respond to this choice once the ability starts resolving."
    );
    ruling!(
        "Ancestral Statue",
        "The triggered ability doesn’t target any permanent. You choose which one to return as the ability resolves. No player can respond to this choice once the ability starts resolving."
    );
    for name in [
        "Dream Stalker",
        "Kor Skyfisher",
        "Shrieking Drake",
        "Whitemane Lion",
        "Ambush Krotiq",
        "Ancestral Statue",
    ] {
        chosen_on_resolution(name);
    }
}

#[test]
fn stormfront_riders_chooses_two_on_resolution() {
    cr!("608.2c", "115.1", "608.2");
    ruling!(
        "Stormfront Riders",
        "The ability of Stormfront Riders doesn't target any creatures. Because of this, no player may take action between the time you choose the creature to return and the time you return it."
    );
    supported("Stormfront Riders");
    let mut t = TestGame::new(2);
    let riders = t.enter(P0, "Stormfront Riders");
    t.settle();
    let trig = top(&t);
    assert!(targets_of(&t, trig).is_empty());
    let a = t.battlefield(P0, "Argothian Enchantress");
    let b = t.battlefield(P0, "Grizzly Bears");
    let from = mark(&t);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.g.resolve_top();
    assert!(priority_since(&t, from).is_empty());
    assert!(in_hand(&t, a) && in_hand(&t, b));
    assert!(t.on_battlefield(riders));
    // Each returned creature triggers the last ability.
    t.settle();
    assert_eq!(triggers_from(&t, riders), 2);
    t.resolve_all();
    assert_eq!(soldiers(&t), 2);
}

/// `name`'s ETB bounce may choose the permanent itself while another candidate exists, and
/// must choose it when it's the only candidate.
fn may_or_must_return_itself(name: &str) {
    supported(name);
    // May: another creature is available, and it chooses itself.
    let mut t = TestGame::new(2);
    let other = t.battlefield(P0, "Grizzly Bears");
    let me = t.enter(P0, name);
    t.settle();
    let from = mark(&t);
    t.answer_choose(P0, &[obj(me)]);
    t.resolve_all();
    let choices = choices_since(&t, P0, from);
    assert!(
        choices[0].0.contains(&obj(me)) && choices[0].0.contains(&obj(other)),
        "{name}: both are candidates"
    );
    assert!(in_hand(&t, me), "{name}: returned itself");
    assert!(t.on_battlefield(other), "{name}: other stays");
    // Must: it's the only candidate (the default answer is the only legal one).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let me = t.enter(P0, name);
    t.settle();
    let from = mark(&t);
    t.resolve_all();
    let choices = choices_since(&t, P0, from);
    if let Some((cands, min, _)) = choices.first() {
        assert_eq!(cands, &vec![obj(me)], "{name}: only itself");
        assert_eq!(*min, 1, "{name}: mandatory");
    }
    assert!(in_hand(&t, me), "{name}: must return itself");
}

#[test]
fn etb_bounce_may_or_must_return_itself() {
    cr!("608.2c", "603.3d");
    ruling!(
        "Dream Stalker",
        "You may return Dream Stalker itself to its owner's hand as its triggered ability resolves."
    );
    ruling!(
        "Kor Skyfisher",
        "If Kor Skyfisher is still on the battlefield as its triggered ability resolves, you may return Kor Skyfisher itself."
    );
    ruling!(
        "Shrieking Drake",
        "You may return Shrieking Drake itself to its owner's hand as its triggered ability resolves. If you don't control any other creature, you must return it."
    );
    ruling!(
        "Whitemane Lion",
        "You may return Whitemane Lion itself to its owner's hand as its triggered ability resolves. If you don't control any other creature, you must return it."
    );
    ruling!(
        "Ancestral Statue",
        "Ancestral Statue’s ability is mandatory. If Ancestral Statue is the only nonland permanent you control when its ability resolves, you must return it to its owner’s hand."
    );
    for name in [
        "Dream Stalker",
        "Kor Skyfisher",
        "Shrieking Drake",
        "Whitemane Lion",
        "Ancestral Statue",
    ] {
        may_or_must_return_itself(name);
    }
    // Ancestral Statue: a land isn't a candidate, so with only lands it returns itself.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    let statue = t.enter(P0, "Ancestral Statue");
    t.resolve_all();
    assert!(in_hand(&t, statue));
    assert!(t.on_battlefield(land));
}

#[test]
fn kor_skyfisher_gone_before_resolution_returns_something_else() {
    cr!("608.2c", "400.7");
    ruling!(
        "Kor Skyfisher",
        "If Kor Skyfisher is still on the battlefield as its triggered ability resolves, you may return Kor Skyfisher itself."
    );
    // Skyfisher leaves first: it isn't a candidate, the other permanent must be returned.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fish = t.enter(P0, "Kor Skyfisher");
    t.settle();
    crate::r_s02_common::destroy(&mut t, fish);
    let from = mark(&t);
    t.resolve_all();
    let choices = choices_since(&t, P0, from);
    assert!(!choices.is_empty());
    assert!(!choices[0].0.contains(&obj(t.g.current(fish))));
    assert!(in_hand(&t, bears));
    assert!(in_graveyard(&t, fish));
}

#[test]
fn stormfront_riders_must_return_itself() {
    cr!("608.2c", "603.10a");
    ruling!(
        "Stormfront Riders",
        "You may return Stormfront Riders itself to its owner's hand as its triggered ability resolves. If you don't control two other creatures, you must return it."
    );
    ruling!(
        "Stormfront Riders",
        "If you control only one creature as the triggered ability resolves, you return it to its owner's hand."
    );
    // One other creature: both are returned, and each return makes a Soldier.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let riders = t.enter(P0, "Stormfront Riders");
    t.resolve_all();
    assert!(in_hand(&t, riders) && in_hand(&t, bears));
    assert_eq!(soldiers(&t), 2, "its own return triggers too (look back)");
    // Only creature: it returns itself.
    let mut t = TestGame::new(2);
    let riders = t.enter(P0, "Stormfront Riders");
    t.resolve_all();
    assert!(in_hand(&t, riders));
    assert_eq!(soldiers(&t), 1);
    // Three others: it may return itself and one other.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Craw Wurm");
    let riders = t.enter(P0, "Stormfront Riders");
    t.settle();
    t.answer_choose(P0, &[obj(riders), obj(a)]);
    t.resolve_all();
    assert!(in_hand(&t, riders) && in_hand(&t, a));
    assert_eq!(soldiers(&t), 2);
}

fn soldiers(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Soldier"))
        .count()
}

#[test]
fn stormfront_riders_triggers_on_a_returned_token() {
    cr!("111.7", "603.10a");
    ruling!(
        "Stormfront Riders",
        "A token you own that's returned to your hand moves to your hand before it ceases to exist. This will cause the last ability of Stormfront Riders to trigger."
    );
    let mut t = TestGame::new(2);
    let riders = t.battlefield(P0, "Stormfront Riders");
    let tok = crate::r_s02_common::create_token(&mut t, P0, "Goblin");
    let before = soldiers(&t);
    let unsummon = t.hand(P0, "Unsummon");
    t.lands(P0, "Island", 1);
    t.cast(P0, unsummon).target(obj(tok)).go();
    t.resolve();
    t.settle();
    assert!(!t.on_battlefield(tok));
    assert_eq!(triggers_from(&t, riders), 1);
    t.resolve_all();
    assert_eq!(soldiers(&t), before + 1);
    // The token ceased to exist in hand.
    assert!(t.g.player(P0).hand.iter().all(|id| !t.g.obj(*id).is_token()));
}

#[test]
fn must_target_itself_when_the_only_creature() {
    cr!("603.3d", "115.1");
    ruling!(
        "Kiri-Onna",
        "If there are no other creatures on the battlefield as Kiri-Onna enters, you must target Kiri-Onna with its own ability."
    );
    ruling!(
        "Flametongue Kavu",
        "Flametongue Kavu’s ability can target itself. If it’s the only creature on the battlefield as its ability triggers, its ability must target itself."
    );
    let mut t = TestGame::new(2);
    let kiri = t.enter(P0, "Kiri-Onna");
    t.settle();
    let trig = top(&t);
    assert_eq!(targets_of(&t, trig), vec![obj(kiri)]);
    t.resolve_all();
    assert!(in_hand(&t, kiri));

    let mut t = TestGame::new(2);
    let kavu = t.enter(P0, "Flametongue Kavu");
    t.settle();
    let trig = top(&t);
    assert_eq!(targets_of(&t, trig), vec![obj(kavu)]);
    t.resolve_all();
    assert!(in_graveyard(&t, kavu));
    // With another creature around, it can still target itself.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let kavu = t.enter(P0, "Flametongue Kavu");
    t.answer_targets(P0, &[obj(kavu)]);
    t.settle();
    let trig = top(&t);
    assert_eq!(targets_of(&t, trig), vec![obj(kavu)]);
    t.resolve_all();
    assert!(in_graveyard(&t, kavu));
    assert!(t.on_battlefield(bears));
}

#[test]
fn wizard_mentor_can_target_itself() {
    cr!("115.1", "608.2b");
    ruling!("Wizard Mentor", "Can target itself.");
    supported("Wizard Mentor");
    let mut t = TestGame::new(2);
    let mentor = t.battlefield(P0, "Wizard Mentor");
    t.activate(P0, mentor, 0, &[obj(mentor)]).unwrap();
    assert_eq!(targets_of(&t, top(&t)), vec![obj(mentor)]);
    t.resolve_all();
    assert!(in_hand(&t, mentor));
    assert_eq!(t.zone(mentor), Zone::Hand(P0));
}
