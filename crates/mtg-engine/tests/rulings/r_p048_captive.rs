//! Rulings batch P048 — permanents whose abilities give things to other players: Captive
//! Audience (under an opponent's control, "choose one that hasn't been chosen", "your life
//! total becomes 4", players leaving the game), Brood Sliver (whose controller decides)
//! and Curse of Opulence (Gold made after attack costs are paid).

use crate::r_p048_common::*;
use crate::r_s09_common::{attacks_now, declare};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A game of `n` players in which P0's Captive Audience entered under `under`'s control.
fn audience_for(n: usize, under: PlayerId) -> (TestGame, ObjectId) {
    supported("Captive Audience");
    let mut t = TestGame::new(n);
    t.answer_choose(P0, &[Entity::Player(under)]);
    let ca = t.enter(P0, "Captive Audience");
    t.resolve_all();
    assert_eq!(controller(&mut t, ca), under);
    (t, ca)
}

/// The modes `p` was offered by its last mode choice since decision `from` (None if not
/// asked).
fn offered_modes(t: &TestGame, p: PlayerId, from: usize) -> Option<Vec<usize>> {
    t.asked()[from..].iter().rev().find_map(|(q, d)| match d {
        Decision::ChooseModes { available, .. } if *q == p => Some(available.clone()),
        _ => None,
    })
}

/// Advances to `p`'s next upkeep, answering its mode choice with `mode`, and resolves
/// everything.
fn upkeep_mode(t: &mut TestGame, p: PlayerId, mode: usize) {
    t.answer(p, DecisionKind::Modes, Answer::Indices(vec![mode]));
    let next = PlayerId(((p.idx() + t.g.players.len() - 1) % t.g.players.len()) as u8);
    t.advance_to(next, Step::PrecombatMain);
    t.advance_to(p, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn captive_audience_under_an_opponents_control_works_for_that_player() {
    cr!("603.3a", "109.5", "700.2");
    ruling!(
        "Captive Audience",
        "Once Captive Audience is controlled by your opponent, its ability triggers during that player’s upkeep and that player makes all choices for it. That player is affected by its first two modes, and that player’s opponents create the tokens for its last mode."
    );
    // First mode: P1's life total becomes 4.
    let (mut t, _) = audience_for(2, P1);
    let from = t.asked().len();
    upkeep_mode(&mut t, P1, 0);
    assert!(offered_modes(&t, P1, from).is_some());
    assert!(offered_modes(&t, P0, from).is_none());
    assert_eq!((t.life(P0), t.life(P1)), (20, 4));
    // Second mode: P1 discards its hand.
    let (mut t, _) = audience_for(2, P1);
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    upkeep_mode(&mut t, P1, 1);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P0), 1);
    // Third mode: P0 (P1's opponent) creates the Zombies.
    let (mut t, _) = audience_for(2, P1);
    upkeep_mode(&mut t, P1, 2);
    assert_eq!(tokens_by_player(&t, "Zombie"), vec![5, 0]);
}

#[test]
fn captive_audience_with_every_mode_chosen_does_nothing_but_stays() {
    cr!("700.2", "603.3c");
    ruling!(
        "Captive Audience",
        "If all three modes have been chosen, Captive Audience’s triggered ability is removed from the stack with no effect, but Captive Audience remains on the battlefield."
    );
    let (mut t, ca) = audience_for(2, P1);
    for mode in [2, 1, 0] {
        upkeep_mode(&mut t, P1, mode);
    }
    assert_eq!(t.life(P1), 4);
    let from = t.asked().len();
    t.advance_to(P0, Step::PrecombatMain);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let offered = offered_modes(&t, P1, from);
    assert!(offered.as_ref().is_none_or(|m| m.is_empty()), "{offered:?}");
    assert!(t.on_battlefield(ca));
    assert_eq!(tokens_by_player(&t, "Zombie"), vec![5, 0]);
    assert_eq!(t.life(P1), 4);
}

#[test]
fn captive_audiences_first_two_modes_can_be_chosen_when_they_would_change_nothing() {
    cr!("700.2");
    ruling!(
        "Captive Audience",
        "You may choose Captive Audience’s first mode even if your life total is already 4. Similarly, you may choose its second mode even if you have no cards in hand."
    );
    // In P1's first upkeep, with no cards in hand (P1 hasn't drawn yet): the second mode.
    let (mut t, _) = audience_for(2, P1);
    let from = t.asked().len();
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.hand_size(P1), 0);
    t.resolve_all();
    let offered = offered_modes(&t, P1, from).expect("asked");
    assert!(offered.contains(&0) && offered.contains(&1), "{offered:?}");
    assert_eq!(t.life(P1), 20);
    // In the next one, at 4 life: the first mode.
    set_life(&mut t, P1, 4);
    let from = t.asked().len();
    upkeep_mode(&mut t, P1, 0);
    let offered = offered_modes(&t, P1, from).expect("asked");
    assert!(offered.contains(&0), "{offered:?}");
    assert!(!offered.contains(&1), "{offered:?}");
    assert_eq!(t.life(P1), 4);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn captive_audience_sets_life_to_4_by_gaining_or_losing_life() {
    cr!("119.5", "603.2");
    ruling!(
        "Captive Audience",
        "For a player’s life total to become 4, that player gains or loses the appropriate amount of life."
    );
    // At 20: P1 loses 16 life, and P0's Exquisite Blood ("Whenever an opponent loses
    // life, you gain that much life.") sees it.
    supported("Exquisite Blood");
    let (mut t, _) = audience_for(2, P1);
    t.battlefield(P0, "Exquisite Blood");
    upkeep_mode(&mut t, P1, 0);
    assert_eq!((t.life(P0), t.life(P1)), (36, 4));
    // At 1: P1 gains 3 life, and P1's Ajani's Pridemate ("Whenever you gain life, put a
    // +1/+1 counter on this creature.") sees it.
    let (mut t, _) = audience_for(2, P1);
    let mate = t.battlefield(P1, "Ajani's Pridemate");
    set_life(&mut t, P1, 1);
    upkeep_mode(&mut t, P1, 0);
    assert_eq!(t.life(P1), 4);
    assert_eq!(t.counters(mate, "+1/+1"), 1);
}

#[test]
fn captive_audience_in_two_headed_giant_sets_the_team_life_total() {
    cr!("810.9c", "810.9a");
    ruling!(
        "Captive Audience",
        "In a Two-Headed Giant game, Captive Audience’s first mode causes its controller to gain or lose life so that their team’s life total becomes 4."
    );
    supported("Captive Audience");
    let mut t = two_headed_giant();
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let ca = t.enter(P0, "Captive Audience");
    t.resolve_all();
    assert_eq!(controller(&mut t, ca), P2);
    let team = t.life(P2);
    assert!(team > 4);
    t.answer(P2, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.advance_to(P2, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P2), 4);
    assert_eq!(t.life(P3), 4);
    assert_ne!(t.life(P0), 4);
}

#[test]
fn captive_audience_leaves_with_its_owner_and_is_exiled_when_its_controller_leaves() {
    cr!("800.4a");
    ruling!(
        "Captive Audience",
        "In a multiplayer game, if Captive Audience’s owner leaves the game, Captive Audience leaves the game with them."
    );
    ruling!(
        "Captive Audience",
        "If Captive Audience’s controller leaves the game, Captive Audience is exiled (assuming that player wasn’t also Captive Audience’s owner)."
    );
    let (mut t, ca) = audience_for(3, P1);
    concede(&mut t, P0);
    assert_eq!(t.zone(ca), Zone::Nowhere);
    assert!(!t.in_exile("Captive Audience"));
    let (mut t, ca) = audience_for(3, P1);
    concede(&mut t, P1);
    assert!(t.in_exile("Captive Audience"));
    assert_eq!(t.zone(t.g.current(ca)), Zone::Exile);
}

#[test]
fn brood_slivers_controller_controls_the_trigger_but_the_slivers_controller_decides() {
    cr!("603.3a", "603.5");
    ruling!(
        "Brood Sliver",
        "This card’s controller controls the triggered ability, but the controller of the Sliver that deals damage to a player decides whether or not to put the token onto the battlefield under their control."
    );
    supported("Brood Sliver");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Brood Sliver");
        let sliver = t.battlefield(P1, "Muscle Sliver");
        to_combat(&mut t, P1);
        t.answer(P1, DecisionKind::YesNo, Answer::Bool(yes));
        attack_with(&mut t, &[(sliver, Entity::Player(P0))]);
        t.advance_to_step(Step::CombatDamage);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        let ab = *t.g.stack.last().unwrap();
        assert_eq!(t.g.obj(ab).controller, P0);
        let from = t.asked().len();
        t.resolve_all();
        let asked: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(asked, vec![P1]);
        let slivers = |t: &TestGame, p| {
            t.g.permanents()
                .filter(|o| o.controller == p && o.is_token())
                .count()
        };
        assert_eq!((slivers(&t, P0), slivers(&t, P1)), (0, yes as usize));
    }
}

#[test]
fn curse_of_opulences_gold_comes_after_attack_costs_are_paid() {
    cr!("508.1h", "508.1m");
    ruling!(
        "Curse of Opulence",
        "Curse of Opulence's triggered ability resolves after attackers have been declared. Any costs to attack must have already been paid before the attacking player creates the Gold."
    );
    supported("Curse of Opulence");
    // P0's Curse of Opulence enchants P1, whose Ghostly Prison makes attacking P1 cost
    // {2} per creature. With two lands, P0 pays and attacks; the Gold arrives after.
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Curse of Opulence", Entity::Player(P1));
    t.battlefield(P1, "Ghostly Prison");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    let attacks = declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    assert_eq!(attacks.len(), 1);
    assert_eq!(tokens_of(&t, P0, "Gold"), 0);
    assert!(t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.is(types::CardType::Land))
        .all(|o| o.tapped));
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Gold"), 1);
    // With no lands, the Gold can't pay for the attack: no attack, no Gold.
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Curse of Opulence", Entity::Player(P1));
    t.battlefield(P1, "Ghostly Prison");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    assert!(attacks_now(&t).is_empty());
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Gold"), 0);
}
