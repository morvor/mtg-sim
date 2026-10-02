//! CR 807.4-807.5 with several turns at once: each turn has its own cleanup step and its
//! own "this turn" (CR 514.2), its own mana (CR 500.5), extra turns keep what happens as
//! they begin and can be skipped (CR 807.4i-j, 614.10), and a player with priority for
//! several stacks chooses the stack for their spells (CR 807.5b).

use super::r800_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::mana::ManaType;
use mtg_engine::multiplayer::grand_melee;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// A Grand Melee game of `n` players in P0's first main phase, with the markers handed
/// out (markers at P0 and P4 for ten players).
fn gm(n: usize) -> TestGame {
    let mut t = TestGame::with_config(
        n,
        GameConfig {
            starting_player: Some(P0),
            ..GameConfig::grand_melee()
        },
    );
    grand_melee::ensure(&mut t.g);
    t
}

fn holds_marker(g: &mtg_engine::Game, p: PlayerId) -> bool {
    grand_melee::markers(g).iter().any(|m| m.holder == p)
}

#[test]
fn until_end_of_turn_effects_end_with_the_turn_that_created_them() {
    cr!("807.4", "514.2");
    let mut t = gm(10);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P4, "Grizzly Bears");
    // P0 pumps their creature during their turn (the first marker's).
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(mine).go();
    t.resolve();
    // P4 pumps theirs during their turn (the second marker's), which is made longer.
    grand_melee::switch_to(&mut t.g, 1);
    t.g.turn.step = Step::PrecombatMain;
    t.g.add_extra_combat(true);
    t.g.add_extra_combat(true);
    t.g.add_extra_combat(true);
    t.lands(P4, "Forest", 1);
    let gg = t.hand(P4, "Giant Growth");
    t.cast(P4, gg).target(theirs).go();
    t.resolve();
    assert_eq!(t.pt(theirs), (5, 5));
    grand_melee::switch_to(&mut t.g, 0);
    assert_eq!(t.pt(mine), (5, 5));
    // P0's turn ends first: only P0's pump ends.
    assert!(t
        .g
        .run_until(40_000, |g| !holds_marker(g, P0) && holds_marker(g, P4)));
    t.g.recompute();
    assert_eq!(t.pt(mine), (2, 2));
    assert_eq!(t.pt(theirs), (5, 5), "P4's turn isn't over");
    // P4's turn ends: now theirs ends.
    assert!(t.g.run_until(40_000, |g| !holds_marker(g, P4)));
    t.g.recompute();
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn mana_empties_as_the_steps_of_the_turn_it_was_added_during_end() {
    cr!("807.4", "500.5");
    let mut t = gm(10);
    // P5 adds mana during P4's turn (the second marker's)...
    grand_melee::switch_to(&mut t.g, 1);
    t.g.players[5].mana_pool.add_type(ManaType::R, 1);
    // ...and P1 during P0's.
    grand_melee::switch_to(&mut t.g, 0);
    t.g.players[1].mana_pool.add_type(ManaType::G, 1);
    // A step of P0's turn ends: only P1's mana empties.
    mtg_engine::mana_abilities::empty_pool(&mut t.g, P1);
    mtg_engine::mana_abilities::empty_pool(&mut t.g, P5);
    assert!(t.g.player(P1).mana_pool.mana.is_empty());
    assert_eq!(t.g.player(P5).mana_pool.mana.len(), 1);
    // A step of P4's turn ends: P5's mana empties.
    grand_melee::switch_to(&mut t.g, 1);
    mtg_engine::mana_abilities::empty_pool(&mut t.g, P5);
    assert!(t.g.player(P5).mana_pool.mana.is_empty());
}

#[test]
fn this_turn_delayed_triggers_last_for_the_turn_that_created_them() {
    cr!("807.4", "603.7b");
    let mut t = gm(10);
    // During P4's turn: "Whenever a creature dies this turn, you gain 1 life."
    grand_melee::switch_to(&mut t.g, 1);
    apply(
        &mut t,
        P4,
        Effect::DelayedTrigger {
            trigger: TriggerCond::Dies(Filter::Type(CardType::Creature)),
            body: Box::new(Body::effect(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::c(1),
            })),
            once: false,
        },
        &[],
    );
    // A creature of P5 (within P4's range) dies while P0's turn is being played: P4's
    // turn is still going on, so it triggers.
    grand_melee::switch_to(&mut t.g, 0);
    let bears = t.battlefield(P5, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(t.g.delayed_triggers.len(), 1);
    grand_melee::switch_to(&mut t.g, 1);
    t.resolve_all();
    assert_eq!(t.life(P4), 21);
}

#[test]
fn an_extra_turn_kept_with_the_marker_keeps_what_happens_as_it_begins() {
    cr!("807.4i", "500.7");
    // Twelve players (markers at P0, P4, P8). Last Chance: "Take an extra turn after this
    // one. At the beginning of that turn's end step, you lose the game." P0 keeps the
    // marker for the extra turn, and loses at its end step.
    let mut t = gm(12);
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Mountain", 2);
    let lc = t.hand(P0, "Last Chance");
    t.cast(P0, lc).go();
    t.resolve();
    let lost = t.g.run_until(60_000, |g| !g.player(P0).in_game());
    assert!(lost, "P0 never lost");
    // In the end step of P0's extra turn.
    grand_melee::switch_to(&mut t.g, 0);
    assert_eq!(t.g.turn.active, P0);
    assert!(t.g.turn.extra);
    assert_eq!(t.g.turn.step, Step::End);
}

#[test]
fn an_extra_turn_waiting_for_the_next_turn_keeps_what_happens_as_it_begins() {
    cr!("807.4j", "500.7");
    // Last Chance's effect for P1 (no marker) during P0's turn (as if copied for P1): P1
    // takes the extra turn immediately before their next turn, and loses at its end step.
    let mut t = gm(10);
    let effect = mtg_engine::card::card("Last Chance").faces[0]
        .chars
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Spell(s) => Some(s.body.effect.clone()),
            _ => None,
        })
        .unwrap();
    assert!(matches!(effect, Effect::ExtraTurnWith { .. }), "{effect:?}");
    apply(&mut t, P1, effect, &[]);
    for _ in 0..5 {
        t.library_top(P1, "Island");
    }
    // (turn number, extra, latest step) of P1's turns.
    let mut p1_turns: Vec<(u32, bool, Step)> = Vec::new();
    let lost = t.g.run_until(80_000, |g| {
        if g.turn.active == P1 {
            let n = g.turn.number;
            match p1_turns.last_mut() {
                Some(last) if last.0 == n => last.2 = g.turn.step,
                _ => p1_turns.push((n, g.turn.extra, g.turn.step)),
            }
        }
        !g.player(P1).in_game()
    });
    assert!(lost, "P1 never lost");
    // During their extra turn, in its end step.
    assert_eq!(p1_turns.len(), 1, "{p1_turns:?}");
    assert!(p1_turns[0].1);
    assert_eq!(p1_turns[0].2, Step::End);
}

#[test]
fn a_marker_holders_extra_turn_can_be_skipped() {
    cr!("807.4i", "614.10");
    // Stranglehold (P1's): "If an opponent would begin an extra turn, that player skips
    // that turn instead." P0's Time Warp extra turn is skipped: the marker passes to P1.
    let mut t = gm(12);
    t.battlefield(P1, "Stranglehold");
    t.lands(P0, "Island", 5);
    let warp = t.hand(P0, "Time Warp");
    t.cast(P0, warp).target(P0).go();
    t.resolve();
    let mut p0_extra = false;
    assert!(t.g.run_until(60_000, |g| {
        p0_extra |= g.turn.active == P0 && g.turn.extra;
        holds_marker(g, P1) && g.turn.active == P1
    }));
    assert!(!p0_extra);
}

#[test]
fn an_extra_turn_before_the_next_turn_can_be_skipped() {
    cr!("807.4j", "614.10");
    // Time Warp gives P1 (no marker) an extra turn; P2's Stranglehold skips it, and P1
    // takes their normal turn.
    let mut t = gm(10);
    t.battlefield(P2, "Stranglehold");
    t.lands(P0, "Island", 5);
    let warp = t.hand(P0, "Time Warp");
    t.cast(P0, warp).target(P1).go();
    t.resolve();
    let mut p1_turns: Vec<bool> = Vec::new();
    let mut last = 0;
    assert!(t.g.run_until(60_000, |g| {
        if g.turn.active == P1 && g.turn.number != last {
            last = g.turn.number;
            p1_turns.push(g.turn.extra);
        }
        !p1_turns.is_empty()
    }));
    assert_eq!(p1_turns, vec![false]);
}

/// Ten players; P1 casts a spell on the first marker's stack (P0's turn) and P3 one on
/// the second's (P4's turn), so P2 has priority for both (CR 807.5a). Both turns are at a
/// point where players have priority; P2 has priority on the first marker's turn.
fn two_stacks() -> TestGame {
    let mut t = gm(10);
    let a = t.custom(P1, super::r114_common::free_instant("Quick Thought"), Zone::Hand(P1));
    t.cast(P1, a).go();
    grand_melee::switch_to(&mut t.g, 1);
    t.g.turn.step = Step::PrecombatMain;
    t.g.turn.stage = Stage::Priority;
    let b = t.custom(P3, super::r114_common::free_instant("Slow Thought"), Zone::Hand(P3));
    t.cast(P3, b).go();
    t.g.turn.priority = Some(P3);
    grand_melee::switch_to(&mut t.g, 0);
    t.g.turn.stage = Stage::Priority;
    t.g.turn.priority = Some(P2);
    t
}

fn chose_stack(t: &TestGame) -> bool {
    t.asked().iter().any(|(p, d)| {
        *p == P2 && matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("stack"))
    })
}

#[test]
fn a_player_with_priority_for_several_stacks_chooses_the_stack_for_a_spell() {
    cr!("807.5b", "117.3c");
    let mut t = two_stacks();
    t.lands(P2, "Island", 1);
    let opt = t.hand(P2, "Opt");
    t.answer(P2, DecisionKind::Option, Answer::Index(1));
    t.g.take_action(
        P2,
        Action::Cast {
            card: opt,
            method: CastMethod::Normal,
        },
    );
    assert!(chose_stack(&t));
    let spell = t.g.current(opt);
    assert!(!t.g.stack.contains(&spell), "not on the first marker's stack");
    grand_melee::switch_to(&mut t.g, 1);
    assert_eq!(t.g.stack.last(), Some(&spell));
    // P2 has priority for that stack now.
    assert_eq!(t.g.turn.priority, Some(P2));
    assert_eq!(t.g.turn.passes, 0);
}

/// The target candidates P2 was offered for Counterspell.
fn counterspell_candidates(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, .. } if *p == P2 => Some(candidates.clone()),
            _ => None,
        })
        .next_back()
        .expect("asked for targets")
}

#[test]
fn a_spell_targets_only_objects_on_the_stack_chosen_for_it() {
    cr!("807.5b");
    // The stack is chosen as Counterspell is announced, before its target: on the first
    // marker's stack it can target only Quick Thought, on the second's only Slow Thought.
    for (stack, on) in [(0usize, "Quick Thought"), (1, "Slow Thought")] {
        let mut t = two_stacks();
        let mine = t.g.stack[0];
        grand_melee::switch_to(&mut t.g, 1);
        let other = t.g.stack[0];
        grand_melee::switch_to(&mut t.g, 0);
        t.lands(P2, "Island", 2);
        let cs = t.hand(P2, "Counterspell");
        t.answer(P2, DecisionKind::Option, Answer::Index(stack));
        // P2 tries to target the spell on the first marker's stack either way.
        t.answer_targets(P2, &[Entity::Object(mine)]);
        t.g.take_action(
            P2,
            Action::Cast {
                card: cs,
                method: CastMethod::Normal,
            },
        );
        assert!(chose_stack(&t));
        let target = if stack == 0 { mine } else { other };
        assert_eq!(t.g.obj(target).name(), on);
        assert_eq!(counterspell_candidates(&t), vec![Entity::Object(target)]);
        grand_melee::switch_to(&mut t.g, stack);
        let spell = t.g.current(cs);
        assert_eq!(t.g.stack.last(), Some(&spell));
        // On the second stack that answer isn't legal: it targets Slow Thought.
        let chosen = &t.g.obj(spell).stack.as_ref().unwrap().chosen[0].targets;
        assert_eq!(chosen, &vec![vec![Entity::Object(target)]]);
    }
}

#[test]
fn a_sorcery_speed_spell_is_offered_only_its_own_turns_stack() {
    cr!("807.5b", "307.1");
    // P0, on their own turn with an empty first stack, casts a spell. P0 has priority for
    // the second marker's stack too, being within range of P1's spell on it (CR 807.5a):
    // an instant may go on either stack, a sorcery only on the stack of P0's own turn.
    for (card, asked) in [("Opt", true), ("Lay of the Land", false)] {
        let mut t = gm(10);
        grand_melee::switch_to(&mut t.g, 1);
        t.g.turn.step = Step::PrecombatMain;
        t.g.turn.stage = Stage::Priority;
        let b = t.custom(P1, super::r114_common::free_instant("Slow Thought"), Zone::Hand(P1));
        t.cast(P1, b).go();
        t.g.turn.priority = Some(P4);
        grand_melee::switch_to(&mut t.g, 0);
        t.g.turn.stage = Stage::Priority;
        t.g.turn.priority = Some(P0);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Island", 1);
        let s = t.hand(P0, card);
        t.g.take_action(
            P0,
            Action::Cast {
                card: s,
                method: CastMethod::Normal,
            },
        );
        let was_asked = t.asked().iter().any(|(p, d)| {
            *p == P0 && matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("stack"))
        });
        assert_eq!(was_asked, asked, "{card}");
        assert_eq!(t.g.stack.last(), Some(&t.g.current(s)), "{card}");
    }
}
