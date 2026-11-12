//! Rulings batch P196 — exchanges (CR 701.12): an exchange with an illegal or missing
//! part doesn't happen (701.12a, 608.2b), an exchange between permanents one player
//! controls does nothing (701.12b), life totals exchanged with a power or toughness
//! (701.12g), sequential exchanges, and changing the targets of an exchange (115.7).

use crate::r_p196_common::*;
use crate::r_s04_common::top_of_stack;
use crate::r_s06_common::give_control;
use crate::r_s25_common::{abilities_from, change_copy_targets, spell_copies, targets_of};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn evra_exchange_needs_evra_but_not_a_positive_power() {
    cr!("701.12a", "701.12g", "104.3b", "704.5a");
    ruling!(
        "Evra, Halcyon Witness",
        "If Evra isn't on the battlefield when its activated ability resolves, the exchange can't happen and the ability will have no effect. However, if Evra is on the battlefield but has power 0 or less, the exchange happens and you'll lose the game."
    );
    supported("Evra, Halcyon Witness");
    // Evra leaves in response: nothing happens.
    let mut t = TestGame::new(2);
    let evra = t.battlefield(P0, "Evra, Halcyon Witness");
    act_only(&mut t, P0, evra, 0, &[]);
    kill(&mut t, evra);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(!t.has_lost(P0));
    // Evra with power 0: the exchange happens and P0 is at 0 life.
    let mut t = TestGame::new(2);
    let evra = t.battlefield(P0, "Evra, Halcyon Witness");
    pump(&mut t, evra, -4, 0);
    assert_eq!(t.pt(evra), (0, 4));
    act(&mut t, P0, evra, 0, &[]);
    assert_eq!(t.life(P0), 0);
    assert!(t.has_lost(P0));
}

#[test]
fn the_trees_exchanges_need_the_tree_on_the_battlefield() {
    cr!("701.12a", "701.12g", "608.2b");
    ruling!(
        "Tree of Perdition",
        "If Tree of Perdition isn't on the battlefield when the ability resolves, the exchange can't happen and the ability will have no effect. Notably, activating the ability and giving Tree of Perdition -13/-13 in response won't cause your opponent to lose the game."
    );
    ruling!(
        "Tree of Redemption",
        "If Tree of Redemption isn't on the battlefield when its activated ability resolves, the exchange can't happen and the ability will have no effect."
    );
    supported("Tree of Perdition");
    supported("Tree of Redemption");
    // Tree of Perdition gets -13/-13 in response: it dies, and P1 keeps 20 life.
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P0, "Tree of Perdition");
    act_only(&mut t, P0, tree, 0, &[pl(P1)]);
    pump(&mut t, tree, -13, -13);
    assert!(!t.on_battlefield(tree));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(!t.has_lost(P1));
    // Tree of Redemption leaves in response: P0's life total is unchanged.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 5;
    let tree = t.battlefield(P0, "Tree of Redemption");
    act_only(&mut t, P0, tree, 0, &[]);
    kill(&mut t, tree);
    t.resolve_all();
    assert_eq!(t.life(P0), 5);
    // Control: on the battlefield it happens.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 5;
    let tree = t.battlefield(P0, "Tree of Redemption");
    act(&mut t, P0, tree, 0, &[]);
    assert_eq!(t.life(P0), 13);
    assert_eq!(t.pt(tree).1, 5);
}

/// Vedalken Plotter enters for P0 targeting P0's Forest and P1's Mountain; its trigger is
/// on the stack.
fn plotter(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let forest = t.battlefield(P0, "Forest");
    let mountain = t.battlefield(P1, "Mountain");
    t.answer_targets(P0, &[obj(forest)]);
    t.answer_targets(P0, &[obj(mountain)]);
    t.enter(P0, "Vedalken Plotter");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (forest, mountain)
}

#[test]
fn vedalken_plotter_never_gains_a_land_without_losing_one() {
    cr!("701.12a", "608.2b");
    ruling!(
        "Vedalken Plotter",
        "If either land becomes an illegal target (or leaves the battlefield) before the ability resolves, the exchange won't happen. There's no way for you to gain a land without losing another one in the process."
    );
    supported("Vedalken Plotter");
    // The opponent's land leaves: P0 keeps the Forest.
    let mut t = TestGame::new(2);
    let (forest, mountain) = plotter(&mut t);
    kill(&mut t, mountain);
    t.resolve_all();
    assert_eq!(ctl(&t, forest), P0);
    // P0's land becomes an illegal target (P1 now controls it): P0 doesn't get the
    // Mountain.
    let mut t = TestGame::new(2);
    let (forest, mountain) = plotter(&mut t);
    give_control(&mut t, forest, P1);
    t.resolve_all();
    assert_eq!(ctl(&t, forest), P1);
    assert_eq!(ctl(&t, mountain), P1);
    // Both legal: exchanged.
    let mut t = TestGame::new(2);
    let (forest, mountain) = plotter(&mut t);
    t.resolve_all();
    assert_eq!(ctl(&t, forest), P1);
    assert_eq!(ctl(&t, mountain), P0);
}

#[test]
fn oko_and_spawnbroker_dont_exchange_with_an_illegal_target() {
    cr!("701.12a", "608.2b");
    ruling!(
        "Oko, Thief of Crowns",
        "If either of the target permanents is an illegal target when Oko's last ability resolves, the exchange won't happen."
    );
    ruling!(
        "Spawnbroker",
        "If either target becomes illegal (say, if a creature’s power has changed such that the creature you control now has less power than the other creature), then the exchange won’t happen."
    );
    supported("Oko, Thief of Crowns");
    supported("Spawnbroker");
    // Oko −5: P1's Hill Giant (power 3) gets +1/+0 in response.
    for grow in [false, true] {
        let mut t = TestGame::new(2);
        let oko = t.battlefield(P0, "Oko, Thief of Crowns");
        t.g.add_counters(obj(oko), "loyalty", 3, None);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        act_only(&mut t, P0, oko, 2, &[obj(bears), obj(giant)]);
        if grow {
            pump(&mut t, giant, 1, 0);
        }
        t.resolve_all();
        assert_eq!(
            ctl(&t, bears),
            if grow { P0 } else { P1 },
            "Oko, grow {grow}"
        );
        assert_eq!(
            ctl(&t, giant),
            if grow { P1 } else { P0 },
            "Oko, grow {grow}"
        );
    }
    // Spawnbroker: P0's Hill Giant (3) for P1's Grizzly Bears (2); the Bears get +2/+0.
    for grow in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(giant)]);
        t.answer_targets(P0, &[obj(bears)]);
        t.enter(P0, "Spawnbroker");
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if grow {
            pump(&mut t, bears, 2, 0);
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(
            ctl(&t, giant),
            if grow { P0 } else { P1 },
            "Spawnbroker {grow}"
        );
        assert_eq!(
            ctl(&t, bears),
            if grow { P1 } else { P0 },
            "Spawnbroker {grow}"
        );
    }
}

#[test]
fn djinn_of_infinite_deceits_exchange_fails_or_does_nothing() {
    cr!("701.12a", "701.12b", "608.2b");
    ruling!(
        "Djinn of Infinite Deceits",
        "If one of the creatures is an illegal target when the ability resolves, the exchange won’t happen. If both creatures are illegal targets, the ability won’t resolve."
    );
    ruling!(
        "Djinn of Infinite Deceits",
        "If the same player controls both creatures when the ability resolves, nothing happens."
    );
    supported("Djinn of Infinite Deceits");
    let setup = |t: &mut TestGame| {
        let djinn = t.battlefield(P0, "Djinn of Infinite Deceits");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        // One target slot for both ("two target nonlegendary creatures").
        t.answer_targets(P0, &[obj(bears), obj(giant)]);
        act_only(t, P0, djinn, 0, &[]);
        assert_eq!(
            crate::r_s25_common::targets_of(t, *t.g.stack.last().unwrap()),
            vec![obj(bears), obj(giant)]
        );
        (djinn, bears, giant)
    };
    // One illegal: no exchange (the ability still resolves).
    let mut t = TestGame::new(2);
    let (djinn, bears, giant) = setup(&mut t);
    crate::r_p189_common::bounce(&mut t, giant);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
    assert!(ability_resolved_from(&t, djinn));
    // Both legal: exchanged.
    let mut t = TestGame::new(2);
    let (_, bears, giant) = setup(&mut t);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P1);
    assert_eq!(ctl(&t, giant), P0);
    // Both illegal: the ability doesn't resolve.
    let mut t = TestGame::new(2);
    let (djinn, bears, giant) = setup(&mut t);
    kill(&mut t, bears);
    kill(&mut t, giant);
    t.resolve_all();
    assert!(!ability_resolved_from(&t, djinn));
    // The same player controls both: nothing happens (CR 701.12b). P0 controls the Giant
    // only until end of turn, so it goes back to P1 then: no exchange gave it to P0.
    let mut t = TestGame::new(2);
    let (_, bears, giant) = setup(&mut t);
    give_control_until_eot(&mut t, giant, P0);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
    assert_eq!(ctl(&t, giant), P0);
    pass_turn(&mut t);
    assert_eq!(ctl(&t, giant), P1);
    assert_eq!(ctl(&t, bears), P0);
}

#[test]
fn two_target_exchange_spells_dont_exchange_with_an_illegal_target() {
    cr!("701.12a", "608.2b");
    ruling!(
        "Shrewd Negotiation",
        "If one of the target permanents is an illegal target as Shrewd Negotiation resolves, the exchange won't happen."
    );
    ruling!(
        "Role Reversal",
        "If one of the target permanents is an illegal target when Role Reversal resolves, the exchange won’t happen."
    );
    ruling!(
        "Shifting Borders",
        "If one of the targeted lands isn’t a legal target as Shifting Borders resolves (say, because it has left the battlefield or become untargetable), the exchange doesn’t occur."
    );
    ruling!(
        "Trade the Helm",
        "If one of the targets is an illegal target when this spell resolves, the exchange won’t happen."
    );
    // (spell, P0's permanent, P1's permanent, one target slot for both)
    let cases = [
        ("Shrewd Negotiation", "Sol Ring", "Grizzly Bears", false),
        ("Role Reversal", "Grizzly Bears", "Hill Giant", true),
        ("Shifting Borders", "Forest", "Mountain", true),
        ("Trade the Helm", "Sol Ring", "Ornithopter", false),
    ];
    for (spell, mine, theirs, one_slot) in cases {
        supported(spell);
        for leave in [false, true] {
            let mut t = TestGame::new(2);
            let a = t.battlefield(P0, mine);
            let b = t.battlefield(P1, theirs);
            t.battlefield(P1, "Llanowar Elves");
            t.battlefield(P1, "Millstone");
            t.battlefield(P1, "Forest");
            let s = if one_slot {
                cast_multi(&mut t, P0, spell, &[obj(a), obj(b)])
            } else {
                cast_at(&mut t, P0, spell, &[obj(a), obj(b)])
            };
            assert_eq!(targets_of(&t, s), vec![obj(a), obj(b)], "{spell}");
            if leave {
                crate::r_p189_common::bounce(&mut t, b);
            }
            t.resolve_all();
            assert_eq!(ctl(&t, a), if leave { P0 } else { P1 }, "{spell} {leave}");
            if !leave {
                assert_eq!(ctl(&t, b), P0, "{spell}");
            }
        }
    }
}

#[test]
fn role_reversal_between_one_players_permanents_does_nothing() {
    cr!("701.12b");
    ruling!(
        "Role Reversal",
        "If the same player controls both permanents when Role Reversal resolves, nothing happens."
    );
    // P0 controls the Giant only until end of turn: after the turn it's P1's again.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let s = cast_multi(&mut t, P0, "Role Reversal", &[obj(bears), obj(giant)]);
    assert_eq!(targets_of(&t, s), vec![obj(bears), obj(giant)]);
    give_control_until_eot(&mut t, giant, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Role Reversal"));
    assert_eq!(ctl(&t, bears), P0);
    assert_eq!(ctl(&t, giant), P0);
    pass_turn(&mut t);
    assert_eq!(ctl(&t, giant), P1);
    assert_eq!(ctl(&t, bears), P0);
}

#[test]
fn shifting_grifts_exchanges_happen_independently() {
    cr!("701.12a", "701.12b", "702.172a", "608.2b");
    ruling!(
        "Shifting Grift",
        "If one of the target permanents is an illegal target when Shifting Grift resolves, the exchange that permanent is involved in won’t happen."
    );
    ruling!(
        "Shifting Grift",
        "If the same player controls both permanents involved in a single exchange when that exchange occurs, nothing happens to those permanents."
    );
    supported("Shifting Grift");
    supported("Millstone");
    // 0: the Hill Giant leaves; 1: P0 gains control of it first, until end of turn.
    for case in 0..2 {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let ring = t.battlefield(P0, "Sol Ring");
        let mill = t.battlefield(P1, "Millstone");
        lots(&mut t, P0);
        let c = t.hand(P0, "Shifting Grift");
        t.cast(P0, c)
            .modes(&[0, 1])
            .targets(&[obj(bears), obj(giant), obj(ring), obj(mill)])
            .go();
        assert_eq!(
            targets_of(&t, t.g.current(c)),
            vec![obj(bears), obj(giant), obj(ring), obj(mill)]
        );
        if case == 0 {
            crate::r_p189_common::bounce(&mut t, giant);
        } else {
            give_control_until_eot(&mut t, giant, P0);
            assert_eq!(ctl(&t, giant), P0);
        }
        t.resolve_all();
        assert_eq!(ctl(&t, bears), P0, "case {case}");
        if case == 1 {
            assert_eq!(ctl(&t, giant), P0);
        }
        // The artifacts are still exchanged.
        assert_eq!(ctl(&t, ring), P1, "case {case}");
        assert_eq!(ctl(&t, mill), P0, "case {case}");
        if case == 1 {
            // Nothing happened to the creatures: the Giant goes back to P1 after the turn.
            pass_turn(&mut t);
            assert_eq!(ctl(&t, giant), P1);
            assert_eq!(ctl(&t, bears), P0);
        }
    }
}

#[test]
fn juxtapose_exchanges_creatures_then_artifacts() {
    cr!("701.12a", "608.2c");
    ruling!(
        "Juxtapose",
        "First creatures are exchanged, then artifacts are exchanged. It's possible that the same artifact creature may be involved in both exchanges."
    );
    supported("Juxtapose");
    supported("Juggernaut");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ring = t.battlefield(P0, "Sol Ring");
    let jugg = t.battlefield(P1, "Juggernaut");
    let mill = t.battlefield(P1, "Millstone");
    cast_at(&mut t, P0, "Juxtapose", &[pl(P1)]);
    t.resolve_all();
    // Bears <-> Juggernaut, then P0's greatest artifact (now Juggernaut) <-> Millstone.
    assert_eq!(ctl(&t, bears), P1);
    assert_eq!(ctl(&t, jugg), P1);
    assert_eq!(ctl(&t, mill), P0);
    assert_eq!(ctl(&t, ring), P0);
}

#[test]
fn gilded_drake_is_sacrificed_if_the_creature_is_gone() {
    cr!("701.12a", "608.2b");
    ruling!(
        "Gilded Drake",
        "If the chosen creature to exchange is not on the battlefield at resolution, then sacrifice this card."
    );
    supported("Gilded Drake");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let drake = t.enter(P0, "Gilded Drake");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, bears);
    t.resolve_all();
    assert!(!t.on_battlefield(drake));
    assert!(t.in_graveyard(P0, "Gilded Drake"));
}

/// The targets a copy of an exchange may be changed to: only a final set that is still
/// legal (CR 115.7e, 707.10c).
#[test]
fn changing_the_targets_of_shifting_loyalties_keeps_the_set_legal() {
    cr!("115.7d", "115.7e", "707.10c");
    ruling!(
        "Shifting Loyalties",
        "If another spell or ability allows you to change the targets of Shifting Loyalties (or perhaps copy it and choose new targets for the copy), you can change the targets only such that the final set of targets is still legal."
    );
    supported("Shifting Loyalties");
    supported("Twincast");
    // new: the copy's new targets (None keeps one); want: its targets afterwards.
    let setup = |t: &mut TestGame| {
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let p1_ring = t.battlefield(P1, "Sol Ring");
        let p2_bears = t.battlefield(P2, "Grizzly Bears");
        let p0_ring = t.battlefield(P0, "Sol Ring");
        let p1_mill = t.battlefield(P1, "Millstone");
        let spell = cast_multi(t, P0, "Shifting Loyalties", &[obj(bears), obj(giant)]);
        [bears, giant, p1_ring, p2_bears, p0_ring, p1_mill, spell]
    };
    for case in 0..3 {
        let mut t = TestGame::new(3);
        let [bears, giant, p1_ring, p2_bears, p0_ring, p1_mill, spell] = setup(&mut t);
        let (new, want) = match case {
            // Just the second target to a noncreature permanent: not allowed.
            0 => (vec![None, Some(obj(p1_ring))], vec![obj(bears), obj(giant)]),
            // Just the second target to another opponent's creature: allowed.
            1 => (
                vec![None, Some(obj(p2_bears))],
                vec![obj(bears), obj(p2_bears)],
            ),
            // Both, to two permanents sharing a different card type: allowed.
            _ => (
                vec![Some(obj(p0_ring)), Some(obj(p1_mill))],
                vec![obj(p0_ring), obj(p1_mill)],
            ),
        };
        cast_at(&mut t, P0, "Twincast", &[obj(spell)]);
        change_copy_targets(&mut t, P0, &new);
        t.resolve();
        let copy = spell_copies(&t)[0];
        assert_eq!(targets_of(&t, copy), want, "case {case}");
    }
}

#[test]
fn changing_the_targets_of_daring_thiefs_ability_keeps_the_set_legal() {
    cr!("115.7d", "115.7e", "707.10c");
    ruling!(
        "Daring Thief",
        "If another spell or ability allows you to change the targets of the ability (or perhaps copy it and choose new targets for the copy), you can change the targets only such that the final set of targets is still legal."
    );
    supported("Strionic Resonator");
    for case in 0..3 {
        let mut t = TestGame::new(3);
        let thief = t.battlefield(P0, "Daring Thief");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let p1_ring = t.battlefield(P1, "Sol Ring");
        let p2_bears = t.battlefield(P2, "Grizzly Bears");
        let p0_ring = t.battlefield(P0, "Sol Ring");
        let p1_mill = t.battlefield(P1, "Millstone");
        let res = t.battlefield(P0, "Strionic Resonator");
        t.g.tap(thief);
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_targets(P0, &[obj(giant)]);
        t.g.untap(thief);
        t.g.flush_events();
        t.settle();
        let trig = top_of_stack(&t);
        assert_eq!(targets_of(&t, trig), vec![obj(bears), obj(giant)]);
        let (new, want) = match case {
            0 => (vec![None, Some(obj(p1_ring))], vec![obj(bears), obj(giant)]),
            1 => (
                vec![None, Some(obj(p2_bears))],
                vec![obj(bears), obj(p2_bears)],
            ),
            _ => (
                vec![Some(obj(p0_ring)), Some(obj(p1_mill))],
                vec![obj(p0_ring), obj(p1_mill)],
            ),
        };
        act_only(&mut t, P0, res, 0, &[obj(trig)]);
        change_copy_targets(&mut t, P0, &new);
        t.resolve();
        let copies = abilities_from(&t, thief);
        assert_eq!(copies.len(), 2, "case {case}");
        assert_eq!(targets_of(&t, copies[1]), want, "case {case}");
    }
}
