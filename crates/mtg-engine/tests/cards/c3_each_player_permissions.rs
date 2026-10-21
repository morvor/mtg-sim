//! Permissions given to other players or to each player (cards3-cast-zones): "Until your
//! next end step, each player may play the card they exiled this way" (Rocco, Street
//! Chef), "Until your next turn, players may play cards they exiled this way, and they
//! can't play cards from their hand" (Memory Vessel), "Until that player's next end step,
//! they may play that card without paying its mana cost" (Casey & Raph, Hotheads), "They
//! may play those cards until the end of their next turn" (Expedited Inheritance). A
//! duration said with "your" is the controller's; one said with "their" or "that
//! player's" is the permission's player's (CR 611.2a).

use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn rocco_each_player_plays_their_own_card_until_your_next_end_step() {
    cr!("611.2a", "305.1");
    ruling!(
        "Rocco, Street Chef",
        "A player may play the card they exiled with Rocco even if Rocco leaves the battlefield"
    );
    assert_supported("Rocco, Street Chef");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rocco, Street Chef");
    t.lands(P1, "Forest", 2);
    let p0_card = t.library_top(P0, "Mountain");
    let p1_card = t.library_top(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let p0_x = t.g.current(p0_card);
    let p1_x = t.g.current(p1_card);
    assert_eq!(t.zone(p0_x), Zone::Exile);
    assert_eq!(t.zone(p1_x), Zone::Exile);
    // Rocco leaves; the permissions remain.
    let rocco = t.named_on_battlefield("Rocco, Street Chef")[0];
    t.g.destroy(rocco, None);
    t.settle();
    // During P1's turn, P1 may cast the card P1 exiled (not P0's, which is P0's).
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.play_land(P1, p0_x).is_err(), "P1 can't play P0's card");
    t.cast_with(P1, p1_x, &[]).expect("P1 casts the card they exiled");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // P0 plays theirs in P0's next turn, before P0's next end step.
    t.advance_to(P0, Step::PrecombatMain);
    let p0_x = t.g.current(p0_card);
    t.play_land(P0, p0_x).expect("P0 plays the card they exiled");
    assert_eq!(t.zone(t.g.current(p0_card)), Zone::Battlefield);
}

#[test]
fn rocco_permission_ends_at_the_controllers_next_end_step() {
    cr!("611.2a");
    ruling!("Rocco, Street Chef", "All cards not played remain exiled.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rocco, Street Chef");
    let p1_card = t.library_top(P1, "Forest");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let p1_x = t.g.current(p1_card);
    // Not played during P1's turn, nor during P0's next turn; P0's next end step ends it,
    // not P1's.
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.zone(p1_x), Zone::Exile);
    assert!(t.play_land(P1, p1_x).is_err(), "{}", t.dump_log());
}

#[test]
fn memory_vessel_players_play_exiled_cards_but_not_from_their_hands() {
    cr!("611.2a", "305.1", "601.3");
    ruling!(
        "Memory Vessel",
        "Players may still play cards from zones other than their hand if something would allow them to do so."
    );
    ruling!(
        "Memory Vessel",
        "Players pay all costs and follow all normal timing rules for cards played from exile with Memory Vessel's ability."
    );
    assert_supported("Memory Vessel");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let vessel = t.battlefield(P0, "Memory Vessel");
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Mountain", 1);
    let p0_bolt = t.library_top(P0, "Lightning Bolt");
    let p1_shock = t.library_top(P1, "Shock");
    let p0_hand = t.hand(P0, "Lightning Bolt");
    let p1_hand = t.hand(P1, "Shock");
    t.activate(P0, vessel, 0, &[]).expect("activate Memory Vessel");
    t.resolve_all();
    let p0_x = t.g.current(p0_bolt);
    let p1_x = t.g.current(p1_shock);
    assert_eq!(t.zone(p0_x), Zone::Exile);
    // P0 can't cast the Bolt in hand, but can cast the exiled one.
    assert!(t.cast_with(P0, p0_hand, &[Entity::Player(P1)]).is_err());
    t.clear_answers();
    t.cast_with(P0, p0_x, &[Entity::Player(P1)]).expect("cast the exiled Bolt");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // P0 can't play P1's card.
    assert!(t.cast_with(P0, p1_x, &[Entity::Player(P1)]).is_err());
    t.clear_answers();
    // During P1's turn: P1 can't cast from hand, but can cast what P1 exiled.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.cast_with(P1, p1_hand, &[Entity::Player(P0)]).is_err());
    t.clear_answers();
    t.cast_with(P1, p1_x, &[Entity::Player(P0)]).expect("cast the exiled Shock");
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // Until P0's next turn: then hands are usable again.
    t.advance_to(P0, Step::PrecombatMain);
    t.cast_with(P0, p0_hand, &[Entity::Player(P1)]).expect("cast from hand again");
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn casey_and_raph_target_player_plays_it_free_until_their_next_end_step() {
    cr!("611.2a", "118.9");
    ruling!(
        "Casey & Raph, Hotheads",
        "The target of the first mode must follow all timing rules for cards played this way."
    );
    assert_supported("Casey & Raph, Hotheads");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.library_top(P1, "Grizzly Bears");
    // The first mode only, targeting P1.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Casey & Raph, Hotheads");
    t.resolve_all();
    let bears_x = t.g.current(bears);
    assert_eq!(t.zone(bears_x), Zone::Exile, "{}", t.dump_log());
    // Not at instant speed during P0's turn (normal timing).
    t.g.turn.priority = Some(P1);
    assert!(t.cast_with(P1, bears_x, &[]).is_err());
    // It lasts until P1's next end step: in P1's main phase it's cast for free.
    t.advance_to(P1, Step::PrecombatMain);
    // Only without paying its mana cost (P1 has no lands).
    t.g.turn.priority = Some(P1);
    assert!(t.cast(P1, bears_x).try_go().is_err());
    t.g.turn.priority = Some(P1);
    t.cast(P1, bears_x)
        .method(CastMethod::Free)
        .try_go()
        .expect("cast for free");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn casey_and_raph_permission_ends_at_that_players_next_end_step() {
    cr!("611.2a");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let forest = t.library_top(P1, "Forest");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Casey & Raph, Hotheads");
    t.resolve_all();
    let forest_x = t.g.current(forest);
    // P0's own end step doesn't end it.
    t.advance_to(P0, Step::End);
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.play_land(P1, forest_x).is_err());
}

#[test]
fn expedited_inheritance_until_the_end_of_their_next_turn() {
    cr!("611.2a");
    ruling!(
        "Expedited Inheritance",
        "Players pay all costs and follow all normal timing rules for cards played from exile with Expedited Inheritance's permission."
    );
    assert_supported("Expedited Inheritance");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Expedited Inheritance");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.lands(P0, "Mountain", 1);
    let forest = t.library_top(P1, "Forest");
    let bolt = t.hand(P0, "Shock");
    // P1 exiles two cards (the Ogre was dealt 2 damage).
    t.answer_yes(P1, true);
    t.cast_with(P0, bolt, &[Entity::Object(ogre)]).unwrap();
    t.resolve_all();
    let forest_x = t.g.current(forest);
    assert_eq!(t.zone(forest_x), Zone::Exile, "{}", t.dump_log());
    // P0's next turn isn't "their next turn": P1 plays the Forest in P1's turn.
    t.advance_to(P1, Step::PrecombatMain);
    t.play_land(P1, forest_x).expect("P1 plays the exiled Forest");
    assert_eq!(t.zone(t.g.current(forest)), Zone::Battlefield);
}

#[test]
fn expedited_inheritance_ends_after_their_next_turn() {
    cr!("611.2a");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Expedited Inheritance");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.lands(P0, "Mountain", 1);
    let forest = t.library_top(P1, "Forest");
    let shock = t.hand(P0, "Shock");
    t.answer_yes(P1, true);
    t.cast_with(P0, shock, &[Entity::Object(ogre)]).unwrap();
    t.resolve_all();
    let forest_x = t.g.current(forest);
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.zone(forest_x), Zone::Exile);
    assert!(t.play_land(P1, forest_x).is_err());
}
