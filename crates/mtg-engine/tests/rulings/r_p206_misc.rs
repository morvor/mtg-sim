//! Rulings batch P206 — dredge replacing the draw of a creature's own sacrifice ability
//! (CR 702.52), earthbend targets (CR 701.66, 115.3), and when echo triggers after a
//! control change (CR 702.30a).

use crate::r_p206_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::target_candidates;
use crate::r_s04_common::next_upkeep;
use crate::r_s06_common::{activate_containing, give_control};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn grave_shell_scarab_can_dredge_itself_back_with_its_own_draw() {
    cr!("702.52a", "601.2h", "614.1a");
    ruling!(
        "Grave-Shell Scarab",
        "You may sacrifice Grave-Shell Scarab to pay for its first ability, then replace that draw using the Scarab's dredge ability. The result is that the top card of your library is put into your graveyard and Grave-Shell Scarab returns to your hand (after a brief trip to the graveyard)."
    );
    supported("Grave-Shell Scarab");
    let mut t = TestGame::new(2);
    let scarab = t.battlefield(P0, "Grave-Shell Scarab");
    let top = t.library_top(P0, "Forest");
    let library = t.library_size(P0);
    rainbow_pool(&mut t, P0, 1);
    activate_containing(&mut t, P0, scarab, "Draw a card").unwrap();
    assert!(t.in_graveyard(P0, "Grave-Shell Scarab"));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grave-Shell Scarab"));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.zone(top), Zone::Graveyard(P0));
    assert_eq!(t.library_size(P0), library - 1);
}

#[test]
fn earthbender_ascension_cant_earthbend_the_land_it_searches_for() {
    cr!("701.66a", "603.3d", "115.1");
    ruling!(
        "Earthbender Ascension",
        "The land that the first ability searches for is not on the battlefield when you select targets for earthbend, so it cannot be the land that you earthbend."
    );
    supported("Earthbender Ascension");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let plains = t.library_top(P0, "Plains");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(forest)]);
    t.enter(P0, "Earthbender Ascension");
    t.answer_choose(P0, &[Entity::Object(plains)]);
    t.resolve_all();
    // The only target offered was the land already on the battlefield.
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered[0], vec![Entity::Object(forest)]);
    assert_eq!(plus1(&t, forest), 2);
    let plains = t.g.current(plains);
    assert!(t.on_battlefield(plains));
    assert_eq!(plus1(&t, plains), 0);
    assert!(!t.obj_now(plains).is(CardType::Creature));
}

/// Casts Cracked Earth Technique (or puts Dai Li Agents onto the battlefield) with its
/// two earthbend targets `a` and `b`, and resolves everything.
fn earthbend_twice(t: &mut TestGame, card: &str, a: ObjectId, b: ObjectId) {
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    if card == "Cracked Earth Technique" {
        rainbow_pool(t, P0, 4);
        let c = t.hand(P0, card);
        t.cast(P0, c).go();
    } else {
        t.enter(P0, card);
    }
    t.resolve_all();
}

#[test]
fn the_two_earthbends_can_target_the_same_land_or_different_lands() {
    cr!("701.66a", "115.3");
    ruling!(
        "Cracked Earth Technique",
        "You can target the same land or two different lands with the multiple instances of earthbend from Cracked Earth Technique."
    );
    ruling!(
        "Dai Li Agents",
        "You can target the same land or two different lands with the multiple instances of earthbend from Dai Li Agents."
    );
    for (card, n) in [("Cracked Earth Technique", 3), ("Dai Li Agents", 1)] {
        supported(card);
        let mut t = TestGame::new(2);
        let forest = t.battlefield(P0, "Forest");
        earthbend_twice(&mut t, card, forest, forest);
        assert_eq!(plus1(&t, forest), 2 * n, "{card}");
        assert_eq!(t.pt(forest), (2 * n as i32, 2 * n as i32), "{card}");
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Forest");
        let b = t.battlefield(P0, "Island");
        earthbend_twice(&mut t, card, a, b);
        assert_eq!((plus1(&t, a), plus1(&t, b)), (n, n), "{card}");
    }
}

#[test]
fn echo_triggers_after_a_control_change() {
    cr!("702.30a");
    ruling!(
        "Orcish Hellraiser",
        "A permanent’s echo ability will trigger during your upkeep if it entered the battlefield since the beginning of your last upkeep, if you gained control of it since the beginning of your last upkeep, or if another player took control of it and it returned to your control since the beginning of your last upkeep."
    );
    supported("Orcish Hellraiser");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Mountain", 1);
    let orc = t.battlefield(P0, "Orcish Hellraiser");
    // It entered since P0's last upkeep: echo triggers; P0 pays.
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "Echo"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(orc));
    // Next upkeep: no echo.
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "Echo"), 0);
    // P1 gains control of it: echo triggers in P1's upkeep.
    give_control(&mut t, orc, P1);
    next_upkeep(&mut t, P1);
    assert_eq!(triggers_on_stack(&t, "Echo"), 1);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.on_battlefield(orc));
    // It returns to P0's control: echo triggers again in P0's upkeep.
    give_control(&mut t, orc, P0);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "Echo"), 1);
}
