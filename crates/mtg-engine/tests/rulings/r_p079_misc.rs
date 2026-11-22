//! Rulings batch P079 — attack requirements and their exceptions (CR 508.1d), effects
//! that don't track controllers (CR 611.2c), "this turn" history (Boarded Window),
//! maximum hand size in timestamp order (CR 402.2, 613.7), and skipping extra turns
//! (CR 614.10).

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s08_common::is_tapped;
use crate::r_s09_common::{legal_attack, to_combat};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `n` players; P0's Kardur, Doomscourge enters (its enters trigger resolves), then it's
/// P1's beginning of combat.
fn kardur(n: usize) -> TestGame {
    let mut t = TestGame::new(n);
    t.enter(P0, "Kardur, Doomscourge");
    t.resolve_all();
    t
}

#[test]
fn kardur_creatures_must_attack_another_player_if_able() {
    cr!("508.1d");
    ruling!(
        "Kardur, Doomscourge",
        "If a creature affected by Kardur's ability could legally attack multiple players and/or planeswalkers, its controller chooses who it attacks."
    );
    ruling!(
        "Kardur, Doomscourge",
        "it must attack a player other than you if able. If the creature can't attack any of those players, or if there are no such players, it must attack you or an opposing planeswalker."
    );
    supported("Kardur, Doomscourge");
    // Four players: P1's Bears may attack P2 or P3, not P0, and must attack.
    let mut t = kardur(4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P2))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P3))]));
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(!legal_attack(&mut t, &[]));
    // Two players: it must attack P0 or P0's planeswalker.
    let mut t = kardur(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(bears, obj(jace))]));
    assert!(!legal_attack(&mut t, &[]));
    // Three players, but P2's Blazing Archon ("creatures can't attack you"): P0.
    let mut t = kardur(3);
    t.battlefield(P2, "Blazing Archon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(!legal_attack(&mut t, &[]));
}

#[test]
fn kardur_doesnt_force_creatures_that_cant_attack_or_attack_costs() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Kardur, Doomscourge",
        "If a creature an opponent controls can't attack for some reason (such as being tapped or having come under its controller's control that turn), then it doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost"
    );
    supported("Ghostly Prison");
    // Tapped, or summoning sick.
    for sick in [false, true] {
        let mut t = kardur(2);
        let bears = if sick {
            t.battlefield_sick(P1, "Grizzly Bears")
        } else {
            t.battlefield(P1, "Grizzly Bears")
        };
        if !sick {
            t.g.tap(bears);
        }
        to_combat(&mut t, P1);
        assert!(legal_attack(&mut t, &[]), "sick: {sick}");
    }
    // P0's Ghostly Prison: attacking P0 costs {2}, which P1 needn't pay.
    let mut t = kardur(2);
    t.battlefield(P0, "Ghostly Prison");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 2);
    to_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn fumiko_doesnt_force_paying_or_unable_creatures() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Fumiko the Lowblood",
        "Fumiko's ability doesn't force an opponent to pay for effects in order to be able to attack with their creatures (such as Ghostly Prison, for example)."
    );
    ruling!(
        "Fumiko the Lowblood",
        "Fumiko's ability doesn't force tapped creatures or creatures with summoning sickness to attack."
    );
    supported("Fumiko the Lowblood");
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Fumiko the Lowblood");
    };
    // An able creature must attack.
    let mut t = TestGame::new(2);
    setup(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    // Tapped, summoning sick, or attacking costs {2} (Ghostly Prison): not forced.
    for case in 0..3 {
        let mut t = TestGame::new(2);
        setup(&mut t);
        let bears = if case == 1 {
            t.battlefield_sick(P1, "Grizzly Bears")
        } else {
            t.battlefield(P1, "Grizzly Bears")
        };
        match case {
            0 => {
                t.g.tap(bears);
            }
            2 => {
                t.battlefield(P0, "Ghostly Prison");
                t.lands(P1, "Wastes", 2);
            }
            _ => {}
        }
        to_combat(&mut t, P1);
        assert!(legal_attack(&mut t, &[]), "case {case}");
    }
}

#[test]
fn clinging_mists_follows_the_creature_to_its_new_controller() {
    cr!("611.2c", "502.3");
    ruling!(
        "Clinging Mists",
        "Clinging Mists doesn’t track the controller of the attacking creatures. If one of them changes controller before its previous controller’s next untap step, it won’t untap during its new controller’s next untap step."
    );
    supported("Clinging Mists");
    // P1 (5 life) casts Clinging Mists at P0's attacking Bears, then gains control of
    // them: they stay tapped through P1's next untap step, and untap in the one after.
    let mut t = TestGame::new(2);
    t.g.players[1].life = 5;
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    cast_card(&mut t, P1, "Clinging Mists", &[]).unwrap();
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    crate::r_s06_common::give_control(&mut t, bears, P1);
    t.advance_to(P1, Step::Upkeep);
    assert!(is_tapped(&t, bears), "P1's next untap step");
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_tapped(&t, bears));
}

#[test]
fn boarded_window_counts_all_damage_dealt_to_you_this_turn() {
    cr!("603.4", "120.3");
    ruling!(
        "Boarded Window",
        "Boarded Window's triggered ability checks the total amount of damage dealt to you during the turn. That damage needn't have been dealt all at once, and Boarded Window needn't have been on the battlefield at the time the damage was dealt. Boarded Window doesn't take into account any life that you gained this turn."
    );
    supported("Boarded Window");
    // Two Shocks to P0 before Boarded Window is on the battlefield, then P0 gains 5 life:
    // exiled at the end step. One Shock: it stays.
    for shocks in [2, 1] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        for _ in 0..shocks {
            cast_card(&mut t, P1, "Shock", &[Entity::Player(P0)]).unwrap();
            t.resolve_all();
        }
        let window = t.battlefield(P0, "Boarded Window");
        t.g.gain_life(P0, 5);
        t.settle();
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.on_battlefield(window), shocks == 1, "{shocks} Shocks");
    }
}

#[test]
fn maximum_hand_size_effects_apply_in_timestamp_order() {
    cr!("402.2", "613.7", "613.11");
    ruling!(
        "Tamiyo, Inquisitive Student // Tamiyo, Seasoned Scholar",
        "If multiple effects modify your hand size, apply them in timestamp order."
    );
    supported("Tamiyo, Inquisitive Student // Tamiyo, Seasoned Scholar");
    supported("Necrodominance");
    for necro_first in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        if necro_first {
            t.battlefield(P0, "Necrodominance");
        }
        let tamiyo = back_face(
            &mut t,
            P0,
            "Tamiyo, Inquisitive Student // Tamiyo, Seasoned Scholar",
        );
        crate::r_s13_common::add(&mut t, tamiyo, "loyalty", 10);
        crate::r_s06_common::activate_containing(&mut t, P0, tamiyo, "emblem").unwrap();
        t.resolve_all();
        if !necro_first {
            t.battlefield(P0, "Necrodominance");
        }
        t.g.recompute();
        let max = t.g.player(P0).max_hand_size;
        if necro_first {
            assert_eq!(max, None, "the emblem's later timestamp wins");
        } else {
            assert_eq!(max, Some(5), "Necrodominance's later timestamp wins");
        }
    }
}

#[test]
fn trouble_in_pairs_skips_extra_turns_only_when_they_would_begin() {
    cr!("614.10", "500.7");
    ruling!(
        "Trouble in Pairs",
        "Extra turns can still be created while Trouble in Pairs is on the battlefield. They're not skipped until they would begin, so if Trouble in Pairs leaves the battlefield before that happens, the extra turns will be unaffected."
    );
    // P1 casts Time Walk with P0's Trouble in Pairs on the battlefield; the extra turn is
    // created. If Trouble in Pairs is gone before it begins, P1 takes it; otherwise it's
    // skipped and P0 is next.
    for removed in [true, false] {
        let mut t = TestGame::new(2);
        let trouble = t.battlefield(P0, "Trouble in Pairs");
        t.advance_to(P1, Step::PrecombatMain);
        let turn = t.g.turn.number;
        cast_card(&mut t, P1, "Time Walk", &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.g.extra_turns.len(), 1, "the extra turn was created");
        if removed {
            crate::r_s02_common::destroy(&mut t, trouble);
        }
        let ok = t.g.run_until(10_000, |g| {
            g.turn.number > turn && g.turn.step == Step::Upkeep
        });
        assert!(ok);
        let expect = if removed { P1 } else { P0 };
        assert_eq!(t.g.turn.active, expect, "removed: {removed}");
    }
}
