//! Dice and coins, round 3 (`oracle/patterns/dice_coins_4.rs`): the results of the dice
//! that triggered an ability, each different result, subtracting from a roll, a results
//! table after a sentence about other results, and cards from among the cards exiled
//! with a permanent.

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn tokens_named(t: &TestGame, subtype: &str) -> usize {
    t.g.zone_objects(Zone::Battlefield)
        .into_iter()
        .filter(|o| t.obj_now(*o).chars.subtypes.iter().any(|s| s == subtype))
        .count()
}

#[test]
fn farideh_draws_if_any_of_the_dice_rolled_was_ten_or_higher() {
    cr!("706.1", "603.2");
    assert_supported(&["Farideh, Devil's Chosen"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let farideh = t.battlefield(P0, "Farideh, Devil's Chosen");
    t.lands(P0, "Mountain", 3);
    // Three six-sided dice: none is 10 or higher.
    let act = t.hand(P0, "Circuits Act");
    t.g.dice.loaded.extend([2, 5, 5]);
    let hand = t.hand_size(P0);
    t.cast(P0, act).go();
    t.resolve_all();
    assert!(t
        .obj_now(farideh)
        .chars
        .has_keyword(keywords::KeywordKind::Flying));
    assert_eq!(t.hand_size(P0), hand - 1);
    // A d20 showing 12 (an effect tells P0 to roll it): draw a card.
    let mut ctx = mtg_engine::eval::Ctx::new(Some(farideh), P0);
    let roll = mtg_engine::dice::DieRoll::new(20);
    t.g.dice.loaded.push_back(12);
    mtg_engine::dice::roll(&mut t.g, &roll, &mut ctx);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn circuits_act_creates_a_token_for_each_different_result() {
    cr!("706.1");
    ruling!(
        "Circuits Act",
        "For example, if you rolled 1, 4, and 4, you'd create two Clown Robots."
    );
    assert_supported(&["Circuits Act"]);
    for (dice, expected) in [([1, 4, 4], 2), ([3, 3, 3], 1), ([1, 2, 6], 3)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.lands(P0, "Mountain", 3);
        let act = t.hand(P0, "Circuits Act");
        t.g.dice.loaded.extend(dice);
        t.cast(P0, act).go();
        t.resolve_all();
        assert_eq!(tokens_named(&t, "Robot"), expected, "{dice:?}");
    }
}

#[test]
fn the_deck_of_many_things_subtracts_cards_in_hand() {
    cr!("706.2", "706.3", "706.3a");
    ruling!(
        "The Deck of Many Things",
        "cares about the result of the die roll after any adjustments"
    );
    assert_supported(&["The Deck of Many Things"]);
    // 12 minus three cards in hand: 9, return a card at random.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let deck = t.battlefield(P0, "The Deck of Many Things");
    t.lands(P0, "Plains", 2);
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    t.graveyard(P0, "Lightning Bolt");
    t.g.dice.loaded.push_back(12);
    t.activate(P0, deck, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(t.hand_size(P0), 4);
    // 2 minus four cards in hand: -2, discard your hand; no row applies.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let deck = t.battlefield(P0, "The Deck of Many Things");
    t.lands(P0, "Plains", 2);
    for _ in 0..4 {
        t.hand(P0, "Grizzly Bears");
    }
    t.graveyard(P0, "Lightning Bolt");
    t.g.dice.loaded.push_back(2);
    t.activate(P0, deck, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    // A natural 20 with an empty hand: a creature card from any graveyard.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let deck = t.battlefield(P0, "The Deck of Many Things");
    t.lands(P0, "Plains", 2);
    let wurm = t.graveyard(P1, "Craw Wurm");
    t.g.dice.loaded.push_back(20);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.activate(P0, deck, 0, &[]).unwrap();
    t.resolve_all();
    let wurms = t.named_on_battlefield("Craw Wurm");
    assert_eq!(wurms.len(), 1);
    assert_eq!(t.obj_now(wurms[0]).controller, P0);
}

#[test]
fn bag_of_devouring_returns_up_to_the_result_of_the_exiled_cards() {
    cr!("706.4", "607.1a");
    ruling!(
        "Bag of Devouring",
        "It won't be a card exiled with Bag of Devouring for the third ability."
    );
    assert_supported(&["Bag of Devouring"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bag = t.battlefield(P0, "Bag of Devouring");
    t.lands(P0, "Plains", 9);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Ornithopter");
    let c = t.battlefield(P0, "Memnite");
    for x in [a, b, c] {
        t.answer_choose(P0, &[Entity::Object(x)]);
        t.activate(P0, bag, 0, &[]).unwrap();
        t.resolve_all();
        t.g.objects[bag.0 as usize].tapped = false;
    }
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Ornithopter"));
    assert!(t.in_exile("Memnite"));
    // A 2: up to two of them return.
    let exiled: Vec<Entity> =
        t.g.zone_objects(Zone::Exile)
            .into_iter()
            .map(Entity::Object)
            .collect();
    assert_eq!(exiled.len(), 3);
    t.answer_choose(P0, &exiled[..2]);
    t.g.dice.loaded.push_back(2);
    t.activate(P0, bag, 1, &[]).unwrap();
    t.resolve_all();
    let in_hand = ["Grizzly Bears", "Ornithopter", "Memnite"]
        .iter()
        .filter(|n| t.in_hand(P0, n))
        .count();
    assert_eq!(in_hand, 2);
}
