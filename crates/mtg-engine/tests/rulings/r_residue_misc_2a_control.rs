//! Item `rulings-residue-misc-2`, pass A — control changes and "you" (CR 108.4, 109.4,
//! 603.3a, 113.8): Captured by the Consulate, Hellish Rebuke, Call to Arms, Promise of
//! Loyalty, Raul, Mirror-Mad Phantasm, Decoy Gambit, Rith and Temple of Power.

use crate::r_residue_misc_2a_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The targets of the stack object now.
fn targets(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    crate::r_s25_common::targets_of(t, id)
}

// --- Captured by the Consulate ----------------------------------------------------------------

/// A three-player game: P0's Captured by the Consulate on P1's Grizzly Bears.
fn captured() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Captured by the Consulate", bears);
    (t, bears)
}

#[test]
fn captured_by_the_consulate_triggers_on_its_controller_s_opponents() {
    cr!("109.5", "603.2", "115.7");
    ruling!(
        "Captured by the Consulate",
        "When you attach Captured by the Consulate to an opponent's creature, you still control Captured by the Consulate. Its last ability triggers whenever your opponents cast a spell with a single target, not when an opponent of the creature's controller casts a spell with a single target."
    );
    supported("Captured by the Consulate");
    // P0 (an opponent of the Bears' controller, but Captured's controller) casts Shock at
    // P2: the target stays.
    let (mut t, bears) = captured();
    cast_new(&mut t, P0, "Shock", &[pl(P2)]);
    t.settle();
    assert_eq!(t.stack_len(), 1, "no trigger");
    t.resolve_all();
    assert_eq!(t.life(P2), 18, "{}", t.dump_log());
    assert_eq!(dmg(&t, bears), 0);
    // P2 (P0's opponent) casts Shock at P0: the target becomes the Bears.
    let (mut t, bears) = captured();
    t.set_step(P2, Step::PrecombatMain);
    cast_new(&mut t, P2, "Shock", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn captured_by_the_consulate_ignores_a_spell_with_one_object_chosen_for_several_targets() {
    cr!("115.10a", "603.4");
    ruling!(
        "Captured by the Consulate",
        "If a spell has multiple targets, it doesn't have a single target even if the same object is chosen for each of those targets."
    );
    supported("Seeds of Strength");
    let (mut t, bears) = captured();
    t.set_step(P2, Step::PrecombatMain);
    let lion = t.battlefield(P2, "Savannah Lions");
    let seeds = cast_new(
        &mut t,
        P2,
        "Seeds of Strength",
        &[obj(lion), obj(lion), obj(lion)],
    );
    t.settle();
    assert_eq!(targets(&t, seeds), vec![obj(lion); 3]);
    t.resolve_all();
    assert_eq!(t.pt(lion), (5, 4), "{}", t.dump_log());
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn several_captured_by_the_consulate_the_first_in_turn_order_resolves_last() {
    cr!("101.4", "603.3b", "115.7");
    ruling!(
        "Captured by the Consulate",
        "If there is more than one Captured by the Consulate on the battlefield, the triggered ability of each one will change the target if able. The last one to resolve determines what that spell finally targets. If one player controls more than one Capture by the Consulate, they choose the order in which they resolve. If multiple players control those Auras, the first player among them in turn order (starting with the player whose turn it is now) is the one whose triggered ability resolves last."
    );
    // P0's Captured on P1's Bears; P2's on P1's Savannah Lions. On P0's turn P1 casts a
    // Shock at P0: P0's trigger is put on the stack first and resolves last.
    for active in [P0, P2] {
        let (mut t, bears) = captured();
        let lions = t.battlefield(P1, "Savannah Lions");
        crate::r_s06_common::attach_new(&mut t, P2, "Captured by the Consulate", lions);
        t.set_step(active, Step::PrecombatMain);
        t.lands(P1, "Mountain", 1);
        let shock = t.hand(P1, "Shock");
        let shock = t.cast(P1, shock).target(pl(P0)).go();
        t.settle();
        assert_eq!(t.stack_len(), 3);
        t.resolve();
        t.resolve();
        let last = if active == P0 { bears } else { lions };
        assert_eq!(targets(&t, shock), vec![obj(last)], "active {active:?}");
        t.resolve_all();
        assert!(!t.on_battlefield(last));
    }
}

// --- Hellish Rebuke -------------------------------------------------------------------------------

#[test]
fn hellish_rebuke_cast_twice_gives_two_instances() {
    cr!("113.2c", "603.2");
    ruling!(
        "Hellish Rebuke",
        "If a player casts more than one Hellish Rebuke in a turn, permanents will gain multiple instances of the triggered ability. Each will trigger and resolve separately. For instance, if you cast Hellish Rebuke twice, each permanent which damaged you will cause its controller to lose 4 life."
    );
    supported("Hellish Rebuke");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    for _ in 0..2 {
        cast_new(&mut t, P0, "Hellish Rebuke", &[]);
        t.resolve_all();
    }
    damage_to(&mut t, pyro, 1, pl(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Prodigal Pyromancer"));
}

#[test]
fn a_hellish_rebuke_copy_nobody_cast_does_nothing() {
    cr!("707.10", "707.10c");
    ruling!(
        "Hellish Rebuke",
        "Copies of Hellish Rebuke that weren't cast will give opponents permanents the ability, but that ability won't do anything because no player cast that copy of Hellish Rebuke."
    );
    supported("Reverberate");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    let rebuke = cast_new(&mut t, P0, "Hellish Rebuke", &[]);
    cast_new(&mut t, P0, "Reverberate", &[obj(rebuke)]);
    t.resolve_all();
    damage_to(&mut t, pyro, 1, pl(P0));
    t.resolve_all();
    // Only the cast Hellish Rebuke's ability does anything: 2 life, not 4.
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Prodigal Pyromancer"));
}

#[test]
fn hellish_rebuke_controlled_by_another_player_still_watches_its_caster() {
    cr!("108.4", "611.2c");
    ruling!(
        "Hellish Rebuke",
        "If a spell or ability causes another player to gain control of Hellish Rebuke while it is on the stack, permanents controlled by the opponents of that player will gain the ability (probably including permanents you control). However, you're still the player that cast Hellish Rebuke, so the ability will trigger whenever one of those permanents deals damage to you, not the player who controlled Hellish Rebuke as it resolved."
    );
    supported("Aethersnatch");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Prodigal Pyromancer");
    let theirs = t.battlefield(P1, "Prodigal Pyromancer");
    let rebuke = cast_new(&mut t, P0, "Hellish Rebuke", &[]);
    t.lands(P1, "Island", 2);
    t.lands(P1, "Wastes", 4);
    let snatch = t.hand(P1, "Aethersnatch");
    t.cast(P1, snatch).target(obj(rebuke)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hellish Rebuke"));
    // P1's permanents don't have it: P1's Pyromancer damaging P0 does nothing.
    damage_to(&mut t, theirs, 1, pl(P0));
    t.resolve_all();
    assert!(t.on_battlefield(theirs));
    assert_eq!(t.life(P1), 20);
    // P0's do, and trigger on damage to P0 (the caster), not to P1.
    damage_to(&mut t, mine, 1, pl(P1));
    t.resolve_all();
    assert!(t.on_battlefield(mine));
    damage_to(&mut t, mine, 1, pl(P0));
    t.resolve_all();
    assert!(
        t.in_graveyard(P0, "Prodigal Pyromancer"),
        "{}",
        t.dump_log()
    );
    assert_eq!(t.life(P0), 20 - 1 - 1 - 2);
}

// --- Call to Arms -----------------------------------------------------------------------------------

/// P0's Call to Arms entering, choosing white and `chosen`.
fn call_to_arms(t: &mut TestGame, chosen: PlayerId) -> ObjectId {
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[pl(chosen)]);
    let c = t.enter(P0, "Call to Arms");
    t.settle();
    c
}

#[test]
fn call_to_arms_pumps_every_white_creature() {
    cr!("611.3a", "105.4");
    ruling!(
        "Call to Arms",
        "Call to Arms affects all white creatures, not just white creatures you control."
    );
    supported("Call to Arms");
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P1, "Savannah Lions");
    t.battlefield(P1, "Glorious Anthem");
    let mine = t.battlefield(P0, "Savannah Lions");
    call_to_arms(&mut t, P1);
    t.g.recompute();
    // P0's Lions too; P1's also get +1/+1 from P1's Glorious Anthem.
    assert_eq!(t.pt(mine), (3, 2));
    assert_eq!(t.pt(lions), (4, 3));
}

#[test]
fn call_to_arms_s_state_trigger_waits_while_on_the_stack_and_retriggers_if_countered() {
    cr!("603.8");
    ruling!(
        "Call to Arms",
        "Call to Arms's third ability is a \"state trigger.\" Once a state trigger triggers, it won't trigger again as long as the ability is on the stack. If the ability is countered and the trigger condition is still true, it will immediately trigger again."
    );
    supported("Stifle");
    let mut t = TestGame::new(2);
    // P1 controls no white nontoken permanents: white isn't the most common color.
    t.battlefield(P1, "Grizzly Bears");
    call_to_arms(&mut t, P1);
    t.settle();
    t.settle();
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "sacrifice"), 1);
    let trigger = *t.g.stack.last().unwrap();
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(obj(trigger)).go();
    t.resolve();
    assert_eq!(
        crate::r_s01_common::triggers_on_stack(&t, "sacrifice"),
        1,
        "it triggers again"
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Call to Arms"));
}

#[test]
fn call_to_arms_keeps_checking_the_chosen_player_after_a_control_change() {
    cr!("607.2a", "108.4");
    ruling!(
        "Call to Arms",
        "If Call to Arms changes controllers, it will continue to check the nontoken permanents controlled by the player chosen as it entered, even if that player isn't an opponent of its current controller."
    );
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P1, "Savannah Lions");
    let c = call_to_arms(&mut t, P1);
    give_control(&mut t, c, P1);
    t.g.recompute();
    assert_eq!(controller(&t, c), P1);
    assert!(t.on_battlefield(c));
    assert_eq!(t.pt(lions), (3, 2));
    // P1's white permanents still count: making P1 control more green ones ends it.
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Call to Arms"), "{}", t.dump_log());
}

// --- Promise of Loyalty ----------------------------------------------------------------------------

#[test]
fn promise_of_loyalty_everyone_keeps_one_and_the_promise_follows_the_creature() {
    cr!("608.2c", "611.2a");
    ruling!(
        "Promise of Loyalty",
        "Each player, including you, must put a vow counter on a creature they control if able. They can't choose to opt out and sacrifice all of their creatures. The creature you put a vow counter on will remember its promise if another player gains control of it."
    );
    supported("Promise of Loyalty");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Savannah Lions");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Craw Wurm");
    t.answer_choose(P0, &[obj(mine)]);
    t.answer_choose(P1, &[obj(theirs)]);
    // Both players try to opt out: an empty answer isn't accepted.
    t.answer_choose(P0, &[]);
    cast_new(&mut t, P0, "Promise of Loyalty", &[]);
    t.resolve_all();
    let p0: Vec<_> = crate::r_s01_common::creatures(&t, P0);
    let p1: Vec<_> = crate::r_s01_common::creatures(&t, P1);
    assert_eq!(p0.len(), 1, "{}", t.dump_log());
    assert_eq!(p1.len(), 1);
    assert_eq!(t.counters(p0[0], "vow"), 1);
    assert_eq!(t.counters(p1[0], "vow"), 1);
    // P1 gains control of P0's vowed creature: it still can't attack P0.
    let vowed = p0[0];
    give_control(&mut t, vowed, P1);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!crate::r_s09_common::legal_attack(
        &mut t,
        &[(vowed, pl(P0))]
    ));
}

// --- Raul, Trouble Shooter -------------------------------------------------------------------------

#[test]
fn raul_lost_after_casting_began_doesnt_affect_the_spell() {
    cr!("601.2", "608.2");
    ruling!(
        "Raul, Trouble Shooter",
        "Once you begin to cast the spell, losing control of Raul won’t affect the spell."
    );
    supported("Raul, Trouble Shooter");
    let mut t = TestGame::new(2);
    let raul = t.battlefield(P0, "Raul, Trouble Shooter");
    t.library_top(P0, "Grizzly Bears");
    t.activate(P0, raul, 0, &[]).expect("Raul mills");
    t.resolve_all();
    let bears = t.g.find_in_zone(Zone::Graveyard(P0), "Grizzly Bears")[0];
    t.lands(P0, "Forest", 2);
    let spell = t.cast(P0, bears).try_go().expect("cast from the graveyard");
    give_control(&mut t, raul, P1);
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(
        controller(&t, t.named_on_battlefield("Grizzly Bears")[0]),
        P0
    );
}

// --- Mirror-Mad Phantasm ------------------------------------------------------------------------------

#[test]
fn mirror_mad_phantasm_is_activated_by_its_controller_not_its_owner() {
    cr!("602.2", "108.3");
    ruling!(
        "Mirror-Mad Phantasm",
        "You can only activate the ability if you control Mirror-Mad Phantasm, even if you don't own it."
    );
    supported("Mirror-Mad Phantasm");
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P1, "Mirror-Mad Phantasm");
    give_control(&mut t, ph, P0);
    t.lands(P0, "Island", 2);
    t.lands(P1, "Island", 2);
    assert!(!can_activate(&mut t, P1, ph), "its owner can't");
    assert!(can_activate(&mut t, P0, ph));
    t.activate(P0, ph, 0, &[])
        .expect("activated by its controller");
    t.resolve_all();
    // Its owner shuffled it into their library and put it back onto the battlefield.
    let back = t.named_on_battlefield("Mirror-Mad Phantasm");
    assert_eq!(back.len(), 1, "{}", t.dump_log());
    assert_eq!(controller(&t, back[0]), P1);
}

// --- Decoy Gambit ---------------------------------------------------------------------------------------

#[test]
fn decoy_gambit_target_that_changed_controller_is_illegal_and_gives_no_draw_choice() {
    cr!("608.2b", "115.7");
    ruling!(
        "Decoy Gambit",
        "If a creature targeted by Decoy Gambit changes controller, it’s no longer a legal target."
    );
    ruling!(
        "Decoy Gambit",
        "For each opponent, if you didn’t choose a target creature they control or if that creature has become an illegal target, that player can’t choose for you to draw a card."
    );
    supported("Decoy Gambit");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    lands_for_cost(&mut t, P0, "Decoy Gambit");
    let gambit = t.hand(P0, "Decoy Gambit");
    t.answer_targets(P0, &[obj(a)]);
    t.answer_targets(P0, &[obj(b)]);
    t.cast(P0, gambit).go();
    // P1's Bears changes controller (to P2) in response.
    give_control(&mut t, a, P2);
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    // P2 would let P0 draw rather than lose its Giant.
    t.answer_yes(P2, true);
    t.answer(P2, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    // The Bears stays where it is; P1 (no legal target) wasn't asked.
    assert!(t.on_battlefield(a));
    assert!(
        asked_since(&t, from).iter().all(|(p, _)| *p != P1),
        "{:?}",
        asked_since(&t, from)
    );
    let _ = hand;
}

// --- Rith, Liberated Primeval -----------------------------------------------------------------------------

#[test]
fn rith_counts_excess_damage_to_a_creature_an_opponent_controlled_then() {
    cr!("120.10", "603.4");
    ruling!(
        "Rith, Liberated Primeval",
        "Rith’s ability will trigger as long as an opponent controlled the creature as excess damage was dealt to it, even if the creature is no longer on the battlefield or you control it at the end of the turn."
    );
    supported("Rith, Liberated Primeval");
    // Dies from the excess damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rith, Liberated Primeval");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage_to(&mut t, pyro, 3, obj(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1, "{}", t.dump_log());
    // Indestructible and then stolen by P0: still counts.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rith, Liberated Primeval");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let myr = t.battlefield(P1, "Darksteel Myr");
    damage_to(&mut t, pyro, 2, obj(myr));
    give_control(&mut t, myr, P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1, "{}", t.dump_log());
    // Baseline: no excess damage (exactly lethal), no Dragon.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rith, Liberated Primeval");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage_to(&mut t, pyro, 2, obj(bears));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
}

// --- Temple of Power ---------------------------------------------------------------------------------------

#[test]
fn temple_of_power_counts_damage_by_a_red_source_you_controlled_then() {
    cr!("602.5b", "120.1");
    ruling!(
        "Ojer Axonil, Deepest Might // Temple of Power",
        "For the purpose of Temple of Power's last ability, you need to have controlled the red source that dealt noncombat damage only at the time that the damage was dealt. If it dies or changes control after that point, that damage will still count toward Temple of Power's activation restriction."
    );
    supported("Ojer Axonil, Deepest Might // Temple of Power");
    for dies in [false, true] {
        let mut t = TestGame::new(2);
        let temple = back_face(&mut t, P0, "Ojer Axonil, Deepest Might // Temple of Power");
        assert_eq!(t.obj_now(temple).chars.name, "Temple of Power");
        let giant = t.battlefield(P0, "Hill Giant");
        t.lands(P0, "Mountain", 3);
        assert!(t.activate(P0, temple, 1, &[]).is_err(), "no damage yet");
        t.g.players[0].mana_pool = Default::default();
        damage_to(&mut t, giant, 4, pl(P1));
        if dies {
            destroy(&mut t, giant);
            assert!(t.in_graveyard(P0, "Hill Giant"));
        } else {
            give_control(&mut t, giant, P1);
        }
        t.activate(P0, temple, 1, &[])
            .unwrap_or_else(|e| panic!("dies {dies}: {e:?}"));
        t.resolve_all();
        assert_eq!(t.obj_now(temple).chars.name, "Ojer Axonil, Deepest Might");
    }
}

#[test]
fn ojer_axonil_and_another_replacement_the_damaged_player_chooses_the_order() {
    cr!("616.1", "616.1e");
    ruling!(
        "Ojer Axonil, Deepest Might // Temple of Power",
        "If Ojer Axonil's second ability and additional replacement effects would modify how damage would be dealt, the player being dealt damage chooses the order in which to apply those effects."
    );
    supported("Torbran, Thane of Red Fell");
    // Shock (2) at P1 with Ojer Axonil (4 power) and Torbran (+2): Ojer first gives 4,
    // then 6; Torbran first gives 4, then Ojer leaves it at 4. P1 chooses.
    for torbran_first in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ojer Axonil, Deepest Might // Temple of Power");
        t.battlefield(P0, "Torbran, Thane of Red Fell");
        let from = t.asked().len();
        // The replacement options name the effects; P1 picks by index.
        t.answer(
            P1,
            DecisionKind::Replacement,
            Answer::Index(if torbran_first { 1 } else { 0 }),
        );
        cast_new(&mut t, P0, "Shock", &[pl(P1)]);
        t.resolve_all();
        let asked: Vec<_> = asked_since(&t, from)
            .into_iter()
            .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseReplacement { .. }))
            .collect();
        assert!(
            !asked.is_empty() && asked.iter().all(|(p, _)| *p == P1),
            "{asked:?}"
        );
        assert_eq!(t.life(P1), if torbran_first { 16 } else { 14 }, "{asked:?}");
    }
}
