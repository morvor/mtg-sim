//! Trigger conditions listing several events (CR 603.2, 603.2c): Trouble in Pairs, Syr
//! Konrad, the Grim, Tergrid, God of Fright.

use crate::combat2_common::*;
use crate::td_next_turn_durations::mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn trouble_in_pairs_triggers_on_each_listed_event() {
    cr!("603.2", "603.2c");
    ruling!(
        "Trouble in Pairs",
        "Trouble in Pairs's last ability can trigger multiple times per opponent per turn"
    );
    supported("Trouble in Pairs");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trouble in Pairs");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    // P1 draws a first card: nothing; a second: P0 draws.
    t.g.draw_cards(P1, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    t.g.draw_cards(P1, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // P1's second spell.
    for i in 0..2 {
        let g = t.hand(P1, "Giant Growth");
        mana(&mut t, P1);
        t.cast_with(P1, g, &[Entity::Object(a)]).unwrap();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1 + i);
    }
    // Attacking P0 with two creatures.
    declare(&mut t, P1, &[(a, Entity::Player(P0)), (b, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn trouble_in_pairs_attack_with_one_creature_does_nothing() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trouble in Pairs");
    let a = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    declare(&mut t, P1, &[(a, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn syr_konrad_triggers_on_each_kind_of_creature_card_event() {
    cr!("603.2", "603.10a");
    ruling!(
        "Syr Konrad, the Grim",
        "If one or more creatures die at the same time as Syr Konrad, its first ability triggers for each of those creatures."
    );
    supported("Syr Konrad, the Grim");
    let mut t = TestGame::new(2);
    let konrad = t.battlefield(P0, "Syr Konrad, the Grim");
    // Another creature dies.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // A creature card is put into a graveyard from a library (milled).
    t.library_top(P1, "Grizzly Bears");
    let mill = t.hand(P0, "Glimpse the Unthinkable");
    mana(&mut t, P0);
    t.cast_with(P0, mill, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // A creature card leaves your graveyard.
    let card = t.graveyard(P0, "Grizzly Bears");
    let raise = t.hand(P0, "Raise Dead");
    mana(&mut t, P0);
    t.cast_with(P0, raise, &[Entity::Object(card)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Syr Konrad and another creature dying together: the other one counts (look back).
    t.battlefield(P0, "Llanowar Elves");
    let wrath = t.hand(P0, "Day of Judgment");
    mana(&mut t, P0);
    t.cast_with(P0, wrath, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(konrad));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn tergrid_takes_sacrificed_and_discarded_permanent_cards() {
    cr!("603.2", "603.10a");
    ruling!(
        "Tergrid, God of Fright // Tergrid's Lantern",
        "it doesn't matter which graveyard it goes to"
    );
    supported("Tergrid, God of Fright // Tergrid's Lantern");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tergrid, God of Fright // Tergrid's Lantern");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let edict = t.hand(P0, "Diabolic Edict");
    mana(&mut t, P0);
    t.answer_yes(P0, true);
    t.cast_with(P0, edict, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let got = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(got.len(), 1);
    assert_eq!(t.obj_now(got[0]).controller, P0);
    let _ = bears;
    // A discarded permanent card; a discarded instant isn't one.
    t.hand(P1, "Llanowar Elves");
    t.hand(P1, "Lightning Bolt");
    let rot = t.hand(P0, "Mind Rot");
    mana(&mut t, P0);
    t.answer_yes(P0, true);
    t.cast_with(P0, rot, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let elves = t.named_on_battlefield("Llanowar Elves");
    assert_eq!(elves.len(), 1);
    assert_eq!(t.obj_now(elves[0]).controller, P0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

