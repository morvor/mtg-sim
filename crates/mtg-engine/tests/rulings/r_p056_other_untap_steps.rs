//! Rulings batch P056 — "Untap all [permanents] you control during each other player's
//! untap step." (Seedborn Muse, Prophet of Kruphix, Drumbellower, Unwinding Clock, Prop
//! Room): those permanents untap at the same time as the active player's, with no choice,
//! once per untap step however many such effects there are (CR 502.3). Effects that say a
//! permanent doesn't untap during its controller's untap step don't apply then; others
//! (a stun counter, CR 122.1d) still do. Also Prophet of Kruphix's "You may cast creature
//! spells as though they had flash." (CR 702.8, 601.3).

use crate::r_p056_common::*;
use crate::r_s01_common::{asked_since, supported};
use crate::r_s29_common::put_counters;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::rooms;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The permanents untapped during P1's untap step, with nothing asked of P0 about it.
fn untap_without_a_choice(t: &mut TestGame, ids: &[ObjectId]) {
    for id in ids {
        tap(t, *id);
    }
    let from = t.asked().len();
    untap_step_of(t, P1);
    for id in ids {
        assert!(!tapped(t, *id), "{}", t.dump_log());
    }
    assert!(
        !asked_since(t, from).iter().any(|(p, d)| *p == P0
            && matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. })),
        "P0 was asked about untapping"
    );
}

#[test]
fn prop_room_untaps_every_creature_without_a_choice() {
    cr!("502.3", "709.5");
    ruling!(
        "Dazzling Theater // Prop Room",
        "All of your creatures untap during each other player's untap step if Prop Room is unlocked."
    );
    supported("Dazzling Theater // Prop Room");
    let mut t = TestGame::new(2);
    let room = t.battlefield(P0, "Dazzling Theater // Prop Room");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let forest = t.battlefield(P0, "Forest");
    // Locked: nothing untaps.
    tap(&mut t, bears);
    untap_step_of(&mut t, P1);
    assert!(tapped(&t, bears));
    assert!(rooms::unlock(&mut t.g, room, 1, P0));
    t.g.recompute();
    tap(&mut t, forest);
    untap_without_a_choice(&mut t, &[bears, giant]);
    // Only creatures.
    assert!(tapped(&t, forest));
}

#[test]
fn drumbellower_untaps_every_creature_without_a_choice() {
    cr!("502.3");
    ruling!(
        "Drumbellower",
        "All your creatures untap during each other player's untap step. You have no choice about what untaps."
    );
    supported("Drumbellower");
    let mut t = TestGame::new(2);
    let drum = t.battlefield(P0, "Drumbellower");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let stone = t.battlefield(P0, "Mind Stone");
    tap(&mut t, stone);
    untap_without_a_choice(&mut t, &[drum, bears]);
    assert!(tapped(&t, stone));
}

#[test]
fn prophet_untaps_creatures_and_lands_with_the_active_players_permanents() {
    cr!("502.3");
    ruling!(
        "Prophet of Kruphix",
        "Creatures and lands you control untap at the same time as the active player's permanents."
    );
    supported("Prophet of Kruphix");
    let mut t = TestGame::new(2);
    let prophet = t.battlefield(P0, "Prophet of Kruphix");
    let forest = t.battlefield(P0, "Forest");
    let stone = t.battlefield(P0, "Mind Stone");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, theirs);
    tap(&mut t, stone);
    untap_without_a_choice(&mut t, &[prophet, forest]);
    assert!(!tapped(&t, theirs));
    assert!(tapped(&t, stone), "artifacts aren't untapped");
    // The untapping happened during the untap step itself, as the active player's did.
    assert!(
        t.g.turn
            .step_log
            .iter()
            .take_while(|s| **s == Step::Untap)
            .count()
            > 0
    );
}

#[test]
fn unwinding_clock_untaps_every_artifact_without_a_choice() {
    cr!("502.3");
    ruling!(
        "Unwinding Clock",
        "Those artifacts untap at the same time as the active player's permanents. You have no choice about what untaps"
    );
    supported("Unwinding Clock");
    let mut t = TestGame::new(2);
    let clock = t.battlefield(P0, "Unwinding Clock");
    let stone = t.battlefield(P0, "Mind Stone");
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, bears);
    untap_without_a_choice(&mut t, &[clock, stone, thopter]);
    assert!(tapped(&t, bears));
}

#[test]
fn two_prophets_untap_each_permanent_once() {
    cr!("502.3");
    ruling!(
        "Prophet of Kruphix",
        "Controlling more than one Prophet of Kruphix doesn't allow you to untap any creature or land more than once"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prophet of Kruphix");
    t.battlefield(P0, "Prophet of Kruphix");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    tap(&mut t, bears);
    tap(&mut t, forest);
    untap_step_of(&mut t, P1);
    assert_eq!(untaps_this_turn(&t, bears), 1);
    assert_eq!(untaps_this_turn(&t, forest), 1);
}

#[test]
fn two_unwinding_clocks_untap_each_artifact_once() {
    cr!("502.3");
    ruling!(
        "Unwinding Clock",
        "Even if you control more than one Unwinding Clock, you'll only untap your artifacts once"
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Unwinding Clock");
    t.battlefield(P0, "Unwinding Clock");
    let stone = t.battlefield(P0, "Mind Stone");
    tap(&mut t, a);
    tap(&mut t, stone);
    untap_step_of(&mut t, P1);
    assert_eq!(untaps_this_turn(&t, a), 1);
    assert_eq!(untaps_this_turn(&t, stone), 1);
}

/// `p` controls `untapper` and a tapped Colossus of Sardia ("doesn't untap during your
/// untap step") and a tapped Grizzly Bears with a stun counter. During P1's untap step
/// the Colossus untaps; the stun counter is removed instead of the Bears untapping.
fn your_untap_step_restrictions_dont_apply(untapper: &str) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, untapper);
    let colossus = t.battlefield(P0, "Colossus of Sardia");
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, colossus);
    tap(&mut t, bears);
    put_counters(&mut t, bears, "stun", 1);
    untap_step_of(&mut t, P1);
    assert!(!tapped(&t, colossus), "{untapper}: {}", t.dump_log());
    assert!(
        tapped(&t, bears),
        "{untapper}: the stun counter still applies"
    );
    assert_eq!(t.counters(bears, "stun"), 0);
    // During P0's own untap step, the Colossus stays tapped.
    tap(&mut t, colossus);
    untap_step_of(&mut t, P0);
    assert!(tapped(&t, colossus));
    assert!(!tapped(&t, bears));
}

#[test]
fn prophet_untaps_what_doesnt_untap_during_your_untap_step() {
    cr!("502.3", "122.1d");
    ruling!(
        "Prophet of Kruphix",
        "Effects that state a creature or land you control doesn't untap during your untap step won't apply during another player's untap step."
    );
    your_untap_step_restrictions_dont_apply("Prophet of Kruphix");
}

#[test]
fn drumbellower_untaps_what_doesnt_untap_during_its_controllers_untap_step() {
    cr!("502.3", "122.1d");
    ruling!(
        "Drumbellower",
        "many effects say that the creature doesn't untap during its controller's untap step, and this card's ability applies during other players' untap steps. If a card does say this, then Drumbellower can untap it."
    );
    your_untap_step_restrictions_dont_apply("Drumbellower");
}

#[test]
fn seedborn_muse_untaps_what_doesnt_untap_during_its_controllers_untap_step() {
    cr!("502.3", "122.1d");
    ruling!(
        "Seedborn Muse",
        "But some other abilities are not written this way and can still prevent a card from untapping."
    );
    your_untap_step_restrictions_dont_apply("Seedborn Muse");
}

#[test]
fn prop_room_untaps_what_doesnt_untap_during_your_untap_step() {
    cr!("502.3", "122.1d");
    ruling!(
        "Dazzling Theater // Prop Room",
        "won't stop that creature from untapping during another player's untap step, and as such, Prop Room's ability will cause that creature to untap."
    );
    let mut t = TestGame::new(2);
    let room = t.battlefield(P0, "Dazzling Theater // Prop Room");
    assert!(rooms::unlock(&mut t.g, room, 1, P0));
    t.g.recompute();
    let colossus = t.battlefield(P0, "Colossus of Sardia");
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, colossus);
    tap(&mut t, bears);
    put_counters(&mut t, bears, "stun", 1);
    untap_step_of(&mut t, P1);
    assert!(!tapped(&t, colossus));
    assert!(tapped(&t, bears));
}

#[test]
fn prophet_gives_flash_to_bestow_only_when_cast_as_a_creature() {
    cr!("702.103b", "702.8a");
    ruling!(
        "Prophet of Kruphix",
        "Prophet of Kruphix won't give flash to a spell with bestow you're casting as an Aura."
    );
    supported("Nimbus Naiad");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prophet of Kruphix");
    let target = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    let naiad = t.hand(P0, "Nimbus Naiad");
    t.set_step(P1, Step::Upkeep);
    // As an Aura (bestow): it doesn't have flash.
    assert!(t
        .cast(P0, naiad)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(target)
        .try_go()
        .is_err());
    t.clear_answers();
    // As a creature: it may be cast as though it had flash.
    t.cast(P0, naiad).go();
    t.resolve_all();
    assert!(t.on_battlefield(naiad));
    assert!(t.obj_now(naiad).is(mtg_engine::types::CardType::Creature));
}

#[test]
fn prophet_gives_flash_to_creature_cards_cast_from_other_zones() {
    cr!("601.3", "702.8a");
    ruling!(
        "Prophet of Kruphix",
        "The last ability applies to creature cards in any zone, provided something is allowing you to cast them."
    );
    supported("Gravecrawler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prophet of Kruphix");
    t.battlefield(P0, "Diregraf Ghoul");
    t.lands(P0, "Swamp", 1);
    let crawler = t.graveyard(P0, "Gravecrawler");
    t.set_step(P1, Step::Upkeep);
    t.cast(P0, crawler).go();
    t.resolve_all();
    assert!(t.on_battlefield(crawler), "{}", t.dump_log());
    // Without the Prophet: not during an opponent's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Diregraf Ghoul");
    t.lands(P0, "Swamp", 1);
    let crawler = t.graveyard(P0, "Gravecrawler");
    t.set_step(P1, Step::Upkeep);
    assert!(t.cast(P0, crawler).try_go().is_err());
}
