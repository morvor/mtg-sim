//! CR 702.75 Hideaway.

use crate::common_k702_011_017::{assert_supported, give_mana_for};
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::{destroy, on_top, stack_triggers};
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::zones;
use mtg_engine::*;

/// Puts four named cards on top of P0's library (the last one on top) and returns them.
fn stack_library(t: &mut TestGame) -> Vec<ObjectId> {
    ["Grizzly Bears", "Counterspell", "Llanowar Elves", "Lightning Bolt"]
        .iter()
        .map(|n| on_top(t, P0, n))
        .collect()
}

/// The ids of the bottom `n` cards of `p`'s library.
fn bottom(t: &TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    t.g.player(p).library[..n].to_vec()
}

#[test]
fn hideaway_exiles_one_of_the_top_n_cards_face_down() {
    cr!("702.75", "702.75a");
    ruling!(
        "Watcher for Tomorrow",
        "Hideaway now causes you to put the rest of the cards on the bottom of your library in a random order instead of any order."
    );
    assert_supported("Watcher for Tomorrow");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let size = t.library_size(P0);
    give_mana_for(&mut t, P0, "Watcher for Tomorrow");
    let watcher = t.hand(P0, "Watcher for Tomorrow");
    t.cast(P0, watcher).go();
    t.resolve();
    t.settle();
    assert_eq!(stack_triggers(&t, "Hideaway 4").len(), 1);
    // Exile the Counterspell (third from the top).
    t.answer_choose(P0, &[Entity::Object(cards[1])]);
    t.resolve();
    let exiled = t.g.current(cards[1]);
    assert_eq!(t.g.obj(exiled).zone, Zone::Exile);
    assert!(t.g.obj(exiled).face_down);
    // The other three are on the bottom.
    let mut rest = bottom(&t, P0, 3);
    rest.sort();
    let mut expected = vec![cards[0], cards[2], cards[3]];
    expected.sort();
    assert_eq!(rest, expected);
    assert_eq!(t.library_size(P0), size - 1);
    // Its controller may look at it; the opponent may not.
    assert!(zones::may_look(&t.g, P0, exiled));
    assert!(!zones::may_look(&t.g, P1, exiled));
}

#[test]
fn the_linked_ability_returns_the_exiled_card() {
    cr!("702.75a", "607.2a");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let watcher = t.enter(P0, "Watcher for Tomorrow");
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.resolve_all();
    assert_eq!(t.zone(cards[3]), Zone::Exile);
    // "When this creature leaves the battlefield, put the exiled card into its owner's
    // hand."
    destroy(&mut t, watcher);
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn leaving_before_hideaway_resolves_strands_the_card() {
    cr!("702.75a");
    ruling!(
        "Watcher for Tomorrow",
        "If Watcher for Tomorrow leaves the battlefield before its triggered ability from hideaway resolves, its leaves-the-battlefield ability resolves and does nothing."
    );
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let watcher = t.enter(P0, "Watcher for Tomorrow");
    t.settle();
    destroy(&mut t, watcher);
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.resolve_all();
    assert_eq!(t.zone(cards[3]), Zone::Exile);
    assert!(!t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn hideaway_n_looks_at_n_cards() {
    cr!("702.75a");
    assert_supported("Fight Rigging");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let fifth = on_top(&mut t, P0, "Island");
    let before = t.library_size(P0);
    t.enter(P0, "Fight Rigging");
    t.settle();
    assert_eq!(stack_triggers(&t, "Hideaway 5").len(), 1);
    // The Grizzly Bears was fifth from the top: it could be chosen.
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.resolve_all();
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    let mut rest = bottom(&t, P0, 4);
    rest.sort();
    let mut expected = vec![cards[1], cards[2], cards[3], fifth];
    expected.sort();
    assert_eq!(rest, expected);
    assert_eq!(t.library_size(P0), before - 1);
}

#[test]
fn a_new_controller_of_the_hideaway_permanent_may_look_at_the_card() {
    cr!("702.75a");
    ruling!(
        "Mosswort Bridge",
        "Any player who has controlled a permanent with a hideaway ability since a card was exiled with it may look at that card."
    );
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let watcher = t.enter(P0, "Watcher for Tomorrow");
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.resolve_all();
    let exiled = t.g.current(cards[3]);
    assert!(!zones::may_look(&t.g, P1, exiled));
    // P1 gains control of it.
    t.lands(P1, "Mountain", 3);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let treason = t.hand(P1, "Act of Treason");
    t.cast(P1, treason).target(watcher).go();
    t.resolve_all();
    assert_eq!(t.obj_now(watcher).controller, P1);
    assert!(zones::may_look(&t.g, P1, exiled));
    // The previous controller still may too.
    assert!(zones::may_look(&t.g, P0, exiled));
}

#[test]
fn older_hideaway_cards_have_hideaway_four_and_enter_tapped() {
    cr!("702.75b");
    ruling!(
        "Mosswort Bridge",
        "Older cards have received errata to have an additional paragraph"
    );
    assert_supported("Mosswort Bridge");
    let def = mtg_engine::card::card("Mosswort Bridge");
    let chars = &def.front().chars;
    let kw: Vec<_> = chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Hideaway)
        .collect();
    assert_eq!(kw.len(), 1);
    assert_eq!(kw[0].n, Some(4));
    let mut t = TestGame::new(2);
    stack_library(&mut t);
    let bridge = t.enter(P0, "Mosswort Bridge");
    assert!(t.obj_now(bridge).tapped);
    t.settle();
    assert_eq!(stack_triggers(&t, "Hideaway 4").len(), 1);
}

#[test]
fn the_exiled_card_can_be_played_by_the_hideaway_permanents_linked_ability() {
    cr!("702.75a", "607.2a");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let bridge = t.enter(P0, "Mosswort Bridge");
    // Exile the Grizzly Bears (fourth from the top).
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.resolve_all();
    let bridge_now = t.g.current(bridge);
    t.g.objects[bridge_now.0 as usize].tapped = false;
    t.lands(P0, "Forest", 1);
    let uid = t
        .obj_now(bridge)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(&a.kind, AbilityKind::Activated(x) if !x.is_mana_ability))
        .map(|a| a.text.clone())
        .expect("play ability");
    // "... if creatures you control have total power 10 or greater": not yet.
    t.answer_yes(P0, true);
    activate_named(&mut t, P0, bridge, &uid, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    // With 12 power, it's played for free.
    t.battlefield(P0, "Craw Wurm");
    t.battlefield(P0, "Craw Wurm");
    let now = t.g.current(bridge);
    t.g.objects[now.0 as usize].tapped = false;
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, true);
    activate_named(&mut t, P0, bridge, &uid, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
