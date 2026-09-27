//! CR 702.168 Disguise: what disguised cards do as they're turned face up (CR 702.168d).

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::decision::SpecialAction;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const DISGUISE: CastMethod = CastMethod::FaceDown(KeywordKind::Disguise);

/// `name` from P0's hand, cast face down for {3} and resolved: the face-down permanent.
fn disguised(t: &mut TestGame, name: &str) -> ObjectId {
    let card = t.hand(P0, name);
    add_mana(t, P0, ManaType::C, 3);
    let spell = t.cast(P0, card).method(DISGUISE).go();
    t.resolve_all();
    let id = t.g.current(spell);
    assert!(t.obj(id).face_down && t.on_battlefield(id));
    id
}

/// Turns P0's face-down `obj` face up, with `mana` in the pool for its disguise cost.
fn turn_up(t: &mut TestGame, obj: ObjectId, mana: &[(ManaType, u32)]) {
    for (m, n) in mana {
        add_mana(t, P0, *m, *n);
    }
    take_special(t, P0, SpecialAction::TurnFaceUp { obj }).expect("turned face up");
    assert!(!t.obj(obj).face_down);
}

#[test]
fn nervous_gardener_searches_for_a_land_with_a_basic_land_type() {
    cr!("702.168d");
    assert_supported("Nervous Gardener");
    // Nervous Gardener: disguise {G}, "When this creature is turned face up, search your
    // library for a land card with a basic land type, reveal it, put it into your hand,
    // then shuffle."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Wastes");
    let gardener = disguised(&mut t, "Nervous Gardener");
    turn_up(&mut t, gardener, &[(ManaType::G, 1)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert!(!t.in_hand(P0, "Wastes"));
}

#[test]
fn faerie_snoop_puts_one_card_into_your_hand_and_the_other_into_your_graveyard() {
    cr!("702.168d");
    assert_supported("Faerie Snoop");
    // Faerie Snoop: disguise {1}{U/B}{U/B}, "When this creature is turned face up, look at
    // the top two cards of your library. Put one into your hand and the other into your
    // graveyard."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Forest");
    let giant = t.library_top(P0, "Hill Giant");
    let snoop = disguised(&mut t, "Faerie Snoop");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    turn_up(&mut t, snoop, &[(ManaType::U, 2), (ManaType::C, 1)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn experiment_twelve_puts_counters_equal_to_power_on_creatures_turned_face_up() {
    cr!("702.168d");
    assert_supported("Experiment Twelve");
    // Experiment Twelve: 4/4 trample, disguise {4}{G}, "Whenever this creature or another
    // creature you control is turned face up, put +1/+1 counters on that creature equal to
    // its power."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let twelve = disguised(&mut t, "Experiment Twelve");
    turn_up(&mut t, twelve, &[(ManaType::G, 5)]);
    t.resolve_all();
    assert_eq!(t.pt(twelve), (8, 8));
    // Nightdrinker Moroii: a 4/2 with disguise {B}{B}.
    let moroii = disguised(&mut t, "Nightdrinker Moroii");
    turn_up(&mut t, moroii, &[(ManaType::B, 2)]);
    t.resolve_all();
    assert_eq!(t.counters(moroii, "+1/+1"), 4);
    assert_eq!(t.counters(twelve, "+1/+1"), 4);
}

#[test]
fn printlifter_ooze_creates_an_ooze_that_enters_with_counters() {
    cr!("702.168d");
    assert_supported("Printlifter Ooze");
    // Printlifter Ooze: "Whenever this creature or another creature you control is turned
    // face up, create a 0/0 green Ooze creature token with trample. The token enters with X
    // +1/+1 counters on it, where X is the number of other creatures you control."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Printlifter Ooze");
    t.battlefield(P1, "Grizzly Bears");
    let moroii = disguised(&mut t, "Nightdrinker Moroii");
    turn_up(&mut t, moroii, &[(ManaType::B, 2)]);
    t.resolve_all();
    let oozes: Vec<ObjectId> = tokens_of_subtype(&t, P0, "Ooze");
    assert_eq!(oozes.len(), 1);
    // The Printlifter Ooze and the Moroii.
    assert_eq!(t.counters(oozes[0], "+1/+1"), 2);
    assert_eq!(t.pt(oozes[0]), (2, 2));
    assert!(t.obj(oozes[0]).chars.has_keyword(KeywordKind::Trample));
}

#[test]
fn crowd_control_warden_gets_counters_as_it_enters_or_is_turned_face_up() {
    cr!("702.168d", "708.8");
    assert_supported("Crowd-Control Warden");
    // Crowd-Control Warden: 4/4, "As this creature enters or is turned face up, put X +1/+1
    // counters on it, where X is the number of other creatures you control." Disguise
    // {3}{G/W}{G/W}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Grizzly Bears");
    // Cast face up: two other creatures (the Bears and the face-down one below).
    let fd = disguised(&mut t, "Nightdrinker Moroii");
    let warden = t.hand(P0, "Crowd-Control Warden");
    add_mana(&mut t, P0, ManaType::G, 4);
    add_mana(&mut t, P0, ManaType::W, 1);
    let spell = t.cast(P0, warden).go();
    t.resolve_all();
    let warden = t.g.current(spell);
    assert_eq!(t.counters(warden, "+1/+1"), 2);
    // Turned face up: it enters face down with none, and gets them as it's turned face up
    // (three other creatures now).
    let hidden = disguised(&mut t, "Crowd-Control Warden");
    assert_eq!(t.counters(hidden, "+1/+1"), 0);
    turn_up(&mut t, hidden, &[(ManaType::W, 5)]);
    assert_eq!(t.counters(hidden, "+1/+1"), 3);
    assert_eq!(t.pt(hidden), (7, 7));
    assert!(t.obj(fd).face_down);
}
