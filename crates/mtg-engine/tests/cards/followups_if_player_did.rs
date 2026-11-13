//! "If [player] does/doesn't/can't, [effect]" after an instruction that player performs
//! (CR 118.12, 608.2c, 609.3), and comma series ending in ", then".

use mtg_engine::keywords::KeywordKind;
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
fn doom_foretold_full_consequences_only_if_the_player_cant_sacrifice() {
    cr!("609.3", "608.2c");
    ruling!(
        "Doom Foretold",
        "If the player is unable to sacrifice a permanent and is unable to discard a card, they still lose 2 life"
    );
    supported("Doom Foretold");
    // P1 controls only a land: nothing to sacrifice, so everything else happens.
    let mut t = TestGame::new(2);
    let doom = t.battlefield(P0, "Doom Foretold");
    t.battlefield(P1, "Forest");
    t.advance_to(P1, Step::Upkeep);
    let (hand0, hand1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P0), hand0 + 1);
    assert_eq!(t.hand_size(P1), hand1.saturating_sub(1));
    let knights = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Knight"))
        .count();
    assert_eq!(knights, 1);
    assert!(!t.on_battlefield(doom));
    // P1 controls a creature: they sacrifice it and nothing else happens.
    let mut t = TestGame::new(2);
    let doom = t.battlefield(P0, "Doom Foretold");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(doom));
}

#[test]
fn cocoon_sacrifices_itself_when_it_cant_remove_a_pupa_counter() {
    cr!("609.3", "122.8");
    supported("Cocoon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cocoon = t.battlefield(P0, "Cocoon");
    assert!(t.g.attach(cocoon, Entity::Object(bears)));
    t.g.add_counters(Entity::Object(cocoon), "pupa", 1, None);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // A counter was removed: nothing else happens.
    assert_eq!(t.counters(cocoon, "pupa"), 0);
    assert!(t.on_battlefield(cocoon));
    assert_eq!(t.pt(bears), (2, 2));
    // Next upkeep: no counter to remove.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(cocoon));
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj_now(bears).chars.has_keyword(KeywordKind::Flying));
}

#[test]
fn nefarious_lich_loses_the_game_without_enough_cards_to_exile() {
    cr!("614.1a", "609.3");
    supported("Nefarious Lich");
    // Enough cards: they're exiled instead of the damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nefarious Lich");
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.graveyard_size(P0), 0);
    assert!(!t.has_lost(P0));
    // Fewer cards than damage: nothing is exiled and they lose.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nefarious Lich");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert!(t.has_lost(P0));
}

#[test]
fn binding_negotiation_they_discard_the_chosen_card() {
    cr!("701.9b", "608.2c");
    supported("Binding Negotiation");
    let mut t = TestGame::new(2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Forest");
    t.lands(P0, "Swamp", 2);
    let spell = t.hand(P0, "Binding Negotiation");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Forest"));
}

#[test]
fn breaking_wave_untaps_and_taps_simultaneously() {
    cr!("608.2c", "701.26a");
    supported("Breaking Wave");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.g.tap(a);
    t.lands(P0, "Island", 4);
    let spell = t.hand(P0, "Breaking Wave");
    t.cast(P0, spell).go();
    t.resolve();
    assert!(!t.obj_now(a).tapped);
    assert!(t.obj_now(b).tapped);
}

#[test]
fn starving_revenant_draws_for_each_card_left_on_top() {
    cr!("701.25a");
    supported("Starving Revenant");
    supported("Chandra, Chill of Compliance");
    supported("Covenant of Minds");
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    t.enter(P0, "Starving Revenant");
    t.g.flush_events();
    t.resolve_all();
    // The default keeps both cards on top: two cards drawn, 6 life lost.
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), 14);
}
