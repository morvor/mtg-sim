//! Item `rulings-residue-misc-1` (pass C) — attack triggers, attack and block requirements:
//! Glorfindel, Dauntless Rescuer (a blocking requirement with a cost, CR 509.1c), Drakuseth
//! (an attacked planeswalker or battle that leaves, CR 506.4), Firemane Commando and Horizon
//! Explorer (attack triggers, CR 508.3a, 603.2), Territorial Hellkite (an attack
//! requirement for a player, CR 508.1d), Portal Manipulator (redirected attackers,
//! CR 506.4, 508.4), Chong and Lily (a lore counter outside the usual timing, CR 714.2b)
//! and Shiko and Narset's must-copy (CR 707.10).

use crate::r_p103_common::{obj, pl};
use crate::r_residue_misc_1_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `decl` is a legal block declaration now (the declare attackers step).
fn legal_blocks(t: &mut TestGame, decl: &[(ObjectId, ObjectId)]) -> bool {
    t.g.recompute();
    let defenders =
        t.g.combat
            .as_ref()
            .map(|c| c.defending_players.clone())
            .unwrap_or_default();
    let options = mtg_engine::combat::block_options(&t.g, &defenders);
    mtg_engine::combat::block_declaration_legal(&t.g, &options, decl)
}

/// What the attacking creature (followed across zone changes) is attacking, if it is.
fn attacking(t: &TestGame, id: ObjectId) -> Option<Entity> {
    let id = t.g.current(id);
    t.g.combat
        .as_ref()?
        .attackers
        .iter()
        .find(|a| a.id == id)
        .and_then(|a| a.target)
}

// ---------------------------------------------------------------------------------------
// Glorfindel, Dauntless Rescuer
// ---------------------------------------------------------------------------------------

/// P0's Glorfindel gets the first mode (P0 casts Opt to scry), then attacks P1 with the
/// other `attackers`; the game stops in the declare attackers step.
fn glorfindel_attacks(t: &mut TestGame, modes: &[usize], others: &[ObjectId]) -> ObjectId {
    supported("Glorfindel, Dauntless Rescuer");
    let g = t.battlefield(P0, "Glorfindel, Dauntless Rescuer");
    t.advance_to(P0, Step::PrecombatMain);
    for m in modes {
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![*m]));
        t.library_top(P0, "Island");
        cast_new(t, P0, "Opt", &[]);
        t.resolve_all();
    }
    let mut a = vec![(g, pl(P1))];
    a.extend(others.iter().map(|o| (*o, pl(P1))));
    attack_with(t, &a);
    t.resolve_all();
    g
}

#[test]
fn glorfindel_must_be_blocked_unless_no_creature_can_or_blocking_costs() {
    cr!("509.1c", "509.1d", "118.5");
    ruling!(
        "Glorfindel, Dauntless Rescuer",
        "If you choose the first mode for Glorfindel, Dauntless Rescuer's triggered ability but each creature the defending player controls can't block for any reason (such as being tapped), then Glorfindel, Dauntless Rescuer isn't blocked. If there's a cost associated with blocking Glorfindel, Dauntless Rescuer, the defending player isn't forced to pay that cost, so it doesn't have to be blocked in that case either."
    );
    // Untapped Bears: it must block.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let g = glorfindel_attacks(&mut t, &[0], &[]);
    assert!(!legal_blocks(&mut t, &[]));
    assert!(legal_blocks(&mut t, &[(bears, g)]));
    // Tapped Bears: no block, Glorfindel (4/3 after the trigger) is unblocked.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    let g = glorfindel_attacks(&mut t, &[0], &[]);
    assert!(legal_blocks(&mut t, &[]));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
    let _ = g;
    // Blocking costs something (War Cadence with X = 0): the Bears needn't block.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cadence = t.battlefield(P1, "War Cadence");
    let g = glorfindel_attacks(&mut t, &[0], &[]);
    t.lands(P1, "Mountain", 1);
    t.answer(P1, DecisionKind::X, Answer::Number(0));
    t.g.turn.priority = Some(P1);
    t.activate(P1, cadence, 0, &[]).unwrap();
    t.resolve();
    assert!(legal_blocks(&mut t, &[]));
    let _ = (bears, g);
}

#[test]
fn glorfindel_needs_only_one_blocker_the_rest_are_free() {
    cr!("509.1c");
    ruling!(
        "Glorfindel, Dauntless Rescuer",
        "If you choose the first mode for Glorfindel, Dauntless Rescuer's triggered ability, only one creature is required to block it. Other creatures may also block it (unless you've also chosen the second mode for Glorfindel, Dauntles Rescuer's triggered ability this turn) and are free to block other creatures or not block at all."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let lions = t.battlefield(P0, "Savannah Lions");
    let g = glorfindel_attacks(&mut t, &[0], &[lions]);
    assert!(legal_blocks(&mut t, &[(a, g)]));
    assert!(legal_blocks(&mut t, &[(a, g), (b, g)]));
    assert!(legal_blocks(&mut t, &[(a, g), (b, lions)]));
    assert!(!legal_blocks(&mut t, &[(b, lions)]));
    // Both modes chosen this turn: one blocker only.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let g = glorfindel_attacks(&mut t, &[0, 1], &[]);
    assert!(legal_blocks(&mut t, &[(a, g)]));
    assert!(!legal_blocks(&mut t, &[(a, g), (b, g)]));
}

// ---------------------------------------------------------------------------------------
// Drakuseth, Maw of Flames
// ---------------------------------------------------------------------------------------

#[test]
fn drakuseth_keeps_attacking_after_its_planeswalker_or_battle_is_gone() {
    cr!("506.4", "506.4c", "510.1c");
    ruling!(
        "Drakuseth, Maw of Flames",
        "If Drakuseth attacks a planeswalker and that planeswalker is reduced to 0 loyalty from the damage caused by Drakuseth's ability, Drakuseth continues to attack. It can be blocked, and it won't deal any combat damage if it's not blocked. The same is true if Drakuseth attacks a battle and that battle is reduced to 0 defense from the damage caused by Drakuseth's ability."
    );
    supported("Drakuseth, Maw of Flames");
    for battle in [false, true] {
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, "Drakuseth, Maw of Flames");
        let what = if battle {
            // P0's Siege, protected by P1: P0 attacks it.
            let id = t.battlefield(P0, "Invasion of Azgol");
            t.g.objects[id.0 as usize].choices.player = Some(P1);
            t.g.objects[id.0 as usize]
                .counters
                .insert(counters::DEFENSE.into(), 4);
            t.g.dirty = true;
            id
        } else {
            let id = t.battlefield(P1, "Liliana of the Veil");
            t.g.objects[id.0 as usize]
                .counters
                .insert(counters::LOYALTY.into(), 3);
            id
        };
        t.advance_to(P0, Step::PrecombatMain);
        // 4 to the planeswalker or battle; 3 each to nothing else.
        t.answer_targets(P0, &[obj(what)]);
        t.answer_targets(P0, &[]);
        attack_with(&mut t, &[(d, obj(what))]);
        t.resolve_all();
        assert!(
            !t.g.is_live(what),
            "battle {battle}: it left the battlefield"
        );
        assert!(is_attacking(&t, d), "battle {battle}: still attacking");
        block_and_finish(&mut t, P1, &[]);
        assert_eq!(t.life(P1), 20, "battle {battle}: no combat damage");
    }
}

#[test]
fn drakuseth_cant_choose_the_same_target_twice() {
    cr!("115.3", "601.2c");
    ruling!(
        "Drakuseth, Maw of Flames",
        "You can't choose the same target multiple times to have it be dealt 6, 7, or 10 damage."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Drakuseth, Maw of Flames");
    t.advance_to(P0, Step::PrecombatMain);
    let from = t.asked().len();
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    attack_with(&mut t, &[(d, pl(P1))]);
    let second = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(second.len() >= 2);
    assert!(!second[1].contains(&pl(P1)), "{second:?}");
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "dealt 4 only, not 7");
}

// ---------------------------------------------------------------------------------------
// Firemane Commando, Horizon Explorer
// ---------------------------------------------------------------------------------------

#[test]
fn firemane_commando_doesnt_care_what_happens_to_the_attackers_later() {
    cr!("603.2", "603.10", "508.3a");
    ruling!(
        "Firemane Commando",
        "For both triggered abilities, once the ability triggers, it doesn't matter what happens to the attacking creatures in response."
    );
    supported("Firemane Commando");
    // P0 attacks with two; one is destroyed in response: P0 still draws.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Firemane Commando");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Savannah Lions");
    t.advance_to(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1))]);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // P1 attacks P2 with two; one is destroyed in response: P1 still draws.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Firemane Commando");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Savannah Lions");
    t.advance_to(P1, Step::PrecombatMain);
    let hand = t.hand_size(P1);
    attack_with(&mut t, &[(a, pl(P2)), (b, pl(P2))]);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn firemane_commando_checks_every_attacker_for_one_attacking_you() {
    cr!("508.3a", "603.4");
    ruling!(
        "Firemane Commando",
        "No matter how many creatures another player attacks with, Firemane Commando's last ability will look at all of them to make sure none of them attacked you. If any of them did, the player won't draw a card."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Firemane Commando");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Savannah Lions");
    let c = t.battlefield(P1, "Hill Giant");
    t.advance_to(P1, Step::PrecombatMain);
    let hand = t.hand_size(P1);
    attack_with(&mut t, &[(a, pl(P2)), (b, pl(P2)), (c, pl(P0))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand);
}

#[test]
fn horizon_explorer_triggers_for_each_player_attacked() {
    cr!("508.3a", "603.2c");
    ruling!(
        "Horizon Explorer",
        "Horizon Explorer’s last ability will trigger once for each player you attack."
    );
    supported("Horizon Explorer");
    for (split, landers) in [(true, 2), (false, 1)] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Horizon Explorer");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Savannah Lions");
        t.advance_to(P0, Step::PrecombatMain);
        let second = if split { P2 } else { P1 };
        attack_with(&mut t, &[(a, pl(P1)), (b, pl(second))]);
        t.resolve_all();
        assert_eq!(tokens_of(&t, P0, "Lander"), landers, "split {split}");
    }
}

// ---------------------------------------------------------------------------------------
// Territorial Hellkite
// ---------------------------------------------------------------------------------------

#[test]
fn territorial_hellkite_may_choose_a_player_who_cant_be_attacked() {
    cr!("508.1d", "508.1c", "706.1");
    ruling!(
        "Territorial Hellkite",
        "If a player can't be attacked, that player can still be chosen at random."
    );
    ruling!(
        "Territorial Hellkite",
        "Players may cast spells and activate abilities after a player is chosen at random for Territorial Hellkite but before it attacks that player."
    );
    supported("Territorial Hellkite");
    // Only P1, behind Blazing Archon ("Creatures can't attack you."): P1 is chosen, the
    // Hellkite can't attack, and it isn't tapped (an opponent was chosen).
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Territorial Hellkite");
    t.battlefield(P1, "Blazing Archon");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.obj_now(h).tapped);
    // Players get priority in the beginning of combat step after the choice.
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    let from = t.asked().len();
    let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.resolve_all();
    assert!(resolved(&t, shock));
    assert!(from < t.asked().len());
    assert!(!crate::r_s02_common::can_attack(&mut t, h));
}

#[test]
fn territorial_hellkite_must_attack_the_player_not_a_planeswalker() {
    cr!("508.1d");
    ruling!(
        "Territorial Hellkite",
        "Territorial Hellkite must attack the chosen player if able, not a planeswalker that player controls."
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Territorial Hellkite");
    let walker = t.battlefield(P1, "Liliana of the Veil");
    t.g.add_counters(obj(walker), counters::LOYALTY, 3, None);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    t.g.recompute();
    let options = mtg_engine::combat::attack_options(&t.g);
    let at_walker =
        mtg_engine::combat::attack_declaration_legal(&t.g, &options, &[(h, obj(walker))]);
    let at_player = mtg_engine::combat::attack_declaration_legal(&t.g, &options, &[(h, pl(P1))]);
    assert!(!at_walker && at_player);
}

// ---------------------------------------------------------------------------------------
// Portal Manipulator
// ---------------------------------------------------------------------------------------

/// P1 attacks P0 with `attackers` in a three-player game; P0 flashes in Portal Manipulator
/// in the declare attackers step, redirecting them to P2. Returns the Manipulator.
fn redirect(t: &mut TestGame, attackers: &[ObjectId]) -> ObjectId {
    supported("Portal Manipulator");
    t.advance_to(P1, Step::PrecombatMain);
    let a: Vec<(ObjectId, Entity)> = attackers.iter().map(|c| (*c, pl(P0))).collect();
    attack_with(t, &a);
    t.resolve_all();
    let mut targets = vec![pl(P2)];
    targets.extend(attackers.iter().map(|c| obj(*c)));
    t.answer_targets(P0, &[pl(P2)]);
    t.answer_targets(P0, &targets[1..]);
    t.g.turn.priority = Some(P0);
    let m = cast_new(t, P0, "Portal Manipulator", &[]);
    t.resolve_all();
    m
}

#[test]
fn portal_manipulator_redirects_but_they_still_attacked_you() {
    cr!("506.4", "508.4");
    ruling!(
        "Portal Manipulator",
        "Once Portal Manipulator's triggered ability resolves, the target attacking creatures are still considered to have attacked the player, planeswalker, or battle they were initially declared as attacking, but they are now attacking the target player."
    );
    // Faramir chose P1 at P0's end step; P1 attacked P0 (then was redirected to P2): at
    // P1's end step P0 gets the soldiers, not the card.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Faramir, Prince of Ithilien");
    t.answer_choose(P0, &[pl(P1)]);
    through_end_step(&mut t, P0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P2, Step::End);
    t.resolve_all();
    redirect(&mut t, &[bears]);
    assert_eq!(attacking(&t, bears), Some(pl(P2)));
    block_and_finish(&mut t, P2, &[]);
    assert_eq!((t.life(P0), t.life(P2)), (20, 18));
    let hand = t.hand_size(P0);
    through_end_step(&mut t, P1);
    assert_eq!(tokens_of(&t, P0, "Soldier"), 3);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn portal_manipulator_doesnt_trigger_attack_abilities_and_ignores_attack_rules() {
    cr!("508.4", "508.1d", "508.1h");
    ruling!(
        "Portal Manipulator",
        "Portal Manipulator's triggered ability doesn't cause \"whenever this creature attacks\" abilities to trigger."
    );
    ruling!(
        "Portal Manipulator",
        "Portal Manipulator's triggered ability ignores all requirements, restrictions, and costs associated with attacking."
    );
    // Accorder Paladin's battle cry doesn't trigger again; P2's Blazing Archon ("Creatures
    // can't attack you.") doesn't stop the redirection.
    let mut t = TestGame::new(3);
    let paladin = t.battlefield(P1, "Accorder Paladin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Blazing Archon");
    redirect(&mut t, &[paladin, bears]);
    assert_eq!(attacking(&t, paladin), Some(pl(P2)));
    assert_eq!(attacking(&t, bears), Some(pl(P2)));
    // One battle cry (from the declaration): the Bears are 3/2, not 4/2.
    assert_eq!(pt_now(&mut t, bears), (3, 2));
}

#[test]
fn portal_manipulator_outside_the_declare_attackers_step_doesnt_trigger() {
    cr!("603.2", "601.3b");
    ruling!(
        "Portal Manipulator",
        "You may cast Portal Manipulator outside of a declare attackers step. If Portal Manipulator enters the battlefield outside of a declare attackers step, its ability simply doesn't trigger."
    );
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Portal Manipulator", &[]);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Portal Manipulator").len(), 1);
    assert_eq!(t.stack_len(), 0);
    // In P1's upkeep (flash): still no trigger.
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::Upkeep);
    t.g.turn.priority = Some(P0);
    cast_new(&mut t, P0, "Portal Manipulator", &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
}

// ---------------------------------------------------------------------------------------
// Chong and Lily, Shiko and Narset
// ---------------------------------------------------------------------------------------

#[test]
fn chong_and_lily_lore_counter_triggers_the_chapter_at_once() {
    cr!("714.2b", "714.3b", "603.2");
    ruling!(
        "Chong and Lily, Nomads",
        "Putting a lore counter on a Saga outside of the normal timing for lore counters will cause the appropriate chapter ability of that Saga to trigger immediately."
    );
    supported("Chong and Lily, Nomads");
    let mut t = TestGame::new(2);
    let chong = t.battlefield(P0, "Chong and Lily, Nomads");
    let saga = t.battlefield(P0, "History of Benalia");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 1);
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(saga)]);
    attack_with(&mut t, &[(chong, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(saga, "lore"), 2);
    assert_eq!(tokens_of(&t, P0, "Knight"), 1, "chapter II");
}

#[test]
fn shiko_and_narset_must_copy_a_targeting_second_spell() {
    cr!("707.10", "603.2");
    ruling!(
        "Shiko and Narset, Unified",
        "If the second spell you cast targets a permanent or player, you must copy that spell, even if it won’t be possible to choose legal targets for the copy. You can’t choose not to copy it in order to draw a card."
    );
    supported("Shiko and Narset, Unified");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shiko and Narset, Unified");
    t.advance_to(P0, Step::PrecombatMain);
    t.library_top(P0, "Island");
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "the Shock and its copy");
    assert_eq!(t.hand_size(P0), hand, "no card drawn");
}
