//! Digs and moves by each player (round-4 card flow): each opponent's own revealed cards
//! (Valki, God of Lies), the cards each player exiled returned to them later (Magus of the
//! Jar), "both cards" (Sword of Hearth and Home), each player's own dig (Explore the
//! Vastlands).

use crate::basic_effects_common::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn valki_exiles_a_creature_card_from_each_opponents_revealed_hand() {
    cr!("701.20a", "610.3");
    assert_supported("Valki, God of Lies // Tibalt, Cosmic Impostor");
    let mut t = TestGame::new(3);
    let a = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Lightning Bolt");
    let b = t.hand(P2, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(b)]);
    let valki = t.enter(P0, "Valki, God of Lies // Tibalt, Cosmic Impostor");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    assert!(t.in_hand(P1, "Lightning Bolt"));
    // Until Valki leaves the battlefield.
    let v = t.g.current(valki);
    t.g.move_object(v, Zone::Graveyard(P0), events::MoveCause::Effect, None);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears") && t.in_hand(P2, "Hill Giant"));
}

#[test]
fn valki_leaving_before_its_ability_resolves_exiles_nothing() {
    cr!("610.3c");
    ruling!(
        "Valki, God of Lies // Tibalt, Cosmic Impostor",
        "each opponent will reveal their hand, but no cards will be exiled"
    );
    let mut t = TestGame::new(2);
    let a = t.hand(P1, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(a)]);
    let valki = t.enter(P0, "Valki, God of Lies // Tibalt, Cosmic Impostor");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.move_object(valki, Zone::Graveyard(P0), events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Hand(P1));
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn magus_of_the_jar_returns_each_players_exiled_hand_at_end_step() {
    cr!("406.3", "603.7a");
    assert_supported("Magus of the Jar");
    let mut t = TestGame::new(2);
    let magus = t.battlefield(P0, "Magus of the Jar");
    let bolt = t.hand(P0, "Lightning Bolt");
    let bears = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.activate(P0, magus, 0, &[]).unwrap();
    t.resolve();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (7, 7));
    assert_eq!(t.g.exile.len(), 3);
    for c in [bolt, bears] {
        let c = t.g.current(c);
        assert_eq!(t.zone(c), Zone::Exile);
        assert!(t.g.obj(c).face_down);
    }
    t.advance_to_step(Step::End);
    t.resolve_all();
    // The seven cards were discarded; each player has their own exiled cards back.
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(t.in_hand(P1, "Grizzly Bears") && t.in_hand(P1, "Hill Giant"));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (1, 2));
}

#[test]
fn sword_of_hearth_and_home_puts_both_cards_onto_the_battlefield() {
    cr!("608.2c", "701.23a");
    ruling!("Sword of Hearth and Home", "the land and the creature will enter the battlefield at the same time");
    assert_supported("Sword of Hearth and Home");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Sword of Hearth and Home");
    t.g.attach(sword, Entity::Object(bears));
    let lynx = t.battlefield(P0, "Steppe Lynx");
    t.library_top(P0, "Forest");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(lynx)]);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    let l = t.named_on_battlefield("Steppe Lynx");
    assert_eq!(l.len(), 1);
    assert_ne!(l[0], lynx);
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    // The returned Lynx saw the Forest enter (landfall): +2/+2.
    assert_eq!(t.pt(l[0]), (2, 3));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn explore_the_vastlands_each_player_digs_their_own_library() {
    cr!("101.4", "608.2e");
    ruling!(
        "Wandering Archaic // Explore the Vastlands",
        "revealing either a land card or an instant or sorcery card is optional"
    );
    assert_supported("Wandering Archaic // Explore the Vastlands");
    let mut t = TestGame::new(2);
    let mut ids = Vec::new();
    for (p, cards) in [
        (P0, ["Island", "Lightning Bolt", "Grizzly Bears", "Forest", "Hill Giant"]),
        (P1, ["Hill Giant", "Craw Wurm", "Shock", "Mountain", "Grizzly Bears"]),
    ] {
        for c in cards {
            ids.push(t.library_top(p, c));
        }
    }
    t.lands(P0, "Plains", 3);
    let e = t.hand(P0, "Wandering Archaic // Explore the Vastlands");
    let (h0, h1) = (t.hand_size(P0) - 1, t.hand_size(P1));
    // P0 takes only the Bolt; P1 a land and an instant from their own cards.
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.answer_choose(P1, &[Entity::Object(ids[7]), Entity::Object(ids[8])]);
    t.cast(P0, e).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1 + 2);
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(t.in_hand(P1, "Shock") && t.in_hand(P1, "Mountain"));
    // The rest went to the bottom of each player's own library.
    let bottom = |p: PlayerId| {
        t.g.player(p).library[..4]
            .iter()
            .map(|o| t.g.obj(*o).name().to_string())
            .collect::<Vec<_>>()
    };
    assert!(bottom(P1).contains(&"Craw Wurm".to_string()));
    assert!(bottom(P0).contains(&"Grizzly Bears".to_string()));
    assert_eq!((t.life(P0), t.life(P1)), (23, 23));
}

#[test]
fn boreas_charger_finds_as_many_plains_as_the_land_difference() {
    cr!("701.23a", "107.1");
    assert_supported("Boreas Charger");
    let mut t = TestGame::new(2);
    let charger = t.battlefield(P0, "Boreas Charger");
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Mountain", 4);
    for _ in 0..4 {
        t.library_top(P0, "Plains");
    }
    let hand = t.hand_size(P0);
    t.g.move_object(charger, Zone::Graveyard(P0), events::MoveCause::Effect, None);
    t.resolve_all();
    // 4 - 1 = 3 Plains: one onto the battlefield tapped, two into the hand.
    let plains = t.named_on_battlefield("Plains");
    assert_eq!(plains.len(), 2);
    assert_eq!(plains.iter().filter(|p| t.obj_now(**p).tapped).count(), 1);
    assert_eq!(t.hand_size(P0), hand + 2);
}

fn verdant_game() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    for n in ["Forest", "Forest", "Island", "Plains"] {
        t.library_top(P0, n);
    }
    let v = t.hand(P0, "Verdant Mastery");
    t.lands(P0, "Forest", 6);
    (t, v)
}

#[test]
fn verdant_mastery_gives_an_opponent_a_land_only_for_the_alternative_cost() {
    cr!("118.9", "701.23a");
    assert_supported("Verdant Mastery");
    let (mut t, v) = verdant_game();
    let alt = t
        .cast_options(P0, v)
        .into_iter()
        .map(|o| o.method)
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .unwrap();
    let hand = t.hand_size(P0) - 1;
    t.cast(P0, v).method(alt).go();
    t.resolve_all();
    let theirs = t.g.battlefield.iter().filter(|o| t.g.obj(**o).controller == P1).count();
    assert_eq!(theirs, 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Without it: two onto the battlefield, the rest into the hand.
    let (mut t, v) = verdant_game();
    let hand = t.hand_size(P0) - 1;
    t.cast(P0, v).go();
    t.resolve_all();
    let theirs = t.g.battlefield.iter().filter(|o| t.g.obj(**o).controller == P1).count();
    assert_eq!(theirs, 0);
    assert_eq!(t.hand_size(P0), hand + 2);
}
