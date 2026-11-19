//! Rulings batch P115 — triggered abilities with an intervening "if" clause (CR 603.4)
//! checked as they trigger and again as they resolve: Bridge from Below (which works only
//! from a graveyard and cares about which graveyard the creature went to), Wrangler of
//! the Damned and Ophiomancer.

use crate::r_p115_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn bridge_from_below_must_be_in_the_graveyard_as_it_triggers_and_resolves() {
    cr!("603.4", "603.10a", "113.6k");
    ruling!(
        "Bridge from Below",
        "If Bridge from Below isn't in your graveyard immediately before a creature dies, its abilities won't trigger at all. If Bridge from Below isn't still in your graveyard as its first ability resolves, you won't create a token."
    );
    supported("Bridge from Below");
    // In the hand: nothing triggers.
    let mut t = TestGame::new(2);
    t.hand(P0, "Bridge from Below");
    let bears = t.battlefield(P0, "Grizzly Bears");
    kill(&mut t, bears);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Zombie"), 0);
    // In the graveyard: it triggers; exiled in response, no token.
    for exile_first in [false, true] {
        let mut t = TestGame::new(2);
        let bridge = t.graveyard(P0, "Bridge from Below");
        let bears = t.battlefield(P0, "Grizzly Bears");
        kill(&mut t, bears);
        assert_eq!(t.stack_len(), 1);
        if exile_first {
            move_to(&mut t, bridge, Zone::Exile);
        }
        t.resolve_all();
        assert_eq!(
            tokens_of(&t, P0, "Zombie"),
            usize::from(!exile_first),
            "exiled first: {exile_first}"
        );
    }
}

#[test]
fn bridge_from_below_simultaneous_deaths_let_you_make_the_token_first() {
    cr!("603.3b", "603.4");
    ruling!(
        "Bridge from Below",
        "You can create a token before you exile Bridge from Below."
    );
    // Pyroclasm kills P0's Grizzly Bears and P1's Llanowar Elves at once. Both abilities
    // trigger; P0 orders them (first = bottom of the stack).
    for (order, token) in [([1usize, 0], true), ([0, 1], false)] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Bridge from Below");
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P1, "Llanowar Elves");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        cast_at(&mut t, P0, "Pyroclasm", &[]);
        t.resolve();
        assert_eq!(t.stack_len(), 2);
        t.resolve_all();
        assert!(t.in_exile("Bridge from Below"));
        assert_eq!(tokens_of(&t, P0, "Zombie"), usize::from(token), "{order:?}");
    }
}

#[test]
fn bridge_from_below_cares_about_the_graveyard_not_the_controller() {
    cr!("603.10a", "404.2", "400.3");
    ruling!(
        "Bridge from Below",
        "Neither ability cares who controlled the creature that died, only which graveyard it was put into."
    );
    // P1 controls a creature P0 owns: when it dies it goes to P0's graveyard, so the
    // first ability triggers (a token), not the second.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Bridge from Below");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    assert_eq!(ctl(&t, bears), P1);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Zombie"), 1);
    assert!(t.in_graveyard(P0, "Bridge from Below"));
    // P0 controls a creature P1 owns: it goes to P1's graveyard and Bridge is exiled.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Bridge from Below");
    let elves = t.battlefield(P1, "Llanowar Elves");
    give_control(&mut t, elves, P0);
    kill(&mut t, elves);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Zombie"), 0);
    assert!(t.in_exile("Bridge from Below"));
}

#[test]
fn bridge_from_below_on_the_battlefield_does_nothing() {
    cr!("113.6", "113.6k");
    ruling!(
        "Bridge from Below",
        "While Bridge from Below is on the battlefield, it has no effect."
    );
    let mut t = TestGame::new(2);
    let bridge = t.battlefield(P0, "Bridge from Below");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    kill(&mut t, bears);
    kill(&mut t, elves);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Zombie"), 0);
    assert!(t.on_battlefield(bridge));
}

#[test]
fn wrangler_of_the_damned_checks_as_it_triggers_and_as_it_resolves() {
    cr!("603.4");
    ruling!(
        "Wrangler of the Damned",
        "If you’ve cast a spell from your hand since the ability triggered, the ability won’t do anything."
    );
    supported("Wrangler of the Damned");
    // No spell cast this turn: a Spirit.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wrangler of the Damned");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Spirit"), 1);
    // A spell cast from hand during the main phase: the ability doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wrangler of the Damned");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Shock", &[obj(bears)]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(tokens_of(&t, P0, "Spirit"), 0);
    // A spell cast from hand in response to the trigger: no Spirit.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wrangler of the Damned");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    cast_at(&mut t, P0, "Shock", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(tokens_of(&t, P0, "Spirit"), 0);
}

#[test]
fn ophiomancer_checks_for_snakes_as_it_triggers_and_resolves() {
    cr!("603.4", "205.3m");
    ruling!(
        "Ophiomancer",
        "Ophiomancer's ability checks if you control a Snake at the beginning of each upkeep. If you do, the ability won't trigger."
    );
    ruling!(
        "Ophiomancer",
        "If the ability does trigger, but you control a Snake when it tries to resolve, the ability will do nothing."
    );
    ruling!(
        "Ophiomancer",
        "Ophiomancer's ability considers any creature you control with the creature type Snake, not just the tokens Ophiomancer creates."
    );
    supported("Ophiomancer");
    // No Snake: a Snake at the opponent's upkeep (each upkeep), then none at the next
    // upkeep (P0 controls that Snake).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ophiomancer");
    to_upkeep(&mut t, P1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Snake"), 1);
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(tokens_of(&t, P0, "Snake"), 1);
    // A nontoken Snake (Coiling Oracle is a Snake Elf Druid): no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ophiomancer");
    t.battlefield(P0, "Coiling Oracle");
    to_upkeep(&mut t, P1);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(tokens_of(&t, P0, "Snake"), 0);
    // A Snake arrives with the trigger on the stack: no token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ophiomancer");
    to_upkeep(&mut t, P1);
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Coiling Oracle");
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Snake"), 0);
}
