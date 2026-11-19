//! Rulings batch P114 — partner commanders' combat and damage abilities: Donna Noble,
//! Madame Vastra, Gorm the Great, Jeska, Francisco, Malcolm, Virtus, Kediss, Rasaad, Ikra
//! Shidiqi, Bruse Tarl, Trynn, Silas Renn (and Renari), Amy Pond, Rowan Kenrith, Prava.

use crate::r_p114_common::*;
use crate::r_s02_common::{can_cast, create_token, destroy};
use crate::r_s06_common::give_control;
use crate::r_s25_common::abilities_from;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Deals `n` damage from P1's `source` to each of `to` at the same time, then settles.
fn damage_together(t: &mut TestGame, source: ObjectId, n: i32, to: &[ObjectId]) {
    let mut ctx = mtg_engine::eval::Ctx::new(Some(source), P1);
    ctx.targets = vec![to.iter().map(|o| ent(*o)).collect()];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::This,
            amount: Value::Const(n),
            to: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

fn pump(t: &mut TestGame, id: ObjectId, p: i32, q: i32) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::Const(p), Value::Const(q))],
    );
}

#[test]
fn donna_noble_triggers_for_lethal_damage_too() {
    cr!("702.95b", "113.7a", "120.3");
    ruling!(
        "Donna Noble",
        "Donna's ability triggers when either creature is dealt damage even if one or both were dealt lethal damage."
    );
    supported("Donna Noble");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let donna = t.enter(P0, "Donna Noble");
    t.settle();
    t.answer_choose(P0, &[ent(bears)]);
    t.resolve();
    assert_eq!(t.obj(donna).paired_with, Some(bears));
    // 5 damage to each: both die.
    let pinger = t.battlefield(P1, "Llanowar Elves");
    damage_together(&mut t, pinger, 5, &[bears, donna]);
    assert!(!t.on_battlefield(donna) && !t.on_battlefield(bears));
    assert_eq!(abilities_from(&t, donna).len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
}

#[test]
fn madame_vastra_sees_a_creature_dying_with_her() {
    cr!("603.10a", "510.2", "704.5g");
    ruling!(
        "Madame Vastra",
        "If Madame Vastra is blocking or being blocked by another creature and both of those creatures are dealt lethal combat damage at the same time, they each die at the same time and the last ability will trigger."
    );
    supported("Madame Vastra");
    let mut t = TestGame::new(2);
    let vastra = t.battlefield(P0, "Madame Vastra");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(vastra, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, vastra)]);
    t.resolve_all();
    assert!(!t.on_battlefield(vastra) && !t.on_battlefield(giant));
    assert_eq!(tokens_named(&t, P0, "Clue").len(), 1);
    assert_eq!(tokens_named(&t, P0, "Food").len(), 1);
}

/// Gorm attacks P1 and P1 declares `blocks`; returns the creatures blocking Gorm.
fn gorm_blocked_by(
    t: &mut TestGame,
    gorm: ObjectId,
    blocks: &[(ObjectId, ObjectId)],
) -> Vec<ObjectId> {
    attack_with(t, &[(gorm, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    t.advance_to(P0, Step::CombatDamage);
    let combat = t.g.combat.as_ref().expect("combat");
    combat
        .blockers
        .iter()
        .filter(|b| b.blocking.contains(&gorm))
        .map(|b| b.id)
        .collect()
}

#[test]
fn gorm_only_creatures_able_to_block_must_block() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Gorm the Great",
        "If only one creature can block Gorm the Great, that creature does so."
    );
    ruling!(
        "Gorm the Great",
        "If a creature can't block Gorm for any reason (such as being tapped), then it doesn't block. If there's a cost associated with having it block Gorm, the player isn't forced to pay that cost, so it doesn't have to block in that case either. If this means that no creatures block Gorm, Gorm isn't blocked."
    );
    supported("Gorm the Great");
    // One untapped creature and one tapped: the untapped one blocks, even if P1 declares
    // no blocks.
    let mut t = TestGame::new(2);
    let gorm = t.battlefield(P0, "Gorm the Great");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tapped = t.battlefield(P1, "Hill Giant");
    t.g.objects[tapped.0 as usize].tapped = true;
    assert_eq!(gorm_blocked_by(&mut t, gorm, &[]), vec![bears]);
    // A blocking cost: War Cadence for X=1. P1 isn't forced to pay: Gorm is unblocked.
    supported("War Cadence");
    let mut t = TestGame::new(2);
    let gorm = t.battlefield(P0, "Gorm the Great");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Wastes", 4);
    let cadence = t.battlefield(P0, "War Cadence");
    t.lands(P0, "Mountain", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, cadence, 0, &[]).expect("War Cadence");
    t.resolve_all();
    assert!(gorm_blocked_by(&mut t, gorm, &[]).is_empty());
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18, "Gorm wasn't blocked");
}

#[test]
fn jeska_twice_multiplies_by_nine() {
    cr!("614.1a", "616.1", "510.2");
    ruling!(
        "Jeska, Thrice Reborn",
        "If a creature is targeted by Jeska's second ability twice, combat damage it would deal one of your opponents is multiplied by nine. Three abilities multiply the damage by twenty-seven, and so on."
    );
    supported("Jeska, Thrice Reborn");
    supported("Adric, Mathematical Genius");
    let mut t = TestGame::new(2);
    let jeska = t.battlefield(P0, "Jeska, Thrice Reborn");
    t.g.add_counters(ent(jeska), counters::LOYALTY, 3, None);
    let adric = t.battlefield(P0, "Adric, Mathematical Genius");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    t.activate(P0, jeska, 0, &[ent(bears)]).expect("Jeska's 0");
    let ability = *t.g.stack.last().unwrap();
    // Adric copies it (same target).
    t.activate(P0, adric, 0, &[ent(ability)]).expect("Adric");
    t.answer_yes(P0, false);
    t.resolve_all();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 2, "2 × 9 = 18");
}

#[test]
fn francisco_triggers_for_each_player_dealt_damage() {
    cr!("603.2c", "510.2", "506.2", "802.2");
    ruling!(
        "Francisco, Fowl Marauder",
        "If one or more Pirates you control deal damage to multiple players at once (for example, in combat, or with an ability such as that of Lightning-Rig Crew), Francisco, Fowl Marauder's ability triggers once for each of those players."
    );
    supported("Francisco, Fowl Marauder");
    supported("Lightning-Rig Crew");
    // Combat: two Pirates attack P1 and P2.
    let mut t = TestGame::new(3);
    let francisco = t.battlefield(P0, "Francisco, Fowl Marauder");
    let a = t.battlefield(P0, "Kitesail Freebooter");
    let b = t.battlefield(P0, "Kitesail Freebooter");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P2))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.answer(P2, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(abilities_from(&t, francisco).len(), 2);
    // Lightning-Rig Crew: "{T}: This creature deals 1 damage to each opponent."
    let mut t = TestGame::new(3);
    let francisco = t.battlefield(P0, "Francisco, Fowl Marauder");
    let crew = t.battlefield(P0, "Lightning-Rig Crew");
    t.activate(P0, crew, 0, &[]).expect("crew");
    t.resolve();
    t.settle();
    assert_eq!(abilities_from(&t, francisco).len(), 2);
}

#[test]
fn malcolm_counts_noncombat_damage() {
    cr!("603.2c", "120.1");
    ruling!(
        "Malcolm, Keen-Eyed Navigator",
        "Malcolm's triggered ability considers all damage dealt by Pirates you control to opponents, not just combat damage."
    );
    supported("Malcolm, Keen-Eyed Navigator");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Malcolm, Keen-Eyed Navigator");
    let crew = t.battlefield(P0, "Lightning-Rig Crew");
    t.activate(P0, crew, 0, &[]).expect("crew");
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Treasure").len(), 2);
}

#[test]
fn virtus_life_loss_follows_combat_damage() {
    cr!("510.2", "603.3", "107.1a");
    ruling!(
        "Virtus the Veiled",
        "Virtus's last ability triggers and resolves after combat damage is dealt. For example, if Virtus deals 1 combat damage to a player with 10 life, combat damage will reduce that player's life total to 9. Then its ability will cause the player to lose 5 life, leaving the player at 4."
    );
    supported("Virtus the Veiled");
    let mut t = TestGame::new(2);
    let virtus = t.battlefield(P0, "Virtus the Veiled");
    t.g.players[1].life = 10;
    attack_with(&mut t, &[(virtus, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(t.life(P1), 9);
    t.resolve_all();
    assert_eq!(t.life(P1), 4);
}

#[test]
fn virtus_in_two_headed_giant_uses_the_team_life() {
    cr!("810.9", "810.9a");
    ruling!(
        "Virtus the Veiled",
        "In a Two-Headed Giant game, a player's life total is the same as their team's life total."
    );
    let mut t = two_headed_giant();
    let virtus = t.battlefield(P0, "Virtus the Veiled");
    assert_eq!(t.life(P2), 30);
    attack_with(&mut t, &[(virtus, Entity::Player(P2))]);
    t.answer(P2, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    // 30 − 1 = 29, then half of 29 rounded up (15).
    assert_eq!(t.life(P2), 14);
    assert_eq!(t.life(P3), 14);
}

/// A three-player Commander game: P0 controls Kediss.
fn kediss_game() -> TestGame {
    supported("Kediss, Emberclaw Familiar");
    let mut t = commander_game_n(3);
    t.battlefield(P0, "Kediss, Emberclaw Familiar");
    t
}

#[test]
fn kediss_triggers_for_any_commander_you_control() {
    cr!("903.3", "510.2");
    ruling!(
        "Kediss, Emberclaw Familiar",
        "Kediss's triggered ability will trigger if any commander you control deals combat damage to an opponent, not just your commander."
    );
    ruling!(
        "Kediss, Emberclaw Familiar",
        "The damage the commander deals as a result of Kediss's triggered ability isn't combat damage. It isn't tracked with combat damage dealt by commanders and won't cause Kediss's ability to trigger again."
    );
    let mut t = kediss_game();
    // P1's commander Ganax, under P0's control.
    let ganax = commander_on_battlefield(&mut t, P1, "Ganax, Astral Hunter");
    give_control(&mut t, ganax, P0);
    // (As if it had been under P0's control since the turn began.)
    t.g.objects[ganax.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(ganax, Entity::Player(P2))]);
    t.answer(P2, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P2), 37);
    assert_eq!(
        t.life(P1),
        37,
        "Kediss: Ganax dealt 3 to the other opponent"
    );
    assert_eq!(t.stack_len(), 0, "no retrigger");
    assert_eq!(
        t.player(P2).commander_damage.values().sum::<u32>(),
        3,
        "combat damage"
    );
    assert_eq!(
        t.player(P1).commander_damage.values().sum::<u32>(),
        0,
        "not combat damage"
    );
}

#[test]
fn kediss_doesnt_trigger_for_damage_to_a_planeswalker() {
    cr!("510.2", "306.8");
    ruling!(
        "Kediss, Emberclaw Familiar",
        "Kediss's triggered ability won't trigger if a commander you control deals combat damage to a planeswalker."
    );
    let mut t = kediss_game();
    let ganax = commander_on_battlefield(&mut t, P0, "Ganax, Astral Hunter");
    let jace = t.battlefield(P1, "Jace Beleren");
    let have = t.counters(jace, counters::LOYALTY);
    t.g.add_counters(ent(jace), counters::LOYALTY, 5 - have, None);
    attack_with(&mut t, &[(ganax, ent(jace))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.counters(jace, counters::LOYALTY), 2, "Ganax dealt 3 to Jace");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 40);
    assert_eq!(t.life(P2), 40);
}

#[test]
fn rasaad_changes_only_the_combat_damage_assigned() {
    cr!("510.1a", "701.14a");
    ruling!(
        "Rasaad yn Bashir",
        "Rasaad yn Bashir's first ability doesn't actually change any creature's power. It changes only the amount of combat damage it assigns. All other rules and effects that check power or toughness use the real values."
    );
    let mut t = TestGame::new(2);
    let rasaad = t.battlefield(P0, "Rasaad yn Bashir");
    assert_eq!(t.pt(rasaad), (0, 3));
    attack_with(&mut t, &[(rasaad, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(rasaad), (0, 3));
    // A fight uses its real power (0): Prey Upon, Rasaad against Grizzly Bears.
    t.advance_to(P0, Step::PostcombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Prey Upon", &[ent(rasaad), ent(bears)]);
    t.resolve_all();
    assert_eq!(t.obj(bears).damage, 0);
    assert_eq!(t.obj(rasaad).damage, 2);
}

#[test]
fn ikra_uses_last_known_toughness() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Ikra Shidiqi, the Usurper",
        "The amount of life you gain is determined as Ikra Shidiqi's triggered ability resolves. If that creature is no longer on the battlefield, use its toughness as it last existed on the battlefield to determine how much life to gain."
    );
    supported("Ikra Shidiqi, the Usurper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ikra Shidiqi, the Usurper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, the Bears get +0/+3 (toughness 5) and then leave the battlefield.
    pump(&mut t, bears, 0, 3);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

#[test]
fn bruse_tarl_triggers_on_entering_and_attacking() {
    cr!("603.2", "508.1m");
    ruling!(
        "Bruse Tarl, Boorish Herder",
        "The triggered ability triggers both when Bruse Tarl enters the battlefield and whenever it attacks. You don't have to choose only one."
    );
    supported("Bruse Tarl, Boorish Herder");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[ent(bears)]);
    let bruse = t.enter(P0, "Bruse Tarl, Boorish Herder");
    t.settle();
    assert_eq!(abilities_from(&t, bruse).len(), 1, "on entering");
    t.resolve_all();
    assert!(t
        .obj(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
    t.g.objects[bruse.0 as usize].summoning_sick = false;
    t.answer_targets(P0, &[ent(bruse)]);
    attack_with(&mut t, &[(bruse, Entity::Player(P1))]);
    assert_eq!(abilities_from(&t, bruse).len(), 1, "on attacking");
    t.resolve_all();
    assert!(t
        .obj(bruse)
        .has_keyword(mtg_engine::keywords::KeywordKind::Lifelink));
}

#[test]
fn trynn_checks_only_that_you_attacked() {
    cr!("603.4", "508.1");
    ruling!(
        "Trynn, Champion of Freedom",
        "Trynn's ability checks only whether you declared an attacking creature during the turn. It doesn't trigger once for each such creature, and it doesn't check whether any attacking creatures dealt damage."
    );
    supported("Trynn, Champion of Freedom");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trynn, Champion of Freedom");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    let wall2 = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(wall, a), (wall2, b)]);
    assert_eq!(t.life(P1), 20, "no damage dealt");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Soldier").len(), 1);
}

/// Silas Renn deals combat damage to P1; its trigger targets Mind Stone in P0's graveyard.
/// The game is left in the combat damage step with the stack empty. Returns (game, Mind
/// Stone).
fn silas_hits(renari: bool) -> (TestGame, ObjectId) {
    supported("Silas Renn, Seeker Adept");
    let mut t = TestGame::new(2);
    let silas = t.battlefield(P0, "Silas Renn, Seeker Adept");
    if renari {
        supported("Renari, Merchant of Marvels");
        t.battlefield(P0, "Renari, Merchant of Marvels");
    }
    let stone = t.graveyard(P0, "Mind Stone");
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[ent(stone)]);
    attack_with(&mut t, &[(silas, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    (t, stone)
}

#[test]
fn silas_renn_doesnt_change_timing_but_renari_does() {
    cr!("601.3", "301.1", "702.8a");
    ruling!(
        "Silas Renn, Seeker Adept",
        "Silas Renn doesn't change when you can cast the target artifact card. For example, if you target an artifact card without flash, you can cast it only during your main phase when the stack is empty."
    );
    ruling!(
        "Renari, Merchant of Marvels",
        "The first ability applies to Dragon spells and artifact spells that are cast from any zone, provided something is allowing you do cast them. For example, if another effect allows you to cast Dragon spells from your graveyard, Renari will allow you to do so as though they had flash."
    );
    let (mut t, stone) = silas_hits(false);
    assert!(!can_cast(&mut t, P0, stone, CastMethod::Normal), "combat");
    t.advance_to(P0, Step::PostcombatMain);
    assert!(
        can_cast(&mut t, P0, stone, CastMethod::Normal),
        "main phase"
    );
    // With Renari: in the combat damage step.
    let (mut t, stone) = silas_hits(true);
    assert_eq!(t.g.turn.step, Step::CombatDamage);
    assert!(can_cast(&mut t, P0, stone, CastMethod::Normal));
    t.cast(P0, stone).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(stone)));
}

#[test]
fn silas_renn_permits_one_cast_and_otherwise_the_card_stays() {
    cr!("400.7", "601.2a");
    ruling!(
        "Silas Renn, Seeker Adept",
        "Casting the card causes it to leave your graveyard and become a new object. You can't cast it multiple times."
    );
    ruling!(
        "Silas Renn, Seeker Adept",
        "If you don't cast the card, it remains in your graveyard."
    );
    let (mut t, stone) = silas_hits(false);
    t.advance_to(P0, Step::PostcombatMain);
    t.cast(P0, stone).go();
    t.resolve_all();
    let on_bf = t.g.current(stone);
    destroy(&mut t, on_bf);
    let back = t.g.current(stone);
    assert_eq!(t.zone(back), Zone::Graveyard(P0));
    t.lands(P0, "Wastes", 2);
    assert!(
        !can_cast(&mut t, P0, back, CastMethod::Normal),
        "a new object"
    );
    // Not cast: it stays in the graveyard.
    let (mut t, stone) = silas_hits(false);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(stone), Zone::Graveyard(P0));
}

#[test]
fn amy_pond_removing_the_last_time_counter_casts_the_card() {
    cr!("702.62a", "702.62d");
    ruling!(
        "Amy Pond",
        "If you remove the last time counter from a suspended card this way, you will cast that card."
    );
    supported("Amy Pond");
    let mut t = TestGame::new(2);
    let amy = t.battlefield(P0, "Amy Pond");
    // Rift Bolt, suspended with two time counters.
    let bolt = t.exile(P0, "Rift Bolt");
    t.g.add_counters(ent(bolt), counters::TIME, 2, None);
    t.answer_choose(P0, &[ent(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(amy, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(t.life(P1), 15, "2 combat damage and Rift Bolt's 3");
    assert!(t.in_graveyard(P0, "Rift Bolt"));
}

#[test]
fn rowan_damages_tapped_hexproof_creatures() {
    cr!("115.10", "702.11b");
    ruling!(
        "Rowan Kenrith",
        "Rowan's second ability targets only the player. Tapped creatures with hexproof that player controls will be dealt damage as that ability resolves."
    );
    let mut t = TestGame::new(2);
    let rowan = t.battlefield(P0, "Rowan Kenrith");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let untapped = t.battlefield(P1, "Gladecover Scout");
    t.g.objects[scout.0 as usize].tapped = true;
    t.activate(P0, rowan, 1, &[Entity::Player(P1)]).expect("-2");
    t.resolve_all();
    assert!(!t.on_battlefield(scout));
    assert!(t.on_battlefield(untapped));
}

#[test]
fn prava_bonus_lasts_through_cleanup_but_ends_when_it_leaves() {
    cr!("514.2", "611.3a", "704.5g");
    ruling!(
        "Prava of the Steel Legion",
        "The bonus provided by Prava's first ability will still be in effect when damage is removed during the cleanup step."
    );
    ruling!(
        "Prava of the Steel Legion",
        "However, if Prava leaves the battlefield, its bonus immediately stops applying. Nonlethal damage dealt to creature tokens you control may become lethal if Prava leaves the battlefield during your turn."
    );
    supported("Prava of the Steel Legion");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let prava = t.battlefield(P0, "Prava of the Steel Legion");
        let token = create_token(&mut t, P0, "Soldier");
        assert_eq!(t.pt(token), (2, 5));
        let pinger = t.battlefield(P1, "Llanowar Elves");
        deal(&mut t, pinger, 4, ent(token));
        assert!(t.on_battlefield(token));
        if leaves {
            destroy(&mut t, prava);
            assert!(!t.on_battlefield(token), "4 damage on a 1/1");
        } else {
            t.advance_to(P1, Step::Upkeep);
            assert!(t.on_battlefield(token), "survived the cleanup step");
            assert_eq!(t.obj(token).damage, 0);
            assert_eq!(t.pt(token), (1, 1));
        }
    }
}
