//! Rulings batch P116 — poison counters (CR 704.5c, 122.1f), "attacks and isn't blocked"
//! triggers (CR 509.1h, 603.2), blocking restrictions checked only as blockers are
//! declared (CR 509.1b, 506.4a), paying life (CR 119.4), and "You can't lose the game and
//! your opponents can't win the game" (CR 104.3, 104.2b, 101.2, 810.8).

use crate::r_p116_common::*;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::game::GameResult;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn poison(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::POISON)
}

/// From P0's main phase: P0 attacks P1 with `attackers` (P1 blocks as given) and the game
/// advances until P0 has priority in `step`.
fn attack_until(
    t: &mut TestGame,
    attackers: &[ObjectId],
    blocks: &[(ObjectId, ObjectId)],
    step: Step,
) {
    let atk: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(atk));
    if !blocks.is_empty() {
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    }
    t.advance_to(P0, step);
    // Triggered abilities are put on the stack before P0 acts.
    t.settle();
}

#[test]
fn virulent_silencer_tenth_poison_counter_loses_immediately() {
    cr!("704.5c", "704.3", "122.1f");
    ruling!(
        "Virulent Silencer",
        "A player with ten or more poison counters loses the game. This is a state-based action and doesn’t use the stack."
    );
    supported("Virulent Silencer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Virulent Silencer");
    let memnite = t.battlefield(P0, "Memnite");
    t.g.add_counters(Entity::Player(P1), counters::POISON, 8, None);
    attack_until(&mut t, &[memnite], &[], Step::CombatDamage);
    // The trigger is on the stack; P1 has 8 poison counters.
    assert_eq!(t.stack_len(), 1);
    let before = n_asked(&t);
    t.g.resolve_top();
    t.settle();
    // P1 lost as the state-based action was checked, before anyone got priority.
    assert_eq!(poison(&t, P1), 10);
    assert!(t.has_lost(P1));
    assert!(!priority_asked_since(&t, before));
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
}

#[test]
fn fynn_doesnt_need_to_be_on_the_battlefield_for_the_poison_loss() {
    cr!("704.5c", "603.2");
    ruling!(
        "Fynn, the Fangbearer",
        "Fynn doesn't have to still be on the battlefield when someone (preferably an opponent) gets their tenth poison counter."
    );
    supported("Fynn, the Fangbearer");
    let mut t = TestGame::new(2);
    let fynn = t.battlefield(P0, "Fynn, the Fangbearer");
    t.g.add_counters(Entity::Player(P1), counters::POISON, 8, None);
    attack_until(&mut t, &[fynn], &[], Step::CombatDamage);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, fynn);
    assert!(!t.on_battlefield(fynn));
    t.resolve_all();
    assert_eq!(poison(&t, P1), 10);
    assert!(t.has_lost(P1));
}

#[test]
fn crypt_cobra_poison_trigger_survives_the_cobra() {
    cr!("509.1h", "603.2", "113.7a");
    ruling!(
        "Crypt Cobra",
        "The player gets a poison counter as a triggered ability on not blocking the Cobra. Killing the Cobra after declaring blockers or preventing its damage will not prevent the poison counter."
    );
    supported("Crypt Cobra");
    let mut t = TestGame::new(2);
    let cobra = t.battlefield(P0, "Crypt Cobra");
    attack_until(&mut t, &[cobra], &[], Step::DeclareBlockers);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, cobra);
    t.resolve_all();
    assert_eq!(poison(&t, P1), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn swamp_mosquito_triggers_right_after_blockers_are_declared() {
    cr!("509.1h", "509.2a", "603.2");
    ruling!(
        "Swamp Mosquito",
        "Triggers immediately after blocking is declared if at that time no blockers are assigned to it."
    );
    supported("Swamp Mosquito");
    // Not blocked: the trigger is on the stack in the declare blockers step.
    let mut t = TestGame::new(2);
    let mosquito = t.battlefield(P0, "Swamp Mosquito");
    attack_until(&mut t, &[mosquito], &[], Step::DeclareBlockers);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(poison(&t, P1), 1);
    // Blocked (by a flier): no trigger.
    let mut t = TestGame::new(2);
    let mosquito = t.battlefield(P0, "Swamp Mosquito");
    let bird = t.battlefield(P1, "Suntail Hawk");
    attack_until(
        &mut t,
        &[mosquito],
        &[(bird, mosquito)],
        Step::DeclareBlockers,
    );
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(poison(&t, P1), 0);
}

#[test]
fn paladin_of_predation_stays_blocked_when_the_blockers_power_drops() {
    cr!("509.1b", "506.4a", "509.1h");
    ruling!(
        "Paladin of Predation",
        "Once it has become blocked, reducing the power of a creature blocking Paladin of Predation to less than 2 won't cause it to become unblocked."
    );
    supported("Paladin of Predation");
    supported("Disfigure");
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Paladin of Predation");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_until(&mut t, &[paladin], &[(giant, paladin)], Step::DeclareBlockers);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(paladin));
    // Hill Giant becomes 1/1.
    t.lands(P0, "Swamp", 1);
    let disfigure = t.hand(P0, "Disfigure");
    t.cast(P0, disfigure).target(giant).go();
    t.resolve();
    assert_eq!(t.pt(giant), (1, 1));
    t.advance_to(P0, Step::EndOfCombat);
    // Still blocked: no damage or poison to P1.
    assert_eq!(t.life(P1), 20);
    assert_eq!(poison(&t, P1), 0);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn phyrexian_unlife_at_zero_or_less_life_only_zero_life_can_be_paid() {
    cr!("119.4", "107.1b");
    ruling!(
        "Phyrexian Unlife",
        "If you’re at 0 or less life, you can’t pay any amount of life except 0."
    );
    supported("Phyrexian Unlife");
    supported("Toxic Deluge");
    // Toxic Deluge: "As an additional cost to cast this spell, pay X life. All creatures
    // get -X/-X until end of turn."
    for (x, can) in [(0, true), (1, false), (3, false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Phyrexian Unlife");
        set_life(&mut t, P0, -2);
        t.settle();
        assert!(!t.has_lost(P0));
        let probe = t.hand(P0, "Gitaxian Probe");
        assert!(!can_cast(&mut t, P0, probe, CastMethod::Normal));
        lands_for_cost(&mut t, P0, "Toxic Deluge");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let deluge = t.hand(P0, "Toxic Deluge");
        let r = t.cast(P0, deluge).x(x).try_go();
        if can {
            assert!(r.is_ok(), "X={x}");
            t.resolve_all();
            assert_eq!(t.life(P0), -2);
            assert!(t.on_battlefield(bears));
        } else {
            assert!(r.is_err(), "X={x}: no life can be paid");
            assert_eq!(t.life(P0), -2);
        }
    }
}

// --- Herald of Eternal Dawn ---------------------------------------------------------

/// P0 has 40 life and Felidar Sovereign; the game advances to P0's next upkeep and the
/// trigger resolves.
fn felidar_upkeep(t: &mut TestGame, winner: PlayerId) {
    t.battlefield(winner, "Felidar Sovereign");
    set_life(t, winner, 40);
    let n = t.g.players.len();
    let before = PlayerId(((winner.idx() + n - 1) % n) as u8);
    t.set_step(before, Step::End);
    t.advance_to(winner, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn herald_no_game_effect_makes_you_lose_or_an_opponent_win() {
    cr!("104.3b", "104.3c", "104.3d", "104.2b", "101.2");
    ruling!(
        "Herald of Eternal Dawn",
        "No game effect can cause you to lose the game or cause any opponent to win the game while you control Herald of Eternal Dawn."
    );
    supported("Herald of Eternal Dawn");
    // 0 or less life, drawing from an empty library, ten poison counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Eternal Dawn");
    set_life(&mut t, P0, -5);
    t.g.players[0].library.clear();
    t.g.draw_cards(P0, 1);
    t.g.add_counters(Entity::Player(P0), counters::POISON, 12, None);
    t.settle();
    assert!(!t.has_lost(P0));
    assert_eq!(t.g.result, None);
    // An effect saying P0 loses (Door to Nothingness).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Eternal Dawn");
    t.set_step(P1, Step::PrecombatMain);
    let door = t.battlefield(P1, "Door to Nothingness");
    for land in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P1, land, 2);
    }
    t.activate(P1, door, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert!(!t.has_lost(P0));
    assert_eq!(t.g.result, None);
    // An effect saying an opponent wins (Felidar Sovereign).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Eternal Dawn");
    felidar_upkeep(&mut t, P1);
    assert_eq!(t.g.result, None);
    assert!(!t.has_lost(P0));
}

#[test]
fn herald_you_can_still_concede() {
    cr!("104.3a", "101.1");
    ruling!(
        "Herald of Eternal Dawn",
        "Other circumstances can still cause you to lose the game. For example, you will lose a game if you concede"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Eternal Dawn");
    t.take_action(P0, Action::Concede);
    assert!(t.has_lost(P0));
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P1])));
}

#[test]
fn herald_in_two_headed_giant_protects_the_whole_team() {
    cr!("810.8", "810.8a", "810.8c", "104.3b");
    ruling!(
        "Herald of Eternal Dawn",
        "If you control Herald of Eternal Dawn in a Two-Headed Giant game, your team can't lose the game and the opposing team can't win the game."
    );
    // The team at 0 life doesn't lose.
    let mut t = two_headed_giant();
    t.battlefield(P0, "Herald of Eternal Dawn");
    t.g.lose_life(P1, 35);
    t.settle();
    assert!(t.life(P0) <= 0);
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    assert_eq!(t.g.result, None);
    // The opposing team can't win (Felidar Sovereign controlled by P2).
    let mut t = two_headed_giant();
    t.battlefield(P0, "Herald of Eternal Dawn");
    t.battlefield(P2, "Felidar Sovereign");
    t.g.gain_life(P2, 20);
    assert_eq!(t.life(P2), 50);
    t.set_step(P0, Step::End);
    t.advance_to(P2, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, None);
    // Without it, that team wins.
    let mut t = two_headed_giant();
    t.battlefield(P2, "Felidar Sovereign");
    t.g.gain_life(P2, 20);
    t.set_step(P0, Step::End);
    t.advance_to(P2, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P2, P3])));
}
