//! Rulings batch P125 — transforming double-faced cards (CR 712): the back face can't be
//! cast and has no mana cost, mana value from the front face (CR 712.8e), color indicator
//! (CR 204), loyalty abilities the turn it transforms (CR 606.3); adventurer cards outside
//! the stack (CR 715.4); total cost with an alternative cost (CR 601.2f).

use crate::r_p125_common::*;
use crate::r_s01_common::watch;
use crate::r_s02_common::can_activate;
use crate::r_s07_common::cast_methods;
use crate::r_s08_common::mana_value;
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color};
use mtg_engine::*;

const LILIANA: &str = "Liliana, Heretical Healer // Liliana, Defiant Necromancer";

/// P0's Liliana, Heretical Healer transforms (another nontoken creature P0 controls dies);
/// returns Liliana, Defiant Necromancer.
fn liliana_walker(t: &mut TestGame) -> ObjectId {
    supported(LILIANA);
    t.battlefield(P0, LILIANA);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(t, bears);
    t.resolve_all();
    let w = t.named_on_battlefield("Liliana, Defiant Necromancer");
    assert_eq!(w.len(), 1);
    w[0]
}

#[test]
fn liliana_back_face_cant_be_cast() {
    cr!("712.8", "712.11");
    ruling!("Liliana, Heretical Healer // Liliana, Defiant Necromancer", "The back face of a double-faced card (in the case of Magic Origins, the planeswalker face) can’t be cast.");
    supported(LILIANA);
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, LILIANA);
    let card = t.hand(P0, LILIANA);
    assert_eq!(cast_methods(&mut t, P0, card), vec![CastMethod::Normal]);
    t.cast(P0, card).go();
    let spell = t.g.stack[0];
    assert_eq!(t.g.obj(spell).chars.name, "Liliana, Heretical Healer");
    assert!(t.g.obj(spell).is(CardType::Creature));
}

#[test]
fn liliana_back_face_has_no_mana_cost_but_a_mana_value_and_color() {
    cr!("712.8e", "204.2", "202.3b");
    ruling!("Liliana, Heretical Healer // Liliana, Defiant Necromancer", "The back face of a double-faced card doesn’t have a mana cost. A double-faced permanent with its back face up has a converted mana cost equal to the converted mana cost of its front face. Each back face has a color indicator that defines its color.");
    let mut t = TestGame::new(2);
    let walker = liliana_walker(&mut t);
    let o = t.obj_now(walker);
    assert!(o.chars.mana_cost.is_none());
    assert_eq!(mana_value(&t, walker), 3);
    assert!(o.chars.colors.contains(Color::Black));
    assert_eq!(o.chars.colors.iter().count(), 1);
}

#[test]
fn liliana_mana_value_off_the_battlefield_is_the_front_faces() {
    cr!("712.8a", "202.3");
    ruling!("Liliana, Heretical Healer // Liliana, Defiant Necromancer", "The converted mana cost of a double-faced card not on the battlefield is the converted mana cost of its front face.");
    let mut t = TestGame::new(2);
    let hand = t.hand(P0, LILIANA);
    let yard = t.graveyard(P0, LILIANA);
    let exiled = t.exile(P0, LILIANA);
    for id in [hand, yard, exiled] {
        assert_eq!(mana_value(&t, id), 3);
    }
}

#[test]
fn liliana_loyalty_abilities_need_a_main_phase_and_empty_stack() {
    cr!("606.3", "606.6");
    ruling!("Liliana, Heretical Healer // Liliana, Defiant Necromancer", "You can activate one of the planeswalker’s loyalty abilities the turn it enters the battlefield. However, you may do so only during one of your main phases when the stack is empty.");
    // It transforms during combat: no loyalty ability until P0's postcombat main phase.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::DeclareAttackers);
    let walker = liliana_walker(&mut t);
    assert!(!can_activate(&mut t, P0, walker));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_activate(&mut t, P0, walker));
    // Not with something on the stack.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    assert!(!can_activate(&mut t, P0, walker));
    t.resolve_all();
    assert!(can_activate(&mut t, P0, walker));
    t.activate(P0, walker, 0, &[]).expect("+2 the turn it transformed");
}

#[test]
fn liliana_plus_two_discards_are_chosen_in_turn_order_then_simultaneous() {
    cr!("101.4", "701.9a");
    ruling!("Liliana, Heretical Healer // Liliana, Defiant Necromancer", "When Liliana, Defiant Necromancer’s first ability resolves, first you choose a card to discard, then each other player in turn order chooses a card to discard, then all those cards are discarded simultaneously.");
    let mut t = TestGame::new(3);
    let walker = liliana_walker(&mut t);
    t.hand(P0, "Hill Giant");
    t.hand(P1, "Gray Ogre");
    t.hand(P1, "Grizzly Bears");
    t.hand(P2, "Savannah Lions");
    t.hand(P2, "Llanowar Elves");
    // When P1 and P2 choose, nobody's card has been discarded yet.
    let seen1 = watch(&mut t, P1, |d| matches!(d, Decision::ChooseEntities { .. }), |g| g.player(PlayerId(0)).graveyard.len());
    let seen2 = watch(&mut t, P2, |d| matches!(d, Decision::ChooseEntities { .. }), |g| g.player(PlayerId(1)).graveyard.len());
    let yard0 = t.graveyard_size(P0);
    let yard1 = t.graveyard_size(P1);
    let from = t.asked().len();
    t.activate(P0, walker, 0, &[]).expect("+2");
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1, P2], "P0 chooses first, then the others in turn order");
    assert_eq!(*seen1.lock().unwrap(), vec![yard0]);
    assert_eq!(*seen2.lock().unwrap(), vec![yard1]);
    assert_eq!(t.graveyard_size(P0), yard0 + 1);
    assert_eq!(t.graveyard_size(P1), yard1 + 1);
    assert_eq!(t.hand_size(P2), 1);
}

#[test]
fn karvanista_is_a_creature_card_outside_the_stack() {
    cr!("715.4", "202.3");
    ruling!(
        "Karvanista, Loyal Lupari // Lupari Shield",
        "This adventurer card is a permanent card in every zone except the stack, as well as while on the stack if not cast as an Adventure."
    );
    supported("Karvanista, Loyal Lupari // Lupari Shield");
    let mut t = TestGame::new(2);
    let yard = t.graveyard(P0, "Karvanista, Loyal Lupari // Lupari Shield");
    let o = t.obj_now(yard);
    assert!(o.is(CardType::Creature) && !o.is(CardType::Sorcery));
    assert_eq!(mana_value(&t, yard), 5);
    // Cast normally, it's a creature spell with mana value 5 on the stack.
    lands_for_cost(&mut t, P0, "Karvanista, Loyal Lupari // Lupari Shield");
    let card = t.hand(P0, "Karvanista, Loyal Lupari // Lupari Shield");
    t.cast(P0, card).go();
    let spell = t.g.stack[0];
    assert!(t.g.obj(spell).is(CardType::Creature));
    assert_eq!(mana_value(&t, spell), 5);
}

#[test]
fn sephara_total_cost_starts_from_the_alternative_cost() {
    cr!("601.2f", "118.9d", "202.3");
    ruling!(
        "Sephara, Sky's Blade",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as Sephara's alternative cost), add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged"
    );
    supported("Sephara, Sky's Blade");
    supported("Sphere of Resistance");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    let fliers: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    // {W} + {1} from the Sphere.
    t.lands(P0, "Plains", 2);
    let card = t.hand(P0, "Sephara, Sky's Blade");
    let alt = cast_methods(&mut t, P0, card)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("alternative cost available");
    let es: Vec<Entity> = fliers.iter().map(|f| obj(*f)).collect();
    t.answer_choose(P0, &es);
    t.cast(P0, card).method(alt).go();
    let spell = t.g.stack[0];
    assert_eq!(mana_value(&t, spell), 7);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
    assert!(fliers.iter().all(|f| t.obj_now(*f).tapped));
}
