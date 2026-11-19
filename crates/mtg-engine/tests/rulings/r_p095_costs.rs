//! Rulings batch P095 — paying life as a cost versus losing life as an effect: a cost is
//! paid as the spell is cast or the ability is activated (CR 601.2h, 602.2b) and can't be
//! paid with more life than you have (CR 119.4); it isn't refunded when the spell is
//! countered. Life lost as an effect happens only if the spell or ability resolves, can
//! take a player below 0 life (CR 119.6), and isn't damage, so it can't be prevented.

use crate::r_p095_common::*;
use crate::r_s01_common::{supported, tokens};
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Life lost as an effect: not a payment
// ---------------------------------------------------------------------------------------

#[test]
fn creatures_that_make_you_lose_life_on_entering_can_be_cast_with_less_life() {
    cr!("119.4", "119.6", "704.5a");
    struct Case {
        card: &'static str,
        loss: i32,
    }
    ruling!(
        "Foul Imp",
        "You can cast this if you have less than 2 life, since the life is lost as an effect and not a payment."
    );
    ruling!(
        "Serpent Warrior",
        "You can cast this if you have less than 3 life, since the life is lost as an effect and not a payment."
    );
    for c in [
        Case {
            card: "Foul Imp",
            loss: 2,
        },
        Case {
            card: "Serpent Warrior",
            loss: 3,
        },
    ] {
        supported(c.card);
        let mut t = TestGame::new(2);
        set_life(&mut t, P0, 1);
        cast_new(&mut t, P0, c.card, &[]);
        t.resolve();
        t.resolve_all();
        assert_eq!(t.life(P0), 1 - c.loss, "{}", c.card);
        assert!(t.has_lost(P0), "{}", c.card);
    }
}

#[test]
fn lord_of_tresserhorn_life_loss_can_take_you_below_zero() {
    cr!("119.6", "119.4");
    ruling!(
        "Lord of Tresserhorn",
        "Since this is a loss of life and not a payment, your life total can be reduced below zero by this."
    );
    supported("Lord of Tresserhorn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    set_life(&mut t, P0, 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let lord = t.enter(P0, "Lord of Tresserhorn");
    t.resolve_all();
    assert_eq!(t.life(P0), -1);
    assert!(t.on_battlefield(lord));
    assert!(t.has_lost(P0));
}

#[test]
fn lord_of_tresserhorn_sacrifices_itself_and_regeneration_doesnt_save_it() {
    cr!("701.21a", "701.19a", "608.2c");
    ruling!(
        "Lord of Tresserhorn",
        "When Lord of Tresserhorn enters, if you have zero or one other creature on the battlefield, Lord of Tresserhorn itself will be sacrificed as part of its ability's effect. Regeneration won't save it. The rest of the effect still happens."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let lord = t.enter(P0, "Lord of Tresserhorn");
    t.settle();
    assert_eq!(abilities_from(&t, lord), 1);
    // A regeneration shield first: "{B}: Regenerate Lord of Tresserhorn."
    t.lands(P0, "Swamp", 1);
    t.activate(P0, lord, 0, &[]).unwrap();
    t.resolve();
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lord of Tresserhorn"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.hand_size(P1), hand + 2);
}

#[test]
fn bitterblossom_life_loss_is_mandatory_and_not_a_payment() {
    cr!("119.4", "119.6", "104.3b");
    ruling!(
        "Bitterblossom",
        "The effect is mandatory. You'll lose 1 life even if you have only 1 life left."
    );
    ruling!(
        "Bitterblossom",
        "The life loss isn't a payment. You'll get a token even if you had 0 life (and another effect is stopping you from losing the game)."
    );
    supported("Bitterblossom");
    // At 1 life: no choice, P0 goes to 0 and loses.
    let mut t = TestGame::new(2);
    let bb = t.battlefield(P0, "Bitterblossom");
    set_life(&mut t, P0, 1);
    let from = t.asked().len();
    to_upkeep(&mut t, P0);
    assert_eq!(abilities_from(&t, bb), 1);
    t.resolve();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    assert_eq!(t.life(P0), 0);
    assert!(t.has_lost(P0));
    // At 0 life with Platinum Angel: the token is still created.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bitterblossom");
    t.battlefield(P0, "Platinum Angel");
    set_life(&mut t, P0, 0);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), -1);
    assert!(!t.has_lost(P0));
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn life_lost_as_a_spells_effect_isnt_lost_if_the_spell_doesnt_resolve() {
    cr!("608.2b", "701.6a");
    ruling!(
        "Boon of Erebos",
        "The loss of life is part of the spell's effect. It's not an additional cost. If Boon of Erebos doesn't resolve, you won't lose life."
    );
    ruling!(
        "Ulcerate",
        "The loss of life isn't a cost. If the target creature is an illegal target when Ulcerate tries to resolve, it won't resolve and none of its effects will happen. You won't lose any life."
    );
    supported("Boon of Erebos");
    supported("Ulcerate");
    // Boon of Erebos countered.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let boon = cast_new(&mut t, P0, "Boon of Erebos", &[Entity::Object(bears)]);
    assert_eq!(t.life(P0), 20);
    counter_with_cancel(&mut t, boon);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Boon of Erebos"));
    assert_eq!(t.life(P0), 20);
    // Resolving, it costs 2 life.
    cast_new(&mut t, P0, "Boon of Erebos", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // Ulcerate whose target left the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Ulcerate", &[Entity::Object(bears)]);
    bounce(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ulcerate"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn eviscerator_life_loss_isnt_damage_and_cant_be_prevented() {
    cr!("119.3", "615.1");
    ruling!(
        "Eviscerator",
        "This is loss of life, and not damage. It can’t be prevented."
    );
    supported("Eviscerator");
    let mut t = TestGame::new(2);
    // Safe Passage: "Prevent all damage that would be dealt this turn to you and
    // creatures you control."
    cast_new(&mut t, P0, "Safe Passage", &[]);
    t.resolve();
    t.enter(P0, "Eviscerator");
    t.resolve_all();
    assert_eq!(t.life(P0), 15);
}

#[test]
fn carnival_of_souls_life_loss_is_not_optional() {
    cr!("603.5", "119.3");
    ruling!("Carnival of Souls", "This is not optional.");
    supported("Carnival of Souls");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Carnival of Souls");
    let from = t.asked().len();
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn fretwork_colony_loses_life_even_if_it_left_the_battlefield() {
    cr!("609.3", "400.7");
    ruling!(
        "Fretwork Colony",
        "You lose 1 life even if you can't put a +1/+1 counter on Fretwork Colony, perhaps because it left the battlefield in response to its triggered ability."
    );
    supported("Fretwork Colony");
    let mut t = TestGame::new(2);
    let colony = t.battlefield(P0, "Fretwork Colony");
    to_upkeep(&mut t, P0);
    assert_eq!(abilities_from(&t, colony), 1);
    bounce(&mut t, colony);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert!(t.in_hand(P0, "Fretwork Colony"));
}

// ---------------------------------------------------------------------------------------
// Life paid as a cost
// ---------------------------------------------------------------------------------------

#[test]
fn life_paid_to_cast_a_spell_isnt_returned_if_it_is_countered() {
    cr!("601.2h", "119.4", "701.6a");
    ruling!(
        "Hatred",
        "The life payment is part of the cost to cast Hatred, so it is lost if this spell is countered."
    );
    ruling!(
        "Withering Boon",
        "The payment of 3 life is an additional cost of casting the spell."
    );
    supported("Hatred");
    supported("Withering Boon");
    // Hatred with X = 3: 3 life paid as it is cast, not returned when Cancel counters it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Hatred");
    let card = t.hand(P0, "Hatred");
    let from = t.g.turn_events.len();
    let hatred = t.cast(P0, card).x(3).target(bears).go();
    assert_eq!(paid_life_since(&t, from, P0), vec![3]);
    assert_eq!(t.life(P0), 17);
    counter_with_cancel(&mut t, hatred);
    assert!(t.in_graveyard(P0, "Hatred"));
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.pt(bears), (2, 2));
    // Withering Boon: 3 life paid as it is cast, while it is on the stack.
    let mut t = TestGame::new(2);
    let spell = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    let from = t.g.turn_events.len();
    cast_new(&mut t, P1, "Withering Boon", &[Entity::Object(spell)]);
    assert_eq!(paid_life_since(&t, from, P1), vec![3]);
    assert_eq!(t.life(P1), 17);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn phyrexian_purge_life_isnt_returned_when_countered_or_targets_become_illegal() {
    cr!("601.2f", "601.2h", "608.2b");
    ruling!(
        "Phyrexian Purge",
        "The life payment is an additional cost, paid at the time you cast Phyrexian Purge. You don’t get any life back if the spell is countered, or if one or more targets have become illegal."
    );
    supported("Phyrexian Purge");
    // Two targets: 6 life paid as it is cast. One target leaves; the other is destroyed.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Raging Goblin");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Phyrexian Purge");
    let card = t.hand(P0, "Phyrexian Purge");
    t.cast(P0, card)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    assert_eq!(t.life(P0), 14);
    bounce(&mut t, a);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Raging Goblin"));
    assert_eq!(t.life(P0), 14);
    // Countered: the life stays paid.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Phyrexian Purge");
    let card = t.hand(P0, "Phyrexian Purge");
    let purge = t.cast(P0, card).targets(&[Entity::Object(a)]).go();
    assert_eq!(t.life(P0), 17);
    counter_with_cancel(&mut t, purge);
    assert!(t.on_battlefield(a));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn bond_of_agony_x_life_and_x_mana() {
    cr!("119.4", "107.3a", "601.2h", "704.5a");
    ruling!(
        "Bond of Agony",
        "You must pay both X life and {X}{B}. For example, if you want each other player to lose 2 life when Bond of Agony resolves, you must pay {2}{B} and 2 life."
    );
    ruling!(
        "Bond of Agony",
        "You can’t choose an X that’s greater than your life total."
    );
    ruling!(
        "Bond of Agony",
        "You can choose an X that’s equal to your life total. If you do, you’ll pay that much life when the spell is cast and will lose the game before it resolves."
    );
    supported("Bond of Agony");
    // X = 2: {2}{B} and 2 life; each other player loses 2.
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 3);
    let card = t.hand(P0, "Bond of Agony");
    t.cast(P0, card).x(2).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (18, 18, 18));
    // X greater than the life total: can't be cast that way.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 3);
    t.lands(P0, "Swamp", 6);
    let card = t.hand(P0, "Bond of Agony");
    assert!(t.cast(P0, card).x(4).try_go().is_err());
    assert_eq!(t.life(P0), 3);
    // X equal to the life total: the life is paid, and P0 loses before it resolves.
    t.clear_answers();
    t.cast(P0, card).x(3).go();
    assert_eq!(t.life(P0), 0);
    t.settle();
    assert!(t.has_lost(P0));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn bond_of_agony_cast_without_paying_its_mana_cost_has_x_zero() {
    cr!("107.3b", "107.3i", "118.9");
    ruling!(
        "Omniscience",
        "If a spell has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    // Bond of Agony ({X}{B}, "As an additional cost to cast this spell, pay X life.") cast
    // with Omniscience: the X of its additional cost is the X of its mana cost (CR 107.3i),
    // which can only be 0, so no life is paid and no one loses any.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let card = t.hand(P0, "Bond of Agony");
    let spell = t.cast(P0, card).method(CastMethod::Free).x(3).go();
    assert_eq!(crate::r_s25_common::x_of(&t, spell), Some(0));
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Hatred has no {X} in its mana cost: its X is still chosen and paid (CR 107.3a).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P0, "Hatred");
    t.cast(P0, card)
        .method(CastMethod::Free)
        .x(5)
        .target(bears)
        .go();
    assert_eq!(t.life(P0), 15);
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 2));
}

#[test]
fn tavern_swindler_needs_three_life_to_activate() {
    cr!("119.4", "602.2b", "704.5a");
    ruling!(
        "Tavern Swindler",
        "You can't activate Tavern Swindler's ability if your life total is less than 3. You can activate it if your life total is exactly 3, but it's probably a bad idea. You'll lose the game before the ability resolves."
    );
    supported("Tavern Swindler");
    let mut t = TestGame::new(2);
    let swindler = t.battlefield(P0, "Tavern Swindler");
    set_life(&mut t, P0, 2);
    assert!(t.activate(P0, swindler, 0, &[]).is_err());
    assert_eq!(t.life(P0), 2);
    set_life(&mut t, P0, 3);
    t.activate(P0, swindler, 0, &[]).unwrap();
    assert_eq!(t.life(P0), 0);
    assert_eq!(abilities_from(&t, swindler), 1);
    t.settle();
    assert!(t.has_lost(P0));
}
