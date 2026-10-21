//! A choice between costs (CR 118.3, 601.2h): "Equip—Pay {3} or discard a card." (an
//! equip ability for each cost, CR 702.6a), "Ward—Discard a card or pay {2}." (the player
//! chooses as the ward ability resolves, CR 702.21a), "you may discard a card or pay {2}.
//! When you do, ..." (an optional cost and a reflexive triggered ability, CR 118.12,
//! 603.12); and optional costs of two parts: "you may sacrifice ~ and pay {2}{G}{G}. If you
//! do, ...", "you may pay {4}{U}{U}{B}{B} and exile eight cards from your graveyard".

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn bloodthorn_flail_equips_for_three_or_a_discarded_card() {
    cr!("702.6a", "118.3");
    assert_supported("Bloodthorn Flail");
    let mut t = TestGame::new(2);
    let flail = t.battlefield(P0, "Bloodthorn Flail");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // No mana and no cards: neither cost can be paid.
    assert!(t.activate(P0, flail, 0, &[Entity::Object(bears)]).is_err());
    assert!(t.activate(P0, flail, 1, &[Entity::Object(bears)]).is_err());
    t.clear_answers();
    // With a card in hand, the discard cost.
    t.hand(P0, "Island");
    t.activate(P0, flail, 1, &[Entity::Object(bears)]).unwrap();
    assert!(t.in_graveyard(P0, "Island"));
    t.resolve();
    assert_eq!(t.pt(bears), (4, 3));
    // With three mana, the other.
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, flail, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve();
    assert_eq!(t.pt(giant), (5, 4));
}

#[test]
fn gavel_of_the_righteous_equips_by_removing_a_counter() {
    cr!("702.6a", "118.3");
    assert_supported("Gavel of the Righteous");
    let mut t = TestGame::new(2);
    let gavel = t.battlefield(P0, "Gavel of the Righteous");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.activate(P0, gavel, 1, &[Entity::Object(bears)]).is_err());
    t.clear_answers();
    t.g.add_counters(Entity::Object(gavel), "charge", 2, None);
    t.activate(P0, gavel, 1, &[Entity::Object(bears)]).unwrap();
    assert_eq!(t.counters(gavel, "charge"), 1);
    t.resolve();
    // +1/+1 for each counter on it: one left.
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn titania_ward_discard_a_card_or_pay_two() {
    cr!("702.21a", "118.3");
    assert_supported("Titania, Rugged Rumbler");
    // P1 has no mana left after casting, but a card to discard: the spell isn't countered.
    let mut t = TestGame::new(2);
    let titania = t.battlefield(P0, "Titania, Rugged Rumbler");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Island");
    t.cast(P1, bolt).target(titania).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Island"));
    assert_eq!(t.g.obj(titania).damage, 3);
    // With {2} and no other card, P1 pays the mana instead.
    let mut t = TestGame::new(2);
    let titania = t.battlefield(P0, "Titania, Rugged Rumbler");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(titania).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(t.g.obj(titania).damage, 3);
    // Neither: the spell is countered.
    let mut t = TestGame::new(2);
    let titania = t.battlefield(P0, "Titania, Rugged Rumbler");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(titania).go();
    t.resolve_all();
    assert_eq!(t.g.obj(titania).damage, 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn anthropede_discards_or_pays_then_destroys_target_room() {
    cr!("603.12", "118.12");
    ruling!("Anthropede", "a second \"reflexive\" ability triggers when you discard a card");
    assert_supported("Anthropede");
    let mut t = TestGame::new(2);
    let room = t.battlefield(P1, "Cramped Vents // Access Maze");
    t.hand(P0, "Island");
    t.enter(P0, "Anthropede");
    t.settle();
    // Yes to the optional cost; the reflexive ability's target is chosen as it's put on
    // the stack.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(room)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Island"));
    assert!(!t.on_battlefield(room));
    // Declined: nothing happens.
    let mut t = TestGame::new(2);
    let room = t.battlefield(P1, "Cramped Vents // Access Maze");
    t.hand(P0, "Island");
    t.enter(P0, "Anthropede");
    t.settle();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_hand(P0, "Island"));
    assert!(t.on_battlefield(room));
}

#[test]
fn nimble_hobbit_pays_mana_without_a_food() {
    cr!("603.12", "118.12", "118.3");
    ruling!("Nimble Hobbit", "triggers when you sacrifice a Food or pay {2}{W} this way");
    assert_supported("Nimble Hobbit");
    let mut t = TestGame::new(2);
    let hobbit = t.battlefield(P0, "Nimble Hobbit");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 3);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.attack(&[(hobbit, Entity::Player(P1))], &[]);
    assert!(t.g.obj(giant).tapped);
    // The mana was paid.
    assert!(t
        .g
        .permanents()
        .filter(|o| o.chars.has_subtype("Plains"))
        .all(|o| o.tapped));
}

#[test]
fn preferred_selection_sacrifices_itself_and_pays() {
    cr!("118.12", "118.3");
    ruling!("Preferred Selection", "no longer a draw");
    assert_supported("Preferred Selection");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Preferred Selection");
    t.lands(P0, "Forest", 4);
    t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Hill Giant");
    t.advance_to(P0, Step::Upkeep);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Preferred Selection"));
    assert!(t.in_hand(P0, "Hill Giant") || t.in_hand(P0, "Craw Wurm"));
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.chars.has_subtype("Forest") && o.tapped)
            .count(),
        4
    );
}

#[test]
fn preferred_selection_unpaid_puts_a_card_on_the_bottom() {
    cr!("118.12");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Preferred Selection");
    // No mana: the cost can't be paid.
    t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Upkeep);
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Preferred Selection").len(), 1);
    assert_eq!(t.hand_size(P0), hand);
    // One of the two is now on the bottom.
    let lib = &t.g.player(P0).library;
    let bottom = t.g.obj(lib[0]).chars.name.clone();
    assert!(bottom == "Hill Giant" || bottom == "Craw Wurm", "{bottom}");
}

#[test]
fn ultimecia_pays_mana_and_exiles_eight_cards_to_transform() {
    cr!("118.12", "118.3", "701.27a");
    assert_supported("Ultimecia, Time Sorceress // Ultimecia, Omnipotent");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ultimecia, Time Sorceress // Ultimecia, Omnipotent");
    t.lands(P0, "Island", 4);
    t.lands(P0, "Swamp", 4);
    for _ in 0..7 {
        t.graveyard(P0, "Grizzly Bears");
    }
    // Seven cards: the cost can't be paid.
    t.advance_to(P0, Step::End);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 7);
    assert!(t.named_on_battlefield("Ultimecia, Omnipotent").is_empty());
    // With eight, it transforms.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ultimecia, Time Sorceress // Ultimecia, Omnipotent");
    t.lands(P0, "Island", 4);
    t.lands(P0, "Swamp", 4);
    for _ in 0..8 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.advance_to(P0, Step::End);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.named_on_battlefield("Ultimecia, Omnipotent").len(), 1);
}
