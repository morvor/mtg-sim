//! Item `rulings-residue-burn-freeze` — the interaction rulings of batch P062: tapping
//! effects that keep a creature from untapping during its controller's next untap step
//! (CR 502.3, 611.2a: they track the creature, not its controller, and are used up by that
//! untap step even if the creature is untapped then), stun counters (CR 122.1d), Koma's
//! "can't be activated" (CR 602.5, 605), control changes with no duration (CR 611.2a),
//! costs (CR 601.2f, 107.3), targets (CR 608.2b), and casting without paying (CR 118.9).

use crate::r_residue_burn_freeze_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::STUN;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// "Doesn't untap during its controller's next untap step"
// ---------------------------------------------------------------------------------------

/// P1's creature `name`, frozen by P0's real card `card` ("[it] doesn't untap during its
/// controller's next untap step"), tapped or not as `tapped` says. Returns the creature.
fn frozen_by(t: &mut TestGame, card: &str, name: &str, tapped: bool) -> ObjectId {
    supported(card);
    t.set_step(P0, Step::PrecombatMain);
    let c = t.battlefield(P1, name);
    // (Attacking taps the creature the Minotaur blocks.)
    if tapped && card != "Labyrinth Minotaur" {
        tap(t, c);
    }
    match card {
        "Elvish Hunter" | "Barl's Cage" => {
            let src = t.battlefield(P0, card);
            lands_for_cost(
                t,
                P0,
                if card == "Barl's Cage" {
                    "Divination"
                } else {
                    "Llanowar Elves"
                },
            );
            t.lands(P0, "Wastes", 1);
            t.activate(P0, src, 0, &[obj(c)])
                .unwrap_or_else(|e| panic!("{card}: {e:?}"));
            t.resolve_all();
        }
        "Permafrost Trap" => {
            cast_new(t, P0, card, &[obj(c)]);
            t.resolve_all();
            assert!(is_tapped(t, c));
        }
        "Guardian of Tazeem" => {
            t.battlefield(P0, card);
            let island = t.hand(P0, "Island");
            t.answer_targets(P0, &[obj(c)]);
            t.play_land(P0, island).expect("land play");
            t.settle();
            t.resolve_all();
            assert!(is_tapped(t, c));
        }
        "Labyrinth Minotaur" => {
            // P1 attacks P0 with it; the Minotaur blocks.
            let m = t.battlefield(P0, card);
            t.set_step(P1, Step::PrecombatMain);
            attack_with(t, &[(c, pl(P0))]);
            block_and_finish(t, P0, &[(m, c)]);
            t.resolve_all();
            // A creature with vigilance stays untapped.
            assert_eq!(is_tapped(t, c), tapped);
            assert_eq!(t.g.rule_effects.len(), 1, "the Minotaur's effect");
        }
        _ => panic!("{card}"),
    }
    c
}

#[test]
fn freezing_tracks_the_creature_not_its_controller() {
    cr!("502.3", "611.2a");
    ruling!(
        "Elvish Hunter",
        "Elvish Hunter doesn’t track the creature’s controller. If the affected creature changes controllers before its old controller’s next untap step, Elvish Hunter will prevent it from being untapped during its new controller’s next untap step."
    );
    ruling!(
        "Permafrost Trap",
        "If an affected creature changes controllers before its previous controller's next untap step, this ability will prevent it from untapping during its new controller's next untap step."
    );
    ruling!(
        "Labyrinth Minotaur",
        "Labyrinth Minotaur doesn’t track the creature’s controller. If the affected creature changes controllers before its old controller’s next untap step, this ability will prevent it from being untapped during its new controller’s next untap step."
    );
    ruling!(
        "Guardian of Tazeem",
        "The ability tracks the creature, but not its controller. If the creature changes controllers before its first controller's next untap step has come around, then it won't untap during its new controller's next untap step."
    );
    for card in [
        "Elvish Hunter",
        "Permafrost Trap",
        "Labyrinth Minotaur",
        "Guardian of Tazeem",
    ] {
        let mut t = TestGame::new(2);
        let bears = frozen_by(&mut t, card, "Grizzly Bears", true);
        assert!(is_tapped(&t, bears), "{card}");
        // P0 gains control of them before P1's next untap step: they don't untap during
        // P0's next untap step, and untap in the one after it.
        give_control(&mut t, bears, P0);
        if t.g.turn.active == P0 {
            through_untap_step(&mut t, P1);
            assert!(is_tapped(&t, bears), "{card}");
        }
        through_untap_step(&mut t, P0);
        assert!(
            is_tapped(&t, bears),
            "{card}: untapped in its new controller's step"
        );
        through_untap_step(&mut t, P0);
        assert!(!is_tapped(&t, bears), "{card}");
    }
}

#[test]
fn freezing_an_untapped_creature_is_used_up_by_that_untap_step() {
    cr!("502.3", "611.2a");
    ruling!(
        "Barl's Cage",
        "If the targeted creature is untapped at the time its controller’s next untap step begins, this ability has no effect. It won’t apply at some later time when the targeted creature is tapped."
    );
    ruling!(
        "Elvish Hunter",
        "If the targeted creature is untapped at the time its controller’s next untap step begins, this ability has no effect. It won’t apply at some later time when the targeted creature is tapped."
    );
    ruling!(
        "Labyrinth Minotaur",
        "If the creature Labyrinth Minotaur blocked is untapped at the time its controller’s next untap step begins, this ability has no effect. It won’t apply at some later time when that creature is tapped."
    );
    ruling!(
        "Permafrost Trap",
        "If a targeted creature is untapped at the time its controller's next untap step begins, the second part of the ability has no effect on it."
    );
    for (card, name) in [
        ("Barl's Cage", "Grizzly Bears"),
        ("Elvish Hunter", "Grizzly Bears"),
        // Steadfast Sentinel has vigilance: it's untapped after attacking.
        ("Labyrinth Minotaur", "Steadfast Sentinel"),
        ("Permafrost Trap", "Grizzly Bears"),
    ] {
        let mut t = TestGame::new(2);
        let c = frozen_by(&mut t, card, name, false);
        if card == "Permafrost Trap" {
            // Untapped again before P1's untap step.
            t.g.untap(c);
        }
        assert!(!is_tapped(&t, c), "{card}");
        through_untap_step(&mut t, P1);
        // Tapped after that untap step: it untaps during the next one.
        tap(&mut t, c);
        untaps_next(&mut t, c, P1);
    }
}

#[test]
fn blinding_beam_affects_every_creature_during_that_players_untap_step() {
    cr!("502.3", "611.2c");
    ruling!(
        "Blinding Beam",
        "The second mode affects all creatures during the player’s next untap step, including creatures controlled by other players (who may have untapped because of Seedborn Muse, for example) and creatures that weren’t on the battlefield when Blinding Beam resolved."
    );
    supported("Blinding Beam");
    supported("Seedborn Muse");
    // Three players: P0 targets P1 on P0's turn. P2 controls Seedborn Muse ("Untap all
    // permanents you control during each other player's untap step.").
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let p1s = tapped_creature(&mut t, P1, "Grizzly Bears");
    t.battlefield(P2, "Seedborn Muse");
    let p2s = tapped_creature(&mut t, P2, "Hill Giant");
    let p2_land = t.lands(P2, "Forest", 1)[0];
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_new(&mut t, P0, "Blinding Beam", &[pl(P1)]);
    t.resolve_all();
    // A creature entering later.
    let later = tapped_creature(&mut t, P1, "Craw Wurm");
    tap(&mut t, p2_land);
    t.advance_to(P1, Step::Upkeep);
    assert!(is_tapped(&t, p1s) && is_tapped(&t, later));
    assert!(
        is_tapped(&t, p2s),
        "P2's creature doesn't untap in P1's untap step"
    );
    assert!(!is_tapped(&t, p2_land), "only creatures");
    // P2's own untap step is unaffected; so is P1's next one.
    t.advance_to(P2, Step::Upkeep);
    assert!(!is_tapped(&t, p2s));
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_tapped(&t, p1s) && !is_tapped(&t, later));
}

#[test]
fn blinding_beam_with_shared_team_turns_affects_the_whole_teams_untap_step() {
    cr!("502.3", "805.4");
    ruling!(
        "Blinding Beam",
        "The second mode affects all creatures during the player’s next untap step"
    );
    // Two-Headed Giant: P0 and P1 against P2 and P3, P2 taking their team's turns. The
    // team's untap step is the untap step of both its players: whichever of them is
    // targeted, neither player's creatures untap during it, and both untap in the next one.
    for target in [P2, P3] {
        let mut t = TestGame::with_config(
            4,
            GameConfig {
                variant: Variant::TwoHeadedGiant,
                teams: Some(vec![0, 0, 1, 1]),
                ..Default::default()
            },
        );
        t.set_step(P0, Step::PrecombatMain);
        let c2 = tapped_creature(&mut t, P2, "Grizzly Bears");
        let c3 = tapped_creature(&mut t, P3, "Hill Giant");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        cast_new(&mut t, P0, "Blinding Beam", &[pl(target)]);
        t.resolve_all();
        t.advance_to(P2, Step::Upkeep);
        assert!(is_tapped(&t, c2) && is_tapped(&t, c3), "target {target:?}");
        t.advance_to(P0, Step::Upkeep);
        t.advance_to(P2, Step::Upkeep);
        assert!(
            !is_tapped(&t, c2) && !is_tapped(&t, c3),
            "target {target:?}"
        );
    }
}

#[test]
fn blinding_beam_waits_for_an_untap_step_that_isnt_skipped() {
    cr!("502.3", "614.10a");
    ruling!(
        "Blinding Beam",
        "The second mode affects all creatures during the player’s next untap step"
    );
    supported("Avizoa");
    // P1 activates Avizoa ("You skip your next untap step."), then P0's Blinding Beam
    // targets P1: P1's next untap step is skipped, so the Beam applies in the one after it,
    // the first that isn't skipped, and P1's creatures untap in the one after that.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    let avizoa = t.battlefield(P1, "Avizoa");
    t.activate(P1, avizoa, 0, &[]).expect("Avizoa");
    t.resolve_all();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_new(&mut t, P0, "Blinding Beam", &[pl(P1)]);
    t.resolve_all();
    tap(&mut t, avizoa);
    // The skipped untap step.
    t.advance_to(P1, Step::Upkeep);
    assert!(is_tapped(&t, bears) && is_tapped(&t, avizoa));
    // The next one: the Beam applies.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(
        is_tapped(&t, bears) && is_tapped(&t, avizoa),
        "the Beam waited"
    );
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_tapped(&t, bears) && !is_tapped(&t, avizoa));
}

// ---------------------------------------------------------------------------------------
// Stun counters
// ---------------------------------------------------------------------------------------

#[test]
fn suppression_ray_stuns_only_creatures_it_tapped() {
    cr!("701.26a", "608.2c", "122.1d");
    ruling!(
        "Suppression Ray // Orderly Plaza",
        "\"Creatures tapped this way\" means creatures that became tapped as a result of Suppression Ray's effect, not creatures that were already tapped before Suppression Ray began to resolve. For example, if a player controls two tapped creatures and one untapped creature and you cast Suppression Ray targeting that player, you'll only be able to choose the creature that was untapped to get a stun counter (as long as you paid at least one {E})."
    );
    supported("Suppression Ray // Orderly Plaza");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 5);
    t.g.add_counters(Entity::Player(P0), "energy", 3, None);
    let a = tapped_creature(&mut t, P1, "Grizzly Bears");
    let b = tapped_creature(&mut t, P1, "Hill Giant");
    let c = t.battlefield(P1, "Craw Wurm");
    let ray = t.hand(P0, "Suppression Ray // Orderly Plaza");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_choose(P0, &[obj(c)]);
    let from = t.asked().len();
    t.cast(P0, ray).target(Entity::Player(P1)).go();
    t.resolve_all();
    let offered: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(
        offered,
        vec![vec![obj(c)]],
        "only the creature tapped this way"
    );
    assert_eq!(t.counters(c, STUN), 1);
    assert_eq!(t.counters(a, STUN) + t.counters(b, STUN), 0);
    // In P1's next untap step the stunned Wurm stays tapped, losing its counter; the
    // others untap.
    through_untap_step(&mut t, P1);
    assert!(is_tapped(&t, c));
    assert_eq!(t.counters(c, STUN), 0);
    assert!(!is_tapped(&t, a) && !is_tapped(&t, b));
}

#[test]
fn fear_of_sleep_paralysis_stops_stun_counters_moving_off_opponents_permanents() {
    cr!("122.5", "101.2");
    ruling!(
        "Fear of Sleep Paralysis",
        "If an ability tries to move a stun counter from a permanent an opponent controls to another permanent while you control Fear of Sleep Paralysis, that counter won't move. No stun counters will be removed from or put onto anything."
    );
    supported("Fear of Sleep Paralysis");
    supported("Nesting Grounds");
    // P1 moves the stun counter on their Bears onto their Hill Giant with Nesting Grounds
    // ("{1}, {T}: Move a counter from target permanent you control onto a second target
    // permanent."), while P0 controls Fear of Sleep Paralysis.
    for fear in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        if fear {
            t.battlefield(P0, "Fear of Sleep Paralysis");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        crate::r_s29_common::put_counters(&mut t, bears, STUN, 1);
        let grounds = t.battlefield(P1, "Nesting Grounds");
        t.lands(P1, "Wastes", 1);
        t.activate(P1, grounds, 1, &[obj(bears), obj(giant)])
            .expect("Nesting Grounds");
        t.resolve_all();
        let moved = (t.counters(bears, STUN), t.counters(giant, STUN));
        assert_eq!(moved, if fear { (1, 0) } else { (0, 1) }, "fear {fear}");
    }
}

#[test]
fn aerith_rescue_mission_chooses_the_stunned_creature_as_it_resolves() {
    cr!("608.2c", "601.2c");
    ruling!(
        "Aerith Rescue Mission",
        "If you choose the second mode, you don't have to choose which of the target creatures you'll put a stun counter on until Aerith Rescue Mission is resolving."
    );
    supported("Aerith Rescue Mission");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    let from = t.asked().len();
    // "Up to three target creatures": one target slot.
    t.answer_targets(P0, &[obj(a), obj(b)]);
    cast_new(&mut t, P0, "Aerith Rescue Mission", &[]);
    let while_casting = asked_since(&t, from).len();
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    t.answer_choose(P0, &[obj(b)]);
    t.resolve_all();
    let resolving = &asked_since(&t, from)[while_casting..];
    assert!(resolving.iter().any(|(p, d)| *p == P0
        && matches!(d, Decision::ChooseEntities { candidates, .. }
            if candidates.contains(&obj(a)) && candidates.contains(&obj(b)))));
    assert!(is_tapped(&t, a) && is_tapped(&t, b));
    assert_eq!((t.counters(a, STUN), t.counters(b, STUN)), (0, 1));
}

#[test]
fn wicked_slumber_puts_both_counters_on_the_remaining_target() {
    cr!("608.2b");
    ruling!(
        "Wicked Slumber",
        "If you choose two target creatures, but one of them is an illegal target as Wicked Slumber resolves, the illegal target will be unaffected. You'll put both stun counters on the remaining legal target. If both targets become illegal, Wicked Slumber won't resolve at all."
    );
    supported("Wicked Slumber");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    // "Up to two target creatures": one target slot. The Giant leaves before it resolves.
    t.answer_targets(P0, &[obj(a), obj(b)]);
    cast_new(&mut t, P0, "Wicked Slumber", &[]);
    let b2 = crate::r_s05_common::move_to(&mut t, b, Zone::Hand(P1)).unwrap();
    t.resolve_all();
    assert!(is_tapped(&t, a));
    assert_eq!(t.counters(a, STUN), 2);
    assert_eq!(t.zone(b2), Zone::Hand(P1));
    // Both legal: the counters may be split.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    cast_new(&mut t, P0, "Wicked Slumber", &[]);
    t.resolve_all();
    assert_eq!((t.counters(a, STUN), t.counters(b, STUN)), (1, 1));
    // Both illegal: it doesn't resolve; it's put into the graveyard (CR 608.2b).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    let spell = cast_new(&mut t, P0, "Wicked Slumber", &[]);
    crate::r_s05_common::move_to(&mut t, a, Zone::Hand(P1));
    crate::r_s05_common::move_to(&mut t, b, Zone::Hand(P1));
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert!(!t.g.turn_events[from..]
        .iter()
        .any(|e| matches!(e, Event::SpellResolved { spell: s } if *s == spell)));
    assert!(t.in_graveyard(P0, "Wicked Slumber"));
}

// ---------------------------------------------------------------------------------------
// Dragon Turtle: an illegal target, and no target
// ---------------------------------------------------------------------------------------

#[test]
fn dragon_turtle_with_an_illegal_target_does_nothing() {
    cr!("608.2b", "603.3d", "115.1");
    ruling!(
        "Dragon Turtle",
        "If you target an opponent's creature with Dragon Turtle's Drag Below ability and that target is illegal as the ability tries to resolve, the entire ability is removed from the stack and does nothing. Dragon Turtle will remain untapped and will untap as normal during its controller's next untap step."
    );
    ruling!(
        "Dragon Turtle",
        "If, however, there are no legal targets for the Drag Below ability when Dragon Turtle enters the battlefield (or you choose not to target anything), Dragon Turtle's ability will still resolve and cause it to be tapped and it won't untap during its controllers next untap step."
    );
    supported("Dragon Turtle");
    // The target leaves before the ability resolves.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let turtle = t.enter(P0, "Dragon Turtle");
    t.g.flush_events();
    t.settle();
    crate::r_s05_common::move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert!(!is_tapped(&t, turtle));
    // No opponent's creature, or no target chosen though there is one: it taps itself and
    // stays tapped through P0's next untap step.
    for opponents_creature in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = opponents_creature.then(|| t.battlefield(P1, "Grizzly Bears"));
        t.answer_targets(P0, &[]);
        let turtle = t.enter(P0, "Dragon Turtle");
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        assert!(is_tapped(&t, turtle), "{opponents_creature}");
        if let Some(b) = bears {
            assert!(!is_tapped(&t, b));
        }
        misses_one_untap(&mut t, turtle, P0);
    }
}

// ---------------------------------------------------------------------------------------
// Koma, Cosmos Serpent
// ---------------------------------------------------------------------------------------

/// P0 controls Koma and a Koma's Coil token (made at P0's upkeep). Returns (Koma, the
/// Coil), in P0's precombat main phase.
fn koma_and_coil(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Koma, Cosmos Serpent");
    let koma = t.battlefield(P0, "Koma, Cosmos Serpent");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    // ("At the beginning of each upkeep": P1's made one too.)
    let coil = t.named_on_battlefield("Koma's Coil");
    assert!(!coil.is_empty());
    (koma, coil[0])
}

#[test]
fn komas_sacrifice_can_be_any_other_serpent() {
    cr!("602.2", "118.3");
    ruling!(
        "Koma, Cosmos Serpent",
        "You can't sacrifice Koma itself to activate its last ability, but you can sacrifice any other Serpent you control, including Koma's Coil."
    );
    let mut t = TestGame::new(2);
    let (koma, coil) = koma_and_coil(&mut t);
    let serpent = t.battlefield(P0, "Serpent of Yawning Depths");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_choose(P0, &[obj(coil)]);
    t.activate(P0, koma, 0, &[]).expect("Koma");
    let offered: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert!(!offered.is_empty());
    assert!(offered[0].contains(&obj(coil)) && offered[0].contains(&obj(serpent)));
    assert!(!offered[0].contains(&obj(koma)));
    t.resolve_all();
    assert!(!t.on_battlefield(coil));
    // With only Koma, it can't be activated.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let koma = t.battlefield(P0, "Koma, Cosmos Serpent");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    assert!(t.activate(P0, koma, 0, &[]).is_err());
    assert!(t.on_battlefield(koma));
}

#[test]
fn komas_tap_mode_stops_activated_abilities_even_of_a_tapped_permanent() {
    cr!("602.5", "605.1a", "604.1", "603.2");
    ruling!(
        "Koma, Cosmos Serpent",
        "If you choose the first mode of Koma's activated ability, you can choose a permanent that's already tapped as the target. The effect stopping its activated abilities will still apply."
    );
    ruling!(
        "Koma, Cosmos Serpent",
        "The first mode of Koma's activated ability doesn't stop static abilities from affecting the game, and it doesn't stop triggered abilities from triggering. Unlike similar effects, it does stop mana abilities from being activated. Most activated abilities that produce mana as they resolve are mana abilities."
    );
    supported("Ashnod's Altar");
    supported("Soul Warden");
    // P1's tapped Ashnod's Altar ("Sacrifice a creature: Add {C}{C}." — a mana ability
    // with no {T}); P1's Glorious Anthem and Soul Warden are also tapped by Koma.
    let mut outcomes = vec![];
    for target in ["Ashnod's Altar", "Glorious Anthem", "Soul Warden"] {
        let mut t = TestGame::new(2);
        let (koma, coil) = koma_and_coil(&mut t);
        let perm = t.battlefield(P1, target);
        tap(&mut t, perm);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
        t.answer_choose(P0, &[obj(coil)]);
        t.activate(P0, koma, 0, &[obj(perm)]).expect("Koma");
        t.resolve_all();
        assert!(is_tapped(&t, perm));
        match target {
            "Ashnod's Altar" => {
                let r = t.activate(P1, perm, 0, &[]);
                assert!(r.is_err(), "the mana ability can't be activated: {r:?}");
                assert!(t.on_battlefield(bears));
            }
            "Glorious Anthem" => assert_eq!(t.pt(bears), (3, 3)),
            _ => {
                let life = t.life(P1);
                t.enter(P1, "Hill Giant");
                t.g.flush_events();
                t.settle();
                t.resolve_all();
                assert_eq!(t.life(P1), life + 1, "Soul Warden still triggers");
            }
        }
        outcomes.push(target);
    }
    assert_eq!(outcomes.len(), 3);
}

// ---------------------------------------------------------------------------------------
// Control changes, costs, and casting without paying
// ---------------------------------------------------------------------------------------

#[test]
fn jinxed_idol_control_change_lasts_and_can_go_back() {
    cr!("611.2a", "613.1b", "602.2");
    ruling!(
        "Jinxed Idol",
        "The control-change effect has no duration. The targeted player retains control of Jinxed Idol until the game ends, Jinxed Idol leaves the battlefield, or an effect causes someone else to gain control of it (perhaps because Jinxed Idol’s ability is activated again)."
    );
    ruling!(
        "Jinxed Idol",
        "Your opponent can use the ability to give this card back to you."
    );
    supported("Jinxed Idol");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let idol = t.battlefield(P0, "Jinxed Idol");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(mine)]);
    t.activate(P0, idol, 0, &[pl(P1)]).expect("Jinxed Idol");
    t.resolve_all();
    assert_eq!(t.obj_now(idol).controller, P1);
    // Through a full round of turns: P1's upkeep, 2 damage to P1.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(idol).controller, P1);
    assert_eq!(t.life(P0), 20);
    // P1 gives it back.
    t.advance_to(P1, Step::PrecombatMain);
    t.answer_choose(P1, &[obj(theirs)]);
    t.activate(P1, idol, 0, &[pl(P0)]).expect("Jinxed Idol");
    t.resolve_all();
    assert_eq!(t.obj_now(idol).controller, P0);
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn zaffai_casts_an_x_spell_free_with_x_zero() {
    cr!("107.3b", "118.9", "601.2b");
    ruling!(
        "Zaffai and the Tempests",
        "If you cast a spell with {X} in its mana cost this way, you must choose 0 as the value of X."
    );
    supported("Zaffai and the Tempests");
    // Blaze ({X}{R}: X damage to any target), cast free with X = 5 asked: X is 0.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Zaffai and the Tempests");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blaze = t.hand(P0, "Blaze");
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[obj(bears)]);
    let spell = t
        .cast(P0, blaze)
        .method(CastMethod::Free)
        .try_go()
        .expect("free cast");
    assert_eq!(crate::r_s25_common::x_of(&t, spell), Some(0));
    t.resolve_all();
    assert_eq!(dmg(&t, bears), 0);
}

#[test]
fn champions_of_the_shoal_countered_leaves_the_beheld_card_in_exile() {
    cr!("701.4a", "601.2h", "701.6a");
    ruling!(
        "Champions of the Shoal",
        "If Champions of the Shoal is countered or otherwise fails to resolve, the beheld card will remain in exile indefinitely."
    );
    supported("Champions of the Shoal");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let merfolk = t.hand(P0, "Merfolk of the Pearl Trident");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[obj(merfolk)]);
    cast_new(&mut t, P0, "Champions of the Shoal", &[]);
    assert_eq!(t.zone(merfolk), Zone::Exile);
    let champions = t.g.stack.last().copied().unwrap();
    cast_new(&mut t, P1, "Cancel", &[obj(champions)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Champions of the Shoal"));
    // Turns later, it's still in exile.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.in_exile("Merfolk of the Pearl Trident"));
}

#[test]
fn flamewar_exiles_face_down_cards_no_one_may_look_at() {
    cr!("406.3", "702.161a");
    ruling!(
        "Flamewar, Brash Veteran // Flamewar, Streetwise Operative",
        "You don't get to look at the cards exiled by Flamewar, Streetwise Operative's triggered ability."
    );
    supported("Flamewar, Brash Veteran // Flamewar, Streetwise Operative");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let fw = crate::r_p079_common::back_face(
        &mut t,
        P0,
        "Flamewar, Brash Veteran // Flamewar, Streetwise Operative",
    );
    t.g.recompute();
    stack_library(&mut t, P0, &["Shock", "Island", "Hill Giant"]);
    attack_with(&mut t, &[(fw, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let exiled: Vec<ObjectId> =
        t.g.objects
            .iter()
            .filter(|o| o.zone == Zone::Exile && o.owner == P0 && o.counters.contains_key("intel"))
            .map(|o| o.id)
            .collect();
    assert_eq!(exiled.len(), 2);
    for e in exiled {
        assert!(t.obj_now(e).face_down);
        assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, e));
        assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, e));
    }
}

// ---------------------------------------------------------------------------------------
// Kaito, Bane of Nightmares; Sharae of Numbing Depths; Venarian Gold
// ---------------------------------------------------------------------------------------

#[test]
fn kaito_counts_opponents_who_lost_life_whatever_they_gained() {
    cr!("119.3", "606.3");
    ruling!(
        "Kaito, Bane of Nightmares",
        "Kaito's fourth ability cares whether opponents lost life this turn, not how their life totals changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life that turn."
    );
    supported("Kaito, Bane of Nightmares");
    // P1 gained 2 life and lost 1 this turn; P2 neither.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let k = pw(&mut t, P0, "Kaito, Bane of Nightmares", 4);
    crate::r_p009_common::gain(&mut t, P1, 2);
    crate::r_p009_common::lose(&mut t, P1, 1);
    assert_eq!(t.life(P1), 21);
    let hand = t.hand_size(P0);
    crate::r_s06_common::activate_containing(&mut t, P0, k, "Surveil").expect("0");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "P1 lost life; P2 didn't");
}

#[test]
fn kaito_as_a_creature_is_dealt_damage_not_loyalty_loss() {
    cr!("306.8", "120.3c", "120.3e", "606.3");
    ruling!(
        "Kaito, Bane of Nightmares",
        "Kaito's second ability causes him to stop being a planeswalker during your turn as long as he has at least one loyalty counter on him. While he's a creature, damage dealt to him won't cause loyalty counters to be removed. You can still activate Kaito's loyalty abilities."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let k = pw(&mut t, P0, "Kaito, Bane of Nightmares", 4);
    t.g.recompute();
    assert!(t.obj_now(k).is_creature());
    assert!(!t
        .obj_now(k)
        .chars
        .is(mtg_engine::types::CardType::Planeswalker));
    let src = t.battlefield(P1, "Grizzly Bears");
    hit(&mut t, src, obj(k), 2);
    assert_eq!(t.counters(k, "loyalty"), 4);
    assert_eq!(t.obj_now(k).damage, 2);
    crate::r_s06_common::activate_containing(&mut t, P0, k, "emblem").expect("+1");
    t.resolve_all();
    assert_eq!(t.counters(k, "loyalty"), 5);
}

#[test]
fn sharae_doesnt_trigger_when_an_opponent_taps_their_own_creature() {
    cr!("603.2", "701.26a");
    ruling!(
        "Sharae of Numbing Depths",
        "Sharae of Numbing Depths's last ability will trigger only when an effect instructs you to tap one or more creatures your opponents control. It won't trigger if a spell or ability you control instructs an opponent to tap a creature they control. For example, if you control Tangle Wire and an opponent taps an untapped creature they control as part of the resolution of Tangle Wire's triggered ability, Sharae's ability won't trigger."
    );
    supported("Sharae of Numbing Depths");
    supported("Tangle Wire");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Sharae of Numbing Depths");
    let wire = t.battlefield(P0, "Tangle Wire");
    t.g.objects[wire.0 as usize]
        .counters
        .insert("fade".into(), 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert!(is_tapped(&t, bears), "P1 tapped it for Tangle Wire");
    assert_eq!(t.hand_size(P0), hand, "no card for P0");
}

#[test]
fn venarian_golds_counters_stay_where_they_were_put() {
    cr!("122.1", "303.4", "603.2");
    ruling!(
        "Venarian Gold",
        "The counters are placed on the creature to which it enters attached. If you later move Venarian Gold to a different creature, the counters remain on the original creature and no counters are put on the new creature."
    );
    supported("Venarian Gold");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 2);
    let gold = t.hand(P0, "Venarian Gold");
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, gold).x(2).go();
    t.resolve_all();
    let gold = t.named_on_battlefield("Venarian Gold")[0];
    assert!(is_tapped(&t, bears));
    assert_eq!(t.counters(bears, "sleep"), 2);
    assert!(t.g.attach(gold, obj(giant)));
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(bears, "sleep"), 2);
    assert_eq!(t.counters(giant, "sleep"), 0);
}

#[test]
fn the_caves_of_androzani_finds_a_doctor_by_type_not_name() {
    cr!("701.23a", "205.3m", "714.2b");
    ruling!(
        "The Caves of Androzani",
        "The last ability of the Caves of Androzani can find any card with the Doctor creature type, not cards with the word \"Doctor\" in the name."
    );
    supported("The Caves of Androzani");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let caves = t.battlefield(P0, "The Caves of Androzani");
    t.g.objects[caves.0 as usize]
        .counters
        .insert("lore".into(), 3);
    stack_library(
        &mut t,
        P0,
        &["Trait Doctoring", "The Thirteenth Doctor", "Shock"],
    );
    let doctor = t.g.player(P0).library[t.g.player(P0).library.len() - 2];
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(doctor)]);
    crate::r_s29_common::put_counters(&mut t, caves, "lore", 1);
    t.resolve_all();
    let offered: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(
        offered,
        vec![vec![obj(doctor)]],
        "only the Doctor creature card"
    );
    assert!(t.in_hand(P0, "The Thirteenth Doctor"));
}

#[test]
fn caribou_range_sacrifices_only_a_caribou_token() {
    cr!("111.1", "205.3m", "602.2");
    ruling!(
        "Caribou Range",
        "You can only sacrifice a Caribou token to this, not any creature with the type Caribou."
    );
    supported("Caribou Range");
    supported("Changeling Outcast");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let land = t.lands(P0, "Plains", 1)[0];
    let range = crate::r_s06_common::attach_new(&mut t, P0, "Caribou Range", obj(land));
    // A Changeling Outcast is a Caribou, but not a token.
    let outcast = t.battlefield(P0, "Changeling Outcast");
    assert!(t.activate(P0, range, 0, &[]).is_err());
    assert!(t.on_battlefield(outcast));
    // A Caribou token made by the enchanted land.
    t.lands(P0, "Plains", 2);
    crate::r_s06_common::activate_containing(&mut t, P0, land, "Caribou").expect("make a Caribou");
    t.resolve_all();
    let caribou = t.named_on_battlefield("Caribou Token");
    assert_eq!(caribou.len(), 1, "{}", t.dump_log());
    t.answer_choose(P0, &[obj(caribou[0])]);
    t.activate(P0, range, 0, &[]).expect("sacrifice the token");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert!(t.on_battlefield(outcast));
}
