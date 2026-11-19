//! Item `rulings-residue-counters` — permanents whose effects depend on counters:
//! Quarantine Field's "until" exile (CR 610.3, 800.4a) and its targets fixed as the
//! ability is put on the stack (CR 603.3d), control and combat (CR 506.4: Shield Broker,
//! Rikku, Rumbling Ruin), a returning Aura's new object (CR 303.4f), effects that last
//! "for as long as" a counter remains (CR 611.2b: Xolatoyac), characteristic-defining
//! abilities of tokens and their copies (CR 604.3, 707.2: Daxos), locked-in sets of
//! affected objects (CR 611.2c: Inspiring Call), creatures put onto the battlefield
//! attacking (CR 508.4), and counting counters on permanents only (Kate Stewart).

use crate::r_residue_counters_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

// --- Quarantine Field --------------------------------------------------------------------

/// `p` casts Quarantine Field with X = `x` at the given targets and resolves it and its
/// enters trigger (unless `stop` is set: then the trigger waits on the stack). Returns
/// the Field.
fn quarantine(t: &mut TestGame, p: PlayerId, x: i64, targets: &[Entity], stop: bool) -> ObjectId {
    t.set_step(p, Step::PrecombatMain);
    t.lands(p, "Plains", 2 + 2 * x as usize);
    let qf = t.hand(p, "Quarantine Field");
    t.cast(p, qf).x(x).go();
    t.answer_targets(p, targets);
    t.resolve();
    if !stop {
        t.resolve_all();
    }
    t.named_on_battlefield("Quarantine Field")[0]
}

#[test]
fn quarantine_field_returns_cards_when_its_owner_leaves_the_game() {
    cr!("800.4a", "610.3c");
    ruling!(
        "Quarantine Field",
        "In a multiplayer game, if Quarantine Field’s owner leaves the game, the exiled cards will return to the battlefield."
    );
    supported("Quarantine Field");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P2, "Grizzly Bears");
    quarantine(&mut t, P1, 1, &[obj(bears)], false);
    assert!(t.in_exile("Grizzly Bears"));
    t.g.lose_game(P1);
    t.settle();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P2);
}

#[test]
fn quarantine_field_returns_cards_before_state_based_actions() {
    cr!("610.3", "704.3");
    ruling!(
        "Quarantine Field",
        "The exiled cards return to the battlefield immediately after Quarantine Field leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let qf = quarantine(&mut t, P0, 1, &[obj(bears)], false);
    assert!(t.in_exile("Grizzly Bears"));
    // Destroyed: once the event is processed, before any state-based action check, the
    // Bears are back.
    t.g.destroy(qf, None);
    t.g.flush_events();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    t.settle();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn quarantine_field_target_count_is_locked_when_the_trigger_is_put_on_the_stack() {
    cr!("603.3d", "601.2c");
    ruling!(
        "Quarantine Field",
        "Changing the number of isolation counters on Quarantine Field won’t change how many nonland permanents are exiled."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let qf = quarantine(&mut t, P0, 2, &[obj(a), obj(b)], true);
    assert_eq!(t.counters(qf, "isolation"), 2);
    remove(&mut t, qf, "isolation", 1);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Hill Giant"));
}

// --- Shield Broker -----------------------------------------------------------------------

#[test]
fn shield_broker_creature_losing_its_counter_in_combat_is_removed_from_combat() {
    cr!("506.4", "611.2b");
    ruling!(
        "Shield Broker",
        "If the creature loses its shield counter during combat, it will be removed from combat when it changes controllers."
    );
    supported("Shield Broker");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Shield Broker");
    t.resolve_all();
    assert_eq!(controller(&mut t, giant), P0);
    // On P0's next turn the Giant attacks; its shield counter is removed after blockers.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::DeclareBlockers);
    remove(&mut t, giant, "shield", 1);
    assert_eq!(controller(&mut t, giant), P1);
    assert!(t.g.combat.as_ref().is_none_or(|c| c.attacker(giant).is_none()));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "no combat damage");
}

// --- Traveling Plague --------------------------------------------------------------------

#[test]
fn traveling_plague_returns_under_the_dead_creatures_controller_who_chooses() {
    cr!("303.4f", "110.2");
    ruling!(
        "Traveling Plague",
        "The controller of the dead creature chooses the new creature."
    );
    supported("Traveling Plague");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    let tp = t.battlefield(P0, "Traveling Plague");
    t.g.attach(tp, obj(bears));
    t.g.recompute();
    let from = t.asked().len();
    t.answer_choose(P1, &[obj(mine)]);
    destroy(&mut t, bears);
    t.resolve_all();
    let plague = t.named_on_battlefield("Traveling Plague");
    assert_eq!(plague.len(), 1);
    assert_eq!(t.g.obj(plague[0]).controller, P1);
    assert_eq!(t.g.obj(plague[0]).attached_to, Some(obj(mine)));
    let asked: Vec<PlayerId> = asked_since(&t, from)
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P1]);
}

// --- Xolatoyac, the Smiling Flood --------------------------------------------------------




#[test]
fn xolatoyac_flooded_land_stays_an_island_after_it_leaves() {
    cr!("611.2b");
    ruling!(
        "Xolatoyac, the Smiling Flood",
        "The land stays an Island until the flood counter is removed, even if Xolatoyac, the Smiling Flood leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let (x, forest) = flood(&mut t);
    destroy(&mut t, x);
    t.g.recompute();
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    remove(&mut t, forest, "flood", 1);
    assert!(!t.obj_now(forest).chars.has_subtype("Island"));
    assert!(t.obj_now(forest).chars.has_subtype("Forest"));
}

// --- Daxos the Returned ------------------------------------------------------------------



#[test]
fn daxos_spirit_copies_have_its_power_and_toughness_ability() {
    cr!("707.2", "604.3");
    ruling!(
        "Daxos the Returned",
        "The ability that defines the token’s power and toughness is part of the token’s copiable values."
    );
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    put(&mut t, Entity::Player(P0), "experience", 3);
    let spirit = daxos_spirit(&mut t);
    cast_resolve(&mut t, P0, "Cackling Counterpart", &[obj(spirit)]);
    let spirits = crate::r_s01_common::with_subtype(&t, P0, "Spirit");
    assert_eq!(spirits.len(), 2);
    for s in &spirits {
        assert_eq!(t.pt(*s), (3, 3));
    }
    put(&mut t, Entity::Player(P0), "experience", 1);
    for s in &spirits {
        assert_eq!(t.pt(*s), (4, 4));
    }
}

// --- Rikku, Resourceful Guardian ---------------------------------------------------------

#[test]
fn rikku_doesnt_unblock_a_creature_already_blocked() {
    cr!("509.1h", "506.4a");
    ruling!(
        "Rikku, Resourceful Guardian",
        "Once a creature has been blocked, Rikku's first ability won't cause it to become unblocked."
    );
    supported("Rikku, Resourceful Guardian");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rikku, Resourceful Guardian");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(wurm, bears)]));
    t.advance_to(P0, Step::DeclareBlockers);
    put(&mut t, bears, "+1/+1", 1);
    assert_eq!(triggers_on_stack(&t, "can't be blocked"), 1);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(marked(&t, wurm), 3);
}

// --- Rumbling Ruin -----------------------------------------------------------------------


#[test]
fn rumbling_ruin_doesnt_remove_a_blocker_whose_power_drops() {
    cr!("506.4a", "509.1b");
    ruling!(
        "Rumbling Ruin",
        "Once a creature with high enough power has blocked, changing the power of the blocking creature won’t remove the blocking creature from combat"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    with_counters(&mut t, P0, "Grizzly Bears", 1);
    t.enter(P0, "Rumbling Ruin");
    t.resolve_all();
    let attacker = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(blocker, attacker)]));
    t.advance_to(P0, Step::DeclareBlockers);
    cast_resolve(&mut t, P0, "Shrink", &[obj(blocker)]);
    assert!(t.pt(blocker).0 <= 1);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "still blocked");
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

// --- Inspiring Call ----------------------------------------------------------------------

#[test]
fn inspiring_call_indestructible_outlasts_the_counters() {
    cr!("611.2c");
    ruling!(
        "Inspiring Call",
        "Once a creature gains indestructible, it will have it for the turn, even if it loses all its +1/+1 counters."
    );
    supported("Inspiring Call");
    let mut t = TestGame::new(2);
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 1);
    cast_resolve(&mut t, P0, "Inspiring Call", &[]);
    remove_plus1(&mut t, bears, 1);
    destroy(&mut t, bears);
    assert!(t.on_battlefield(bears));
}

// --- Sun-Blessed Guardian ----------------------------------------------------------------

#[test]
fn furnace_blessed_conqueror_token_entering_attacking_doesnt_trigger_its_attack_ability() {
    cr!("508.4", "508.3a");
    ruling!(
        "Sun-Blessed Guardian // Furnace-Blessed Conqueror",
        "Although the token enters the battlefield attacking, it was never declared as an attacking creature"
    );
    supported("Sun-Blessed Guardian // Furnace-Blessed Conqueror");
    let mut t = TestGame::new(2);
    let c = enter_with(
        &mut t,
        P0,
        "Sun-Blessed Guardian // Furnace-Blessed Conqueror",
        1,
        true,
    );
    let c = t.g.current(c);
    t.g.objects[c.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "copy of it"), 1);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "copy of it"), 0, "the token's doesn't trigger");
    assert_eq!(
        t.named_on_battlefield("Furnace-Blessed Conqueror").len()
            + t.named_on_battlefield("Sun-Blessed Guardian // Furnace-Blessed Conqueror")
                .len(),
        2
    );
}

// --- Kate Stewart, Armageddon Clock ------------------------------------------------------



// --- Spark Rupture -----------------------------------------------------------------------

#[test]
fn spark_rupture_removes_an_attacked_planeswalker_from_combat() {
    // Spark Rupture's "is a creature" sets the card type (CR 205.1a): the planeswalker
    // stops being one and is removed from combat (CR 506.4); its attackers keep attacking
    // nothing (CR 506.4c).
    cr!("205.1a", "506.4", "506.4c");
    ruling!(
        "Spark Rupture",
        "In some unusual cases, Spark Rupture may enter the battlefield during a combat in which a planeswalker has been attacked. In such a case, that planeswalker will be removed from combat."
    );
    supported("Spark Rupture");
    // (blocks: does the former planeswalker block the creature that was attacking it)
    for blocks in [false, true] {
        let mut t = TestGame::new(2);
        let jace = t.battlefield(P1, "Jace Beleren");
        let bears = t.battlefield(P0, "Grizzly Bears");
        attack_with(&mut t, &[(bears, obj(jace))]);
        t.battlefield(P0, "Spark Rupture");
        t.settle();
        assert!(!t.obj_now(jace).is(CardType::Planeswalker));
        let c = t.g.combat.as_ref().unwrap();
        assert!(c.attacker(bears).is_some(), "still attacking");
        assert_eq!(c.attack_target(bears), None, "attacking nothing");
        let b = if blocks { vec![(jace, bears)] } else { vec![] };
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(b));
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20);
        if blocks {
            // The 3/3 blocked it and was dealt its damage.
            assert!(t.in_graveyard(P0, "Grizzly Bears"));
            assert_eq!(marked(&t, jace), 2);
        } else {
            // Unblocked, it dealt no combat damage.
            assert_eq!(marked(&t, jace), 0);
            assert_eq!(t.counters(jace, "loyalty"), 3);
        }
    }
}
