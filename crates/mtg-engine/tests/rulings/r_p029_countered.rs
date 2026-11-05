//! Rulings batch P029 — an ability that copies a spell resolves before that spell and
//! copies it even if the spell was countered by then, using its last known information
//! (CR 707.10, 608.2h, 405.5); a countered spell was still cast (CR 601.2i, 701.6a).

use crate::r_p028_common::*;
use crate::r_p029_common::*;
use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::{supported, with_subtype};
use crate::r_s17_common::enter_transformed;
use crate::r_s18_common::ADVENTURE;
use crate::r_s20_common::tap_for_mana;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The position of `id` on the stack (0 is the bottom).
fn pos(t: &TestGame, id: ObjectId) -> Option<usize> {
    t.g.stack.iter().position(|x| *x == id)
}

/// The triggered abilities on the stack, bottom first.
fn triggers_on_stack(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| {
            matches!(
                t.g.obj(*id).stack.as_deref().map(|s| &s.kind),
                Some(mtg_engine::object::StackKind::Triggered { .. })
            )
        })
        .collect()
}

/// P0 casts Divination; `c` sets up a copy of it. If `counter`, P1 counters the
/// Divination with Counterspell before the copying ability resolves: the copy is still
/// made and draws two cards. Otherwise the copying ability and then the copy are above the
/// Divination and resolve first.
fn copies_divination_countered_or_not(c: &Copier, counter: bool) {
    supported(c.name);
    let mut t = TestGame::new(2);
    (c.setup)(&mut t);
    let div = cast_new(&mut t, P0, "Divination", &[]);
    (c.start)(&mut t, div);
    t.settle();
    let trig = *triggers_on_stack(&t)
        .last()
        .unwrap_or_else(|| panic!("{}: no copying trigger", c.name));
    assert!(pos(&t, trig) > pos(&t, div), "{}", c.name);
    if counter {
        cast_new(&mut t, P1, "Counterspell", &[obj(div)]);
        t.resolve();
        assert_eq!(pos(&t, div), None, "{}: countered", c.name);
        assert!(t.in_graveyard(P0, "Divination"));
    }
    let hand = t.hand_size(P0);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1, "{}", c.name);
    if !counter {
        assert!(pos(&t, copies[0]) > pos(&t, div), "{}", c.name);
        // The copy resolves while the Divination is still on the stack.
        t.resolve();
        assert_eq!(t.hand_size(P0), hand + 2, "{}", c.name);
        assert!(pos(&t, div).is_some(), "{}", c.name);
    }
    t.resolve_all();
    let want = hand + if counter { 2 } else { 4 };
    assert_eq!(t.hand_size(P0), want, "{}", c.name);
}

#[test]
fn copying_triggers_resolve_first_and_copy_a_countered_spell() {
    cr!("707.10", "608.2h", "405.5", "603.3");
    ruling!(
        "Mica, Reader of Ruins",
        "Mica's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Adaptive Training Post",
        "The copy created by Adaptive Training Post’s last ability resolves before the spell it’s copying. It resolves even if the original spell is countered before the copy is created."
    );
    ruling!(
        "Swarm Intelligence",
        "Swarm Intelligence can copy the spell even if it’s countered before Swarm Intelligence’s triggered ability resolves."
    );
    ruling!(
        "Storm King's Thunder",
        "Storm King's Thunder will create copies of the spell that caused its delayed triggered ability to trigger even if that spell has been countered by the time that ability resolves."
    );
    for c in [MICA, ADAPTIVE_TRAINING_POST, SWARM_INTELLIGENCE, STORM_KINGS_THUNDER] {
        copies_divination_countered_or_not(&c, true);
        copies_divination_countered_or_not(&c, false);
    }
    // Mica's "otherwise leaves the stack": the Divination is exiled from the stack.
    let mut t = TestGame::new(2);
    (MICA.setup)(&mut t);
    let div = cast_new(&mut t, P0, "Divination", &[]);
    (MICA.start)(&mut t, div);
    t.settle();
    t.g.exile_object(div, None);
    t.g.flush_events();
    let hand = t.hand_size(P0);
    assert_eq!(resolve_until_copies(&mut t).len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn primal_wellspring_copies_a_countered_spell() {
    cr!("707.10", "608.2h", "603.7c");
    ruling!(
        "Primal Amulet // Primal Wellspring",
        "Primal Wellspring's delayed triggered ability can copy the spell even if that spell is countered before the ability resolves."
    );
    for counter in [true, false] {
        let mut t = TestGame::new(2);
        let w = enter_transformed(&mut t, P0, "Primal Amulet // Primal Wellspring");
        // {U}: Index(1) is blue.
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        assert!(tap_for_mana(&mut t, P0, w, "Add one mana"));
        t.lands(P0, "Wastes", 2);
        let card = t.hand(P0, "Divination");
        let div = t.cast(P0, card).go();
        keep_copy_targets(&mut t, P0);
        t.settle();
        if counter {
            cast_new(&mut t, P1, "Counterspell", &[obj(div)]);
            t.resolve();
            assert_eq!(pos(&t, div), None);
        }
        let hand = t.hand_size(P0);
        assert_eq!(resolve_until_copies(&mut t).len(), 1);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + if counter { 2 } else { 4 });
    }
}

#[test]
fn lucky_clover_s_ability_and_copy_resolve_first_even_if_the_adventure_is_countered() {
    cr!("707.10", "608.2h", "405.5", "715.4");
    ruling!(
        "Lucky Clover",
        "Lucky Clover's ability and the copy it creates both resolve before the Adventure spell. They resolve even if the Adventure spell is countered before the copy is created."
    );
    // Heart's Desire: "Create a 1/1 white Human creature token."
    for counter in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lucky Clover");
        t.lands(P0, "Forest", 1);
        let beast = t.hand(P0, "Lovestruck Beast // Heart's Desire");
        let spell = t.cast(P0, beast).method(ADVENTURE).go();
        keep_copy_targets(&mut t, P0);
        t.settle();
        let trig = triggers_on_stack(&t)[0];
        assert!(pos(&t, trig) > pos(&t, spell));
        if counter {
            cast_new(&mut t, P1, "Counterspell", &[obj(spell)]);
            t.resolve();
            assert_eq!(pos(&t, spell), None);
        }
        t.resolve();
        let copies = spell_copies(&t);
        assert_eq!(copies.len(), 1);
        if !counter {
            assert!(pos(&t, copies[0]) > pos(&t, spell));
        }
        t.resolve();
        assert_eq!(with_subtype(&t, P0, "Human").len(), 1);
        t.resolve_all();
        assert_eq!(
            with_subtype(&t, P0, "Human").len(),
            if counter { 1 } else { 2 }
        );
    }
}

#[test]
fn a_countered_spell_still_counts_for_thousand_year_storm() {
    cr!("601.2i", "701.6a", "707.10");
    ruling!(
        "Thousand-Year Storm",
        "Spells you've cast that were countered were still cast, and so will add copies when Thousand-Year Storm's ability resolves for later spells in the turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thousand-Year Storm");
    let opt = cast_new(&mut t, P0, "Opt", &[]);
    cast_new(&mut t, P1, "Counterspell", &[obj(opt)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Opt"));
    cast_new(&mut t, P0, "Divination", &[]);
    keep_copy_targets(&mut t, P0);
    let copies = resolve_collecting_copies(&mut t);
    assert_eq!(copies.len(), 1);
}
