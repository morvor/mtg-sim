//! Rulings batch P194 — "cards put into your graveyard from the battlefield this turn":
//! a countered permanent spell or a discarded card never was on the battlefield, a card
//! put into another player's graveyard isn't in yours, and timing doesn't matter
//! (before the card that asks entered, or in the same combat damage step).

use crate::r_p194_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Runeclaw Bear and P1 counters it with Counterspell. Returns the card in P0's
/// graveyard.
fn countered_bears(t: &mut TestGame) -> ObjectId {
    let b = t.hand(P0, "Runeclaw Bear");
    mana(t, P0, ManaType::G, 2);
    let spell = t.cast(P0, b).go();
    let cs = t.hand(P1, "Counterspell");
    mana(t, P1, ManaType::U, 2);
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    let now = t.g.current(spell);
    assert_eq!(t.g.obj(now).zone, Zone::Graveyard(P0));
    now
}

/// A Grizzly Bears of `owner`'s that died this turn (the card in the graveyard).
fn died_bears(t: &mut TestGame, owner: PlayerId) -> ObjectId {
    let b = t.battlefield(owner, "Grizzly Bears");
    kill(t, b);
    t.g.current(b)
}

/// A Hill Giant `p` discarded this turn.
fn discarded_giant(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let g = t.hand(p, "Hill Giant");
    t.g.discard(p, g, None);
    t.g.flush_events();
    t.g.current(g)
}

#[test]
fn faiths_reward_returns_only_your_cards_that_died_this_turn() {
    cr!("700.4", "400.7", "701.6a");
    ruling!(
        "Faith's Reward",
        "Permanent cards that were put into your graveyard from anywhere else, such as a card that was discarded, stay in the graveyard. Permanent cards that you gained control of and went to another player's graveyard from the battlefield will likewise stay in the graveyard."
    );
    ruling!(
        "Faith's Reward",
        "Permanent spells that were countered earlier in the turn never entered, so they won't return to the battlefield because of Faith's Reward."
    );
    supported("Faith's Reward");
    let mut t = TestGame::new(2);
    let died = died_bears(&mut t, P0);
    let discarded = discarded_giant(&mut t, P0);
    let countered = countered_bears(&mut t);
    // P1's creature that P0 controlled when it died went to P1's graveyard.
    let borrowed = t.battlefield(P1, "Hill Giant");
    crate::r_p117_common::gain_control(&mut t, P0, borrowed);
    kill(&mut t, borrowed);
    assert_eq!(t.zone(borrowed), Zone::Graveyard(P1));
    let fr = t.hand(P0, "Faith's Reward");
    mana(&mut t, P0, ManaType::W, 4);
    t.cast(P0, fr).go();
    t.resolve_all();
    assert!(t.on_battlefield(died));
    assert_eq!(t.zone(discarded), Zone::Graveyard(P0));
    assert_eq!(t.zone(countered), Zone::Graveyard(P0));
    assert_eq!(t.zone(borrowed), Zone::Graveyard(P1));
}

#[test]
fn faiths_reward_aura_cant_enchant_what_returns_with_it() {
    cr!("303.4f", "303.4g");
    ruling!(
        "Faith's Reward",
        "You choose what an Aura card put onto the battlefield this way will enchant. You can't choose any permanent cards entering at the same time as that Aura. If there's nothing legal for the Aura to enchant, it stays in the graveyard."
    );
    supported("Holy Strength");
    // Without and with another creature on the battlefield.
    for other in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let aura = t.battlefield(P0, "Holy Strength");
        assert!(t.g.attach(aura, obj(bears)));
        let giant = other.then(|| t.battlefield(P1, "Hill Giant"));
        kill(&mut t, bears);
        t.settle();
        assert!(t.in_graveyard(P0, "Holy Strength"));
        let fr = t.hand(P0, "Faith's Reward");
        mana(&mut t, P0, ManaType::W, 4);
        t.cast(P0, fr).go();
        t.resolve_all();
        assert!(t.on_battlefield(bears));
        match giant {
            None => assert!(t.in_graveyard(P0, "Holy Strength")),
            Some(g) => {
                assert!(t.on_battlefield(aura));
                assert_eq!(t.obj_now(aura).attached_to, Some(obj(g)));
                assert_eq!(t.pt(g), (4, 5));
                assert_eq!(t.pt(bears), (2, 2));
            }
        }
    }
}

#[test]
fn countered_permanent_spells_arent_cards_that_died() {
    cr!("700.4", "701.6a", "115.1");
    ruling!(
        "Salvager of Ruin",
        "Permanent spells that were countered earlier in the turn never entered the battlefield, so they are not legal targets for Salvager of Ruin’s ability."
    );
    ruling!(
        "Salvager of Ruin",
        "Salvager of Ruin’s ability can target a permanent card in your graveyard that was put there from the battlefield before Salvager of Ruin entered the battlefield during the same turn."
    );
    ruling!(
        "Brought Back",
        "Permanent spells that were countered earlier in the turn never entered the battlefield, so they won't be legal targets for Brought Back."
    );
    ruling!(
        "Gerrard, Weatherlight Hero",
        "Permanent spells that were countered earlier in the turn never entered the battlefield, so they won't be returned by Gerrard's ability."
    );
    supported("Salvager of Ruin");
    supported("Brought Back");
    supported("Gerrard, Weatherlight Hero");
    // Salvager entered after the Bears died.
    let mut t = TestGame::new(2);
    let died = died_bears(&mut t, P0);
    let countered = countered_bears(&mut t);
    let salvager = t.battlefield(P0, "Salvager of Ruin");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(died)]);
    t.activate(P0, salvager, 0, &[]).unwrap();
    let choices = target_choices(&t, P0, from, "");
    assert_eq!(choices, vec![vec![obj(died)]]);
    assert!(!choices[0].contains(&obj(countered)));
    t.resolve_all();
    assert_eq!(t.zone(died), Zone::Hand(P0));

    // Brought Back: the countered spell can't be targeted.
    let mut t = TestGame::new(2);
    let died = died_bears(&mut t, P0);
    let countered = countered_bears(&mut t);
    let bb = t.hand(P0, "Brought Back");
    mana(&mut t, P0, ManaType::W, 2);
    let from = t.asked().len();
    t.cast(P0, bb).targets(&[obj(died)]).go();
    let choices = target_choices(&t, P0, from, "");
    assert!(choices.iter().all(|c| !c.contains(&obj(countered))));
    assert!(choices[0].contains(&obj(died)));
    t.resolve_all();
    assert!(t.on_battlefield(died));
    assert!(t.obj_now(died).tapped);
    assert_eq!(t.zone(countered), Zone::Graveyard(P0));

    // Gerrard: returns the creature that died, not the countered spell.
    let mut t = TestGame::new(2);
    let died = died_bears(&mut t, P0);
    let countered = countered_bears(&mut t);
    let gerrard = t.battlefield(P0, "Gerrard, Weatherlight Hero");
    kill(&mut t, gerrard);
    t.resolve_all();
    assert!(t.on_battlefield(died));
    assert_eq!(t.zone(countered), Zone::Graveyard(P0));
    assert_eq!(t.zone(gerrard), Zone::Exile);
}

#[test]
fn grim_return_targets_any_graveyard_but_not_discarded_cards() {
    cr!("700.4", "115.1");
    ruling!(
        "Grim Return",
        "You can choose a creature card in any player’s graveyard as long as that card was put there from the battlefield during the turn. A creature card that was put into a graveyard from anywhere else, such as a card that was discarded, can’t be chosen as the target."
    );
    supported("Grim Return");
    let mut t = TestGame::new(2);
    let theirs = died_bears(&mut t, P1);
    let mine = died_bears(&mut t, P0);
    let discarded = discarded_giant(&mut t, P1);
    let gr = t.hand(P0, "Grim Return");
    mana(&mut t, P0, ManaType::B, 3);
    let from = t.asked().len();
    t.cast(P0, gr).target(theirs).go();
    let choices = target_choices(&t, P0, from, "");
    assert_eq!(choices.len(), 1);
    let mut c = choices[0].clone();
    c.sort();
    let mut want = vec![obj(theirs), obj(mine)];
    want.sort();
    assert_eq!(c, want);
    assert!(!choices[0].contains(&obj(discarded)));
    t.resolve_all();
    assert!(t.on_battlefield(theirs));
    assert_eq!(t.obj_now(theirs).controller, P0);
}

#[test]
fn moira_can_return_a_creature_that_died_in_the_same_damage_step() {
    cr!("510.2", "603.3d", "700.4");
    ruling!(
        "Moira, Urborg Haunt",
        "You may target a creature card with Moira's triggered ability that was put into your graveyard as a result of combat damage dealt in the same combat damage step."
    );
    supported("Moira, Urborg Haunt");
    let mut t = TestGame::new(2);
    let moira = t.battlefield(P0, "Moira, Urborg Haunt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(moira, pl(P1)), (bears, pl(P1))]);
    t.answer_targets(P0, &[obj(bears)]);
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.on_battlefield(bears));
    assert_ne!(t.g.current(bears), bears);
}
