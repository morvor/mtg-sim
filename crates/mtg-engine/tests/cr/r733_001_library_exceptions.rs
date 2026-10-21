//! CR 733.1's exceptions: "Players may not reverse actions that moved cards to a library,
//! moved cards from a library to any zone other than the stack, caused a library to be
//! shuffled, or caused cards from a library to be revealed." The engine reverses an
//! illegal action by restoring the whole game (register entry
//! `illegal-action-reversal-restores-libraries`); no supported card can reach the
//! exceptions, because a cost's library actions are paid after everything that can make
//! the action illegal.

use crate::r506_common::custom_card;
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn library_costs_are_paid_after_every_other_cost() {
    cr!("601.2h", "602.2b");
    interp!("illegal-action-reversal-restores-libraries");
    let mut t = TestGame::new(2);
    // "{B}, Pay 1 life, Mill a card, Discard a card, Sacrifice ~: Destroy target creature."
    let concoction = t.battlefield(P0, "Sinister Concoction");
    t.lands(P0, "Swamp", 1);
    let discard = t.hand(P0, "Grizzly Bears");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let top = t.g.library_top(P0).unwrap();
    let start = t.g.turn_events.len();
    t.activate(P0, concoction, 0, &[Entity::Object(bear)])
        .expect("activates");
    let events = &t.g.turn_events[start..];
    let pos = |f: &dyn Fn(&Event) -> bool| events.iter().position(f);
    let moved = |id: ObjectId| {
        pos(&|e| matches!(e, Event::ZoneChange { old, to: Zone::Graveyard(_), .. } if *old == id))
    };
    let life = pos(&|e| matches!(e, Event::LifeLost { player, amount: 1 } if *player == P0));
    let (discarded, sacrificed, milled) = (moved(discard), moved(concoction), moved(top));
    assert!(life.is_some() && discarded.is_some() && sacrificed.is_some());
    // The mill (moving a card from the library to a public zone) is paid last.
    assert!(milled > life && milled > discarded && milled > sacrificed);
    assert_eq!(t.g.player(P0).graveyard.len(), 3);
}

#[test]
fn a_reversed_action_also_reverses_a_draw_made_while_attempting_it() {
    cr!("605.1b");
    interp!("illegal-action-reversal-restores-libraries");
    interp!("reversal-undoes-mana-abilities");
    let mut t = TestGame::new(2);
    // A triggered mana ability may move cards from a library (CR 605.1b, unlike an
    // activated one, 605.1a).
    let insight = custom_card(
        "Fertile Insight",
        "Enchantment",
        None,
        "Whenever you tap a creature for mana, add an additional {G} and draw a card.",
    );
    t.custom(P0, insight, Zone::Battlefield);
    // "{T}: Add {W}." — P0's only creature.
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
    let top = t.g.library_top(P0).unwrap();
    let (hand, library) = (t.hand_size(P0), t.library_size(P0));
    // Paying {W} taps the Pilgrim for mana, and the triggered mana ability draws a card;
    // then the additional cost can't be paid, so casting the spell is illegal.
    let r = t
        .cast(P0, spell)
        .target(Entity::Object(bear))
        .method(CastMethod::Normal)
        .try_go();
    assert!(r.is_err());
    // CR 733.1 says the draw can't be reversed. The engine restores the whole game: the
    // card is back on top of the library, and the hand is as it was.
    assert_eq!(t.g.library_top(P0), Some(top));
    assert_eq!(t.g.obj(top).zone, Zone::Library(P0));
    assert_eq!((t.hand_size(P0), t.library_size(P0)), (hand, library));
    assert!(!t.g.obj(pilgrim).tapped);
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    assert_eq!(t.g.obj(spell).zone, Zone::Hand(P0));
    // With a second creature to tap, the same payment is legal and the draw stays.
    t.battlefield(P0, "Grizzly Bears");
    t.cast(P0, spell)
        .target(Entity::Object(bear))
        .method(CastMethod::Normal)
        .go();
    let drawn = t.g.current(top);
    assert_eq!(t.g.obj(drawn).zone, Zone::Hand(P0));
    assert_eq!((t.hand_size(P0), t.library_size(P0)), (hand, library - 1));
}

#[test]
fn a_reversed_cast_reverses_its_mana_abilities() {
    cr!("733.1", "601.2g");
    interp!("reversal-undoes-mana-abilities");
    use mtg_engine::game::EventObserver;
    use std::sync::{Arc, Mutex};
    let mut t = TestGame::new(2);
    // "{T}, Sacrifice Lotus Petal: Add one mana of any color." P0 controls no creature.
    let petal = t.battlefield(P0, "Lotus Petal");
    let mut offering = custom_card(
        "Hollow Offering",
        "Instant",
        None,
        "As an additional cost to cast this spell, sacrifice a creature.\nYou gain 1 life.",
    );
    let cost = mtg_engine::mana::ManaCost::parse("{W}").unwrap();
    offering.faces[0].chars.colors = cost.colors();
    offering.faces[0].chars.mana_cost = Some(cost);
    let spell = t.custom(P0, offering, Zone::Hand(P0));
    let sacrificed: Arc<Mutex<bool>> = Arc::default();
    let s2 = sacrificed.clone();
    t.g.observer = Some(EventObserver::new(move |_, ev| {
        if matches!(ev, Event::ZoneChange { old, to: Zone::Graveyard(_), .. } if *old == petal) {
            *s2.lock().unwrap() = true;
        }
    }));
    let r = t.cast(P0, spell).method(CastMethod::Normal).try_go();
    assert!(r.is_err());
    // Paying {W} sacrificed the Petal for mana; then the creature to sacrifice was missing.
    assert!(*sacrificed.lock().unwrap());
    // The mana ability is reversed with the cast: the Petal is back, the mana is gone.
    assert!(t.on_battlefield(petal));
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    assert_eq!(t.g.obj(spell).zone, Zone::Hand(P0));
}
