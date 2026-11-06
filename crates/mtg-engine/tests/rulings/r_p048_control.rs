//! Rulings batch P048 — giving permanents to other players: "target player gains control
//! of target permanent you control" (Donate, Bazaar Trader, Zedruu, Harmless Offering,
//! Iroh), control changes with no duration (CR 613.1b, 611.2a), players leaving the game
//! (CR 800.4a), and the permanents that pass themselves around (Witch Hunt, Chaos Lord,
//! Contested War Zone, Scrambleverse).

use crate::r_p048_common::*;
use crate::r_s06_common::{attached_to, give_control};
use crate::r_s25_common::targets_of;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Zedruu's "{U}{R}{W}: Target opponent gains control of target permanent you control."
fn zedruu_gives(t: &mut TestGame, zedruu: ObjectId, to: PlayerId, what: ObjectId) {
    for ty in [ManaType::U, ManaType::R, ManaType::W] {
        mana(t, P0, ty, 1);
    }
    activate_with(
        t,
        P0,
        zedruu,
        "Target opponent gains control",
        &[Entity::Player(to), obj(what)],
    );
}

#[test]
fn donating_an_aura_leaves_it_attached() {
    cr!("613.1b", "303.4e");
    ruling!(
        "Donate",
        "If you Donate an Aura, it stays where it is, but the targeted player now controls it."
    );
    supported("Donate");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Pacifism", obj(bears));
    cast_new(&mut t, P0, "Donate", &[Entity::Player(P1), obj(aura)]);
    t.resolve_all();
    assert_eq!(controller(&mut t, aura), P1);
    assert_eq!(attached_to(&t, aura), Some(obj(bears)));
    assert!(t.on_battlefield(aura));
}

#[test]
fn harmless_offering_cant_be_refused() {
    cr!("613.1b", "608.2c");
    ruling!(
        "Harmless Offering",
        "Your opponent can't refuse your generous donation."
    );
    supported("Harmless Offering");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    cast_new(
        &mut t,
        P0,
        "Harmless Offering",
        &[Entity::Player(P1), obj(ring)],
    );
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(controller(&mut t, ring), P1);
    // P1 was never asked anything but priority.
    assert!(t.asked()[from..]
        .iter()
        .all(|(p, d)| *p != P1 || matches!(d, Decision::Priority { .. })));
}

#[test]
fn giving_a_permanent_does_nothing_if_either_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Bazaar Trader",
        "If either the targeted player or the targeted permanent is an illegal target by the time the ability resolves, the ability does nothing."
    );
    ruling!(
        "Zedruu the Greathearted",
        "If either the opponent or the permanent you control becomes an illegal target by the time Zedruu's last ability tries to resolve, the ability does nothing."
    );
    supported("Bazaar Trader");
    supported("Zedruu the Greathearted");
    for card in ["Bazaar Trader", "Zedruu the Greathearted"] {
        for player_illegal in [true, false] {
            let mut t = TestGame::new(3);
            let src = t.battlefield(P0, card);
            let bears = t.battlefield(P0, "Grizzly Bears");
            if card == "Bazaar Trader" {
                activate_with(
                    &mut t,
                    P0,
                    src,
                    "Target player gains control",
                    &[Entity::Player(P1), obj(bears)],
                );
            } else {
                zedruu_gives(&mut t, src, P1, bears);
            }
            if player_illegal {
                // P1 gets hexproof (Leyline of Sanctity: "You have hexproof."): P0 keeps
                // the Bears.
                t.battlefield(P1, "Leyline of Sanctity");
                t.resolve_all();
                assert_eq!(controller(&mut t, bears), P0, "{card}");
            } else {
                // P2 takes the Bears in response: they're no longer a permanent P0
                // controls, so P1 doesn't get them.
                give_control(&mut t, bears, P2);
                t.resolve_all();
                assert_eq!(controller(&mut t, bears), P2, "{card}");
            }
            assert_eq!(controller(&mut t, src), P0, "{card}");
        }
    }
}

#[test]
fn bazaar_traders_control_change_has_no_duration() {
    cr!("611.2a", "613.1b", "613.7");
    ruling!(
        "Bazaar Trader",
        "This effect has no duration. The targeted player retains control of the targeted permanent until the game ends, the permanent leaves the battlefield, or an effect causes someone else to gain control of it."
    );
    let mut t = TestGame::new(2);
    let trader = t.battlefield(P0, "Bazaar Trader");
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_with(
        &mut t,
        P0,
        trader,
        "Target player gains control",
        &[Entity::Player(P1), obj(bears)],
    );
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P1);
    // Turns go by: P1 keeps it.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(controller(&mut t, bears), P1);
    // Until another effect gives it to someone else: P0 takes it back with Act of
    // Treason, until end of turn; then it's P1's again.
    cast_new(&mut t, P0, "Act of Treason", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&mut t, bears), P1);
}

#[test]
fn bazaar_trader_targeting_yourself_makes_a_temporary_control_change_permanent() {
    cr!("613.7", "611.2a", "514.2");
    ruling!(
        "Bazaar Trader",
        "You may target yourself with Bazaar Trader’s ability. Normally, this won’t have any visible effect."
    );
    supported("Act of Treason");
    let mut t = TestGame::new(2);
    let trader = t.battlefield(P0, "Bazaar Trader");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    // Targeting yourself and a permanent you control: nothing visible.
    activate_with(
        &mut t,
        P0,
        trader,
        "Target player gains control",
        &[Entity::Player(P0), obj(mine)],
    );
    t.resolve_all();
    assert_eq!(controller(&mut t, mine), P0);
    // Act of Treason on P1's Bears, then Bazaar Trader (untapped again) targeting P0 and
    // the Bears: P0 keeps them after the turn ends.
    cast_new(&mut t, P0, "Act of Treason", &[obj(theirs)]);
    t.resolve_all();
    assert_eq!(controller(&mut t, theirs), P0);
    t.g.objects[trader.0 as usize].tapped = false;
    activate_with(
        &mut t,
        P0,
        trader,
        "Target player gains control",
        &[Entity::Player(P0), obj(theirs)],
    );
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&mut t, theirs), P0);
}

#[test]
fn permanents_given_to_a_player_who_leaves_return_to_you() {
    cr!("800.4a");
    ruling!(
        "Zedruu the Greathearted",
        "If an opponent leaves the game, all permanents you gave to them courtesy of Zedruu will return to your control."
    );
    let mut t = TestGame::new(3);
    let zedruu = t.battlefield(P0, "Zedruu the Greathearted");
    let ring = t.battlefield(P0, "Sol Ring");
    zedruu_gives(&mut t, zedruu, P1, ring);
    t.resolve_all();
    assert_eq!(controller(&mut t, ring), P1);
    concede(&mut t, P1);
    assert!(t.on_battlefield(ring));
    assert_eq!(controller(&mut t, ring), P0);
}

#[test]
fn permanents_you_own_leave_the_game_with_you() {
    cr!("800.4a");
    ruling!(
        "Zedruu the Greathearted",
        "If you leave the game, all permanents you own leave the game with you."
    );
    ruling!(
        "Iroh, Tea Master",
        "If you leave the game, all permanents you own leave the game with you."
    );
    // Zedruu gives P1 a Sol Ring.
    let mut t = TestGame::new(3);
    let zedruu = t.battlefield(P0, "Zedruu the Greathearted");
    let ring = t.battlefield(P0, "Sol Ring");
    zedruu_gives(&mut t, zedruu, P1, ring);
    t.resolve_all();
    assert_eq!(controller(&mut t, ring), P1);
    concede(&mut t, P0);
    assert_eq!(t.zone(ring), Zone::Nowhere);
    assert!(!t.in_exile("Sol Ring"));
    // Iroh, at the beginning of combat on P0's turn, gives P2 a Sol Ring.
    supported("Iroh, Tea Master");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Iroh, Tea Master");
    let ring = t.battlefield(P0, "Sol Ring");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.answer_targets(P0, &[obj(ring)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(controller(&mut t, ring), P2);
    concede(&mut t, P0);
    assert_eq!(t.zone(ring), Zone::Nowhere);
    assert!(!t.in_exile("Sol Ring"));
}

#[test]
fn chaos_lords_upkeep_control_change_overrides_earlier_control_effects() {
    cr!("613.7", "613.1b");
    ruling!(
        "Chaos Lord",
        "This creature's changing of controllers is a new effect each upkeep so it will take precedence over any other control effects."
    );
    supported("Chaos Lord");
    supported("Control Magic");
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P1, "Chaos Lord");
    // P0 takes it with Control Magic (4 lands): Chaos Lord, Control Magic and four lands
    // make six permanents, an even number.
    cast_new(&mut t, P0, "Control Magic", &[obj(lord)]);
    t.resolve_all();
    assert_eq!(controller(&mut t, lord), P0);
    assert_eq!(t.g.permanents().count() % 2, 0);
    // At P0's upkeep, P0's Chaos Lord trigger gives it to P1 despite Control Magic.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(controller(&mut t, lord), P1);
    assert_eq!(named_by(&t, P0, "Control Magic").len(), 1);
}

#[test]
fn witch_hunt_makes_life_gain_impossible() {
    cr!("119.7", "119.9", "614.17c");
    ruling!(
        "Witch Hunt",
        "Abilities that trigger whenever a player gains life"
    );
    supported("Witch Hunt");
    // P0's Ajani's Pridemate doesn't trigger, and P1's Tainted Remedy ("If an opponent
    // would gain life, that player loses that much life instead.") doesn't apply.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Witch Hunt");
    let mate = t.battlefield(P0, "Ajani's Pridemate");
    t.battlefield(P1, "Tainted Remedy");
    cast_new(&mut t, P0, "Revitalize", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(mate, "+1/+1"), 0);
}

#[test]
fn witch_hunt_stops_life_totals_from_becoming_higher() {
    cr!("119.7", "119.5");
    ruling!(
        "Witch Hunt",
        "If an effect sets a player’s life total to a specific number and that number is higher than the player’s current life total, that part of the effect won’t do anything."
    );
    // Captive Audience under P1's control, first mode ("Your life total becomes 4.").
    for (before, after) in [(2, 2), (10, 4)] {
        let mut t = TestGame::new(2);
        t.answer_choose(P0, &[Entity::Player(P1)]);
        t.enter(P0, "Captive Audience");
        t.resolve_all();
        set_life(&mut t, P1, before);
        t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![0]));
        t.advance_to(P1, Step::Upkeep);
        t.settle();
        // Witch Hunt arrives now (at P0's end step it would have changed hands).
        t.battlefield(P0, "Witch Hunt");
        t.resolve_all();
        assert_eq!(t.life(P1), after);
    }
}

#[test]
fn witch_hunt_targets_an_opponent_at_random_as_its_ability_is_put_on_the_stack() {
    cr!("115.1", "603.3d", "613.1b");
    ruling!(
        "Witch Hunt",
        "If you control Witch Hunt, you are not a legal target."
    );
    ruling!(
        "Witch Hunt",
        "The target is chosen at random as you put the last ability on the stack."
    );
    ruling!(
        "Witch Hunt",
        "To choose a target at random, all possible legal targets must have an equal chance of being chosen."
    );
    // Three players, several random streams: the ability, on the stack in P0's end step,
    // already has its target, an opponent; both opponents get chosen.
    let mut chosen = [0; 3];
    for seed in 0..24u64 {
        let mut t = TestGame::new(3);
        use rand::SeedableRng;
        t.g.rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let hunt = t.battlefield(P0, "Witch Hunt");
        t.set_step(P0, Step::PostcombatMain);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        let top = *t.g.stack.last().unwrap();
        let target = match targets_of(&t, top).as_slice() {
            [Entity::Player(p)] => *p,
            other => panic!("targets {other:?}"),
        };
        assert_ne!(target, P0);
        chosen[target.idx()] += 1;
        t.resolve_all();
        assert_eq!(controller(&mut t, hunt), target);
    }
    assert_eq!(chosen[0], 0);
    assert!(chosen[1] >= 5 && chosen[2] >= 5, "{chosen:?}");
    // The choice is among the legal targets only: with P1 hexproof (Leyline of Sanctity),
    // it's always P2.
    for seed in 0..12u64 {
        let mut t = TestGame::new(3);
        use rand::SeedableRng;
        t.g.rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let hunt = t.battlefield(P0, "Witch Hunt");
        t.battlefield(P1, "Leyline of Sanctity");
        t.set_step(P0, Step::PostcombatMain);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), 1, "seed {seed}");
        let top = *t.g.stack.last().unwrap();
        assert_eq!(targets_of(&t, top), vec![Entity::Player(P2)], "seed {seed}");
        t.resolve_all();
        assert_eq!(controller(&mut t, hunt), P2);
    }
}

#[test]
fn witch_hunt_with_no_legal_target_stays_with_you() {
    cr!("603.3d", "115.1");
    ruling!(
        "Witch Hunt",
        "If there are no legal targets available, the ability is removed from the stack and you retain control of Witch Hunt."
    );
    let mut t = TestGame::new(2);
    let hunt = t.battlefield(P0, "Witch Hunt");
    t.battlefield(P1, "Leyline of Sanctity");
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&mut t, hunt), P0);
}

/// `attacker` attacks `defender` with a new Grizzly Bears that isn't blocked (P1's turn), and combat ends.
fn bears_hit(t: &mut TestGame, attacker: PlayerId, defender: PlayerId) -> ObjectId {
    let bears = t.battlefield(attacker, "Grizzly Bears");
    to_combat(t, attacker);
    t.attack(&[(bears, Entity::Player(defender))], &[]);
    t.resolve_all();
    bears
}

#[test]
fn gaining_control_of_contested_war_zone_doesnt_tap_or_untap_it() {
    cr!("613.1b");
    ruling!(
        "Contested War Zone",
        "Gaining control of Contested War Zone doesn’t cause it to tap or untap."
    );
    supported("Contested War Zone");
    for tapped in [true, false] {
        let mut t = TestGame::new(2);
        let zone = t.battlefield(P0, "Contested War Zone");
        t.g.objects[zone.0 as usize].tapped = tapped;
        bears_hit(&mut t, P1, P0);
        assert_eq!(controller(&mut t, zone), P1);
        assert_eq!(t.obj_now(zone).tapped, tapped);
    }
}

#[test]
fn you_in_contested_war_zones_trigger_is_its_current_controller() {
    cr!("109.5", "603.2");
    ruling!(
        "Contested War Zone",
        "The word “you” in a permanent’s trigger condition always refers to the permanent’s current controller."
    );
    // P0 owns it, P1 controls it. P1's creature hitting P0 (its owner) does nothing; P0's
    // creature hitting P1 (its controller) gives it back to P0.
    let mut t = TestGame::new(2);
    let zone = t.battlefield(P0, "Contested War Zone");
    give_control(&mut t, zone, P1);
    bears_hit(&mut t, P1, P0);
    assert_eq!(t.life(P0), 18);
    assert_eq!(controller(&mut t, zone), P1);
    bears_hit(&mut t, P0, P1);
    assert_eq!(controller(&mut t, zone), P0);
}

#[test]
fn contested_war_zone_pumps_every_attacking_creature() {
    cr!("611.2c");
    ruling!(
        "Contested War Zone",
        "The last activated ability affects all attacking creatures, not just those you control."
    );
    // During P1's attack, P0 activates its Contested War Zone.
    let mut t = TestGame::new(2);
    let zone = t.battlefield(P0, "Contested War Zone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let idle = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.lands(P0, "Wastes", 1);
    activate_with(&mut t, P0, zone, "Attacking creatures get", &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(idle), (3, 3));
    // The set of creatures it affects was fixed as it resolved (CR 611.2c): after combat,
    // when the Bears aren't attacking any more, they keep the bonus until end of turn.
    t.advance_to(P1, Step::PostcombatMain);
    assert!(!crate::r_s10_common::attacking(&t, bears));
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn contested_war_zone_hit_by_two_players_creatures_goes_to_the_last_to_resolve() {
    cr!("603.3b", "810.7", "101.4");
    ruling!(
        "Contested War Zone",
        "In some multiplayer formats, creatures controlled by different players may deal combat damage to you simultaneously."
    );
    // Two-Headed Giant: P0 and P1 (one team) attack P2 (the other), who controls the
    // Contested War Zone. P2 controls both triggers and orders them; the two orders give
    // the land to different players: the last to resolve decides.
    let mut results = vec![];
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = two_headed_giant();
        let zone = t.battlefield(P2, "Contested War Zone");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P0, Step::BeginningOfCombat);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(a, Entity::Player(P2)), (b, Entity::Player(P2))]),
        );
        t.answer(
            P1,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(a, Entity::Player(P2)), (b, Entity::Player(P2))]),
        );
        let from = t.asked().len();
        t.answer(P2, DecisionKind::Order, Answer::Indices(order));
        t.advance_to_step(Step::CombatDamage);
        t.settle();
        assert_eq!(t.stack_len(), 2);
        assert!(t.g.stack.iter().all(|s| t.g.obj(*s).controller == P2));
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P2 && matches!(d, Decision::Order { .. })));
        t.resolve_all();
        results.push(controller(&mut t, zone));
    }
    results.sort();
    assert_eq!(results, vec![P0, P1]);
}

#[test]
fn scrambleverse_changes_control_of_everything_at_once_without_responses() {
    cr!("608.2f", "608.2c");
    ruling!(
        "Scrambleverse",
        "All of the control-change effects happen at the same time."
    );
    ruling!(
        "Scrambleverse",
        "Once Scrambleverse starts to resolve, no player may respond until it has finished resolving."
    );
    supported("Scrambleverse");
    let mut t = TestGame::new(3);
    use rand::SeedableRng;
    t.g.rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);
    // Two tapped permanents each (they're untapped after the control changes).
    let perms: Vec<ObjectId> = (0..3)
        .flat_map(|p| {
            let p = PlayerId(p);
            vec![
                t.battlefield(p, "Grizzly Bears"),
                t.battlefield(p, "Sol Ring"),
            ]
        })
        .collect();
    for p in &perms {
        tap(&mut t, *p);
    }
    let before: Vec<PlayerId> = perms.iter().map(|p| t.obj(*p).controller).collect();
    cast_new(&mut t, P0, "Scrambleverse", &[]);
    t.settle();
    let from = t.asked().len();
    let ev_from = t.g.turn_events.len();
    t.g.resolve_top();
    t.g.flush_events();
    // No one was asked anything while it resolved.
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    // Some permanents changed hands (with this random stream)...
    let after: Vec<PlayerId> = perms.iter().map(|p| controller(&mut t, *p)).collect();
    let moved = before.iter().zip(&after).filter(|(b, a)| b != a).count();
    assert!(moved > 0, "{after:?}");
    // ... all at once: every control change is reported together, before any of the
    // permanents untaps ("Untap those permanents" comes after all of them).
    let evs = &t.g.turn_events[ev_from..];
    let idx = |pred: fn(&Event) -> bool| -> Vec<usize> {
        evs.iter()
            .enumerate()
            .filter(|(_, e)| pred(e))
            .map(|(i, _)| i)
            .collect()
    };
    let changes = idx(|e| matches!(e, Event::ControlChanged { .. }));
    let untaps = idx(|e| matches!(e, Event::Untapped { .. }));
    assert_eq!(changes.len(), moved, "{evs:?}");
    assert_eq!(untaps.len(), perms.len(), "{evs:?}");
    let (f, l) = (changes[0], changes[changes.len() - 1]);
    assert_eq!(
        l - f + 1,
        changes.len(),
        "control changes interleaved: {evs:?}"
    );
    assert!(untaps.iter().all(|u| *u > l), "{evs:?}");
}
