//! Rulings batch S10 — kicker (CR 702.33): "You may pay an additional [cost] as you cast
//! this spell." A spell whose kicker cost was paid is "kicked".

use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s02_common::destroy;
use crate::r_s04_common::{add_mana, asked_of_since, hand_names};
use crate::r_s05_common::enter;
use crate::r_s07_common::resolved;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Queues the answer to a "choose a creature type" prompt.
fn choose_creature_type(t: &mut TestGame, p: PlayerId, ty: &str) {
    let i = subtype_lists()
        .creature
        .iter()
        .position(|s| s == ty)
        .expect("creature type");
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn a_copy_of_a_kicked_permanent_spell_becomes_a_kicked_token() {
    cr!("702.33d", "707.10", "707.10f", "608.3f");
    ruling!(
        "Kavu Titan",
        "If you copy a kicked spell on the stack, the copy is also kicked. If the copied spell is a permanent spell, the token the copy of that spell becomes when it enters is also kicked."
    );
    supported("Kavu Titan");
    supported("Reflections of Littjara");
    // Reflections of Littjara (Kavu chosen): "Whenever you cast a spell of the chosen
    // type, copy that spell." Kavu Titan: "If this creature was kicked, it enters with
    // three +1/+1 counters on it and with trample."
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Kavu");
    enter(&mut t, P0, "Reflections of Littjara");
    t.lands(P0, "Forest", 5);
    let titan = t.hand(P0, "Kavu Titan");
    t.cast(P0, titan).kicked(true).go();
    t.resolve_all();
    let kavus: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.chars.name == "Kavu Titan")
            .map(|o| o.id)
            .collect();
    assert_eq!(kavus.len(), 2);
    assert!(kavus.iter().any(|k| t.g.obj(*k).is_token()));
    for k in kavus {
        assert_eq!(t.counters(k, counters::PLUS1), 3);
        assert!(t.obj_now(k).has_keyword(KeywordKind::Trample));
        assert_eq!(t.pt(k), (5, 5));
    }
    // Unkicked, neither is.
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Kavu");
    enter(&mut t, P0, "Reflections of Littjara");
    t.lands(P0, "Forest", 2);
    let titan = t.hand(P0, "Kavu Titan");
    t.cast(P0, titan).kicked(false).go();
    t.resolve_all();
    let kavus = t.named_on_battlefield("Kavu Titan");
    assert_eq!(kavus.len(), 2);
    for k in kavus {
        assert_eq!(t.counters(k, counters::PLUS1), 0);
    }
}

#[test]
fn a_copy_of_a_kicked_spell_is_kicked() {
    cr!("702.33d", "707.10");
    ruling!(
        "Firebending Lesson",
        "If you copy a kicked spell on the stack, the copy is also kicked."
    );
    supported("Firebending Lesson");
    supported("Twincast");
    // Firebending Lesson: "deals 2 damage to target creature. If this spell was kicked,
    // it deals 5 damage to that creature instead." Twincast copies it with a new target.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Serra Angel");
    let b = t.battlefield(P1, "Serra Angel");
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 4);
    let lesson = t.hand(P0, "Firebending Lesson");
    let spell = t.cast(P0, lesson).kicked(true).target(a).go();
    add_mana(&mut t, P0, ManaType::U, 2);
    let twin = t.hand(P0, "Twincast");
    t.cast(P0, twin).target(spell).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve_all();
    // Both Angels (4/4) were dealt 5 damage.
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn chosen_modes_are_performed_in_order_with_triggers_waiting() {
    cr!("700.2", "608.2c", "603.3");
    ruling!(
        "Inscription of Ruin",
        "If more than one mode is chosen, perform them in the order written. Nothing can happen in between, however, and no player may choose to take actions. Any abilities that trigger will be put onto the stack after the spell has finished resolving."
    );
    supported("Inscription of Ruin");
    supported("Blood Artist");
    // Kicked, all three modes: P1 discards two cards; P0's Grizzly Bears returns from the
    // graveyard; P1's Grizzly Bears is destroyed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Soul Warden");
    let own = t.graveyard(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.hand(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::B, 3);
    add_mana(&mut t, P0, ManaType::C, 4);
    let card = t.hand(P0, "Inscription of Ruin");
    let spell = t
        .cast(P0, card)
        .kicked(true)
        .modes(&[0, 1, 2])
        .targets(&[Entity::Player(P1)])
        .targets(&[Entity::Object(own)])
        .targets(&[Entity::Object(theirs)])
        .go();
    let from_event = t.g.turn_events.len();
    t.g.resolve_top();
    // Nothing triggered went on the stack while it resolved.
    assert!(resolved(&t, spell));
    assert!(t.g.stack.is_empty());
    let events = t.g.turn_events[from_event..].to_vec();
    let discard = events
        .iter()
        .position(|e| matches!(e, Event::Discarded { player, .. } if *player == P1))
        .expect("discarded");
    let returned = events
        .iter()
        .position(
            |e| matches!(e, Event::ZoneChange { old, to: Zone::Battlefield, .. } if *old == own),
        )
        .expect("returned");
    let destroyed = events
        .iter()
        .position(|e| {
            matches!(e, Event::ZoneChange { old, cause: MoveCause::Destroy, .. } if *old == theirs)
        })
        .expect("destroyed");
    assert!(discard < returned && returned < destroyed);
    assert_eq!(t.hand_size(P1), 0);
    // Blood Artist's and Soul Warden's triggers go on the stack now.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn a_second_target_only_if_kicked() {
    cr!("702.33d", "601.2c");
    ruling!(
        "Rushing River",
        "You choose a second target only if you choose to pay the Kicker cost."
    );
    supported("Rushing River");
    // Rushing River: "Kicker—Sacrifice a land. Return target nonland permanent to its
    // owner's hand. If this spell was kicked, return another target nonland permanent to
    // its owner's hand."
    let is_targets = |d: &Decision| matches!(d, Decision::ChooseTargets { .. });
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P0, "Rushing River");
    let river = t.hand(P0, "Rushing River");
    let from = t.asked().len();
    t.cast(P0, river).kicked(false).target(a).go();
    assert_eq!(asked_of_since(&t, P0, from, is_targets), 1);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(b));
    // Kicked (sacrificing a land): a second target.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P0, "Rushing River");
    t.lands(P0, "Forest", 1);
    let river = t.hand(P0, "Rushing River");
    let from = t.asked().len();
    t.cast(P0, river).kicked(true).target(a).target(b).go();
    assert_eq!(asked_of_since(&t, P0, from, is_targets), 2);
    t.resolve_all();
    assert_eq!(hand_names(&t, P1).len(), 2);
}

#[test]
fn illegal_targets_dont_stop_the_other_modes() {
    cr!("608.2b", "700.2");
    ruling!(
        "Inscription of Ruin",
        "If any targets become illegal, the other targets will still be affected as appropriate."
    );
    // Kicked, the last two modes: return Grizzly Bears from P0's graveyard; destroy P1's
    // Grizzly Bears, which is destroyed in response.
    let mut t = TestGame::new(2);
    let own = t.graveyard(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::B, 3);
    add_mana(&mut t, P0, ManaType::C, 4);
    let card = t.hand(P0, "Inscription of Ruin");
    let spell = t
        .cast(P0, card)
        .kicked(true)
        .modes(&[1, 2])
        .targets(&[Entity::Object(own)])
        .targets(&[Entity::Object(theirs)])
        .go();
    destroy(&mut t, theirs);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert!(t.on_battlefield(own));
    assert_eq!(t.zone(own), Zone::Battlefield);
}
