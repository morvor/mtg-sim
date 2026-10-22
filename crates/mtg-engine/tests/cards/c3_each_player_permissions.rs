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
    // "Until your next end step" is P0's (Rocco's controller's), not the grantee's: P1 may
    // still cast the Bolt P1 exiled after P1's own end step, during P0's next turn, and
    // not after P0's next end step.
    for cast_before_p0_end in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Rocco, Street Chef");
        t.lands(P1, "Mountain", 1);
        let p1_card = t.library_top(P1, "Lightning Bolt");
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let p1_x = t.g.current(p1_card);
        assert_eq!(t.zone(p1_x), Zone::Exile);
        t.advance_to(P1, Step::End);
        t.resolve_all();
        if cast_before_p0_end {
            t.advance_to(P0, Step::PrecombatMain);
            t.cast_with(P1, p1_x, &[Entity::Player(P0)])
                .expect("P1's permission lasts until P0's next end step");
            t.resolve_all();
            assert_eq!(t.life(P0), 17, "{}", t.dump_log());
        } else {
            t.advance_to(P0, Step::End);
            t.resolve_all();
            t.advance_to(P1, Step::PrecombatMain);
            assert_eq!(t.zone(p1_x), Zone::Exile);
            assert!(
                t.cast_with(P1, p1_x, &[Entity::Player(P0)]).is_err(),
                "{}",
                t.dump_log()
            );
        }
    }
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

#[test]
fn ian_malcolm_the_active_player_casts_a_card_they_dont_own_exiled_with_it() {
    cr!("607.2a", "601.3", "118.14");
    ruling!(
        "Ian Malcolm, Chaotician",
        "Players must follow all applicable timing rules for casting spells with the permission granted by Ian Malcolm, Chaotician's last ability."
    );
    assert_supported("Ian Malcolm, Chaotician");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Ian Malcolm, Chaotician");
    let bolt = t.library_top(P0, "Lightning Bolt");
    // P0 draws two cards this turn: the second one makes P0 exile the top card.
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.resolve_all();
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Exile, "{}", t.dump_log());
    // P0 owns it: not P0.
    t.lands(P0, "Mountain", 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.turn.priority = Some(P0);
    assert!(t.cast_with(P0, bolt_x, &[Entity::Player(P1)]).is_err());
    t.clear_answers();
    // During P1's turn, P1 may cast it, paying with a Forest (mana of any type).
    t.lands(P1, "Forest", 1);
    t.advance_to(P1, Step::PrecombatMain);
    t.cast_with(P1, bolt_x, &[Entity::Player(P0)])
        .expect("P1 casts the Bolt exiled with Ian Malcolm");
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn ian_malcolm_only_the_active_player_casts_them() {
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Ian Malcolm, Chaotician");
    // P0's Shock and P1's Bolt and Lava Spike end up exiled with Ian Malcolm.
    let shock = t.library_top(P0, "Shock");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.g.draw_cards(P0, 2);
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.library_top(P1, "Island");
    t.library_top(P1, "Island");
    t.g.draw_cards(P1, 2);
    t.g.flush_events();
    t.resolve_all();
    let shock_x = t.g.current(shock);
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(shock_x), Zone::Exile);
    assert_eq!(t.zone(bolt_x), Zone::Exile);
    t.lands(P0, "Mountain", 2);
    t.lands(P1, "Mountain", 2);
    // During P0's turn, P1 can't cast P0's Shock.
    t.g.turn.priority = Some(P1);
    assert!(t.cast_with(P1, shock_x, &[Entity::Player(P0)]).is_err());
    t.clear_answers();
    // P0 casts P1's Bolt.
    t.cast_with(P0, bolt_x, &[Entity::Player(P1)]).expect("P0 casts it");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn suspend_aggression_each_owner_plays_their_card_until_the_end_of_their_next_turn() {
    cr!("611.2a", "305.1");
    ruling!(
        "Suspend Aggression",
        "Players pay all costs and follow all timing rules for cards played this way."
    );
    assert_supported("Suspend Aggression");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let spell = t.hand(P0, "Suspend Aggression");
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    let bears_x = t.g.current(bears);
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bears_x), Zone::Exile, "{}", t.dump_log());
    assert_eq!(t.zone(bolt_x), Zone::Exile);
    // P1 casts the Bears (their own card) in their next turn; not P0's Bolt.
    t.lands(P1, "Forest", 2);
    t.lands(P1, "Mountain", 1);
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.cast_with(P1, bolt_x, &[Entity::Player(P0)]).is_err());
    t.clear_answers();
    t.cast_with(P1, bears_x, &[]).expect("P1 casts the Bears");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // P0's Bolt: still in P0's next turn.
    t.advance_to(P0, Step::PrecombatMain);
    t.cast_with(P0, bolt_x, &[Entity::Player(P1)]).expect("P0 casts the Bolt");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn apple_of_eden_plays_the_opponents_exiled_hand_and_returns_the_rest() {
    cr!("406.3b", "611.2a", "118.14", "603.7");
    ruling!(
        "Apple of Eden, Isu Relic",
        "Only cards still exiled with Apple of Eden’s ability will be returned to their owner’s hand at the beginning of the next end step."
    );
    assert_supported("Apple of Eden, Isu Relic");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let apple = t.battlefield(P0, "Apple of Eden, Isu Relic");
    t.lands(P0, "Plains", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let forest = t.hand(P1, "Forest");
    t.activate(P0, apple, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Exile);
    assert!(t.g.obj(bolt_x).face_down);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.life(P0), 16);
    // A Plains pays for the Bolt; its owner draws a card.
    t.cast_with(P0, bolt_x, &[Entity::Player(P1)]).expect("cast the Bolt");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P1), 1);
    // The Forest goes back to P1's hand at the end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(forest)), Zone::Hand(P1));
}
