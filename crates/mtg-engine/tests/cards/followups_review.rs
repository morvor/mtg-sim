//! Review tests for cards2-followups: repeated punishers with all choices made first
//! (Rottenmouth Viper), "[instruction] or [instruction]" choices (Turnabout), "may
//! choose to" (Covenant of Minds), and per-player outcomes with more than two players.

use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
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
fn rottenmouth_viper_choices_for_each_counter_then_all_at_once() {
    cr!("118.12", "122.1");
    ruling!(
        "Rottenmouth Viper",
        "Cards or permanents they’ve chosen previously during this process can’t be chosen again"
    );
    supported("Rottenmouth Viper");
    let mut t = TestGame::new(3);
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P2, "Forest", 1);
    // Three blight counters after the trigger: P1 pays with the creature and the card
    // (and so loses 4 life once); P2 has neither and loses 4 life three times.
    t.answer_choose(P1, &[Entity::Object(bears), Entity::Object(bolt)]);
    let viper = t.enter(P0, "Rottenmouth Viper");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.add_counters(Entity::Object(viper), "blight", 2, None);
    t.resolve_all();
    assert_eq!(t.counters(viper, "blight"), 3);
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(t.on_battlefield(forest), "a land can't be chosen");
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P2), 8);
}

#[test]
fn rottenmouth_viper_uses_counters_it_last_had() {
    cr!("608.2h");
    ruling!(
        "Rottenmouth Viper",
        "use the number of blight counters that were on it as it last existed on the battlefield"
    );
    supported("Rottenmouth Viper");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let viper = t.enter(P0, "Rottenmouth Viper");
    t.settle();
    // The trigger is on the stack; the Viper gets a counter, then leaves before the
    // trigger resolves (so the trigger can't put its counter on it).
    assert_eq!(t.stack_len(), 1);
    t.g.add_counters(Entity::Object(viper), "blight", 1, None);
    t.g.destroy(viper, None);
    t.resolve_all();
    assert!(!t.on_battlefield(viper));
    // The one counter it had as it last existed: 4 life once.
    assert_eq!(t.life(P1), 16);
}

#[test]
fn turnabout_untaps_the_chosen_type_on_resolution() {
    cr!("608.2d");
    ruling!(
        "Turnabout",
        "You decide on resolution if you are tapping or untapping"
    );
    supported("Turnabout");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    t.g.tap(forest);
    t.g.tap(mine);
    t.lands(P0, "Island", 4);
    let spell = t.hand(P0, "Turnabout");
    // Creature, then the second instruction: untap.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert!(
        !t.obj_now(bears).tapped,
        "the target player's creature untaps"
    );
    assert!(t.obj_now(forest).tapped, "lands weren't chosen");
    assert!(
        t.obj_now(mine).tapped,
        "only the target player's permanents"
    );
}

#[test]
fn covenant_of_minds_opponent_chooses_the_three_cards_or_five_draws() {
    cr!("608.2c", "118.12");
    ruling!(
        "Covenant of Minds",
        "Let you have the three revealed cards or let you have five unknown cards"
    );
    supported("Covenant of Minds");
    for accept in [true, false] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        let top: Vec<_> = (0..3).map(|_| t.library_top(P0, "Grizzly Bears")).collect();
        t.lands(P0, "Island", 5);
        let spell = t.hand(P0, "Covenant of Minds");
        t.answer_yes(P1, accept);
        let hand = t.hand_size(P0);
        let gy = t.graveyard_size(P0);
        t.cast(P0, spell).target(Entity::Player(P1)).go();
        t.resolve();
        if accept {
            assert_eq!(t.hand_size(P0), hand - 1 + 3);
            for c in &top {
                assert_eq!(t.zone(*c), Zone::Hand(P0));
            }
            assert_eq!(t.graveyard_size(P0), gy + 1);
        } else {
            // The three revealed cards go to the graveyard (with the spell), then five
            // cards are drawn.
            assert_eq!(t.hand_size(P0), hand - 1 + 5);
            assert_eq!(t.graveyard_size(P0), gy + 4);
        }
    }
}

#[test]
fn zoyowa_damages_each_opponent_who_didnt_including_one_who_couldnt() {
    cr!("101.4", "118.12", "609.3");
    ruling!(
        "Zoyowa Lava-Tongue",
        "An opponent with zero cards in hand can't choose to discard a card"
    );
    supported("Zoyowa Lava-Tongue");
    let mut t = TestGame::new(4);
    t.battlefield(P0, "Zoyowa Lava-Tongue");
    // P0 descended this turn.
    let land = t.battlefield(P0, "Swamp");
    t.advance_to(P0, Step::PrecombatMain);
    t.g.destroy(land, None);
    // P1 discards a card; P2 has no cards or permanents but accepts; P3 declines.
    let card1 = t.hand(P1, "Lightning Bolt");
    t.hand(P3, "Lightning Bolt");
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P1, &[Entity::Object(card1)]);
    t.answer_yes(P2, true);
    t.answer_yes(P3, false);
    let hands = (t.hand_size(P1), t.hand_size(P2), t.hand_size(P3));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hands.0 - 1);
    assert_eq!(t.hand_size(P3), hands.2);
    assert_eq!((t.life(P1), t.life(P2), t.life(P3)), (20, 17, 17));
}

#[test]
fn professor_onyx_each_opponent_who_doesnt_discard_loses_3_seven_times() {
    cr!("101.4", "608.2e", "118.12");
    ruling!(
        "Professor Onyx",
        "the next opponent in turn order may choose a card in hand without revealing it"
    );
    supported("Professor Onyx");
    let mut t = TestGame::new(3);
    t.advance_to(P0, Step::PrecombatMain);
    let onyx = t.battlefield(P0, "Professor Onyx");
    let have = t.counters(onyx, "loyalty");
    t.g.add_counters(Entity::Object(onyx), "loyalty", 8 - have, None);
    // P1 has two cards and discards them in the first two of the seven processes; P2
    // has none.
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Lightning Bolt");
    for _ in 0..2 {
        t.answer_yes(P1, true);
    }
    let (h1, h2) = (t.hand_size(P1), t.hand_size(P2));
    t.activate(P0, onyx, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 - 2);
    assert_eq!(t.hand_size(P2), h2);
    assert_eq!(t.life(P1), 20 - 3 * 5);
    assert_eq!(t.life(P2), 20 - 3 * 7);
}

#[test]
fn rhystic_scrying_discards_only_if_a_player_pays() {
    cr!("118.12");
    ruling!(
        "Rhystic Scrying",
        "Each player (starting with the current player and going in turn order) gets the option to pay"
    );
    supported("Rhystic Scrying");
    for pays in [true, false] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        t.lands(P0, "Island", 4);
        t.lands(P1, "Island", 2);
        let spell = t.hand(P0, "Rhystic Scrying");
        let hand = t.hand_size(P0);
        // P0 (the current player) declines first; then P1 decides.
        t.answer_yes(P0, false);
        t.answer_yes(P1, pays);
        t.cast(P0, spell).go();
        t.resolve();
        let drawn = hand - 1 + 3;
        if pays {
            assert_eq!(t.hand_size(P0), drawn - 3);
        } else {
            assert_eq!(t.hand_size(P0), drawn);
        }
    }
}

#[test]
fn brain_pry_draws_if_the_player_cant_discard_the_named_card() {
    cr!("609.3", "608.2c");
    supported("Brain Pry");
    for has_it in [true, false] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        t.lands(P0, "Swamp", 2);
        if has_it {
            t.hand(P1, "Grizzly Bears");
        } else {
            t.hand(P1, "Hill Giant");
        }
        let spell = t.hand(P0, "Brain Pry");
        t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
        let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
        t.cast(P0, spell).target(Entity::Player(P1)).go();
        t.resolve();
        if has_it {
            assert_eq!(t.hand_size(P1), h1 - 1);
            assert_eq!(t.hand_size(P0), h0 - 1);
        } else {
            assert_eq!(t.hand_size(P1), h1);
            assert_eq!(t.hand_size(P0), h0);
        }
    }
}

#[test]
fn chandra_chill_of_compliance_keeps_a_surveiled_noncreature_nonland_card() {
    cr!("701.25a", "608.2c");
    supported("Chandra, Chill of Compliance");
    for (name, to_hand) in [("Lightning Bolt", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        let chandra = t.battlefield(P0, "Chandra, Chill of Compliance");
        let top = t.library_top(P0, name);
        // Put it into the graveyard.
        t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], vec![top]));
        t.activate(P0, chandra, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.in_hand(P0, name), to_hand, "{name}");
        assert_eq!(t.in_graveyard(P0, name), !to_hand, "{name}");
    }
}

#[test]
fn infernal_offering_both_players_who_sacrificed_draw_in_multiplayer() {
    cr!("101.4", "608.2c");
    ruling!(
        "Infernal Offering",
        "If you control no creatures, you won’t sacrifice one and you won’t draw two cards"
    );
    supported("Infernal Offering");
    // You and the chosen opponent (P2) both sacrifice a creature and both draw two cards;
    // P1, not chosen, does neither.
    let mut t = TestGame::new(3);
    t.advance_to(P0, Step::PrecombatMain);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let p1 = t.battlefield(P1, "Grizzly Bears");
    let p2 = t.battlefield(P2, "Hill Giant");
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Infernal Offering");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let (h0, h1, h2) = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    t.cast(P0, spell).go();
    t.resolve();
    assert!(!t.g.is_live(mine));
    assert!(!t.g.is_live(p2));
    assert!(t.on_battlefield(p1));
    assert_eq!(t.hand_size(P0), h0 - 1 + 2);
    assert_eq!(t.hand_size(P1), h1);
    assert_eq!(t.hand_size(P2), h2 + 2);
}
