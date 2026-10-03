//! CR 702.93 Undying.

use crate::common_k702_011_017::{assert_supported, bf, custom_card};
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::{destroy, run_effect, stack_triggers};
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
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

fn on_bf(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
}

#[test]
fn undying_returns_it_with_a_plus_one_counter() {
    cr!("702.93", "702.93a");
    assert_supported("Strangleroot Geist");
    let mut t = TestGame::new(2);
    let geist = t.battlefield(P0, "Strangleroot Geist");
    t.g.objects[geist.0 as usize].damage = 1;
    destroy(&mut t, geist);
    t.settle();
    assert_eq!(stack_triggers(&t, "Undying").len(), 1);
    t.resolve();
    let back = on_bf(&t, "Strangleroot Geist");
    assert_eq!(back.len(), 1);
    let back = back[0];
    assert_ne!(back, geist);
    assert_eq!(t.g.obj(back).controller, P0);
    assert_eq!(t.g.obj(back).counter(counters::PLUS1), 1);
    assert_eq!(t.pt(back), (3, 2));
    assert_eq!(t.g.obj(back).damage, 0);
}

#[test]
fn undying_doesnt_return_it_if_it_had_a_plus_one_counter() {
    cr!("702.93a");
    let mut t = TestGame::new(2);
    let geist = t.battlefield(P0, "Strangleroot Geist");
    destroy(&mut t, geist);
    t.resolve_all();
    let back = on_bf(&t, "Strangleroot Geist")[0];
    destroy(&mut t, back);
    t.settle();
    assert!(stack_triggers(&t, "Undying").is_empty());
    t.resolve_all();
    assert!(on_bf(&t, "Strangleroot Geist").is_empty());
    assert!(t.in_graveyard(P0, "Strangleroot Geist"));
}

#[test]
fn counters_annihilated_by_minus_one_counters_let_it_return_again() {
    cr!("702.93a", "704.5q");
    ruling!(
        "Endling",
        "Endling's undying ability can bring it back again if its +1/+1 counters are removed this way."
    );
    let mut t = TestGame::new(2);
    let geist = t.battlefield(P0, "Strangleroot Geist");
    destroy(&mut t, geist);
    t.resolve_all();
    let back = on_bf(&t, "Strangleroot Geist")[0];
    put_counters(&mut t, back, counters::MINUS1, 1);
    t.settle();
    assert_eq!(t.g.obj(back).counter(counters::PLUS1), 0);
    destroy(&mut t, back);
    t.settle();
    assert_eq!(stack_triggers(&t, "Undying").len(), 1);
    t.resolve_all();
    assert_eq!(on_bf(&t, "Strangleroot Geist").len(), 1);
}

#[test]
fn a_creature_killed_by_minus_one_counters_despite_plus_one_counters_doesnt_return() {
    cr!("702.93a");
    ruling!(
        "Endling",
        "undying won't trigger and the card won't return to the battlefield. That's because undying checks the creature as it last existed on the battlefield"
    );
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Young Wolf");
    put_counters(&mut t, wolf, counters::PLUS1, 1);
    t.settle();
    // Three -1/-1 counters on a 2/2: 0/0 with both kinds of counters on it as it dies.
    put_counters(&mut t, wolf, counters::MINUS1, 3);
    t.settle();
    assert!(t.in_graveyard(P0, "Young Wolf"));
    assert!(stack_triggers(&t, "Undying").is_empty());
}

#[test]
fn a_pump_that_isnt_a_counter_doesnt_stop_undying() {
    cr!("702.93a");
    ruling!(
        "Evernight Shade",
        "If Evernight Shade has no +1/+1 counters on it and dies after its ability has resolved, undying will still return it to the battlefield."
    );
    assert_supported("Evernight Shade");
    let mut t = TestGame::new(2);
    let shade = t.battlefield(P0, "Evernight Shade");
    t.lands(P0, "Swamp", 1);
    activate_named(&mut t, P0, shade, "{B}: ~ gets +1/+1 until end of turn.", 0).unwrap();
    t.resolve();
    assert_eq!(t.pt(shade), (2, 2));
    destroy(&mut t, shade);
    t.settle();
    assert_eq!(stack_triggers(&t, "Undying").len(), 1);
    t.resolve_all();
    let back = on_bf(&t, "Evernight Shade");
    assert_eq!(back.len(), 1);
    assert_eq!(t.pt(back[0]), (2, 2));
}

#[test]
fn undying_does_nothing_if_the_card_left_the_graveyard() {
    cr!("702.93a");
    ruling!(
        "Endling",
        "If a card leaves the graveyard after it dies but before the undying trigger resolves, it won't be returned to the battlefield."
    );
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Young Wolf");
    destroy(&mut t, wolf);
    t.settle();
    let in_gy = t.g.current(wolf);
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
    assert!(on_bf(&t, "Young Wolf").is_empty());
    assert!(t.in_exile("Young Wolf"));
}

#[test]
fn redundant_instances_of_undying_return_it_once() {
    cr!("702.93a");
    ruling!(
        "Endling",
        "If a creature has multiple instances of undying, they'll each trigger separately, but once one of those abilities returns the card to the battlefield, any others will have no effect. The creature won't receive multiple +1/+1 counters."
    );
    let def = custom_card(
        "Twice-Undying Wolf",
        "Creature — Wolf",
        Some((1, 1)),
        "Undying\nUndying",
    );
    let mut t = TestGame::new(2);
    let wolf = bf(&mut t, P0, def);
    destroy(&mut t, wolf);
    t.settle();
    assert_eq!(stack_triggers(&t, "Undying").len(), 2);
    t.resolve_all();
    let back = on_bf(&t, "Twice-Undying Wolf");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).counter(counters::PLUS1), 1);
}

#[test]
fn a_token_with_undying_cant_return() {
    cr!("702.93a");
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Young Wolf");
    run_effect(
        &mut t,
        Some(wolf),
        P0,
        Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &[Entity::Object(wolf)],
    );
    t.resolve_all();
    let token = t
        .g
        .permanents()
        .find(|o| o.is_token() && o.chars.name == "Young Wolf")
        .map(|o| o.id)
        .expect("token copy");
    destroy(&mut t, token);
    t.settle();
    assert_eq!(stack_triggers(&t, "Undying").len(), 1);
    t.resolve_all();
    assert_eq!(on_bf(&t, "Young Wolf"), vec![wolf]);
}

#[test]
fn undying_granted_by_mikaeus_returns_creatures_dying_with_it() {
    cr!("702.93a", "603.10a");
    ruling!(
        "Mikaeus, the Unhallowed",
        "If a non-Human creature you control without a +1/+1 counter dies at the same time as Mikaeus, that creature’s undying ability granted by Mikaeus triggers and will return it to the battlefield."
    );
    ruling!(
        "Mikaeus, the Unhallowed",
        "The +1/+1 bonus that Mikaeus gives to other non-Human creatures you control isn’t a counter. It won’t prevent undying from triggering."
    );
    assert_supported("Mikaeus, the Unhallowed");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mikaeus, the Unhallowed");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let human = t.battlefield(P0, "Elite Vanguard");
    assert_eq!(t.pt(bears), (3, 3));
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
    // Only the Bears had undying: Mikaeus doesn't grant it to itself, nor to Humans.
    assert_eq!(stack_triggers(&t, "Undying").len(), 1);
    t.resolve_all();
    let back = on_bf(&t, "Grizzly Bears");
    assert_eq!(back.len(), 1);
    // Mikaeus is gone: just the +1/+1 counter.
    assert_eq!(t.pt(back[0]), (3, 3));
    assert!(t.in_graveyard(P0, "Elite Vanguard"));
    assert!(t.in_graveyard(P0, "Mikaeus, the Unhallowed"));
    let _ = human;
}

#[test]
fn a_returned_creatures_enters_abilities_trigger_again() {
    cr!("702.93a");
    ruling!(
        "Demonlord of Ashmouth",
        "Demonlord of Ashmouth's \"enters\" ability will trigger no matter how it entered, including because of its undying ability."
    );
    assert_supported("Geralf's Messenger");
    let mut t = TestGame::new(2);
    let messenger = t.battlefield(P0, "Geralf's Messenger");
    destroy(&mut t, messenger);
    t.settle();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    let back = on_bf(&t, "Geralf's Messenger")[0];
    // It enters tapped (a replacement effect), and its enters ability triggers.
    assert!(t.g.obj(back).tapped);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(back), (4, 3));
}
