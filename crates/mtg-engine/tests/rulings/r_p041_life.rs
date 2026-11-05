//! Rulings batch P041 — life totals that can't go down or below 1 (CR 119.7, 119.8,
//! 120.3a), not losing the game (CR 104.3), commander damage (CR 903.10a), Two-Headed
//! Giant team life (CR 810.9), Soul Echo's "for each 1 damage" replacement, and an effect
//! "until your next turn" whose controller left the game (CR 800.4m).

use crate::r_p041_common::*;
use mtg_engine::decision::Action;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts Courageous Resolve for `p` (no creature target) with fateful hour on.
fn resolve_fateful(t: &mut TestGame, p: PlayerId) {
    supported("Courageous Resolve");
    lands_for_cost(t, p, "Courageous Resolve");
    let c = t.hand(p, "Courageous Resolve");
    t.answer_targets(p, &[]);
    t.cast_with(p, c, &[]).unwrap();
    t.resolve_all();
}

/// `p` can't lose the game whatever happens: 0 life, ten poison counters, a draw from an
/// empty library.
fn everything_that_would_lose(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].life = 0;
    t.g.players[p.idx()].counters.insert("poison".into(), 10);
    while let Some(c) = t.g.players[p.idx()].library.pop() {
        t.g.objects[c.0 as usize].zone = Zone::Nowhere;
    }
    t.g.draw_cards(p, 1);
    t.g.flush_events();
    t.settle();
}

#[test]
fn sustaining_spirit_doesnt_let_you_pay_more_life_than_you_have() {
    cr!("119.4");
    ruling!(
        "Sustaining Spirit",
        "If an effect asks you to pay life, you can’t pay more life than you have."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sustaining Spirit");
    t.g.players[0].life = 1;
    // Gitaxian Probe ({U/P}) with no mana: payable only with 2 life.
    let probe = t.hand(P0, "Gitaxian Probe");
    assert!(t.cast(P0, probe).target(pl(P1)).try_go().is_err());
    assert_eq!(t.life(P0), 1);
    t.g.players[0].life = 2;
    assert!(t.cast(P0, probe).target(pl(P1)).try_go().is_ok());
    assert_eq!(t.life(P0), 0, "paying life isn't damage");
}

#[test]
fn courageous_resolve_life_total_cant_be_set_or_exchanged_lower() {
    cr!("119.7", "119.5", "701.12c");
    ruling!(
        "Courageous Resolve",
        "If an effect says to set your life total to a certain number and that number is less than your current life total, that part of the effect won't do anything."
    );
    ruling!(
        "Courageous Resolve",
        "If an effect would cause you to exchange life totals with another player whose life total is lower than yours, the exchange won't happen. Neither player's life total changes."
    );
    supported("Biorhythm");
    supported("Magus of the Mirror");
    // Biorhythm: each player's life total becomes the number of creatures they control.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 5;
    resolve_fateful(&mut t, P0);
    t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Biorhythm", &[]);
    assert_eq!(t.life(P0), 5);
    assert_eq!(t.life(P1), 1);
    // Magus of the Mirror: P1 (3 life) exchanges life totals with P0 (5) in P1's upkeep.
    let mut t = TestGame::new(2);
    let magus = t.battlefield(P1, "Magus of the Mirror");
    t.set_step(P1, Step::Upkeep);
    t.g.players[0].life = 5;
    t.g.players[1].life = 3;
    resolve_fateful(&mut t, P0);
    activate_resolve(&mut t, P1, magus, 0, &[pl(P0)]);
    assert!(!t.on_battlefield(magus));
    assert_eq!((t.life(P0), t.life(P1)), (5, 3));
}

#[test]
fn courageous_resolve_fateful_hour_you_cant_lose() {
    cr!("104.3b", "104.3c", "104.3d", "104.3h");
    ruling!(
        "Courageous Resolve",
        "If you cast Courageous Resolve and your life total is 5 or less when it resolves, for the rest of that turn, no game effect can cause you to lose the game or cause any opponent to win the game. It doesn't matter whether you have 0 or less life, you're forced to draw a card while your library is empty, you have ten or more poison counters"
    );
    let mut t = TestGame::new(2);
    t.g.players[0].life = 5;
    resolve_fateful(&mut t, P0);
    everything_that_would_lose(&mut t, P0);
    assert!(!t.has_lost(P0));
    assert!(t.g.result.is_none());
    // At 6 life, fateful hour doesn't apply.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 6;
    resolve_fateful(&mut t, P0);
    t.g.players[0].life = 0;
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn courageous_resolve_in_two_headed_giant() {
    cr!("810.8a", "810.9h");
    ruling!(
        "Courageous Resolve",
        "If you cast Courageous Resolve in a Two-Headed Giant game and your team's life total is 5 or less when it resolves, your team can't lose the game that turn and the opposing team can't win the game that turn."
    );
    let mut t = crate::r_p108_common::two_headed_giant();
    t.g.players[0].life = 5;
    t.g.players[1].life = 5;
    resolve_fateful(&mut t, P0);
    t.g.players[0].life = 0;
    t.g.players[1].life = 0;
    t.settle();
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    assert!(t.g.result.is_none());
}

#[test]
fn commander_damage_is_tracked_even_if_life_doesnt_change() {
    cr!("903.10a", "119.7", "120.3a");
    ruling!(
        "Courageous Resolve",
        "In a Commander game, combat damage you're dealt by a commander is still tracked, even if you don't lose life as a result."
    );
    ruling!(
        "Elderscale Wurm",
        "In a Commander game, combat damage you’re dealt by a commander is still tracked, even if it doesn’t change your life total."
    );
    ruling!(
        "Serra the Benevolent",
        "In the Commander variant, combat damage you're dealt by a commander is still tracked, even if it doesn't change your life total."
    );
    supported("Serra the Benevolent");
    for case in ["Courageous Resolve", "Elderscale Wurm", "Serra"] {
        let mut t = crate::r_s13_common::commander_game();
        let life = match case {
            "Courageous Resolve" => {
                t.g.players[0].life = 5;
                resolve_fateful(&mut t, P0);
                5
            }
            "Elderscale Wurm" => {
                t.battlefield(P0, "Elderscale Wurm");
                t.g.players[0].life = 7;
                7
            }
            _ => {
                let serra = t.battlefield(P0, "Serra the Benevolent");
                set_loyalty(&mut t, serra, 6);
                activate_resolve(&mut t, P0, serra, 2, &[]);
                t.battlefield(P0, "Grizzly Bears");
                t.g.players[0].life = 1;
                1
            }
        };
        let giant = t.battlefield(P1, "Hill Giant");
        t.g.objects[giant.0 as usize].is_commander = true;
        t.g.players[P1.idx()].commander_names.push("Hill Giant".into());
        to_beginning_of_combat(&mut t, P1);
        attack_with(&mut t, &[(giant, pl(P0))]);
        block_and_finish(&mut t, P0, &[]);
        assert_eq!(t.life(P0), life, "{case}");
        assert_eq!(
            t.g.player(P0).commander_damage.get("Hill Giant").copied(),
            Some(3),
            "{case}"
        );
    }
}

#[test]
fn elderscale_wurm_damage_that_would_reduce_life_below_7() {
    cr!("120.3a", "614.1a");
    ruling!(
        "Elderscale Wurm",
        "If damage is dealt to you and Elderscale Wurm at the same time, and that damage is enough to destroy Elderscale Wurm and reduce your life total to 7 or less, Elderscale Wurm’s ability will still apply."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Elderscale Wurm");
    let src = t.battlefield(P1, "Hill Giant");
    t.g.players[0].life = 12;
    damage_at_once(&mut t, &[(src, pl(P0), 9), (src, obj(wurm), 7)], true);
    assert_eq!(t.life(P0), 7);
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn fortune_thief_does_nothing_below_1_life() {
    cr!("120.3a", "104.3b");
    ruling!(
        "Fortune Thief",
        "If your life total is less than 1 but you've somehow avoided losing the game, Fortune Thief's effect won't apply at all if you're dealt damage."
    );
    supported("Fortune Thief");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fortune Thief");
    t.battlefield(P0, "Platinum Angel");
    let src = t.battlefield(P1, "Hill Giant");
    t.g.players[0].life = 0;
    damage(&mut t, src, pl(P0), 3);
    assert_eq!(t.life(P0), -3);
    assert!(!t.has_lost(P0));
    // At 2 life it applies.
    t.g.players[0].life = 2;
    damage(&mut t, src, pl(P0), 3);
    assert_eq!(t.life(P0), 1);
}

// --- Two-Headed Giant ---------------------------------------------------------------------

#[test]
fn damage_to_either_teammate_cant_take_the_team_below_1() {
    cr!("810.9a", "120.3a", "614.1a");
    ruling!(
        "Angel of Grace",
        "In a Two-Headed Giant game, after resolving Angel of Grace's triggered ability, damage dealt to you and/or your teammate that would reduce your team's life total to less than 1 reduces it to 1 instead."
    );
    ruling!(
        "Serra the Benevolent",
        "In a Two-Headed Giant game, damage dealt to you and/or your teammate that would reduce your team's life total to less than 1 reduces it to 1 instead, as long as you control a creature. It doesn't matter whether your teammate controls a creature, even if they're the player being dealt damage"
    );
    supported("Angel of Grace");
    for serra in [false, true] {
        let mut t = crate::r_p108_common::two_headed_giant();
        if serra {
            let s = t.battlefield(P0, "Serra the Benevolent");
            set_loyalty(&mut t, s, 6);
            activate_resolve(&mut t, P0, s, 2, &[]);
            t.battlefield(P0, "Grizzly Bears");
        } else {
            t.enter(P0, "Angel of Grace");
            t.g.flush_events();
            t.resolve_all();
        }
        let src = t.battlefield(P2, "Hill Giant");
        t.g.players[0].life = 3;
        t.g.players[1].life = 3;
        // P0's teammate, who controls no creature, is dealt the damage.
        damage(&mut t, src, pl(P1), 5);
        assert_eq!((t.life(P0), t.life(P1)), (1, 1), "Serra: {serra}");
        assert!(!t.has_lost(P1));
    }
}

#[test]
fn angel_of_grace_sets_the_team_life_total_to_10() {
    cr!("810.9c", "119.5");
    ruling!(
        "Angel of Grace",
        "In a Two-Headed Giant game, the activated ability of Angel of Grace causes you to gain or lose life so that your team's life total becomes 10."
    );
    for start in [25, 4] {
        let mut t = crate::r_p108_common::two_headed_giant();
        let angel = t.graveyard(P0, "Angel of Grace");
        t.lands(P0, "Plains", 6);
        t.g.players[0].life = start;
        t.g.players[1].life = start;
        activate_resolve(&mut t, P0, angel, 0, &[]);
        assert_eq!((t.life(P0), t.life(P1)), (10, 10), "from {start}");
    }
}

// --- Soul Echo ----------------------------------------------------------------------------

/// Soul Echo under P0's control with `n` echo counters.
fn soul_echo(t: &mut TestGame, n: u32) -> ObjectId {
    let e = t.battlefield(P0, "Soul Echo");
    t.g.add_counters(obj(e), "echo", n, None);
    e
}

#[test]
fn soul_echo_doesnt_affect_damage_before_its_trigger_resolves() {
    cr!("603.3", "614.1a");
    ruling!(
        "Soul Echo",
        "If you take damage between the start of the upkeep step and the time in which the upkeep trigger resolves, then Soul Echo will not affect that damage and you will be affected by the damage as normal."
    );
    supported("Soul Echo");
    let mut t = TestGame::new(2);
    let echo = soul_echo(&mut t, 5);
    let src = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P1, true);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the upkeep trigger");
    damage(&mut t, src, pl(P0), 2);
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    damage(&mut t, src, pl(P0), 2);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.counters(echo, "echo"), 3);
}

#[test]
fn several_soul_echoes_one_takes_all_the_damage() {
    cr!("616.1", "616.1e");
    ruling!(
        "Soul Echo",
        "If you have more than one Soul Echo, the opponent chooses which ones will be replacing damage that turn (if any). And when damage happens, you decide which one of the ones the opponent selected that you want the damage to be replaced with, but the damage all goes to one of them."
    );
    let mut t = TestGame::new(2);
    let a = soul_echo(&mut t, 5);
    let b = soul_echo(&mut t, 5);
    let src = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::End);
    // P1 chooses that both replace damage.
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let from = t.asked().len();
    damage(&mut t, src, pl(P0), 3);
    assert_eq!(t.life(P0), 20);
    assert_eq!(replacement_choosers(&t, from), vec![P0]);
    let (ca, cb) = (t.counters(a, "echo"), t.counters(b, "echo"));
    assert!((ca, cb) == (2, 5) || (ca, cb) == (5, 2), "{ca}, {cb}");
}

#[test]
fn soul_echo_targets_an_opponent_each_upkeep() {
    cr!("603.3d", "115.1");
    ruling!(
        "Soul Echo",
        "In a multi-player game, you choose a new opponent during each of your upkeeps."
    );
    let mut t = TestGame::new(3);
    soul_echo(&mut t, 5);
    let from = t.asked().len();
    t.answer_targets(P0, &[pl(P2)]);
    t.set_step(P2, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    let offered = crate::r_p016_common::target_choices(&t, P0, from, "");
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&pl(P1)) && offered[0].contains(&pl(P2)));
    assert!(!offered[0].contains(&pl(P0)));
}

// --- Dovin, a player leaving the game -------------------------------------------------------

#[test]
fn dovin_until_your_next_turn_after_you_left_the_game() {
    cr!("800.4m", "611.2b");
    ruling!(
        "Dovin, Hand of Control",
        "In a multiplayer game, if you leave the game after Dovin’s last ability resolves but before your next turn begins, its effect lasts until your next turn would have begun. It neither expires immediately nor lasts indefinitely."
    );
    let mut t = TestGame::new(3);
    let dovin = t.battlefield(P0, "Dovin, Hand of Control");
    let giant = t.battlefield(P1, "Hill Giant");
    activate_resolve(&mut t, P0, dovin, 0, &[obj(giant)]);
    t.g.perform_action(P0, Action::Concede).unwrap();
    assert!(t.has_lost(P0));
    t.advance_to(P1, Step::PrecombatMain);
    damage(&mut t, giant, pl(P2), 3);
    assert_eq!(t.life(P2), 20, "P1's turn");
    t.advance_to(P2, Step::PrecombatMain);
    damage(&mut t, giant, pl(P2), 3);
    assert_eq!(t.life(P2), 20, "P2's turn");
    // P0's turn would have begun after P2's: the effect ended.
    t.advance_to(P1, Step::Upkeep);
    damage(&mut t, giant, pl(P2), 3);
    assert_eq!(t.life(P2), 17);
}
