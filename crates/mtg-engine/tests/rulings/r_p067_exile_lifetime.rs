//! Rulings batch P067 — cards that let a player cast cards from exile: which cards stay
//! exiled, which permissions outlast the card that granted them, and which are tied to
//! the object that exiled the cards (CR 400.7, 406.6, 611.2a).

use crate::r_p067_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use crate::r_s02_common::destroy;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn ornate_kanzashi_exiles_face_up_and_the_card_stays_after_it_leaves() {
    cr!("406.3", "611.2a", "400.7");
    ruling!(
        "Ornate Kanzashi",
        "Cards exiled by Ornate Kanzashi’s ability are exiled face up."
    );
    ruling!(
        "Ornate Kanzashi",
        "Cards exiled by Ornate Kanzashi’s ability stay exiled even if the Kanzashi leaves the battlefield."
    );
    supported("Ornate Kanzashi");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Ornate Kanzashi");
    t.lands(P0, "Wastes", 2);
    let bears = t.library_top(P1, "Grizzly Bears");
    t.activate(P0, k, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, bears));
    assert!(!t.obj_now(bears).face_down, "face up");
    destroy(&mut t, k);
    assert!(t.in_graveyard(P0, "Ornate Kanzashi"));
    assert!(exiled(&t, bears));
    // And it can still be played this turn ("You may play that card this turn").
    mana(&mut t, P0, ManaType::G, 2);
    assert!(castable(&mut t, P0, bears));
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(exiled(&t, bears));
    mana(&mut t, P0, ManaType::G, 2);
    assert!(!castable(&mut t, P0, bears));
}

#[test]
fn plargg_and_nassari_cards_not_cast_including_lands_remain_exiled() {
    cr!("608.2g", "305.9");
    ruling!(
        "Plargg and Nassari",
        "Cards not cast this way, including any land cards, remain in exile."
    );
    supported("Plargg and Nassari");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Plargg and Nassari");
    let mine = stack_library(&mut t, P0, &["Forest", "Lightning Bolt"]);
    let theirs = stack_library(&mut t, P1, &["Mountain", "Grizzly Bears"]);
    t.set_step(P1, Step::End);
    // P1 chooses its Grizzly Bears; P0 casts nothing.
    t.answer_choose_moved(P1, &[o(theirs[1])]);
    t.answer_choose_moved(P0, &[]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    for c in mine.iter().chain(theirs.iter()) {
        assert!(exiled(&t, *c), "{}", t.dump_log());
    }
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!playable_land(&mut t, P0, mine[0]));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, mine[1]));
}

#[test]
fn kefka_cards_not_cast_remain_exiled() {
    cr!("608.2g", "601.2c");
    ruling!(
        "Kefka, Dancing Mad",
        "Cards you choose not to cast this way (or can't cast this way) remain in exile."
    );
    supported("Kefka, Dancing Mad");
    // Shatter with no artifact to target can't be cast; Divination is declined.
    for (name, yes) in [("Shatter", true), ("Divination", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Kefka, Dancing Mad");
        let c = t.graveyard(P1, name);
        t.answer_choose_moved(P0, if yes { vec![o(c)] } else { vec![] }.as_slice());
        t.answer_yes(P0, yes);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert!(exiled(&t, c), "{name}: {}", t.dump_log());
        assert_eq!(t.hand_size(P0), 0, "{name}");
        assert_eq!(t.life(P1), 20, "{name}");
    }
}

#[test]
fn gonti_casting_the_card_makes_it_leave_exile_once() {
    cr!("601.2a", "400.7");
    ruling!(
        "Gonti, Lord of Luxury",
        "Casting the card causes it to leave exile. You can't cast it multiple times."
    );
    supported("Gonti, Lord of Luxury");
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose_moved(P0, &[o(bolt)]);
    t.enter(P0, "Gonti, Lord of Luxury");
    t.resolve_all();
    assert!(exiled(&t, bolt));
    mana(&mut t, P0, ManaType::B, 1);
    assert!(castable(&mut t, P0, bolt));
    t.cast(P0, t.g.current(bolt)).target(P1).go();
    assert_eq!(t.zone(bolt), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    mana(&mut t, P0, ManaType::B, 1);
    assert!(!castable(&mut t, P0, bolt));
}

#[test]
fn mindleecher_casting_the_card_makes_it_leave_exile_once() {
    cr!("601.2a", "400.7", "702.140a");
    ruling!(
        "Mindleecher",
        "Casting the card causes it to leave exile. You can’t cast it multiple times."
    );
    supported("Mindleecher");
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P1, "Lightning Bolt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let m = t.hand(P0, "Mindleecher");
    mana(&mut t, P0, ManaType::B, 5);
    t.cast(P0, m)
        .method(mtg_engine::object::CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Mutate))
        .target(bears)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert!(exiled(&t, bolt), "{}", t.dump_log());
    assert!(t.obj_now(bolt).face_down);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(castable(&mut t, P0, bolt));
    t.cast(P0, t.g.current(bolt)).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, bolt));
}

#[test]
fn crabomination_cant_play_lands_and_uncast_cards_remain_exiled() {
    cr!("305.9", "608.2g");
    ruling!(
        "Crabomination",
        "Crabomination's triggered ability doesn't allow you to play land cards, and as such, any land cards exiled with Crabomination's triggered ability will remain in exile. If you choose not to cast any of the exiled nonland cards (either because you can't or don't want to), those cards will remain in exile as well."
    );
    supported("Crabomination");
    let mut t = TestGame::new(2);
    let forest = t.library_top(P1, "Forest");
    let bears = t.graveyard(P1, "Grizzly Bears");
    let shatter = t.hand(P1, "Shatter");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose_moved(P0, &[]);
    let from = t.asked().len();
    t.enter(P0, "Crabomination");
    t.resolve_all();
    // The land wasn't offered; nothing was cast.
    assert!(!was_offered(&t, from, forest));
    for c in [forest, bears, shatter] {
        assert!(exiled(&t, c), "{}", t.dump_log());
    }
    assert!(!playable_land(&mut t, P0, forest));
    mana(&mut t, P0, ManaType::G, 2);
    assert!(!castable(&mut t, P0, bears));
}

#[test]
fn shared_fate_tracks_only_the_cards_it_exiled() {
    cr!("400.7", "607.2a", "614.11");
    ruling!(
        "Shared Fate",
        "Each Shared Fate tracks only the cards it exiled. If the Shared Fate which was responsible for a card being exiled leaves the battlefield, putting another Shared Fate onto the battlefield will not allow you to play that card again."
    );
    supported("Shared Fate");
    let mut t = TestGame::new(2);
    let fate = t.battlefield(P0, "Shared Fate");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    mana(&mut t, P0, ManaType::G, 2);
    assert!(castable(&mut t, P0, bears));
    destroy(&mut t, fate);
    t.battlefield(P0, "Shared Fate");
    mana(&mut t, P0, ManaType::G, 2);
    assert!(!castable(&mut t, P0, bears));
    // The new one lets P0 play what it exiles.
    let giant = t.library_top(P1, "Hill Giant");
    t.g.draw_cards(P0, 1);
    t.settle();
    mana(&mut t, P0, ManaType::R, 4);
    assert!(castable(&mut t, P0, giant));
    assert!(!castable(&mut t, P0, bears));
}

#[test]
fn you_find_some_prisoners_cards_not_chosen_or_not_played_remain_exiled() {
    cr!("700.2", "611.2a");
    ruling!(
        "You Find Some Prisoners",
        "For the Interrogate Them mode, the cards you didn't choose remain in exile. If you don't play the chosen card, it also remains in exile."
    );
    supported("You Find Some Prisoners");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P1, &["Grizzly Bears", "Forest", "Lightning Bolt"]);
    let spell = t.hand(P0, "You Find Some Prisoners");
    t.lands(P0, "Mountain", 2);
    t.answer_choose_moved(P0, &[o(cards[0])]);
    t.cast(P0, spell).modes(&[1]).target(P1).go();
    t.resolve_all();
    for c in &cards {
        assert!(exiled(&t, *c), "{}", t.dump_log());
    }
    mana(&mut t, P0, ManaType::R, 2);
    assert!(castable(&mut t, P0, cards[0]));
    assert!(!castable(&mut t, P0, cards[2]));
    assert!(!playable_land(&mut t, P0, cards[1]));
    // Not played: it stays exiled after its permission ends ("until the end of your next
    // turn").
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::R, 2);
    assert!(castable(&mut t, P0, cards[0]));
    t.advance_to(P1, Step::PrecombatMain);
    for c in &cards {
        assert!(exiled(&t, *c));
    }
    t.set_step(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::R, 2);
    assert!(!castable(&mut t, P0, cards[0]));
}

#[test]
fn elite_arcanist_loses_its_card_when_it_leaves_and_a_new_one_exiles_another() {
    cr!("400.7", "607.2a", "707.12");
    ruling!(
        "Elite Arcanist",
        "If Elite Arcanist leaves the battlefield, you will no longer be able to copy the exiled card. If it returns to the battlefield, you'll be able to exile another instant card from your hand, but Elite Arcanist will be a different permanent with no connection to the first exiled card."
    );
    supported("Elite Arcanist");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[o(bolt)]);
    let a = t.enter(P0, "Elite Arcanist");
    t.resolve_all();
    assert!(exiled(&t, bolt));
    // It leaves and returns; this time Shock is exiled.
    move_to(&mut t, a, Zone::Hand(P0));
    let shock = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[o(shock)]);
    let a2 = t.enter(P0, "Elite Arcanist");
    t.resolve_all();
    assert!(exiled(&t, shock) && exiled(&t, bolt));
    t.g.objects[a2.0 as usize].summoning_sick = false;
    t.lands(P0, "Wastes", 1);
    // The copy is of Shock (2 damage), not of the first Arcanist's Lightning Bolt.
    t.answer_yes(P0, true);
    activate_x(&mut t, P0, a2, 0, 1, &[]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
}

#[test]
fn hedonists_trove_cards_cant_be_played_once_it_leaves() {
    cr!("400.7", "607.2a", "611.2a");
    ruling!(
        "Hedonist's Trove",
        "If Hedonist’s Trove leaves the battlefield, any cards it exiled remain exiled. Those cards can no longer be played. Any future instance of Hedonist’s Trove (even one represented by the same card) will be a new object and won’t allow you to play those cards."
    );
    supported("Hedonist's Trove");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let forest = t.graveyard(P1, "Forest");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let trove = t.enter(P0, "Hedonist's Trove");
    t.resolve_all();
    mana(&mut t, P0, ManaType::G, 2);
    assert!(castable(&mut t, P0, bears));
    assert!(playable_land(&mut t, P0, forest));
    // The same card returns as a new object.
    let card = move_to(&mut t, trove, Zone::Hand(P0));
    let _ = card;
    assert!(exiled(&t, bears) && exiled(&t, forest));
    assert!(!castable(&mut t, P0, bears));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let back = t.g.current(card);
    t.g.move_object(back, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0))
        .unwrap();
    t.resolve_all();
    mana(&mut t, P0, ManaType::G, 2);
    assert!(!castable(&mut t, P0, bears));
    assert!(!playable_land(&mut t, P0, forest));
}

#[test]
fn jeleva_without_being_cast_exiles_nothing() {
    cr!("107.3", "601.2h");
    ruling!(
        "Jeleva, Nephalia's Scourge",
        "If Jeleva enters the battlefield without being cast, then no mana was spent to cast it. This means X will be 0, so no cards will be exiled."
    );
    supported("Jeleva, Nephalia's Scourge");
    let mut t = TestGame::new(2);
    let (l0, l1) = (t.library_size(P0), t.library_size(P1));
    t.enter(P0, "Jeleva, Nephalia's Scourge");
    t.resolve_all();
    assert_eq!((t.library_size(P0), t.library_size(P1)), (l0, l1));
    assert!(t.g.exile.is_empty());
}

/// Casts Jeleva from P0's hand with four mana and resolves it and its trigger.
fn cast_jeleva(t: &mut TestGame) -> ObjectId {
    let j = t.hand(P0, "Jeleva, Nephalia's Scourge");
    for ty in [ManaType::U, ManaType::B, ManaType::R, ManaType::C] {
        mana(t, P0, ty, 1);
    }
    t.cast(P0, j).go();
    t.resolve_all();
    j
}

#[test]
fn jeleva_new_object_has_no_connection_to_the_first_exiled_cards() {
    cr!("400.7", "607.2a");
    ruling!(
        "Jeleva, Nephalia's Scourge",
        "If Jeleva leaves the battlefield, the exiled cards will remain exiled indefinitely. If Jeleva enters the battlefield again, it won't be associated with the cards the \"other\" Jeleva exiled. The new Jeleva will exile a new set of cards. Only those cards can be cast using Jeleva's last ability."
    );
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P1, "Lightning Bolt");
    let j = cast_jeleva(&mut t);
    assert!(exiled(&t, bolt));
    assert_eq!(t.g.exile.len(), 8);
    move_to(&mut t, j, Zone::Hand(P0));
    assert!(exiled(&t, bolt));
    let shock = t.library_top(P1, "Shock");
    let j2 = cast_jeleva(&mut t);
    assert!(exiled(&t, shock));
    assert!(exiled(&t, bolt));
    let j2 = t.g.current(j2);
    t.g.obj_mut(j2).summoning_sick = false;
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[o(shock)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(j2, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!was_offered(&t, from, bolt), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Shock"), "{}", t.dump_log());
    assert!(exiled(&t, bolt));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - 2 - 1);
}

#[test]
fn planeswalkers_mischief_card_is_still_castable_and_returned_after_it_leaves() {
    cr!("611.2a", "603.7c");
    ruling!(
        "Planeswalker's Mischief",
        "If Planeswalker’s Mischief leaves the battlefield, any cards it exiled are still under the effect of the ability and can be cast (and if they are not cast, they are returned to the owner’s hand)."
    );
    supported("Planeswalker's Mischief");
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Planeswalker's Mischief");
        let div = t.hand(P1, "Divination");
        t.lands(P0, "Island", 4);
        t.activate(P0, m, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        assert!(exiled(&t, div), "{}", t.dump_log());
        destroy(&mut t, m);
        assert!(castable(&mut t, P0, div));
        if cast {
            cast_free(&mut t, P0, div);
            t.resolve_all();
            assert_eq!(t.hand_size(P0), 2);
            assert!(t.in_graveyard(P1, "Divination"));
        } else {
            t.advance_to(P0, Step::End);
            t.resolve_all();
            assert_eq!(t.zone(div), Zone::Hand(P1), "{}", t.dump_log());
        }
    }
}

#[test]
fn robber_of_the_rich_permission_outlasts_it_on_later_turns_you_attack_with_a_rogue() {
    cr!("611.2a", "508.1");
    ruling!(
        "Robber of the Rich",
        "If Robber of the Rich leaves the battlefield, the effect allowing you to cast the exiled card if you attacked with a Rogue continues to apply, even during later turns—as long as you attack with a Rogue during those later turns."
    );
    let mut t = TestGame::new(2);
    let robber = t.battlefield(P0, "Robber of the Rich");
    let agent = t.battlefield(P0, "Opposition Agent");
    t.hand(P1, "Forest");
    let bears = t.library_top(P1, "Grizzly Bears");
    attack_with(&mut t, &[(robber, Entity::Player(P1))]);
    t.resolve_all();
    assert!(exiled(&t, bears));
    block_and_finish(&mut t, P1, &[]);
    destroy(&mut t, robber);
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::R, 2);
    assert!(castable(&mut t, P0, bears), "this turn, attacked with a Rogue");
    // Next turn: not before attacking with a Rogue; after Opposition Agent (a Rogue)
    // attacks, yes.
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::R, 2);
    assert!(!castable(&mut t, P0, bears));
    attack_with(&mut t, &[(agent, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::R, 2);
    assert!(castable(&mut t, P0, bears));
    t.cast(P0, t.g.current(bears)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(t.on_battlefield(bears));
}

#[test]
fn release_to_the_wind_token_cant_be_cast() {
    cr!("111.7", "704.5d");
    ruling!(
        "Release to the Wind",
        "If a token is exiled this way, it can’t be cast."
    );
    supported("Release to the Wind");
    let mut t = TestGame::new(2);
    let tok = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    t.lands(P0, "Island", 3);
    let r = t.hand(P0, "Release to the Wind");
    t.cast(P0, r).target(tok).go();
    t.resolve_all();
    assert!(!t.g.is_live(t.g.current(tok)) || t.zone(tok) != Zone::Battlefield);
    assert!(t.g.exile.iter().all(|x| !t.g.obj(*x).is_token()));
    assert!(!castable(&mut t, P1, tok));
}

#[test]
fn a_token_exiled_by_hostage_taker_or_abstruse_appropriation_ceases_to_exist() {
    cr!("111.7", "704.5d", "610.3c");
    ruling!(
        "Hostage Taker",
        "If a token is exiled this way, it will cease to exist and won't return to the battlefield. You can't cast it."
    );
    ruling!(
        "Abstruse Appropriation",
        "If a token is exiled this way, it will cease to exist and won't return to the battlefield. You can't cast it."
    );
    supported("Hostage Taker");
    supported("Abstruse Appropriation");
    // Hostage Taker: the token doesn't return when Hostage Taker leaves.
    let mut t = TestGame::new(2);
    let tok = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    t.answer_targets(P0, &[o(tok)]);
    let taker = t.enter(P0, "Hostage Taker");
    t.resolve_all();
    assert!(t.g.exile.iter().all(|x| !t.g.obj(*x).is_token()));
    assert!(!castable(&mut t, P0, tok));
    destroy(&mut t, taker);
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    // Abstruse Appropriation: can't cast it.
    let mut t = TestGame::new(2);
    let tok = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    crate::r_s25_common::cast_new(&mut t, P0, "Abstruse Appropriation", &[o(tok)]);
    t.resolve_all();
    assert!(t.g.exile.iter().all(|x| !t.g.obj(*x).is_token()));
    mana(&mut t, P0, ManaType::C, 4);
    assert!(!castable(&mut t, P0, tok));
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
}

/// Kaho enters for P0, exiling Lightning Bolt from P0's library.
fn kaho_with_bolt(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_choose_moved(P0, &[o(bolt)]);
    let k = t.enter(P0, "Kaho, Minamo Historian");
    t.resolve_all();
    assert!(exiled(t, bolt), "{}", t.dump_log());
    t.g.objects[k.0 as usize].summoning_sick = false;
    (k, bolt)
}

#[test]
fn kaho_cant_cast_a_card_an_effect_took_out_of_exile() {
    cr!("400.7", "607.2a");
    ruling!(
        "Kaho, Minamo Historian",
        "If an effect takes a card Kaho exiled out of the exile zone, then you can't use Kaho's ability to cast that card."
    );
    supported("Kaho, Minamo Historian");
    let mut t = TestGame::new(2);
    let (k, bolt) = kaho_with_bolt(&mut t);
    // Back to exile from the graveyard: a new object, not "exiled with Kaho".
    move_to(&mut t, bolt, Zone::Graveyard(P0));
    move_to(&mut t, bolt, Zone::Exile);
    t.lands(P0, "Wastes", 1);
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[o(bolt)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_x(&mut t, P0, k, 0, 1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(exiled(&t, bolt));
    assert!(!was_offered(&t, from, bolt));
}

#[test]
fn kaho_new_controller_may_cast_the_exiled_cards() {
    cr!("607.2a", "108.4");
    ruling!(
        "Kaho, Minamo Historian",
        "If another player gains control of Kaho, that player may cast any of the cards exiled with Kaho's ability."
    );
    let mut t = TestGame::new(2);
    let (k, bolt) = kaho_with_bolt(&mut t);
    crate::r_s06_common::give_control(&mut t, k, P1);
    let k = t.g.current(k);
    t.g.obj_mut(k).summoning_sick = false;
    t.lands(P1, "Wastes", 1);
    t.answer_choose_moved(P1, &[o(bolt)]);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.set_step(P1, Step::PrecombatMain);
    activate_x(&mut t, P1, k, 0, 1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

/// Kotose enters for P0, exiling P1's Grizzly Bears from P1's graveyard and the one in
/// P1's library.
fn kotose(t: &mut TestGame) -> (ObjectId, Vec<ObjectId>) {
    let gy = t.graveyard(P1, "Grizzly Bears");
    let lib = t.library_top(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(gy)]);
    t.answer_choose_moved(P0, &[o(lib)]);
    let k = t.enter(P0, "Kotose, the Silent Spider");
    t.resolve_all();
    assert!(exiled(t, gy) && exiled(t, lib), "{}", t.dump_log());
    (k, vec![gy, lib])
}

#[test]
fn kotose_permission_ends_with_control_and_with_the_object() {
    cr!("611.2b", "400.7");
    ruling!(
        "Kotose, the Silent Spider",
        "If another player gains control of Kotose, the Silent Spider, you won't be able to play any of the exiled cards, even if you later regain control of it. Similarly, if Kotose leaves the battlefield and later returns, you won't be able to play the cards that were exiled the first time it was on the battlefield."
    );
    supported("Kotose, the Silent Spider");
    // Control changes and changes back.
    let mut t = TestGame::new(2);
    let (k, bears) = kotose(&mut t);
    mana(&mut t, P0, ManaType::C, 2);
    assert!(castable(&mut t, P0, bears[0]));
    crate::r_s06_common::give_control(&mut t, k, P1);
    mana(&mut t, P0, ManaType::C, 2);
    assert!(!castable(&mut t, P0, bears[0]));
    crate::r_s06_common::give_control(&mut t, k, P0);
    mana(&mut t, P0, ManaType::C, 2);
    assert!(!castable(&mut t, P0, bears[0]));
    assert!(!castable(&mut t, P0, bears[1]));
    // Leaves and returns.
    let mut t = TestGame::new(2);
    let (k, bears) = kotose(&mut t);
    let card = move_to(&mut t, k, Zone::Hand(P0));
    t.g.move_object(card, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0))
        .unwrap();
    t.clear_answers();
    t.resolve_all();
    mana(&mut t, P0, ManaType::C, 2);
    assert!(!castable(&mut t, P0, bears[0]));
    assert!(!castable(&mut t, P0, bears[1]));
}
