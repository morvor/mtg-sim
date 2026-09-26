//! CR 733: handling illegal actions.

use crate::r506_common::custom_card;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Stage;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn an_illegal_cast_is_reversed_with_the_mana_abilities_activated_for_it() {
    cr!("733.1");
    let mut t = TestGame::new(2);
    // "Whenever an opponent casts a spell, you gain 1 life."
    let watcher = custom_card(
        "Spell Watcher",
        "Enchantment",
        None,
        "Whenever an opponent casts a spell, you gain 1 life.",
    );
    t.custom(P1, watcher, Zone::Battlefield);
    // "Whenever a creature you control becomes tapped, you gain 1 life."
    let tap_watcher = custom_card(
        "Tap Watcher",
        "Enchantment",
        None,
        "Whenever a creature you control becomes tapped, you gain 1 life.",
    );
    t.custom(P0, tap_watcher, Zone::Battlefield);
    // "{T}: Add {W}." — P0's only creature and only mana source.
    let pilgrim = t.battlefield(P0, "Avacyn's Pilgrim");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bear.0 as usize].tapped = true;
    let mut swallow = custom_card(
        "Swallowing Maw",
        "Instant",
        None,
        "As an additional cost to cast this spell, tap an untapped creature you control.\nExile target tapped creature.",
    );
    let cost = mtg_engine::mana::ManaCost::parse("{W}").unwrap();
    swallow.faces[0].chars.colors = cost.colors();
    swallow.faces[0].chars.mana_cost = Some(cost);
    let spell = t.custom(P0, swallow, Zone::Hand(P0));
    let events = t.g.turn_events.len();
    // Paying {W} taps the Pilgrim for mana; then the additional cost (tapping an untapped
    // creature) can't be paid: casting the spell is illegal.
    let r = t
        .cast(P0, spell)
        .target(Entity::Object(bear))
        .method(CastMethod::Normal)
        .try_go();
    assert!(r.is_err());
    // The whole action was reversed: the spell is back in hand (the same card), the mana
    // ability that tapped the Pilgrim was reversed and no mana is left over...
    assert_eq!(t.g.obj(spell).zone, Zone::Hand(P0));
    assert!(t.g.is_live(spell));
    assert!(t.g.stack.is_empty());
    assert!(!t.g.obj(pilgrim).tapped);
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    assert!(t.on_battlefield(bear));
    // ...and nothing triggered as a result: neither the tapping nor the casting.
    t.settle();
    assert!(t.g.stack.is_empty());
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(t.g.turn_events.len(), events);
}

#[test]
fn the_player_who_attempted_an_illegal_action_retains_priority() {
    cr!("733.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let giant = t.hand(P0, "Hill Giant");
    let forest = t.hand(P0, "Forest");
    t.g.turn.passes = 0;
    // P0 tries to cast Hill Giant without enough mana: illegal.
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: giant,
            method: CastMethod::Normal,
        }),
    );
    // Then takes another action: plays a land.
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::PlayLand { card: forest }),
    );
    t.g.advance();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.g.turn.priority, Some(P0));
    assert_eq!(t.g.turn.passes, 0);
    assert_eq!(t.g.turn.stage, Stage::Priority);
    t.g.advance();
    assert!(t.on_battlefield(forest));
    assert_eq!(t.g.turn.priority, Some(P0));
    // Now with two lands the player can redo the action legally.
    t.lands(P0, "Mountain", 2);
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: giant,
            method: CastMethod::Normal,
        }),
    );
    t.g.advance();
    assert_eq!(t.g.stack.len(), 1);
}
