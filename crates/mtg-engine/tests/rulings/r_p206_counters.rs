//! Rulings batch P206 — doubling the number of counters on a permanent or player
//! (CR 701.10e): put as many more as are already there, so replacement effects that modify
//! how many counters are put apply; "each kind" doubles every kind. Also experience
//! counters (CR 122.1) and renown (CR 702.112).

use crate::r_p206_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::create_token;
use crate::r_s06_common::activate_containing;
use crate::r_s29_common::put_counters;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// One counter-doubling ability: `setup` builds the board and returns the permanent whose
/// counters will be doubled; `fire` makes the ability happen and resolve.
struct Case {
    card: &'static str,
    /// Doubles each kind of counter (not only +1/+1 counters).
    each_kind: bool,
    /// +1/+1 counters the ability puts on first ("put a +1/+1 counter on it, then double").
    first: u32,
    setup: fn(&mut TestGame) -> ObjectId,
    fire: fn(&mut TestGame, ObjectId),
}

fn giant(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Hill Giant")
}

fn activate(t: &mut TestGame, source: ObjectId, needle: &str, target: Option<ObjectId>) {
    rainbow_pool(t, P0, 10);
    if let Some(s) = target {
        t.answer_targets(P0, &[Entity::Object(s)]);
    }
    activate_containing(t, P0, source, needle).expect("activation failed");
    t.resolve_all();
    empty_pool(t, P0);
}

fn attack_alone(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            card: "Deepglow Skate",
            each_kind: true,
            first: 0,
            setup: giant,
            fire: |t, s| {
                t.answer_targets(P0, &[Entity::Object(s)]);
                t.enter(P0, "Deepglow Skate");
                t.resolve_all();
            },
        },
        Case {
            card: "Vorel of the Hull Clade",
            each_kind: true,
            first: 0,
            setup: giant,
            fire: |t, s| {
                let v = t.battlefield(P0, "Vorel of the Hull Clade");
                activate(t, v, "Double", Some(s));
            },
        },
        Case {
            card: "Ferrafor, Young Yew",
            each_kind: true,
            first: 0,
            setup: giant,
            fire: |t, s| {
                let f = t.battlefield(P0, "Ferrafor, Young Yew");
                activate(t, f, "Double", Some(s));
            },
        },
        Case {
            card: "Arcade Cabinet",
            each_kind: true,
            first: 0,
            setup: giant,
            fire: |t, s| {
                let a = t.battlefield(P0, "Arcade Cabinet");
                create_token(t, P0, "Treasure");
                activate(t, a, "Double", Some(s));
            },
        },
        Case {
            card: "Solarion",
            each_kind: false,
            first: 0,
            setup: |t| t.battlefield(P0, "Solarion"),
            fire: |t, s| activate(t, s, "Double", None),
        },
        Case {
            card: "Kalonian Hydra",
            each_kind: false,
            first: 0,
            setup: |t| t.battlefield(P0, "Kalonian Hydra"),
            fire: attack_alone,
        },
        Case {
            card: "Mossborn Hydra",
            each_kind: false,
            first: 0,
            setup: |t| t.battlefield(P0, "Mossborn Hydra"),
            fire: |t, _| {
                t.enter(P0, "Forest");
                t.resolve_all();
            },
        },
        Case {
            card: "Byrke, Long Ear of the Law",
            each_kind: false,
            first: 0,
            setup: |t| {
                t.battlefield(P0, "Byrke, Long Ear of the Law");
                giant(t)
            },
            fire: attack_alone,
        },
        Case {
            card: "Growth Curve",
            each_kind: false,
            first: 1,
            setup: giant,
            fire: |t, s| {
                crate::r_s25_common::cast_new(t, P0, "Growth Curve", &[Entity::Object(s)]);
                t.resolve_all();
            },
        },
    ]
}

/// Runs the case with 2 +1/+1 counters (and 1 charge counter) on its permanent; with
/// `menace`, Corpsejack Menace doubles the +1/+1 counters put on creatures P0 controls.
fn run_case(c: &Case, menace: bool) {
    supported(c.card);
    let mut t = TestGame::new(2);
    if menace {
        t.battlefield(P0, "Corpsejack Menace");
    }
    let s = (c.setup)(&mut t);
    // Placed directly (not "put"): unaffected by Corpsejack Menace.
    t.g.objects[s.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 2);
    t.g.objects[s.0 as usize]
        .counters
        .insert(counters::CHARGE.into(), 1);
    t.g.recompute();
    (c.fire)(&mut t, s);
    let m = if menace { 2 } else { 1 };
    let before = 2 + c.first * m;
    assert_eq!(
        plus1(&t, s),
        before + before * m,
        "{} (Corpsejack Menace: {menace})",
        c.card
    );
    let charge = if c.each_kind { 2 } else { 1 };
    assert_eq!(t.counters(s, counters::CHARGE), charge, "{}", c.card);
}

#[test]
fn doubling_counters_puts_as_many_more_on_it() {
    cr!("701.10e", "122.1");
    ruling!(
        "Vorel of the Hull Clade",
        "The effect of Vorel's ability will essentially double the counters on the target artifact, creature, or land. For example, if a creature has three +1/+1 counters and a divinity counter on it before the ability resolves, it will have six +1/+1 counters and two divinity counters on it after the ability resolves."
    );
    ruling!(
        "Arcade Cabinet",
        "To double the number of a kind of counters on a permanent, put a number of that kind of counters on it equal to the number it already has. Other cards that interact with putting counters on it will interact with this effect accordingly."
    );
    ruling!(
        "Ferrafor, Young Yew",
        "To double the number of each kind of counter on a creature, put another counter on it for each counter it already has. Effects that interact with counters being put onto creatures apply as appropriate."
    );
    ruling!(
        "Deepglow Skate",
        "To double the number of each kind of counter on a permanent, put another counter on it for each counter it already has. Effects that interact with counters being put onto permanents apply as appropriate."
    );
    ruling!(
        "Solarion",
        "“Double” has its normal English meaning. If Solarion had three +1/+1 counters on it, it would end up with six +1/+1 counters on it."
    );
    for c in cases() {
        run_case(&c, false);
    }
}

#[test]
fn doubling_counters_is_putting_counters_so_replacements_apply() {
    cr!("701.10e", "614.1a");
    ruling!(
        "Mossborn Hydra",
        "To double the number of +1/+1 counters on Mossborn Hydra, put a number of +1/+1 counters on it equal to the number it already has. Other cards that interact with putting counters on it will interact with this effect accordingly."
    );
    ruling!(
        "Kalonian Hydra",
        "To double the number of +1/+1 counters on a creature, determine how many +1/+1 counters are on the creature and put that many more on it. Effects that interact with counters (such as the one created by Corpsejack Menace's ability) may change the number of counters ultimately put on the creature."
    );
    ruling!(
        "Growth Curve",
        "To double the number of +1/+1 counters on a creature, put a number of +1/+1 counters on it equal to the number it already has. Other effects that interact with putting counters on it will interact with this effect accordingly."
    );
    ruling!(
        "Byrke, Long Ear of the Law",
        "To double the number of +1/+1 counters on a creature, put a number of +1/+1 counters on it equal to the number it already has. Replacement effects that modify the number of counters being placed on creatures you control, such as the effect of Branching Evolution, apply to this ability as normal."
    );
    for c in cases() {
        run_case(&c, true);
    }
}

#[test]
fn deepglow_skate_doubles_each_kind_on_each_target() {
    cr!("701.10e", "115.1");
    ruling!(
        "Deepglow Skate",
        "As Deepglow Skate's ability resolves, you must double each kind of counter on the permanents it targets."
    );
    supported("Deepglow Skate");
    let mut t = TestGame::new(2);
    let a = giant(&mut t);
    let b = t.battlefield(P1, "Hill Giant");
    put_counters(&mut t, a, counters::PLUS1, 1);
    put_counters(&mut t, a, counters::STUN, 2);
    put_counters(&mut t, b, counters::MINUS1, 1);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.enter(P0, "Deepglow Skate");
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert_eq!(t.counters(a, counters::STUN), 4);
    assert_eq!(t.counters(b, counters::MINUS1), 2);
}

#[test]
fn deepglow_skate_can_have_no_targets() {
    cr!("115.1");
    ruling!(
        "Deepglow Skate",
        "You can choose no targets if you don't want to double the counters on any permanents."
    );
    let mut t = TestGame::new(2);
    let a = giant(&mut t);
    put_counters(&mut t, a, counters::PLUS1, 1);
    t.answer_targets(P0, &[]);
    t.enter(P0, "Deepglow Skate");
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn elvish_vatkeeper_transforms_the_incubator_before_doubling() {
    cr!("701.10e", "701.53a");
    ruling!(
        "Elvish Vatkeeper",
        "The target Incubator token transforms before its +1/+1 counters are doubled, so anything that cares about +1/+1 counters being placed on a creature will see them being placed on the Phyrexian artifact creature."
    );
    supported("Elvish Vatkeeper");
    let mut t = TestGame::new(2);
    let keeper = t.enter(P0, "Elvish Vatkeeper");
    t.resolve_all();
    let incubator =
        t.g.permanents()
            .find(|o| o.is_token() && o.chars.name.contains("Incubator"))
            .map(|o| o.id)
            .expect("an Incubator token");
    assert_eq!(plus1(&t, incubator), 2);
    // Corpsejack Menace only affects creatures: it sees the counters put on the
    // transformed Phyrexian artifact creature.
    t.battlefield(P0, "Corpsejack Menace");
    activate(
        &mut t,
        keeper,
        "Transform target Incubator",
        Some(incubator),
    );
    assert!(t.obj_now(incubator).is(CardType::Creature));
    assert_eq!(plus1(&t, incubator), 2 + 4);
}

#[test]
fn aragorn_doesnt_double_counters_of_a_creature_that_becomes_renowned_then() {
    cr!("702.112a", "702.112b");
    ruling!(
        "Aragorn, Hornburg Hero",
        "When a creature you control that isn't renowned deals combat damage to a player, you'll put a +1/+1 counter on it and it will become renowned. Since it wasn't renowned when it dealt combat damage, Aragorn, Hornburg Hero's last ability won't trigger, and you won't double the number of +1/+1 counters on it."
    );
    supported("Aragorn, Hornburg Hero");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aragorn, Hornburg Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
    assert!(t.obj_now(bears).renowned);
}

#[test]
fn toph_counts_all_your_experience_counters_and_they_stay_on_you() {
    cr!("122.1", "701.66a");
    ruling!(
        "Toph, Earthbending Master",
        "All experience counters are identical, no matter how you got them. For example, the last ability will count experience counters that you got from the first ability, from another ability, from another copy of Toph, Earthbending Master, and so on."
    );
    ruling!(
        "Toph, Earthbending Master",
        "The experience counter goes on you, the player, not on Toph. You will keep that counter even if Toph, Earthbending Master dies."
    );
    supported("Toph, Earthbending Master");
    let mut t = TestGame::new(2);
    let toph = t.battlefield(P0, "Toph, Earthbending Master");
    let forest = t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter(counters::EXPERIENCE), 1);
    // An experience counter from elsewhere is just the same.
    t.g.add_counters(Entity::Player(P0), counters::EXPERIENCE, 1, None);
    t.answer_targets(P0, &[Entity::Object(forest)]);
    attack_alone(&mut t, toph);
    assert_eq!(plus1(&t, forest), 2);
    // The counters are on the player: Toph dying doesn't remove them.
    crate::r_s02_common::destroy(&mut t, toph);
    assert!(!t.on_battlefield(toph));
    assert_eq!(t.g.player(P0).counter(counters::EXPERIENCE), 2);
}
