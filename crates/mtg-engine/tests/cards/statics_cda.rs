//! Characteristic-defining P/T abilities (CR 604.3, 613.4a), blocker limits, attacking
//! despite defender, and more conditions, compiled by `oracle/patterns/statics*.rs`.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn tarmogoyf_counts_card_types_in_all_graveyards_in_every_zone() {
    cr!("604.3", "613.4a");
    compiles("Tarmogoyf");
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P0, "Tarmogoyf");
    let in_hand = t.hand(P0, "Tarmogoyf");
    assert_eq!(t.pt(goyf), (0, 1));
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Forest");
    t.graveyard(P1, "Hill Giant");
    t.settle();
    // Instant, creature, land.
    assert_eq!(t.pt(goyf), (3, 4));
    // A characteristic-defining ability works in every zone.
    assert_eq!(t.pt(in_hand), (3, 4));
}

#[test]
fn power_and_toughness_equal_to_creature_cards_in_all_graveyards() {
    cr!("604.3", "613.4a");
    compiles("Mortivore");
    let mut t = TestGame::new(2);
    let mortivore = t.battlefield(P0, "Mortivore");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Lightning Bolt");
    t.settle();
    assert_eq!(t.pt(mortivore), (2, 2));
}

#[test]
fn cant_be_blocked_by_more_than_one_creature() {
    cr!("509.1b");
    compiles("Charging Rhino");
    // Two blockers is an illegal block; the defending player's declaration is rejected.
    let mut t = TestGame::new(2);
    let rhino = t.battlefield(P0, "Charging Rhino");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(rhino, Entity::Player(P1))], &[(b1, rhino), (b2, rhino)]);
    assert_eq!(t.life(P1), 16);
    assert!(t.on_battlefield(b1) && t.on_battlefield(b2));

    // One blocker is fine.
    let mut t = TestGame::new(2);
    let rhino = t.battlefield(P0, "Charging Rhino");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(rhino, Entity::Player(P1))], &[(b1, rhino)]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(b1));
}

#[test]
fn walls_can_attack_as_though_they_didnt_have_defender() {
    cr!("702.3b");
    compiles("Rolling Stones");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!t.can_attack(wall));
    t.battlefield(P0, "Rolling Stones");
    t.settle();
    assert!(t.can_attack(wall));
    // Defender is still there; the effect only lets it attack.
    assert!(t
        .obj_now(wall)
        .has_keyword(mtg_engine::keywords::KeywordKind::Defender));
}

#[test]
fn activated_can_attack_this_turn_as_though_it_didnt_have_defender() {
    cr!("702.3b", "611.2a");
    compiles("Krotiq Nestguard");
    let mut t = TestGame::new(2);
    let nest = t.battlefield(P0, "Krotiq Nestguard");
    t.lands(P0, "Forest", 3);
    assert!(!t.can_attack(nest));
    t.activate(P0, nest, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.can_attack(nest));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(nest, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    // Until end of turn only.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.can_attack(nest));
}

#[test]
fn as_long_as_an_opponent_has_many_cards_in_their_graveyard() {
    cr!("611.3a", "108.2b");
    compiles("Jace's Phantasm");
    let mut t = TestGame::new(2);
    let phantasm = t.battlefield(P0, "Jace's Phantasm");
    for _ in 0..9 {
        t.graveyard(P1, "Grizzly Bears");
    }
    // Your own graveyard doesn't count.
    t.graveyard(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(phantasm), (1, 1));
    // Neither does a token in theirs: it isn't a card (CR 108.2b).
    super::statics::token_in_graveyard(&mut t, P1);
    t.g.recompute();
    assert_eq!(t.g.player(P1).graveyard.len(), 10);
    assert_eq!(t.pt(phantasm), (1, 1));
    t.graveyard(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(phantasm), (5, 5));
}

#[test]
fn as_long_as_there_is_a_land_card_in_your_graveyard() {
    cr!("611.3a");
    compiles("Murasa Behemoth");
    let mut t = TestGame::new(2);
    let behemoth = t.battlefield(P0, "Murasa Behemoth");
    t.graveyard(P1, "Forest");
    t.graveyard(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(behemoth), (5, 5));
    t.graveyard(P0, "Forest");
    t.settle();
    assert_eq!(t.pt(behemoth), (8, 8));
}
