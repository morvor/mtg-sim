//! Rulings batch P195 — effects that last indefinitely and ownership: control changes
//! that outlast their source (Agent of Treachery, Eyes Everywhere, Kiora Bests the Sea
//! God), a token's owner (CR 111.2), Aminatou's blink of a permanent you own, abilities
//! that look back over the turn (Admiral Beckett Brass), choices made on resolution
//! (Liliana, Waker of the Dead; Ashiok, Nightmare Weaver) and The Soul Stone's harnessed
//! designation.

use crate::r_p195_common::*;
use crate::r_s01_common::{custom_card, stack_library};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 casts Raise the Alarm: two 1/1 Soldier tokens P1 owns. Returns them.
fn p1_tokens(t: &mut TestGame) -> Vec<ObjectId> {
    cast_at(t, P1, "Raise the Alarm", &[]);
    t.resolve_all();
    tokens(t, P1)
}

/// `p` gains control of `id` until end of turn with Ray of Command.
fn ray(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    cast_at(t, p, "Ray of Command", &[obj(id)]);
    t.resolve_all();
    assert_eq!(ctl(t, id), p);
}

#[test]
fn a_tokens_owner_is_the_player_who_created_it() {
    cr!("111.2", "108.3");
    ruling!("Brand", "A token's owner is the player who created it.");
    ruling!("Misfortune's Gain", "A token's owner is the player who created it.");
    ruling!("Enslave", "A token's owner is the player who created it.");
    ruling!("Coveted Falcon", "A token's owner is the player who created it.");
    ruling!("Stormfront Riders", "A token's owner is the player who created it.");
    supported("Brand");
    supported("Misfortune's Gain");
    supported("Coveted Falcon");
    supported("Stormfront Riders");
    // Brand: P1 gets back its token that P0 took.
    let mut t = TestGame::new(2);
    let tk = p1_tokens(&mut t)[0];
    ray(&mut t, P0, tk);
    cast_at(&mut t, P1, "Brand", &[]);
    t.resolve_all();
    assert_eq!(ctl(&t, tk), P1);
    // Misfortune's Gain: the token's owner (P1) gains the life, not its controller.
    let mut t = TestGame::new(2);
    let tk = p1_tokens(&mut t)[0];
    ray(&mut t, P0, tk);
    cast_at(&mut t, P0, "Misfortune's Gain", &[obj(tk)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 24);
    assert_eq!(t.life(P0), 20);
    // Enslave: the token deals damage to its owner.
    let mut t = TestGame::new(2);
    let tk = p1_tokens(&mut t)[0];
    cast_at(&mut t, P0, "Enslave", &[obj(tk)]);
    t.resolve_all();
    assert_eq!(ctl(&t, tk), P0);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
    // Coveted Falcon: gain control of a token you created that another player controls.
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Coveted Falcon");
    cast_at(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    let tk = tokens(&t, P0)[0];
    ray(&mut t, P1, tk);
    t.answer_targets(P0, &[obj(tk)]);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(f, pl(P1))]));
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(ctl(&t, tk), P0);
    // Stormfront Riders: P1's token P0 controls goes to its owner's hand, not P0's, so
    // it doesn't make a Soldier for P0.
    let mut t = TestGame::new(2);
    let tk = p1_tokens(&mut t)[0];
    ray(&mut t, P0, tk);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(tk), obj(mine)]);
    t.enter(P0, "Stormfront Riders");
    t.resolve_all();
    assert!(!t.on_battlefield(tk));
    assert!(!t.in_hand(P0, "Soldier"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(tokens(&t, P1).len(), 1);
}

#[test]
fn agent_of_treachery_control_lasts_until_you_leave_the_game() {
    cr!("611.2a", "800.4a", "613.1b");
    ruling!(
        "Agent of Treachery",
        "Agent of Treachery's effect lasts indefinitely. It doesn't wear off during the cleanup step, and it doesn't expire if Agent of Treachery leaves the battlefield. In a multiplayer game, it does expire if you leave the game."
    );
    supported("Agent of Treachery");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let a = t.enter(P0, "Agent of Treachery");
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
    kill(&mut t, a);
    assert_eq!(ctl(&t, bears), P0);
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert_eq!(ctl(&t, bears), P0);
    t.g.lose_game(P0);
    t.settle();
    t.g.recompute();
    assert!(t.on_battlefield(bears));
    assert_eq!(ctl(&t, bears), P1);
}

#[test]
fn aminatou_blinks_a_permanent_you_own_that_another_player_controls() {
    cr!("108.3", "610.3c", "115.1");
    ruling!(
        "Aminatou, the Fateshifter",
        "Aminatou's second ability can target any permanent you own, including those another player controls."
    );
    supported("Aminatou, the Fateshifter");
    let mut t = TestGame::new(2);
    let am = t.battlefield(P0, "Aminatou, the Fateshifter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    ray(&mut t, P1, bears);
    act(&mut t, P0, am, 1, &[obj(bears)]);
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(ctl(&t, bears[0]), P0);
    // It's a new object: Ray of Command's effect no longer applies to it.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(ctl(&t, bears[0]), P0);
}

#[test]
fn kiora_does_not_stop_permanents_gained_later_from_untapping() {
    cr!("611.2c", "502.3");
    ruling!(
        "Kiora Bests the Sea God",
        "Any nonland permanents that come under the target player’s control after the second chapter ability resolves will untap as normal."
    );
    supported("Kiora Bests the Sea God");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Kiora Bests the Sea God");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 1);
    let old = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[pl(P1)]);
    t.g.add_counters(obj(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(tapped(&t, old));
    // A creature that comes under P1's control afterwards, then becomes tapped.
    let later = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, later);
    t.advance_to(P1, Step::Upkeep);
    assert!(tapped(&t, old));
    assert!(!tapped(&t, later));
}

#[test]
fn eyes_everywhere_must_be_there_to_exchange() {
    cr!("701.12a", "608.2b");
    ruling!(
        "Eyes Everywhere",
        "As Eyes Everywhere’s last ability resolves, Eyes Everywhere must be on the battlefield and the target nonland permanent must be a legal target. If either of these things isn’t true, the ability does nothing."
    );
    supported("Eyes Everywhere");
    // Normal exchange.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Eyes Everywhere");
    let bears = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, e, 0, &[obj(bears)]);
    assert_eq!(ctl(&t, bears), P0);
    assert_eq!(ctl(&t, e), P1);
    // Eyes Everywhere gone: nothing.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Eyes Everywhere");
    let bears = t.battlefield(P1, "Grizzly Bears");
    act_only(&mut t, P0, e, 0, &[obj(bears)]);
    kill(&mut t, e);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P1);
    // The target gone: nothing (Eyes stays with P0).
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Eyes Everywhere");
    let bears = t.battlefield(P1, "Grizzly Bears");
    act_only(&mut t, P0, e, 0, &[obj(bears)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(ctl(&t, e), P0);
}

#[test]
fn admiral_beckett_brass_looks_back_at_the_turn() {
    cr!("603.10", "608.2h");
    ruling!(
        "Admiral Beckett Brass",
        "Admiral Beckett Brass's triggered ability checks what happened earlier in the turn. It doesn't care whether Admiral Beckett Brass was on the battlefield when those Pirates dealt combat damage, whether those creatures are still on the battlefield, or whether they're still Pirates."
    );
    supported("Admiral Beckett Brass");
    let mut t = TestGame::new(2);
    let pirate = custom_card("Test Pirate", "Creature — Human Pirate", "{1}", Some((1, 1)), "");
    let ps: Vec<ObjectId> = (0..3)
        .map(|_| t.custom(P0, pirate.clone(), Zone::Battlefield))
        .collect();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(&ps.iter().map(|p| (*p, pl(P1))).collect::<Vec<_>>(), &[]);
    assert_eq!(t.life(P1), 17);
    // One is gone, one isn't a Pirate any more; the Admiral arrives afterwards.
    kill(&mut t, ps[0]);
    cast_at(&mut t, P0, "Turn to Frog", &[obj(ps[1])]);
    t.resolve_all();
    assert!(!t.obj_now(ps[1]).chars.has_subtype("Pirate"));
    t.battlefield(P0, "Admiral Beckett Brass");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
}

#[test]
fn liliana_each_player_discards_and_only_those_who_cant_lose_life() {
    cr!("101.4", "701.9a", "608.2c");
    ruling!(
        "Liliana, Waker of the Dead",
        "A player who has a card in hand can't choose to lose 3 life instead of discarding a card."
    );
    ruling!(
        "Liliana, Waker of the Dead",
        "As Liliana's first ability resolves, first the player whose turn it is chooses a card in hand without revealing it, then each other player in turn order does the same. Then all the chosen cards are discarded at the same time. Finally, each opponent who couldn't discard a card loses 3 life."
    );
    supported("Liliana, Waker of the Dead");
    let mut t = TestGame::new(3);
    let l = t.battlefield(P0, "Liliana, Waker of the Dead");
    for p in [P0, P1] {
        t.hand(p, "Grizzly Bears");
        t.hand(p, "Hill Giant");
    }
    // P2 has no cards in hand.
    let p2_hand: Vec<ObjectId> = t.g.player(P2).hand.clone();
    for c in p2_hand {
        t.g.move_object(c, Zone::Graveyard(P2), events::MoveCause::Effect, None);
    }
    let from = t.asked().len();
    act(&mut t, P0, l, 0, &[]);
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 17);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn ashiok_minus_x_chooses_on_resolution_without_targeting() {
    cr!("115.1", "608.2c", "107.3");
    ruling!(
        "Ashiok, Nightmare Weaver",
        "Ashiok's second ability doesn't target any creature card. You choose which creature card to return when that ability resolves, but you must choose one with mana value equal to the value you chose for X when activating the ability."
    );
    supported("Ashiok, Nightmare Weaver");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ashiok, Nightmare Weaver");
    let cards = stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant", "Runeclaw Bear"]);
    act(&mut t, P0, a, 0, &[pl(P1)]);
    assert_eq!(t.counters(a, "loyalty"), 5);
    pass_turn(&mut t);
    pass_turn(&mut t);
    let rb = t.g.current(cards[2]);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(rb)]);
    act(&mut t, P0, a, 1, &[]);
    assert!(target_choices(&t, P0, from, "").is_empty());
    let cands = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .expect("no choice on resolution");
    let mut cands = cands;
    cands.sort_by_key(|e| e.object().map(|o| o.0));
    let mut want = vec![obj(t.g.current(cards[0])), obj(rb)];
    want.sort_by_key(|e| e.object().map(|o| o.0));
    assert_eq!(cands, want);
    let rb = t.g.current(rb);
    assert!(t.on_battlefield(rb));
    assert_eq!(ctl(&t, rb), P0);
    assert!(t.obj_now(rb).chars.has_subtype("Nightmare"));
}

#[test]
fn the_soul_stone_infinity_ability_needs_it_harnessed() {
    cr!("701.64a", "701.64b", "702.186b");
    ruling!(
        "The Soul Stone",
        "\"Harnessed\" is a designation that the permanent has once an ability instructs you to harness it. It has no special rules meaning other than being a designation that the ∞ ability (and, theoretically, other effects) can see."
    );
    supported("The Soul Stone");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "The Soul Stone");
    let dead = t.graveyard(P0, "Hill Giant");
    // Not harnessed: nothing at upkeep.
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::PrecombatMain);
    let fodder = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(fodder)]);
    act(&mut t, P0, s, 1, &[]);
    assert!(t.in_exile("Grizzly Bears"));
    t.answer_targets(P0, &[obj(dead)]);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(dead));
}
