//! Rulings batch P137 — resolving sacrifice and discard effects: each instruction does as
//! much as possible (CR 608.2c, 609.3), "unless" choices a player can't make (CR 118.12),
//! hidden discard choices in turn order (CR 101.4a), simultaneous steps (CR 101.4),
//! votes (CR 701.38), greatest-power edicts with ties, and delayed triggers (CR 603.7).

use crate::r_p137_common::*;
use crate::r_s17_common::put_onto_battlefield;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

fn poison(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::POISON)
}

#[test]
fn remorseless_punishment_without_options_loses_ten_life() {
    cr!("118.12", "608.2c");
    ruling!(
        "Remorseless Punishment",
        "The player can’t choose to discard two cards unless they have at least two cards in hand. Similarly, that player can’t choose to sacrifice a creature or planeswalker unless they control one. This means that if the target opponent doesn’t have two cards in hand and doesn’t control a creature or planeswalker, that player will lose 5 life and then lose 5 life again."
    );
    supported("Remorseless Punishment");
    let mut t = TestGame::new(2);
    t.hand(P1, "Island");
    give_mana_for(&mut t, P0, "Remorseless Punishment");
    let spell = t.hand(P0, "Remorseless Punishment");
    // P1 tries to take the other options; it can't.
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn remorseless_punishment_repeats_for_the_same_opponent() {
    cr!("115.1", "608.2c");
    ruling!(
        "Remorseless Punishment",
        "The same player chooses both times. In a multiplayer game, you can’t choose a different opponent to repeat the process."
    );
    supported("Remorseless Punishment");
    let mut t = TestGame::new(3);
    for _ in 0..4 {
        t.hand(P1, "Island");
        t.hand(P2, "Island");
    }
    give_mana_for(&mut t, P0, "Remorseless Punishment");
    let spell = t.hand(P0, "Remorseless Punishment");
    let from = t.asked().len();
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    // Only P1 chose (and was punished) both times; P2 was untouched.
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
        .map(|(p, _)| *p)
        .filter(|p| *p != P0)
        .collect();
    assert!(!choosers.is_empty() && choosers.iter().all(|p| *p == P1));
    assert_eq!(t.hand_size(P2), 4);
    assert_eq!(t.life(P2), 20);
    // Two rounds: each either two discarded cards or 5 life.
    assert_eq!((20 - t.life(P1)) / 5 + (4 - t.hand_size(P1) as i32) / 2, 2);
}

#[test]
fn angrath_and_vraskas_fall_do_the_rest_even_if_part_is_impossible() {
    cr!("608.2c", "609.3");
    ruling!(
        "Angrath, the Flame-Chained",
        "Angrath’s first ability causes each opponent to lose 2 life even if some or all of those players were unable to discard a card."
    );
    ruling!(
        "Vraska's Fall",
        "Each opponent gets a poison counter even if they can't sacrifice any permanents."
    );
    supported("Angrath, the Flame-Chained");
    supported("Vraska's Fall");
    let mut t = TestGame::new(3);
    let angrath = t.battlefield(P0, "Angrath, the Flame-Chained");
    t.hand(P2, "Island");
    t.activate(P0, angrath, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (18, 18));
    assert_eq!(t.hand_size(P2), 0);

    let mut t = TestGame::new(3);
    t.battlefield(P2, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Vraska's Fall");
    let spell = t.hand(P0, "Vraska's Fall");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!((poison(&t, P1), poison(&t, P2)), (1, 1));
    assert!(t.in_graveyard(P2, "Grizzly Bears"));
}

#[test]
fn lilianas_triumph_two_lilianas_still_mean_one_discard() {
    cr!("608.2c");
    ruling!(
        "Liliana's Triumph",
        "Controlling more than one planeswalker of the appropriate type won't have any more of a benefit than just controlling one."
    );
    supported("Liliana's Triumph");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liliana of the Veil");
    t.battlefield(P0, "Liliana, Death's Majesty");
    for _ in 0..3 {
        t.hand(P1, "Island");
    }
    t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Liliana's Triumph");
    let spell = t.hand(P0, "Liliana's Triumph");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2, "one card discarded");
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn braskas_final_aeon_discards_are_chosen_hidden_in_turn_order() {
    cr!("101.4", "101.4a", "714.2b");
    ruling!(
        "Jecht, Reluctant Guardian // Braska's Final Aeon",
        "As Braska's Final Aeon's first or second chapter ability resolves, the next opponent in turn order (or, if it's an opponent's turn, that opponent) chooses a card in hand without revealing it, then each other opponent in turn order does the same. Then the chosen cards are discarded at the same time. Finally, you draw a card."
    );
    const JECHT: &str = "Jecht, Reluctant Guardian // Braska's Final Aeon";
    supported(JECHT);
    let mut t = TestGame::new(3);
    let a = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.hand(P2, "Grizzly Bears");
    let b = t.hand(P2, "Hill Giant");
    t.answer_choose(P1, &[obj(a)]);
    t.answer_choose(P2, &[obj(b)]);
    // When P2 chooses, P1's card is still in hand and P1's choice is hidden.
    let seen = watch(&mut t, P2, is_choose, |g| {
        (
            g.player(PlayerId(1)).hand.len(),
            g.known_apnap_choices(PlayerId(2))
                .iter()
                .map(|(p, c)| (*p, c.is_some()))
                .collect::<Vec<_>>(),
        )
    });
    let card = t.graveyard(P0, JECHT);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    let saga = put_onto_battlefield(&mut t, P0, card, true).expect("entered");
    assert_eq!(t.obj(saga).chars.name, "Braska's Final Aeon");
    t.resolve_all();
    assert_eq!(choosers_since(&t, from)[..2], [P1, P2]);
    assert_eq!(*seen.lock().unwrap(), vec![(2, vec![(P1, false)])]);
    assert!(t.in_graveyard(P1, "Grizzly Bears") && t.in_graveyard(P2, "Hill Giant"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn living_death_exiles_sacrifices_and_returns_in_three_simultaneous_steps() {
    cr!("101.4", "608.2c");
    ruling!(
        "Living Death",
        "As Living Death resolves, all players exile their creature cards from graveyards at the same time. Then all players sacrifice all creatures they control at the same time. Then all players put all creatures they exiled onto the battlefield at the same time."
    );
    supported("Living Death");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.battlefield(P1, "Craw Wurm");
    t.graveyard(P1, "Runeclaw Bear");
    give_mana_for(&mut t, P0, "Living Death");
    let spell = t.hand(P0, "Living Death");
    t.cast(P0, spell).go();
    t.resolve_all();
    // The creatures that were sacrificed stay in the graveyards.
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P1, "Craw Wurm"));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Runeclaw Bear").len(), 1);
    assert_eq!(creatures(&t, P0).len(), 1);
    assert_eq!(creatures(&t, P1).len(), 1);
}

/// Capital Punishment's votes: P0 then the others, death = 0, taxes = 1.
fn vote(t: &mut TestGame, p: PlayerId, death: bool) {
    t.answer(p, DecisionKind::Option, Answer::Index(usize::from(!death)));
}

#[test]
fn capital_punishment_sacrifices_first_then_discards() {
    cr!("101.4", "701.38a");
    ruling!(
        "Capital Punishment",
        "Each opponent chooses an appropriate number of creatures to sacrifice in turn order, then all of those creatures are sacrificed simultaneously. After that is complete, each opponent sets aside an appropriate number of cards to discard in turn order, then all cards are revealed and discarded simultaneously."
    );
    supported("Capital Punishment");
    // One death vote (P0) and two taxes votes.
    let mut t = TestGame::new(3);
    vote(&mut t, P0, true);
    vote(&mut t, P1, false);
    vote(&mut t, P2, false);
    for p in [P1, P2] {
        t.battlefield(p, "Grizzly Bears");
        t.battlefield(p, "Hill Giant");
        for _ in 0..3 {
            t.hand(p, "Island");
        }
    }
    // When P1 chooses cards to discard, both sacrifices have happened.
    let seen = watch(&mut t, P1, is_choose, |g| {
        g.permanents()
            .filter(|o| o.controller == PlayerId(2))
            .count()
    });
    give_mana_for(&mut t, P0, "Capital Punishment");
    let spell = t.hand(P0, "Capital Punishment");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    let order: Vec<PlayerId> = choosers_since(&t, from)
        .into_iter()
        .filter(|p| *p != P0)
        .collect();
    assert_eq!(order, vec![P1, P2, P1, P2]);
    assert_eq!(seen.lock().unwrap()[..2], [2, 1]);
    for p in [P1, P2] {
        assert_eq!(creatures(&t, p).len(), 1);
        assert_eq!(t.hand_size(p), 1);
    }
}

#[test]
fn capital_punishment_takes_everything_when_there_are_fewer() {
    cr!("701.38a", "701.21a", "701.9a");
    ruling!(
        "Capital Punishment",
        "If an opponent controls fewer creatures or has fewer cards in hand than the number of appropriate votes, all those creatures are sacrificed or all those cards are discarded, as applicable."
    );
    supported("Capital Punishment");
    let mut t = TestGame::new(2);
    // One death vote, one taxes vote; P1 has no creatures and one card... and then two
    // death votes against a single creature.
    vote(&mut t, P0, false);
    vote(&mut t, P1, false);
    t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Island");
    give_mana_for(&mut t, P0, "Capital Punishment");
    let spell = t.hand(P0, "Capital Punishment");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(creatures(&t, P1).len(), 1);

    let mut t = TestGame::new(2);
    vote(&mut t, P0, true);
    vote(&mut t, P1, true);
    t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Island");
    give_mana_for(&mut t, P0, "Capital Punishment");
    let spell = t.hand(P0, "Capital Punishment");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(creatures(&t, P1).is_empty());
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn szats_will_each_opponent_compares_only_their_own_creatures() {
    cr!("701.21a");
    ruling!(
        "Szat's Will",
        "Each opponent considers their creatures independently. For example, if one opponent controls creatures with powers 4 and 5, and another opponent controls creatures with powers 2 and 7, the creatures with powers 5 and 7 will be sacrificed."
    );
    ruling!(
        "Szat's Will",
        "If an opponent controls more than one creature with the greatest power among creatures they control, they choose which one to sacrifice."
    );
    supported("Szat's Will");
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let b1 = t.battlefield(P2, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Runeclaw Bear");
    t.answer_choose(P2, &[obj(b2)]);
    give_mana_for(&mut t, P0, "Szat's Will");
    let spell = t.hand(P0, "Szat's Will");
    let from = t.asked().len();
    t.cast(P0, spell).modes(&[0]).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant) && !t.on_battlefield(wurm));
    assert!(t.on_battlefield(b1) && !t.on_battlefield(b2));
    // P2 chose between its two creatures tied for the greatest power.
    assert_eq!(offered_to(&t, P2, from), vec![vec![b1, b2]]);
}

#[test]
fn szats_will_uses_powers_as_the_cards_were_in_the_graveyard() {
    cr!("608.2h");
    ruling!(
        "Szat's Will",
        "For the second mode, use the powers of the creature cards as they existed in the graveyard to determine the value of X."
    );
    supported("Szat's Will");
    supported("Tarmogoyf");
    let mut t = TestGame::new(2);
    // In the graveyard, Tarmogoyf is 2/3 (a creature card and an instant card are in
    // graveyards); exiled with the instant, it would be 0/1.
    t.graveyard(P1, "Tarmogoyf");
    t.graveyard(P1, "Lightning Bolt");
    give_mana_for(&mut t, P0, "Szat's Will");
    let spell = t.hand(P0, "Szat's Will");
    t.cast(P0, spell).modes(&[1]).go();
    t.resolve_all();
    assert!(t.in_exile("Tarmogoyf"));
    assert_eq!(t.named_on_battlefield("Thrull Token").len(), 2);
}

#[test]
fn killing_wave_players_choose_in_turn_order_then_pay_and_sacrifice_together() {
    cr!("101.4", "118.12");
    ruling!(
        "Killing Wave",
        "First, the active player chooses whether to pay X life for each creature they control. Then each other player in turn order chooses for their creatures. Then each player pays life and sacrifices creatures at the same time. Players will know the decisions of players who chose before them."
    );
    supported("Killing Wave");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    // P0 casts it on P1's turn: "You may cast spells as though they had flash."
    t.battlefield(P0, "Leyline of Anticipation");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P2, "Runeclaw Bear");
    // P1 (the active player) pays for its creature, P2 doesn't, P0 pays.
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    t.answer_yes(P0, true);
    let look = |g: &mtg_engine::game::Game| {
        (
            g.player(PlayerId(1)).life,
            g.permanents()
                .filter(|o| o.is(mtg_engine::types::CardType::Creature))
                .count(),
        )
    };
    let s2 = watch(&mut t, P2, |d| matches!(d, Decision::YesNo { .. }), look);
    let s0 = watch(&mut t, P0, |d| matches!(d, Decision::YesNo { .. }), look);
    t.lands(P0, "Swamp", 4);
    let spell = t.hand(P0, "Killing Wave");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.cast(P0, spell).go();
    let decided_from = t.asked().len();
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[decided_from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P1, P2, P0]);
    // Nobody had paid or sacrificed while the others decided.
    assert_eq!(*s2.lock().unwrap(), vec![(20, 3)]);
    assert_eq!(*s0.lock().unwrap(), vec![(20, 3)]);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (17, 17, 20));
    assert!(t.on_battlefield(mine) && t.on_battlefield(theirs));
    assert!(!t.on_battlefield(other));
}

#[test]
fn killing_wave_a_player_cant_commit_more_life_than_they_have() {
    cr!("119.4", "101.4");
    // The payments are made together, so the life a player already chose to pay counts
    // against what they can pay for their next creature.
    supported("Killing Wave");
    let mut t = TestGame::new(2);
    t.g.players[P1.idx()].life = 5;
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Runeclaw Bear");
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.lands(P0, "Swamp", 4);
    let spell = t.hand(P0, "Killing Wave");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.cast(P0, spell).go();
    let from = t.asked().len();
    t.resolve_all();
    let asked = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(asked, 1, "only one payment of 3 life fits in 5");
    assert_eq!(t.life(P1), 2);
    assert_eq!(usize::from(t.on_battlefield(a)) + usize::from(t.on_battlefield(b)), 1);
}

#[test]
fn the_eternal_wanderer_returns_the_card_even_if_it_is_gone() {
    cr!("603.7c", "610.3");
    ruling!(
        "The Eternal Wanderer",
        "For the first loyalty ability, the exiled card will return to the battlefield at the beginning of that player's next end step even if The Eternal Wanderer is no longer on the battlefield at that time."
    );
    supported("The Eternal Wanderer");
    let mut t = TestGame::new(2);
    let wanderer = t.battlefield(P0, "The Eternal Wanderer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, wanderer, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    destroy(&mut t, wanderer);
    // Not at P0's end step: at P1's.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).controller, P1);
}

#[test]
fn extract_the_truth_you_dont_have_to_choose_a_card() {
    cr!("608.2d");
    ruling!(
        "Extract the Truth",
        "For the first mode, you don't have to choose a card if you don't want to, even if the opponent has a creature, enchantment, or planeswalker card in their hand."
    );
    supported("Extract the Truth");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Extract the Truth");
    let spell = t.hand(P0, "Extract the Truth");
    t.answer_choose(P0, &[]);
    t.cast(P0, spell)
        .modes(&[0])
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn pirs_whim_friends_act_before_foes() {
    cr!("101.4", "603.3");
    ruling!(
        "Pir's Whim",
        "Friends perform their specified actions before foes. This means that a friend’s action may cause a triggered ability of a foe’s permanent to trigger before that foe’s action causes that permanent to leave the battlefield."
    );
    supported("Pir's Whim");
    let mut t = TestGame::new(2);
    // P1's Ankh of Mishra: "Whenever a land enters, this artifact deals 2 damage to that
    // land's controller."
    let ankh = t.battlefield(P1, "Ankh of Mishra");
    let forest = t.library_top(P0, "Forest");
    give_mana_for(&mut t, P0, "Pir's Whim");
    let spell = t.hand(P0, "Pir's Whim");
    // P0 is a friend, P1 a foe.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P0, &[obj(forest)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(ankh));
    assert_eq!(t.named_on_battlefield("Forest").len(), 2);
    assert_eq!(t.life(P0), 18, "the Ankh triggered before it was sacrificed");
}

#[test]
fn greatest_power_ties_let_the_player_choose() {
    cr!("701.21a");
    ruling!(
        "Shadowgrange Archfiend",
        "If a player controls more than one creature tied for the greatest power among creatures they control while Shadowgrange Archfiend's first ability is resolving, that player chooses which one of those creatures to sacrifice."
    );
    ruling!(
        "Riveteers Charm",
        "For Riveteers Charm's first mode, if the target opponent controls more than one creature or planeswalker with the highest mana value among creatures and planeswalkers they control, that player chooses which one to sacrifice as Riveteers Charm resolves."
    );
    supported("Shadowgrange Archfiend");
    supported("Riveteers Charm");
    let mut t = TestGame::new(2);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Runeclaw Bear");
    let elf = t.battlefield(P1, "Elvish Mystic");
    t.answer_choose(P1, &[obj(b2)]);
    let from = t.asked().len();
    t.enter(P0, "Shadowgrange Archfiend");
    t.settle();
    t.resolve_all();
    assert_eq!(offered_to(&t, P1, from), vec![vec![b1, b2]]);
    assert!(t.on_battlefield(b1) && !t.on_battlefield(b2) && t.on_battlefield(elf));
    assert_eq!(t.life(P0), 22);

    let mut t = TestGame::new(2);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Runeclaw Bear");
    let elf = t.battlefield(P1, "Elvish Mystic");
    t.answer_choose(P1, &[obj(b1)]);
    give_mana_for(&mut t, P0, "Riveteers Charm");
    let spell = t.hand(P0, "Riveteers Charm");
    let from = t.asked().len();
    t.cast(P0, spell)
        .modes(&[0])
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(offered_to(&t, P1, from), vec![vec![b1, b2]]);
    assert!(!t.on_battlefield(b1) && t.on_battlefield(b2) && t.on_battlefield(elf));
}

#[test]
fn call_to_the_grave_does_nothing_to_a_player_with_only_zombies() {
    cr!("608.2c");
    ruling!(
        "Call to the Grave",
        "If a player controls no creatures, or each creature they control is a Zombie, that player does nothing when the first ability resolves."
    );
    supported("Call to the Grave");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Call to the Grave");
    let corpse = t.battlefield(P1, "Walking Corpse");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(corpse));
    // And with a non-Zombie, it's sacrificed.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(corpse));
}

#[test]
fn nazgul_battle_mace_triggers_when_the_sacrificed_card_is_exiled_instead() {
    cr!("614.1a", "701.21a", "603.10a");
    ruling!(
        "Nazgûl Battle-Mace",
        "If a replacement effect, such as that of Rest in Peace, causes a sacrificed nontoken permanent to be put into a different public zone than a graveyard, the last ability granted by Nazgûl Battle-Mace will still trigger and you will still put the card onto the battlefield under your control if its original controller doesn't pay 3 life."
    );
    supported("Nazgûl Battle-Mace");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let mace = t.battlefield(P0, "Nazgûl Battle-Mace");
    t.g.attach(mace, obj(attacker));
    t.battlefield(P0, "Rest in Peace");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(bears)]);
    t.answer_yes(P1, false);
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
    let now = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(now.len(), 1);
    assert_eq!(t.obj(now[0]).controller, P0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn in_thrall_to_the_pit_cast_in_the_end_step_cant_sacrifice_later() {
    cr!("603.7c", "603.7b");
    ruling!(
        "In Thrall to the Pit",
        "If an effect allows In Thrall to the Pit to be cast during the end step and it is kicked, you will gain control of the target creature until this turn ends. The delayed triggered ability will trigger at the beginning of the next turn's end step, but you won't be able to sacrifice that creature because you no longer control it."
    );
    supported("In Thrall to the Pit");
    let mut t = TestGame::new(2);
    // "You may cast spells as though they had flash."
    t.battlefield(P0, "Leyline of Anticipation");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 5);
    let spell = t.hand(P0, "In Thrall to the Pit");
    t.cast(P0, spell).kicked(true).target(obj(giant)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(giant).controller, P1);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).controller, P1);
}

#[test]
fn glistening_goremonger_opponent_must_sacrifice_what_they_have() {
    cr!("701.21a");
    ruling!(
        "Pyretic Prankster // Glistening Goremonger",
        "If an opponent controls only artifacts and no creatures, or vice versa, they must sacrifice one. They can’t choose to sacrifice an artifact and then fail to do so while they control a creature."
    );
    const PRANKSTER: &str = "Pyretic Prankster // Glistening Goremonger";
    supported(PRANKSTER);
    let mut t = TestGame::new(3);
    let ring = t.battlefield(P1, "Sol Ring");
    let bears = t.battlefield(P2, "Grizzly Bears");
    let card = t.graveyard(P0, PRANKSTER);
    let goremonger = put_onto_battlefield(&mut t, P0, card, true).expect("entered");
    assert_eq!(t.obj(goremonger).chars.name, "Glistening Goremonger");
    // They try not to sacrifice anything.
    t.answer_choose(P1, &[]);
    t.answer_choose(P2, &[]);
    destroy(&mut t, goremonger);
    t.resolve_all();
    assert!(!t.on_battlefield(ring) && !t.on_battlefield(bears));
}

#[test]
fn blood_on_the_snow_returns_a_card_whichever_mode() {
    cr!("700.2a", "608.2c", "107.4h");
    ruling!(
        "Blood on the Snow",
        "You choose a mode for Blood on the Snow as you cast it. The last part of the effect (returning a card to the battlefield) happens no matter which mode you choose."
    );
    supported("Blood on the Snow");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lili = t.battlefield(P1, "Liliana of the Veil");
    let giant = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Snow-Covered Swamp", 6);
    let spell = t.hand(P0, "Blood on the Snow");
    t.answer_choose(P0, &[obj(giant)]);
    t.cast(P0, spell).modes(&[1]).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(lili));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn price_of_betrayal_removes_a_players_counters() {
    cr!("122.1", "115.1");
    ruling!(
        "Price of Betrayal",
        "Players don’t normally have counters, but some cards in other sets say that a player “gets” a counter of a certain kind. Notably, emblems aren’t counters."
    );
    supported("Price of Betrayal");
    let mut t = TestGame::new(2);
    t.g.add_counters(Entity::Player(P1), counters::POISON, 3, None);
    assert_eq!(poison(&t, P1), 3);
    t.lands(P0, "Swamp", 1);
    let spell = t.hand(P0, "Price of Betrayal");
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(poison(&t, P1), 0);
}

#[test]
fn brain_gorgers_a_player_without_creatures_cant_sacrifice() {
    cr!("118.12", "603.2");
    ruling!(
        "Brain Gorgers",
        "A player can't choose to sacrifice a creature unless they actually control a creature that can be sacrificed. For example, if the action were \"discard three cards,\" a player with two or fewer cards in hand couldn't take the option to counter the spell."
    );
    supported("Brain Gorgers");
    let mut t = TestGame::new(2);
    t.answer_yes(P1, true);
    t.answer_yes(P0, false);
    give_mana_for(&mut t, P0, "Brain Gorgers");
    let gorgers = t.hand(P0, "Brain Gorgers");
    t.cast(P0, gorgers).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Brain Gorgers").len(), 1);
    // With a creature, P1 can sacrifice it to counter Brain Gorgers.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(bears)]);
    give_mana_for(&mut t, P0, "Brain Gorgers");
    let gorgers = t.hand(P0, "Brain Gorgers");
    t.cast(P0, gorgers).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Brain Gorgers"));
}

#[test]
fn rakdos_the_defiler_cant_sacrifice_demons() {
    cr!("701.21a", "107.1a");
    ruling!(
        "Rakdos the Defiler",
        "A permanent with the subtype Demon, including Rakdos the Defiler itself, can't be sacrificed this way."
    );
    supported("Rakdos the Defiler");
    let mut t = TestGame::new(2);
    let rakdos = t.battlefield(P0, "Rakdos the Defiler");
    let demon = t.battlefield(P0, "Demon of Loathing");
    let lands = t.lands(P0, "Swamp", 3);
    let from = t.asked().len();
    attack_with(&mut t, &[(rakdos, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(rakdos) && t.on_battlefield(demon));
    // Half of the three non-Demon permanents, rounded up.
    assert_eq!(lands.iter().filter(|l| t.on_battlefield(**l)).count(), 1);
    for offered in offered_to(&t, P0, from) {
        assert!(!offered.contains(&rakdos) && !offered.contains(&demon));
    }
}
