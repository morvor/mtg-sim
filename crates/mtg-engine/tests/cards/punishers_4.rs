//! Punishers whose payer or follow-up is another player (CR 118.12): "that player may
//! exile a card from their graveyard. If the player doesn't, creatures they control can't
//! attack you this turn", "that opponent may pay {X}, where X is the number of cards in
//! their hand. If they don't, they can't attack you this combat", "If the player doesn't,
//! you mill three cards, then ~ deals damage to that player equal to the total mana value
//! of those cards".

use mtg_engine::decision::Answer;
use mtg_engine::designations::become_monarch;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn web_of_inertia_creatures_cant_attack_unless_a_card_is_exiled() {
    cr!("118.12", "611.2c");
    supported("Web of Inertia");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Web of Inertia");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1's graveyard is empty: they can't exile a card.
    t.advance_to(P1, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.g.can_attack_target(bears, Entity::Player(P0)));
    // A creature they control later this turn can't attack you either.
    let goblin = t.battlefield(P1, "Raging Goblin");
    assert!(!t.g.can_attack_target(goblin, Entity::Player(P0)));
}

#[test]
fn web_of_inertia_exiling_a_card_lets_them_attack() {
    cr!("118.12");
    supported("Web of Inertia");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Web of Inertia");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.graveyard(P1, "Lightning Bolt");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(card)]);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.g.can_attack_target(bears, Entity::Player(P0)));
}

#[test]
fn champions_of_minas_tirith_pay_x_equal_to_cards_in_hand() {
    cr!("118.12", "107.3c");
    ruling!(
        "Champions of Minas Tirith",
        "An opponent who chooses not to pay {X} for Champions of Minas Tirith's last ability can still attack planeswalkers you control"
    );
    supported("Champions of Minas Tirith");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Champions of Minas Tirith");
    become_monarch(&mut t.g, P0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.lands(P1, "Mountain", 3);
    // P1 declines to pay.
    t.answer_yes(P1, false);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.g.can_attack_target(bears, Entity::Player(P0)));
    assert!(t.g.can_attack_target(bears, Entity::Object(jace)));
}

#[test]
fn champions_of_minas_tirith_paying_lets_them_attack() {
    cr!("118.12", "107.3c");
    supported("Champions of Minas Tirith");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Champions of Minas Tirith");
    become_monarch(&mut t.g, P0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lands = t.lands(P1, "Mountain", 3);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::BeginningOfCombat);
    // X is P1's hand size as the ability resolves (one card drawn this turn plus none).
    let x = t.hand_size(P1);
    t.resolve_all();
    let tapped = lands.iter().filter(|l| t.obj_now(**l).tapped).count();
    assert_eq!(tapped, x);
    assert!(t.g.can_attack_target(bears, Entity::Player(P0)));
}

#[test]
fn combustible_gearhulk_mills_and_deals_their_total_mana_value() {
    cr!("118.12");
    ruling!(
        "Combustible Gearhulk",
        "If any of the cards put into your graveyard have {X} in their mana cost, X is considered to be 0."
    );
    supported("Combustible Gearhulk");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let g = t.hand(P0, "Combustible Gearhulk");
    t.library_top(P0, "Hill Giant"); // 4
    t.library_top(P0, "Fireball"); // {X}{R}: 1
    t.library_top(P0, "Grizzly Bears"); // 2
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, false);
    let hand = t.hand_size(P0);
    t.cast(P0, g).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1);
    assert!(t.in_graveyard(P0, "Fireball"));
    assert_eq!(t.life(P1), 13);
}

#[test]
fn combustible_gearhulk_opponent_has_you_draw() {
    cr!("118.12");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let g = t.hand(P0, "Combustible Gearhulk");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, true);
    let hand = t.hand_size(P0);
    t.cast(P0, g).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn palantir_of_orthanc_mills_x_and_that_player_loses_their_total() {
    cr!("118.12", "107.3c");
    supported("Palantír of Orthanc");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Palantír of Orthanc");
    // One influence counter already: X is 2 after this one.
    t.g.add_counters(Entity::Object(p), "influence", 1, None);
    let bears = t.library_top(P0, "Grizzly Bears"); // 2
    let giant = t.library_top(P0, "Hill Giant"); // 4
                                                 // Scry 2: keep both on top (Hill Giant first).
    t.answer(
        P0,
        DecisionKind::Scry,
        Answer::Split(vec![giant, bears], vec![]),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, false);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(p, "influence"), 2);
    assert_eq!(t.life(P1), 20 - 6);
}

fn sirocco(pay: bool) -> TestGame {
    supported("Sirocco");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.hand(P1, "Counterspell");
    t.hand(P1, "Opt");
    t.hand(P1, "Lightning Bolt");
    t.answer_yes(P1, pay);
    t.answer_yes(P1, pay);
    let s = t.hand(P0, "Sirocco");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve_all();
    t
}

#[test]
fn sirocco_each_blue_instant_is_discarded_unless_its_owner_pays_four_life() {
    cr!("118.12a");
    let t = sirocco(false);
    assert!(t.in_graveyard(P1, "Counterspell"));
    assert!(t.in_graveyard(P1, "Opt"));
    assert!(t.in_hand(P1, "Lightning Bolt"));
    assert_eq!(t.life(P1), 20);
    let t = sirocco(true);
    assert!(t.in_hand(P1, "Counterspell"));
    assert!(t.in_hand(P1, "Opt"));
    assert_eq!(t.life(P1), 12);
}

#[test]
fn sivriss_mills_for_each_opponent_and_returns_unless_they_pay() {
    cr!("118.12a");
    ruling!(
        "Sivriss, Nightmare Speaker",
        "first mill a card for the first opponent in turn order"
    );
    supported("Sivriss, Nightmare Speaker");
    let mut t = TestGame::new(3);
    let s = t.battlefield(P0, "Sivriss, Nightmare Speaker");
    let orni = t.battlefield(P0, "Ornithopter");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(orni)]);
    // P1 (first) doesn't pay; P2 pays.
    t.answer_yes(P1, false);
    t.answer_yes(P2, true);
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!((t.life(P1), t.life(P2)), (20, 17));
}
