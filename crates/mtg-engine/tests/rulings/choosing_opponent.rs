//! Which opponent makes a choice "an opponent" makes (register
//! `controller-picks-the-choosing-opponent`, decided by the user on Nov 19): the controller
//! of the spell or ability picks the opponent, as CR 601.7a and 602.3a say for choices made
//! while casting or activating, and as the rulings say for choices made as it resolves.
//! The controller is offered the opponents in turn order starting with the next one, who
//! is the default. Every wording below is checked in a three-player game where the
//! controller picks the second opponent in turn order.

use crate::r_s01_common::supported;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The players asked since `from` to pick among players, with the players offered.
fn player_picks(t: &TestGame, from: usize) -> Vec<(PlayerId, Vec<Entity>)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. }
                if !candidates.is_empty()
                    && candidates.iter().all(|e| matches!(e, Entity::Player(_))) =>
            {
                Some((*p, candidates.clone()))
            }
            _ => None,
        })
        .collect()
}

/// The players asked since `from` a decision `f` matches.
fn asked_whom(t: &TestGame, from: usize, f: impl Fn(&Decision) -> bool) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| f(d))
        .map(|(p, _)| *p)
        .collect()
}

/// A choice among cards (not players).
fn card_choice(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { candidates, .. }
        if candidates.iter().any(|e| matches!(e, Entity::Object(_))))
}

const P2_PICKED: &str = "the controller was offered P1 then P2 and picked P2";

#[test]
fn burning_rune_demon_controller_picks_the_second_opponent() {
    interp!("controller-picks-the-choosing-opponent");
    supported("Burning-Rune Demon");
    // "An opponent chooses one of them. Put the chosen card into your hand and the other
    // into your graveyard." P0 finds Shock and Lightning Bolt and picks P2, who chooses
    // Shock: Shock goes to P0's hand.
    let mut t = TestGame::new(3);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let shock = t.library_top(P0, "Shock");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(shock), Entity::Object(bolt)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_choose(P2, &[Entity::Object(shock)]);
    t.enter(P0, "Burning-Rune Demon");
    t.resolve_all();
    let picks = player_picks(&t, from);
    assert_eq!(
        picks,
        vec![(P0, vec![Entity::Player(P1), Entity::Player(P2)])],
        "{P2_PICKED}"
    );
    let choosers = asked_whom(&t, from, card_choice);
    assert_eq!(choosers, vec![P0, P2], "P0 searched, P2 chose");
    assert!(t.in_hand(P0, "Shock"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn by_default_the_next_opponent_in_turn_order_chooses() {
    interp!("controller-picks-the-choosing-opponent");
    supported("Burning-Rune Demon");
    // It's P0's turn. P1's Burning-Rune Demon enters: the first opponent in APNAP order is
    // P0, but the next in turn order after P1 is P2, who chooses by default.
    let mut t = TestGame::new(3);
    let bolt = t.library_top(P1, "Lightning Bolt");
    let shock = t.library_top(P1, "Shock");
    let from = t.asked().len();
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(shock), Entity::Object(bolt)]);
    t.enter(P1, "Burning-Rune Demon");
    t.resolve_all();
    let picks = player_picks(&t, from);
    assert_eq!(
        picks,
        vec![(P1, vec![Entity::Player(P2), Entity::Player(P0)])]
    );
    assert_eq!(asked_whom(&t, from, card_choice), vec![P1, P2]);
}

#[test]
fn sphinx_of_uthuun_controller_picks_the_opponent_who_separates() {
    interp!("controller-picks-the-choosing-opponent");
    ruling!(
        "Sphinx of Uthuun",
        "you choose an opponent to separate the cards when the ability resolves"
    );
    supported("Sphinx of Uthuun");
    // "An opponent separates those cards into two piles."
    let mut t = TestGame::new(3);
    for name in ["Shock", "Lightning Bolt", "Forest", "Island", "Swamp"] {
        t.library_top(P0, name);
    }
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.enter(P0, "Sphinx of Uthuun");
    t.resolve_all();
    assert_eq!(
        player_picks(&t, from),
        vec![(P0, vec![Entity::Player(P1), Entity::Player(P2)])],
        "{P2_PICKED}"
    );
    // P2 separates the revealed cards; P1 isn't asked.
    let separators = asked_whom(&t, from, card_choice);
    assert!(separators.contains(&P2), "{separators:?}");
    assert!(!separators.contains(&P1), "{separators:?}");
}

#[test]
fn steam_augury_controller_picks_the_opponent_who_chooses_a_pile() {
    interp!("controller-picks-the-choosing-opponent");
    ruling!(
        "Steam Augury",
        "In a multiplayer game, you choose the opponent as the spell resolves."
    );
    supported("Steam Augury");
    // "An opponent chooses one of those piles."
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    for name in ["Lightning Bolt", "Forest", "Island", "Swamp"] {
        t.library_top(P0, name);
    }
    let shock = t.library_top(P0, "Shock");
    let augury = t.hand(P0, "Steam Augury");
    t.cast(P0, augury).go();
    let from = t.asked().len();
    // P0 separates (Shock alone in the first pile), then picks P2.
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.resolve();
    assert_eq!(
        player_picks(&t, from),
        vec![(P0, vec![Entity::Player(P1), Entity::Player(P2)])],
        "{P2_PICKED}"
    );
    let choosers = asked_whom(&t, from, |d| matches!(d, Decision::ChooseOption { .. }));
    assert_eq!(choosers, vec![P2]);
}

#[test]
fn akroan_horse_controller_picks_the_opponent_who_gains_control() {
    interp!("controller-picks-the-choosing-opponent");
    ruling!(
        "Akroan Horse",
        "In a multiplayer game, you choose the opponent as the ability resolves."
    );
    supported("Akroan Horse");
    // "When this creature enters, an opponent gains control of it."
    let mut t = TestGame::new(3);
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let horse = t.enter(P0, "Akroan Horse");
    t.resolve_all();
    assert_eq!(
        player_picks(&t, from),
        vec![(P0, vec![Entity::Player(P1), Entity::Player(P2)])],
        "{P2_PICKED}"
    );
    assert_eq!(t.obj_now(horse).controller, P2);
}

#[test]
fn murmurs_from_beyond_controller_picks_the_opponent_who_chooses_a_card() {
    interp!("controller-picks-the-choosing-opponent");
    supported("Murmurs from Beyond");
    // "Reveal the top three cards of your library. An opponent chooses one of them. Put that
    // card into your graveyard and the rest into your hand."
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 3);
    for name in ["Shock", "Lightning Bolt", "Forest"] {
        t.library_top(P0, name);
    }
    let murmurs = t.hand(P0, "Murmurs from Beyond");
    t.cast(P0, murmurs).go();
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.resolve();
    assert_eq!(
        player_picks(&t, from),
        vec![(P0, vec![Entity::Player(P1), Entity::Player(P2)])],
        "{P2_PICKED}"
    );
    assert_eq!(asked_whom(&t, from, card_choice), vec![P2]);
}

#[test]
fn fatal_lore_controller_picks_the_opponent_who_chooses_the_mode() {
    cr!("601.7", "601.7a");
    interp!("controller-picks-the-choosing-opponent");
    supported("Fatal Lore");
    // "An opponent chooses one —": while the spell is cast (CR 601.7), its controller
    // decides which opponent (601.7a).
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 4);
    let lore = t.hand(P0, "Fatal Lore");
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.cast(P0, lore).go();
    assert_eq!(
        player_picks(&t, from),
        vec![(P0, vec![Entity::Player(P1), Entity::Player(P2)])],
        "{P2_PICKED}"
    );
    let choosers = asked_whom(&t, from, |d| matches!(d, Decision::ChooseModes { .. }));
    assert_eq!(choosers, vec![P2]);
}
