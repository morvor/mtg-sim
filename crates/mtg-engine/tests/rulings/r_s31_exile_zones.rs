//! Rulings batch S31 — cards in exile: a search of a player's graveyard, hand, and
//! library for cards with a name leaves the permanents with that name alone (CR 701.23,
//! 201.2), and face-down cards in exile have no characteristics (CR 406.3a).

use crate::r_s01_common::supported;
use crate::r_s04_common::add_mana;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn eradicate_doesnt_exile_permanents_with_the_same_name() {
    cr!("701.23a", "201.2", "608.2h");
    ruling!(
        "Eradicate",
        "Does not exile other cards of the same name that are on the battlefield. Just from the graveyard, hand, and library."
    );
    supported("Eradicate");
    // "Exile target nonblack creature. Search its controller's graveyard, hand, and
    // library for all cards with the same name as that creature and exile them. Then that
    // player shuffles."
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let in_hand = t.hand(P1, "Grizzly Bears");
    let in_graveyard = t.graveyard(P1, "Grizzly Bears");
    let in_library = t.library_top(P1, "Grizzly Bears");
    let unrelated = t.graveyard(P1, "Hill Giant");
    // P0's own Grizzly Bears card isn't in that player's zones.
    let mine = t.graveyard(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::B, 4);
    let spell = t.hand(P0, "Eradicate");
    t.cast(P0, spell).target(target).go();
    t.resolve_all();
    for c in [target, in_hand, in_graveyard, in_library] {
        assert_eq!(t.zone(c), Zone::Exile);
    }
    assert!(t.on_battlefield(other));
    assert_eq!(t.zone(unrelated), Zone::Graveyard(P1));
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
}

#[test]
fn splinter_searches_the_artifacts_controllers_zones() {
    cr!("701.23a", "201.2");
    ruling!(
        "Splinter",
        "Does not exile other cards of the same name that are on the battlefield. Just from the graveyard, hand, and library."
    );
    supported("Splinter");
    // "Exile target artifact. Search its controller's graveyard, hand, and library for all
    // cards with the same name as that artifact and exile them. Then that player
    // shuffles."
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Ornithopter");
    let other = t.battlefield(P0, "Ornithopter");
    let in_hand = t.hand(P1, "Ornithopter");
    add_mana(&mut t, P0, ManaType::G, 4);
    let spell = t.hand(P0, "Splinter");
    t.cast(P0, spell).target(target).go();
    t.resolve_all();
    assert_eq!(t.zone(target), Zone::Exile);
    assert_eq!(t.zone(in_hand), Zone::Exile);
    assert!(t.on_battlefield(other));
}

#[test]
fn crackling_drake_doesnt_count_face_down_exiled_cards() {
    cr!("406.3a", "702.143a", "604.3");
    ruling!(
        "Crackling Drake",
        "If any exiled cards you own are face down, they have no characteristics. If they're normally instants or sorceries, they won't be counted."
    );
    supported("Crackling Drake");
    supported("Saw It Coming");
    // "Crackling Drake's power is equal to the total number of instant and sorcery cards
    // you own in exile and in your graveyard."
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Crackling Drake");
    t.graveyard(P0, "Lightning Bolt");
    t.exile(P0, "Shock");
    // Cards that aren't instants or sorceries, or aren't P0's, don't count.
    t.graveyard(P0, "Grizzly Bears");
    t.exile(P1, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.pt(drake).0, 2);
    // Saw It Coming (an instant) foretold: exiled face down, it isn't counted.
    let card = t.hand(P0, "Saw It Coming");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Foretell { card }))
        .expect("foretell");
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(t.obj_now(card).face_down);
    t.g.recompute();
    assert_eq!(t.pt(drake).0, 2);
}

#[test]
fn beacon_bolt_doesnt_count_face_down_exiled_cards() {
    cr!("406.3a", "702.143a");
    ruling!(
        "Beacon Bolt",
        "If any exiled cards you own are face down, they have no characteristics. If they're normally instants or sorceries, they won't be counted."
    );
    supported("Beacon Bolt");
    // "Beacon Bolt deals damage to target creature equal to the total number of instant
    // and sorcery cards you own in exile and in your graveyard."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.graveyard(P0, "Lightning Bolt");
    t.exile(P0, "Shock");
    let card = t.hand(P0, "Saw It Coming");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Foretell { card }))
        .expect("foretell");
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    let spell = t.hand(P0, "Beacon Bolt");
    t.cast(P0, spell).target(giant).go();
    t.resolve_all();
    // 2 damage: the face-down card isn't counted (nor Beacon Bolt itself, on the stack).
    assert_eq!(t.obj_now(giant).damage, 2);
}
