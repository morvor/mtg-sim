//! CR 702.79 Persist.

use crate::common_k702_011_017::{assert_supported, bf, custom_card};
use crate::common_k702_052_066::{destroy, run_effect, stack_triggers};
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;
use smol_str::SmolStr;

fn put_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: i32) {
    let id = t.g.current(id);
    run_effect(
        t,
        None,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: SmolStr::new(kind),
            n: Value::c(n),
        },
        &[Entity::Object(id)],
    );
}

/// The permanents named `name` on the battlefield.
fn on_bf(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
}

#[test]
fn persist_returns_it_with_a_minus_one_counter() {
    cr!("702.79", "702.79a");
    ruling!(
        "Murderous Redcap",
        "When a permanent with persist returns to the battlefield, it’s a new object with no memory of or connection to its previous existence."
    );
    ruling!(
        "Kitchen Finks",
        "When a permanent with persist returns to the battlefield, it's a new object with no memory of or connection to its previous existence."
    );
    assert_supported("Kitchen Finks");
    let mut t = TestGame::new(2);
    let finks = t.battlefield(P0, "Kitchen Finks");
    t.g.objects[finks.0 as usize].tapped = true;
    t.g.objects[finks.0 as usize].damage = 1;
    destroy(&mut t, finks);
    t.settle();
    assert_eq!(stack_triggers(&t, "Persist").len(), 1);
    t.resolve();
    let back = on_bf(&t, "Kitchen Finks");
    assert_eq!(back.len(), 1);
    let back = back[0];
    assert_ne!(back, finks);
    assert_eq!(t.g.obj(back).controller, P0);
    assert_eq!(t.g.obj(back).counter(counters::MINUS1), 1);
    assert_eq!(t.pt(back), (2, 1));
    // A new object: untapped, undamaged, summoning sick.
    assert!(!t.g.obj(back).tapped);
    assert_eq!(t.g.obj(back).damage, 0);
    assert!(t.g.obj(back).summoning_sick);
    // Its enters-the-battlefield ability triggers again.
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn persist_doesnt_return_it_if_it_had_a_minus_one_counter() {
    cr!("702.79a");
    ruling!(
        "Kitchen Finks",
        "Its last known information (that is, how the creature last existed on the battlefield) is used to determine whether it had a -1/-1 counter on it."
    );
    let mut t = TestGame::new(2);
    let finks = t.battlefield(P0, "Kitchen Finks");
    destroy(&mut t, finks);
    t.resolve_all();
    let back = on_bf(&t, "Kitchen Finks")[0];
    destroy(&mut t, back);
    t.settle();
    // The card in the graveyard has no counters, but the permanent had one.
    assert_eq!(t.obj_now(back).counter(counters::MINUS1), 0);
    assert!(stack_triggers(&t, "Persist").is_empty());
    t.resolve_all();
    assert!(on_bf(&t, "Kitchen Finks").is_empty());
    assert!(t.in_graveyard(P0, "Kitchen Finks"));
}

#[test]
fn a_plus_one_counter_cancelling_the_minus_one_counter_lets_it_persist_again() {
    cr!("702.79a", "704.5q");
    let mut t = TestGame::new(2);
    let finks = t.battlefield(P0, "Kitchen Finks");
    destroy(&mut t, finks);
    t.resolve_all();
    let back = on_bf(&t, "Kitchen Finks")[0];
    put_counters(&mut t, back, counters::PLUS1, 1);
    t.settle();
    // The counters annihilate as a state-based action.
    assert_eq!(t.g.obj(back).counter(counters::MINUS1), 0);
    assert_eq!(t.g.obj(back).counter(counters::PLUS1), 0);
    destroy(&mut t, back);
    t.settle();
    assert_eq!(stack_triggers(&t, "Persist").len(), 1);
    t.resolve_all();
    assert_eq!(on_bf(&t, "Kitchen Finks").len(), 1);
}

#[test]
fn a_creature_killed_by_minus_one_counters_despite_plus_one_counters_doesnt_persist() {
    cr!("702.79a");
    ruling!(
        "Murderous Redcap",
        "persist won’t trigger and the card won’t return to the battlefield"
    );
    ruling!(
        "Kitchen Finks",
        "persist won't trigger and the card won't return to the battlefield. That's because persist checks the creature's existence just before it leaves the battlefield"
    );
    let mut t = TestGame::new(2);
    let finks = t.battlefield(P0, "Kitchen Finks");
    put_counters(&mut t, finks, counters::PLUS1, 1);
    t.settle();
    assert_eq!(t.pt(finks), (4, 3));
    // Three -1/-1 counters: 1/0 with both kinds of counters on it.
    put_counters(&mut t, finks, counters::MINUS1, 3);
    t.settle();
    assert!(t.in_graveyard(P0, "Kitchen Finks"));
    assert!(stack_triggers(&t, "Persist").is_empty());
    t.resolve_all();
    assert!(on_bf(&t, "Kitchen Finks").is_empty());
}

#[test]
fn a_persisted_creature_enters_with_the_counter_already_on_it() {
    cr!("702.79a");
    assert_supported("Murderous Redcap");
    let mut t = TestGame::new(2);
    // Murderous Redcap (2/2): "When this creature enters, it deals damage equal to its
    // power to any target."
    let redcap = t.battlefield(P0, "Murderous Redcap");
    destroy(&mut t, redcap);
    t.settle();
    assert_eq!(stack_triggers(&t, "Persist").len(), 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    let back = on_bf(&t, "Murderous Redcap")[0];
    assert_eq!(t.pt(back), (1, 1));
    // It deals damage equal to its power now: 1.
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn a_token_with_persist_cant_return() {
    cr!("702.79a");
    ruling!(
        "Murderous Redcap",
        "However, the token will cease to exist and can’t return to the battlefield."
    );
    ruling!(
        "Kitchen Finks",
        "If a token with no -1/-1 counters on it has persist, the ability will trigger when the token is put into the graveyard. However, the token will cease to exist and can't return to the battlefield."
    );
    let mut t = TestGame::new(2);
    let finks = t.battlefield(P0, "Kitchen Finks");
    run_effect(
        &mut t,
        Some(finks),
        P0,
        Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &[Entity::Object(finks)],
    );
    t.resolve_all();
    let token = t
        .g
        .permanents()
        .find(|o| o.is_token() && o.chars.name == "Kitchen Finks")
        .map(|o| o.id)
        .expect("token copy");
    destroy(&mut t, token);
    t.settle();
    assert_eq!(stack_triggers(&t, "Persist").len(), 1);
    t.resolve_all();
    assert_eq!(on_bf(&t, "Kitchen Finks"), vec![finks]);
}

#[test]
fn redundant_instances_of_persist_trigger_but_return_it_once() {
    cr!("702.79a");
    ruling!(
        "Murderous Redcap",
        "they’ll each trigger separately, but the redundant instances will have no effect"
    );
    ruling!(
        "Kitchen Finks",
        "If a permanent has multiple instances of persist, they'll each trigger separately, but the redundant instances will have no effect."
    );
    let def = custom_card(
        "Twice-Persistent Ouphe",
        "Creature — Ouphe",
        Some((2, 2)),
        "Persist\nPersist",
    );
    let mut t = TestGame::new(2);
    let ouphe = bf(&mut t, P0, def);
    destroy(&mut t, ouphe);
    t.settle();
    assert_eq!(stack_triggers(&t, "Persist").len(), 2);
    t.resolve_all();
    let back = on_bf(&t, "Twice-Persistent Ouphe");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).counter(counters::MINUS1), 1);
}

#[test]
fn persist_works_on_a_permanent_that_stopped_being_a_creature() {
    cr!("702.79a");
    ruling!(
        "Kitchen Finks",
        "If a creature with persist stops being a creature, persist will still work."
    );
    let mut t = TestGame::new(2);
    let finks = t.battlefield(P0, "Kitchen Finks");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::RemoveTypes(vec![CardType::Creature]),
                Modification::AddTypes(vec![CardType::Artifact]),
            ],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(finks)],
    );
    assert!(!t.obj_now(finks).is(CardType::Creature));
    destroy(&mut t, finks);
    t.settle();
    assert_eq!(stack_triggers(&t, "Persist").len(), 1);
    t.resolve();
    let back = on_bf(&t, "Kitchen Finks");
    assert_eq!(back.len(), 1);
    assert!(t.g.obj(back[0]).is(CardType::Creature));
}

#[test]
fn persist_does_nothing_if_the_card_left_the_graveyard() {
    cr!("702.79a");
    let mut t = TestGame::new(2);
    let finks = t.battlefield(P0, "Kitchen Finks");
    destroy(&mut t, finks);
    t.settle();
    assert_eq!(stack_triggers(&t, "Persist").len(), 1);
    // In response, the card is exiled from the graveyard.
    let in_gy = t.g.current(finks);
    assert_eq!(t.g.obj(in_gy).zone, Zone::Graveyard(P0));
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Exile {
            what: Sel::Target(0),
            face_down: false,
            link: false,
        },
        &[Entity::Object(in_gy)],
    );
    t.resolve_all();
    assert!(on_bf(&t, "Kitchen Finks").is_empty());
    assert!(t.in_exile("Kitchen Finks"));
}

#[test]
fn simultaneous_persist_triggers_are_put_on_the_stack_in_apnap_order() {
    cr!("702.79a", "603.3b");
    ruling!(
        "Murderous Redcap",
        "the nonactive player’s persist creatures will return to the battlefield first"
    );
    ruling!(
        "Kitchen Finks",
        "That means that in a two-player game, the nonactive player's persist creatures will return to the battlefield first"
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Kitchen Finks");
    let theirs = t.battlefield(P1, "Kitchen Finks");
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::All(Filter::creature()),
            no_regen: false,
        },
        &[],
    );
    t.settle();
    let trig = stack_triggers(&t, "Persist");
    assert_eq!(trig.len(), 2);
    // The nonactive player's trigger is on top.
    t.resolve();
    let back = on_bf(&t, "Kitchen Finks");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).owner, P1);
    let _ = (mine, theirs);
}

#[test]
fn persist_granted_by_an_effect_works_for_each_creature() {
    cr!("702.79a");
    assert_supported("Cauldron Haze");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 2);
    let haze = t.hand(P0, "Cauldron Haze");
    // "Choose any number of target creatures. Each of those creatures gains persist
    // until end of turn."
    t.cast(P0, haze)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    t.resolve_all();
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::All(Filter::creature()),
            no_regen: false,
        },
        &[],
    );
    t.settle();
    assert_eq!(stack_triggers(&t, "Persist").len(), 2);
    t.resolve_all();
    let bears_back = on_bf(&t, "Grizzly Bears");
    let giant_back = on_bf(&t, "Hill Giant");
    assert_eq!(bears_back.len(), 1);
    assert_eq!(giant_back.len(), 1);
    assert_eq!(t.pt(giant_back[0]), (2, 2));
    assert_eq!(t.pt(bears_back[0]), (1, 1));
    // The returned creatures are new objects: they don't have persist any more.
    destroy(&mut t, bears_back[0]);
    t.settle();
    assert!(stack_triggers(&t, "Persist").is_empty());
}

#[test]
fn a_persisting_creatures_enters_ability_uses_its_last_known_power() {
    cr!("702.79a", "608.2h");
    ruling!(
        "Murderous Redcap",
        "If it’s left the battlefield by then, its last known information is used."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // The Redcap enters (2/2); in response to its trigger it gets +2/+0 and then dies:
    // its power as it last existed (4) is used.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let redcap = t.enter(P0, "Murderous Redcap");
    t.settle();
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(redcap)],
    );
    destroy(&mut t, redcap);
    t.settle();
    // Persist triggered on top: resolve it (the returned Redcap's trigger targets the
    // Bears), then the first trigger.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // The returned Redcap (1/1) dealt 1 damage to the Bears.
    assert_eq!(t.obj_now(bears).damage, 1);
}
