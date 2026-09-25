//! CR 117: timing and priority.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn cast_action(card: ObjectId) -> Action {
    Action::Cast {
        card,
        method: CastMethod::Normal,
    }
}

fn priority_log(g: &Game, p: PlayerId, d: &Decision) -> Option<String> {
    matches!(d, Decision::Priority { .. })
        .then(|| format!("{:?} P{} stack={}", g.turn.step, p.0, g.stack.len()))
}

#[test]
fn instants_any_time_with_priority_noninstants_only_in_own_main_phase_with_empty_stack() {
    cr!("117.1a");
    let mut t = TestGame::new(2);
    let quick = t.custom(P0, free_instant("Quick"), Zone::Hand(P0));
    let slow = t.custom(P0, free_sorcery("Slow"), Zone::Hand(P0));
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    // With a spell on the stack, only the instant can be cast.
    t.cast(P0, quick).go();
    assert!(t.cast(P0, slow).try_go().is_err());
    assert!(t.cast(P0, bears).try_go().is_err());
    t.resolve();
    // Main phase, empty stack: noninstant spells can be cast.
    t.cast(P0, slow).go();
    t.resolve();
    t.cast(P0, bears).go();
    t.resolve();
    // During the opponent's turn, P0 (with priority) may cast only instants.
    let quick2 = t.custom(P0, free_instant("Quick 2"), Zone::Hand(P0));
    let slow2 = t.custom(P0, free_sorcery("Slow 2"), Zone::Hand(P0));
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P0, slow2).try_go().is_err());
    t.cast(P0, quick2).go();
    t.resolve();
    // Not during P0's own combat phase either.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.cast(P0, slow2).try_go().is_err());
    // A player without priority can't cast even an instant.
    let quick3 = t.custom(P0, free_instant("Quick 3"), Zone::Hand(P0));
    t.g.turn.priority = Some(P1);
    assert!(t.g.cast_spell(P0, quick3, CastMethod::Normal).is_err());
    assert!(!t.g.legal_actions(P0).contains(&cast_action(quick3)));
}

#[test]
fn activated_abilities_any_time_with_priority() {
    cr!("117.1b");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    // On the opponent's turn, with a spell on the stack.
    t.set_step(P1, Step::DeclareBlockers);
    let quick = t.custom(P1, free_instant("Quick"), Zone::Hand(P1));
    t.cast(P1, quick).go();
    // Without priority the ability can't be activated.
    t.g.turn.priority = Some(P1);
    let uid = activated_uid(&t, pyro, 0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(t.g.activate_ability(P0, pyro, uid).is_err());
    t.clear_answers();
    // With priority it can.
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn some_special_actions_only_in_main_phase_with_empty_stack() {
    cr!("117.1c");
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    let quick = t.custom(P0, free_instant("Quick"), Zone::Hand(P0));
    // Playing a land is a special action taken during the player's main phase while the
    // stack is empty.
    t.cast(P0, quick).go();
    assert!(t.play_land(P0, forest).is_err());
    t.resolve();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.play_land(P0, forest).is_err());
    t.set_step(P0, Step::PostcombatMain);
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(forest));
}

#[test]
fn mana_abilities_with_priority_while_paying_and_when_an_effect_asks_for_mana() {
    cr!("117.1d");
    let mut t = TestGame::new(2);
    // With priority: the mana ability doesn't use the stack.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, elves, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.player(P0).mana_pool.total(), 1);
    // A player without priority can't activate a mana ability when no payment is asked for.
    let p1_elves = t.battlefield(P1, "Llanowar Elves");
    let uid = activated_uid(&t, p1_elves, 0);
    t.g.turn.priority = Some(P0);
    assert!(t.g.activate_ability(P1, p1_elves, uid).is_err());
    // While casting a spell: the lands' mana abilities are activated as the cost is paid.
    let forests = t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert!(t.obj(forests[0]).tapped);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    t.resolve();
    // When an effect asks for a mana payment during resolution: P1 pays for Mana Leak by
    // tapping lands while no player has priority.
    let islands = t.lands(P1, "Island", 3);
    t.lands(P0, "Island", 2);
    let quick = t.custom(P1, free_instant("Quick"), Zone::Hand(P1));
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.cast(P1, quick).go();
    let leak = t.hand(P0, "Mana Leak");
    t.cast(P0, leak).target(spell).go();
    t.answer_yes(P1, true);
    t.resolve();
    assert!(islands.iter().all(|i| t.obj(*i).tapped));
    assert!(t.stack.contains(&spell));
}

#[test]
fn abilities_triggering_during_resolution_wait_until_a_player_would_receive_priority() {
    cr!("117.2a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ajani's Pridemate");
    let spell = CB::new("Gain Then Draw")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::seq(vec![gain(1), draw(1)])))
        .build();
    let s = t.custom(P0, spell, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.g.resolve_top();
    // The spell finished resolving (the card was drawn) before the trigger was put on
    // the stack.
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.stack.is_empty());
    assert_eq!(t.pending_triggers.len(), 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn static_abilities_apply_continuously_without_priority() {
    cr!("117.2b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let bears = t.enter(P0, "Grizzly Bears");
    // Nothing used the stack and nobody received priority.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn turn_based_actions_happen_before_priority_and_at_the_end_of_steps() {
    cr!("117.2c");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    t.g.players[1].mana_pool.add_type(mana::ManaType::R, 1);
    let log = spy(&mut t, P1, |g, p, d| {
        matches!(d, Decision::Priority { .. }).then(|| {
            format!(
                "{:?} hand={} pool={}",
                g.turn.step,
                g.player(p).hand.len(),
                g.player(p).mana_pool.total()
            )
        })
    });
    t.advance_to(P1, Step::PrecombatMain);
    let lines = probe_lines(&log);
    // In the upkeep P1 still has the mana; the step ended (emptying mana pools) with no
    // priority afterward, and the draw step's card draw happened before P1 received
    // priority in that step.
    assert!(lines.contains(&"Upkeep hand=0 pool=1".to_string()));
    assert!(!lines.contains(&"Upkeep hand=0 pool=0".to_string()));
    assert_eq!(
        lines.iter().find(|l| l.starts_with("Draw")).unwrap(),
        "Draw hand=1 pool=0"
    );
}

#[test]
fn state_based_actions_happen_before_a_player_receives_priority() {
    cr!("117.2d");
    let mut t = TestGame::new(2);
    let zero = t.custom(
        P1,
        CB::new("Zero").creature(0, 0).build(),
        Zone::Battlefield,
    );
    let log = spy(&mut t, P0, |g, _p, d| {
        matches!(d, Decision::Priority { .. }).then(|| format!("creatures={}", g.creatures().len()))
    });
    t.g.advance();
    assert_eq!(probe_lines(&log)[0], "creatures=0");
    assert!(!t.on_battlefield(zero));
}

#[test]
fn no_player_has_priority_while_a_spell_resolves() {
    cr!("117.2e");
    let mut t = TestGame::new(2);
    let spell = CB::new("Optional Draw")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(draw(1)),
        }))
        .build();
    let s = t.custom(P0, spell, Zone::Hand(P0));
    let log = spy(&mut t, P0, |g, _p, d| {
        matches!(d, Decision::YesNo { .. }).then(|| format!("priority={:?}", g.turn.priority))
    });
    t.g.take_action(P0, cast_action(s));
    t.g.take_action(P0, Action::Pass);
    t.g.take_action(P1, Action::Pass);
    // The player made a choice during resolution without having priority.
    assert_eq!(probe_lines(&log), vec!["priority=None".to_string()]);
    assert_eq!(t.hand_size(P0), 1);
    // Afterward the active player receives priority.
    assert_eq!(t.turn.priority, Some(P0));
}

#[test]
fn active_player_receives_priority_at_the_beginning_of_steps_after_actions_and_triggers() {
    cr!("117.3a");
    let mut t = TestGame::new(2);
    let upkeep = CB::new("Upkeep Blessing")
        .enchantment()
        .ability(trig(
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            Body::effect(gain(1)),
        ))
        .build();
    t.custom(P1, upkeep, Zone::Battlefield);
    let l0 = spy(&mut t, P0, priority_log);
    let l1 = spy(&mut t, P1, priority_log);
    t.advance_to(P1, Step::Draw);
    let mut all = probe_lines(&l0);
    all.extend(probe_lines(&l1));
    // No player receives priority during the untap step or (normally) the cleanup step.
    assert!(all
        .iter()
        .all(|l| !l.starts_with("Untap") && !l.starts_with("Cleanup")));
    // In P1's upkeep, P1 is the first to receive priority, after its "beginning of
    // upkeep" trigger was put on the stack.
    let first_upkeep = probe_lines(&l1)
        .into_iter()
        .find(|l| l.starts_with("Upkeep"))
        .unwrap();
    assert_eq!(first_upkeep, "Upkeep P1 stack=1");
    // During P0's end step P0 received priority first too.
    assert_eq!(
        probe_lines(&l0)
            .iter()
            .find(|l| l.starts_with("End "))
            .unwrap(),
        "End P0 stack=0"
    );
}

#[test]
fn active_player_receives_priority_after_a_spell_resolves() {
    cr!("117.3b", "117.3c");
    let mut t = TestGame::new(2);
    let quick = t.custom(P1, free_instant("Quick"), Zone::Hand(P1));
    // P0 (active) passes; P1 casts a spell and receives priority again (117.3c).
    t.g.take_action(P0, Action::Pass);
    assert_eq!(t.turn.priority, Some(P1));
    t.g.take_action(P1, cast_action(quick));
    assert_eq!(t.turn.priority, Some(P1));
    assert_eq!(t.stack_len(), 1);
    // Both pass: it resolves and the active player (not its caster) receives priority.
    t.g.take_action(P1, Action::Pass);
    t.g.take_action(P0, Action::Pass);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 21);
    assert_eq!(t.turn.priority, Some(P0));
}

#[test]
fn player_who_takes_a_special_action_receives_priority_again() {
    cr!("117.3c");
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    t.g.take_action(P0, Action::PlayLand { card: forest });
    assert_eq!(t.turn.priority, Some(P0));
    assert!(t.on_battlefield(forest));
}

#[test]
fn passing_gives_priority_to_the_next_player_in_turn_order() {
    cr!("117.3d");
    let mut t = TestGame::new(3);
    t.g.take_action(P0, Action::Pass);
    assert_eq!(t.turn.priority, Some(P1));
    t.g.take_action(P1, Action::Pass);
    assert_eq!(t.turn.priority, Some(P2));
    // The step hasn't ended: not everyone has passed yet... now all three have.
    assert_eq!(t.turn.step, Step::PrecombatMain);
    t.g.take_action(P2, Action::Pass);
    assert_ne!(t.turn.stage, mtg_engine::turn::Stage::Priority);
}

#[test]
fn top_of_stack_resolves_only_when_all_players_pass_in_succession() {
    cr!("117.4");
    let mut t = TestGame::new(2);
    let a = t.custom(P0, free_instant("First"), Zone::Hand(P0));
    let b = t.custom(P1, free_instant("Second"), Zone::Hand(P1));
    t.g.take_action(P0, cast_action(a));
    t.g.take_action(P0, Action::Pass);
    // P1 acts instead of passing: the passes weren't in succession.
    t.g.take_action(P1, cast_action(b));
    t.g.take_action(P1, Action::Pass);
    assert_eq!(t.stack_len(), 2);
    t.g.take_action(P0, Action::Pass);
    // Only the top object resolved.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P1), 21);
    assert_eq!(t.life(P0), 20);
    t.g.take_action(P0, Action::Pass);
    t.g.take_action(P1, Action::Pass);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 21);
    // With an empty stack, all players passing ends the step.
    assert_eq!(t.turn.step, Step::PrecombatMain);
    t.g.take_action(P0, Action::Pass);
    t.g.take_action(P1, Action::Pass);
    assert_eq!(t.turn.stage, mtg_engine::turn::Stage::End);
}

#[test]
fn state_based_actions_and_triggers_repeat_before_priority() {
    cr!("117.5");
    let mut t = TestGame::new(2);
    let traveler = t.battlefield(P0, "Doomed Traveler");
    t.g.obj_mut(traveler).damage = 1;
    let log = spy(&mut t, P0, |g, _p, d| {
        matches!(d, Decision::Priority { .. }).then(|| {
            format!(
                "stack={} travelers={}",
                g.stack.len(),
                g.find_in_zone(Zone::Battlefield, "Doomed Traveler").len()
            )
        })
    });
    t.g.advance();
    // The creature died as a state-based action, then its dies trigger was put on the
    // stack, and only then did P0 receive priority.
    assert_eq!(probe_lines(&log)[0], "stack=1 travelers=0");
}

#[test]
fn a_spell_cast_in_response_resolves_first() {
    cr!("117.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P1, "Mountain", 1);
    let growth = t.hand(P0, "Giant Growth");
    let shock = t.hand(P1, "Shock");
    t.cast(P0, growth).target(bears).go();
    // P1 responds.
    t.cast(P1, shock).target(bears).go();
    t.resolve();
    // Shock resolved first: the 2/2 died before Giant Growth could resolve.
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(t.in_graveyard(P0, "Giant Growth"));
}
