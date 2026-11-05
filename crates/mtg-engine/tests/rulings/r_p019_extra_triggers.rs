//! Rulings batch P019 — abilities that trigger an additional time (CR 603.2d): Virtue of
//! Knowledge ("If a permanent entering causes a triggered ability of a permanent you
//! control to trigger, that ability triggers an additional time."), Delney, Streetwise
//! Lookout, Echoes of Eternity, Fractured Realm and Wizard's Staff; and abilities that
//! stop entering permanents from causing triggers (Torpor Orb, Doorkeeper Thrull). They
//! see keyword triggered abilities too, and don't touch replacement effects or "as ...
//! enters" abilities (CR 614.1c, 614.12).

use crate::r_p019_common::*;
use crate::r_s01_common::{supported, with_subtype};
use crate::r_s06_common::attach_new;
use crate::r_s25_common::{abilities_from, cast_new};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const VIRTUE: &str = "Virtue of Knowledge // Vantress Visions";

/// `p` puts the card onto the battlefield (its enters abilities trigger) and the triggers
/// are put on the stack.
fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.settle();
    id
}

#[test]
fn virtue_of_knowledge_doesnt_affect_as_enters_abilities() {
    cr!("614.12", "603.2d");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "Abilities that apply \"as [this permanent] enters the battlefield,\" such as choosing a number with Talion, the Kindly Lord, are also unaffected."
    );
    supported("Talion, the Kindly Lord");
    let mut t = TestGame::new(2);
    t.battlefield(P0, VIRTUE);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    enter(&mut t, P0, "Talion, the Kindly Lord");
    assert_eq!(
        asked_count(&t, from, |d| matches!(d, Decision::ChooseNumber { .. })),
        1
    );
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn virtue_of_knowledge_sees_itself_entering() {
    cr!("603.2d", "603.10");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "If a permanent entering the battlefield at the same time as Virtue of Knowledge (including Virtue of Knowledge itself) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    supported("Eidolon of Blossoms");
    // Eidolon of Blossoms: "Constellation — Whenever this creature or another enchantment
    // you control enters, draw a card."
    let mut t = TestGame::new(2);
    let eidolon = t.battlefield(P0, "Eidolon of Blossoms");
    enter(&mut t, P0, VIRTUE);
    assert_eq!(abilities_from(&t, eidolon).len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn virtue_of_knowledge_copies_stack_one_extra_time_each() {
    cr!("603.2d");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "If you control two copies of Virtue of Knowledge, a permanent entering the battlefield causes abilities to trigger three times, not four. A third Virtue of Knowledge causes abilities to trigger four times, a fourth causes abilities to trigger five times, and so on."
    );
    supported("Elvish Visionary");
    // Elvish Visionary: "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, VIRTUE);
    t.battlefield(P0, VIRTUE);
    let visionary = enter(&mut t, P0, "Elvish Visionary");
    assert_eq!(abilities_from(&t, visionary).len(), 3);
    t.battlefield(P0, VIRTUE);
    let second = enter(&mut t, P0, "Elvish Visionary");
    assert_eq!(abilities_from(&t, second).len(), 4);
}

#[test]
fn virtue_of_knowledge_doesnt_affect_a_two_headed_giant_teammates_abilities() {
    cr!("603.2d");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "In a Two-Headed Giant game, Virtue of Knowledge does not affect triggered abilities of permanents your teammate controls."
    );
    // P0 and P1 are teammates. P1's Elvish Visionary triggers once; P0's twice.
    let mut t = crate::r_p050_common::two_headed_giant();
    t.battlefield(P0, VIRTUE);
    let theirs = enter(&mut t, P1, "Elvish Visionary");
    assert_eq!(abilities_from(&t, theirs).len(), 1);
    t.resolve_all();
    let mine = enter(&mut t, P0, "Elvish Visionary");
    assert_eq!(abilities_from(&t, mine).len(), 2);
}

#[test]
fn virtue_of_knowledge_doesnt_affect_replacement_effects() {
    cr!("614.1c", "603.2d");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "Replacement effects are unaffected by Virtue of Knowledge's ability. For example, a creature that enters the battlefield under your control with one +1/+1 counter on it won't receive an additional +1/+1 counter."
    );
    supported("Festercreep");
    // Festercreep: "This creature enters with a +1/+1 counter on it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, VIRTUE);
    let creep = enter(&mut t, P0, "Festercreep");
    assert_eq!(t.counters(creep, counters::PLUS1), 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn virtue_of_knowledge_doubles_the_entering_permanents_own_and_others_triggers() {
    cr!("603.2d", "603.6a", "702.100a", "603.4");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "Virtue of Knowledge's ability affects a permanent's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that permanent enters the battlefield. Such triggered abilities start with \"when\" or \"whenever.\" Some keyword abilities also include a triggered ability that happens when a permanent enters the battlefield."
    );
    supported("Soul Warden");
    supported("Cloudfin Raptor");
    // Elvish Visionary's own "When this creature enters, draw a card", Soul Warden's
    // "Whenever another creature enters, you gain 1 life" and Cloudfin Raptor's evolve
    // ("Whenever a creature you control enters, if that creature has greater power or
    // toughness than this creature, put a +1/+1 counter on this creature.").
    let mut t = TestGame::new(2);
    t.battlefield(P0, VIRTUE);
    let warden = t.battlefield(P0, "Soul Warden");
    let raptor = t.battlefield(P0, "Cloudfin Raptor");
    let visionary = enter(&mut t, P0, "Elvish Visionary");
    assert_eq!(abilities_from(&t, visionary).len(), 2);
    assert_eq!(abilities_from(&t, warden).len(), 2);
    assert_eq!(abilities_from(&t, raptor).len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 22);
    // Evolve's intervening "if" is checked again on resolution: the second instance
    // finds the Raptor (now 1/2) no smaller than the Visionary (1/1).
    assert_eq!(t.counters(raptor, counters::PLUS1), 1);
}

#[test]
fn virtue_of_knowledge_instances_make_their_choices_separately() {
    cr!("603.2d", "603.3d", "608.2");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "Virtue of Knowledge's ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    supported("Shriekmaw");
    // Shriekmaw: "When this creature enters, destroy target nonartifact, nonblack
    // creature." One target for each instance.
    let mut t = TestGame::new(2);
    t.battlefield(P0, VIRTUE);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    enter(&mut t, P0, "Shriekmaw");
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    // Keldon Raider: "When this creature enters, you may discard a card. If you do, draw a
    // card." — decided for each instance.
    let mut t = TestGame::new(2);
    t.battlefield(P0, VIRTUE);
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    enter(&mut t, P0, "Keldon Raider");
    assert_eq!(t.stack_len(), 2);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(yes_nos(&t, from), 2);
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn extra_trigger_effects_see_keyword_triggered_abilities() {
    cr!("603.2d", "702.108a", "702.92a");
    ruling!(
        "Delney, Streetwise Lookout",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities are triggered abilities and will have \"when,\" \"whenever,\" or \"at the beginning of\" in their reminder text."
    );
    ruling!(
        "Mirror Room // Fractured Realm",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities are triggered abilities and will have \"when,\" \"whenever,\" or \"at the beginning of\" in their reminder text."
    );
    ruling!(
        "Wizard's Staff",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities are triggered abilities and will have \"when,\" \"whenever,\" or \"at the beginning of\" in their reminder text."
    );
    ruling!(
        "Echoes of Eternity",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities are triggered abilities and will have \"when,\" \"whenever,\" or \"at the beginning of\" in their reminder text."
    );
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities are triggered abilities and will have \"when,\" \"whenever,\" or \"at the beginning of\" in their reminder text."
    );
    supported("Delney, Streetwise Lookout");
    supported("Mirror Room // Fractured Realm");
    supported("Wizard's Staff");
    supported("Echoes of Eternity");
    // Prowess (Monastery Swiftspear, 1/2) triggers twice under Delney ("a creature you
    // control with power 2 or less") and Fractured Realm ("a permanent you control").
    for setup in ["Delney", "Fractured Realm"] {
        let mut t = TestGame::new(2);
        match setup {
            "Delney" => {
                t.battlefield(P0, "Delney, Streetwise Lookout");
            }
            _ => {
                let room = t.battlefield(P0, "Mirror Room // Fractured Realm");
                assert!(mtg_engine::rooms::unlock(&mut t.g, room, 1, P0));
                t.settle();
                t.resolve_all();
            }
        }
        let swiftspear = t.battlefield(P0, "Monastery Swiftspear");
        cast_new(&mut t, P0, "Angel's Mercy", &[]);
        t.settle();
        assert_eq!(abilities_from(&t, swiftspear).len(), 2, "{setup}");
        t.resolve_all();
        assert_eq!(t.pt(swiftspear), (3, 4), "{setup}");
    }
    // Wizard's Staff: "Equipped creature has prowess. If a triggered ability of equipped
    // creature triggers, that ability triggers an additional time."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Wizard's Staff", bears);
    cast_new(&mut t, P0, "Angel's Mercy", &[]);
    t.settle();
    assert_eq!(abilities_from(&t, bears).len(), 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    // Echoes of Eternity ("a triggered ability of ... another colorless permanent you
    // control"): Flayer Husk's living weapon triggers twice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoes of Eternity");
    let husk = enter(&mut t, P0, "Flayer Husk");
    assert_eq!(abilities_from(&t, husk).len(), 2);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Germ").len(), 1);
    // Virtue of Knowledge: Cloudfin Raptor's evolve triggers twice as the Bears enter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, VIRTUE);
    let raptor = t.battlefield(P0, "Cloudfin Raptor");
    enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(abilities_from(&t, raptor).len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(raptor, counters::PLUS1), 2);
}

#[test]
fn torpor_orb_and_doorkeeper_thrull_stop_keyword_enters_triggers() {
    cr!("603.2", "702.100a", "702.92a");
    ruling!(
        "Torpor Orb",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities are triggered abilities and will have \"when,\" \"whenever,\" or \"at the beginning of\" in their reminder text."
    );
    ruling!(
        "Doorkeeper Thrull",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities are triggered abilities and will have \"when,\" \"whenever,\" or \"at the beginning of\" in their reminder text."
    );
    supported("Torpor Orb");
    supported("Doorkeeper Thrull");
    // Torpor Orb ("Creatures entering don't cause abilities to trigger."): Cloudfin
    // Raptor's evolve doesn't trigger as the Bears enter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Torpor Orb");
    t.battlefield(P0, "Cloudfin Raptor");
    enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.stack_len(), 0);
    // Doorkeeper Thrull ("Artifacts and creatures entering don't cause abilities to
    // trigger."): Flayer Husk's living weapon doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doorkeeper Thrull");
    enter(&mut t, P0, "Flayer Husk");
    assert_eq!(t.stack_len(), 0);
    assert!(with_subtype(&t, P0, "Germ").is_empty());
    // Without them, both trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cloudfin Raptor");
    enter(&mut t, P0, "Grizzly Bears");
    enter(&mut t, P0, "Flayer Husk");
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn fractured_realm_doesnt_affect_as_enters_abilities() {
    cr!("614.12", "603.2d");
    ruling!(
        "Mirror Room // Fractured Realm",
        "Abilities that apply \"as [this creature] enters\" or \"as [this creature] is turned face up\" are also unaffected by Fractured Realm's ability."
    );
    let mut t = TestGame::new(2);
    let room = t.battlefield(P0, "Mirror Room // Fractured Realm");
    assert!(mtg_engine::rooms::unlock(&mut t.g, room, 1, P0));
    t.settle();
    t.resolve_all();
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    enter(&mut t, P0, "Talion, the Kindly Lord");
    assert_eq!(
        asked_count(&t, from, |d| matches!(d, Decision::ChooseNumber { .. })),
        1
    );
    assert_eq!(t.stack_len(), 0);
}
