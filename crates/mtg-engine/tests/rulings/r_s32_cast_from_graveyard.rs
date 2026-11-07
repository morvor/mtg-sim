//! Rulings batch S32 — casting a card from the graveyard with its own ability: "You may
//! cast this card from your graveyard by discarding two cards in addition to paying its
//! other costs." (Rona, Sheoldred's Faithful), "... by paying {3}{R} and exiling four other
//! cards from your graveyard rather than paying its mana cost." (Squee, Dubious Monarch).
//! The card moves to the stack as casting begins (CR 601.2a) and the costs are paid last
//! (CR 601.2h).

use crate::r_s01_common::*;
use crate::r_s04_common::graveyard_names;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether a card named `name` is on the stack / in `P0`'s graveyard.
fn where_is(g: &mtg_engine::game::Game, name: &str) -> (bool, bool) {
    let on_stack = g.stack.iter().any(|s| g.obj(*s).chars.name.starts_with(name));
    let in_gy = g
        .player(P0)
        .graveyard
        .iter()
        .any(|c| g.obj(*c).chars.name.starts_with(name));
    (on_stack, in_gy)
}

/// The casting method of the card's "you may cast this card from your graveyard" ability.
fn graveyard_method(t: &mut TestGame, card: ObjectId) -> CastMethod {
    let methods = legal_cast_methods(t, P0, card);
    let alt: Vec<CastMethod> = methods
        .into_iter()
        .filter(|m| matches!(m, CastMethod::Alternative(_)))
        .collect();
    assert_eq!(alt.len(), 1, "{alt:?}");
    alt[0].clone()
}

#[test]
fn a_card_cast_from_the_graveyard_is_on_the_stack_while_its_costs_are_paid() {
    cr!("601.2a", "601.2h", "601.2f", "113.6m");
    ruling!(
        "Rona, Sheoldred's Faithful",
        "Once you begin casting a spell from your graveyard, it immediately moves to the stack. Players can't take any other actions until you've finished casting the spell."
    );
    supported("Rona, Sheoldred's Faithful");
    let mut t = TestGame::new(2);
    let rona = t.graveyard(P0, "Rona, Sheoldred's Faithful");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 3);
    // One card in hand: it can't be cast that way.
    let shock = t.hand(P0, "Shock");
    assert!(legal_cast_methods(&mut t, P0, rona).is_empty());
    let forest = t.hand(P0, "Forest");
    // Karador ("Once during each of your turns, you may cast a creature spell from your
    // graveyard") would let P0 cast it normally too.
    t.battlefield(P0, "Karador, Ghost Chieftain");
    let method = graveyard_method(&mut t, rona);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| where_is(g, "Rona"),
    );
    t.answer_choose(P0, &[Entity::Object(shock), Entity::Object(forest)]);
    let from = t.asked().len();
    t.cast(P0, rona).method(method).go();
    // As the two cards were chosen to discard, Rona was already on the stack, and no
    // other player was asked anything while it was being cast.
    assert_eq!(seen.lock().unwrap().clone(), vec![(true, false)]);
    assert!(t.asked()[from..].iter().all(|(p, _)| *p == P0));
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Rona, Sheoldred's Faithful").len(), 1);
    let mut gy = graveyard_names(&t, P0);
    gy.sort();
    assert_eq!(gy, vec!["Forest", "Shock"]);
    // Cast with its own permission, it didn't use Karador's.
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert_eq!(legal_cast_methods(&mut t, P0, bears), vec![CastMethod::Normal]);
}

#[test]
fn squee_is_cast_from_the_graveyard_for_its_alternative_cost() {
    cr!("601.2a", "118.9", "601.2h");
    ruling!(
        "Squee, Dubious Monarch",
        "Once you begin casting a spell from your graveyard, it immediately moves to the stack. Players can't take any other actions until you've finished casting the spell."
    );
    supported("Squee, Dubious Monarch");
    let mut t = TestGame::new(2);
    let squee = t.graveyard(P0, "Squee, Dubious Monarch");
    t.lands(P0, "Mountain", 4);
    // Three other cards aren't enough.
    for _ in 0..3 {
        t.graveyard(P0, "Shock");
    }
    assert!(legal_cast_methods(&mut t, P0, squee).is_empty());
    t.graveyard(P0, "Forest");
    let method = graveyard_method(&mut t, squee);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| where_is(g, "Squee"),
    );
    t.cast(P0, squee).method(method).go();
    // Squee was on the stack, not among the cards that could be exiled.
    assert!(seen.lock().unwrap().iter().all(|s| *s == (true, false)));
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.g.exile.len(), 4);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Squee, Dubious Monarch").len(), 1);
}
