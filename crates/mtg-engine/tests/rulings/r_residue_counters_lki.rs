//! Item `rulings-residue-counters` — counters and power read from objects that left the
//! battlefield or changed (CR 608.2h, 603.10a: Rootwire Amalgam, Thought Sponge, Vastwood
//! Hydra), state-based actions after counters move or a 0/0 token is created (CR 704.5f,
//! 704.5g), Dismantle's counters (CR 608.2b, 122.1a), Descent into Madness, Nissa's
//! uninterrupted resolution (CR 608.2), Rottenmouth Viper's choices in multiplayer
//! (CR 101.4), Hallar in Two-Headed Giant (CR 810.9), "entered under your control this
//! turn" (Shaile), Afiya Grove's target, and choices made on resolution (Exuberant
//! Wolfbear).

use crate::r_residue_counters_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Rootwire Amalgam, Thought Sponge ----------------------------------------------------

#[test]
fn rootwire_amalgam_uses_its_last_power_on_the_battlefield() {
    cr!("608.2h", "602.2");
    ruling!(
        "Rootwire Amalgam",
        "Use Rootwire Amalgam's power as it existed the last time it was on the battlefield"
    );
    supported("Rootwire Amalgam");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P0, "Rootwire Amalgam");
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(a)]);
    assert_eq!(t.pt(a), (8, 8));
    t.lands(P0, "Forest", 5);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    let golems = crate::r_s01_common::with_subtype(&t, P0, "Golem");
    assert_eq!(golems.len(), 1);
    assert_eq!(t.pt(golems[0]), (24, 24));
}

#[test]
fn thought_sponge_draws_its_last_power_and_nothing_if_negative() {
    cr!("608.2h", "107.1b");
    ruling!(
        "Thought Sponge",
        "Use Thought Sponge's power as it last existed on the battlefield to determine how many cards to draw for its last ability. If that number is negative, you neither draw nor discard cards."
    );
    supported("Thought Sponge");
    // Pumped to 4/4 as it dies: four cards.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Island"; 6]);
    let s = t.battlefield(P0, "Thought Sponge");
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(s)]);
    let hand = t.hand_size(P0);
    destroy(&mut t, s);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4);
    // -4/1 as it dies: nothing.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Island"; 6]);
    let s = t.battlefield(P0, "Thought Sponge");
    cast_resolve(&mut t, P1, "Shrink", &[obj(s)]);
    let hand = t.hand_size(P0);
    let lib = t.library_size(P0);
    destroy(&mut t, s);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.library_size(P0)), (hand, lib));
}

// --- Vastwood Hydra ----------------------------------------------------------------------

#[test]
fn vastwood_hydra_distributes_its_plus1_counters_from_before_the_minus1_counters() {
    cr!("704.5f", "704.5q", "603.10a");
    ruling!(
        "Vastwood Hydra",
        "the ability will still distribute a number of +1/+1 counters equal to the number of +1/+1 counters on Vastwood Hydra before the -1/-1 counters were put on it"
    );
    supported("Vastwood Hydra");
    let mut t = TestGame::new(2);
    let hydra = with_counters(&mut t, P0, "Vastwood Hydra", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(bears)]);
    give_minus1(&mut t, hydra, 3);
    assert!(t.in_graveyard(P0, "Vastwood Hydra"));
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
}

// --- Corpse Cobble, Fractal Harness ------------------------------------------------------

#[test]
fn corpse_cobble_with_nothing_sacrificed_makes_a_dying_zero_zero_that_still_triggers() {
    cr!("704.5f", "603.6a", "603.10a");
    ruling!(
        "Corpse Cobble",
        "You may choose to sacrifice no creatures to pay the additional cost. If you do, you'll create a 0/0 Zombie token."
    );
    supported("Corpse Cobble");
    supported("Soul Warden");
    supported("Blood Artist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P0, "Blood Artist");
    t.answer_choose(P0, &[]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_resolve(&mut t, P0, "Corpse Cobble", &[]);
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Zombie").is_empty());
    assert_eq!(t.life(P0), 22, "Soul Warden and Blood Artist");
    assert_eq!(t.life(P1), 19);
}

#[test]
fn fractal_harness_put_onto_the_battlefield_makes_a_zero_zero_fractal() {
    cr!("107.3m", "704.5f");
    ruling!(
        "Fractal Harness",
        "If you put Fractal Harness onto the battlefield without casting it, or if you choose 0 as the value of X, the token you create will be 0/0."
    );
    supported("Fractal Harness");
    // Put onto the battlefield (X is 0).
    let mut t = TestGame::new(2);
    let h = t.enter(P0, "Fractal Harness");
    t.resolve_all();
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Fractal").is_empty());
    assert_eq!(t.obj_now(h).attached_to, None);
    // Cast with X = 0.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 3);
    let h = t.hand(P0, "Fractal Harness");
    t.cast(P0, h).x(0).go();
    t.resolve_all();
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Fractal").is_empty());
    // With X = 2: a 2/2 wearing it.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 5);
    let h = t.hand(P0, "Fractal Harness");
    t.cast(P0, h).x(2).go();
    t.resolve_all();
    let f = crate::r_s01_common::with_subtype(&t, P0, "Fractal");
    assert_eq!(f.len(), 1);
    assert_eq!(t.pt(f[0]), (2, 2));
}

// --- Aetherborn Marauder -----------------------------------------------------------------

#[test]
fn aetherborn_marauder_taking_counters_from_a_damaged_creature_kills_it() {
    cr!("704.5g", "122.5");
    ruling!(
        "Aetherborn Marauder",
        "If Aetherborn Marauder takes counters from a creature that has been dealt damage, that creature will be destroyed if the damage is now lethal."
    );
    let mut t = TestGame::new(2);
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 2);
    damage(&mut t, bears, 3);
    assert!(t.on_battlefield(bears), "a 4/4 with 3 damage");
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let m = t.enter(P0, "Aetherborn Marauder");
    t.resolve_all();
    assert_eq!(plus1(&t, m), 2);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

// --- Dismantle ---------------------------------------------------------------------------




#[test]
fn dismantle_with_an_illegal_target_puts_no_counters_but_indestructible_does() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Dismantle",
        "If the target artifact is an illegal target by the time Dismantle tries to resolve, the spell won't resolve."
    );
    supported("Darksteel Ingot");
    // Gone in response: nothing.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Ornithopter");
    put(&mut t, target, "fuse", 2);
    let stone = dismantle(&mut t, target, |t| {
        let o = t.g.current(target);
        exile_now(t, o);
    });
    assert_eq!(dismantled(&t, stone), 0);
    // Indestructible: not destroyed, but the counters are put.
    let mut t = TestGame::new(2);
    let ingot = t.battlefield(P1, "Darksteel Ingot");
    put(&mut t, ingot, "fuse", 3);
    let stone = dismantle(&mut t, ingot, |_| {});
    assert!(t.on_battlefield(ingot));
    assert_eq!(dismantled(&t, stone), 3);
}

#[test]
fn dismantle_plus1_counters_on_a_noncreature_artifact_count_once_it_animates() {
    cr!("122.1a", "208.3");
    ruling!(
        "Dismantle",
        "You can put +1/+1 counters on a noncreature artifact. They won't do anything unless the artifact becomes a creature"
    );
    supported("March of the Machines");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Ornithopter");
    put(&mut t, target, "fuse", 2);
    let stone = dismantle(&mut t, target, |_| {});
    assert_eq!(plus1(&t, stone), 2);
    assert!(!t.obj_now(stone).is(mtg_engine::types::CardType::Creature));
    // March of the Machines: a 2/2 (mana value 2) plus two counters.
    t.battlefield(P0, "March of the Machines");
    assert_eq!(t.pt(stone), (4, 4));
}

// --- Descent into Madness ----------------------------------------------------------------


// --- Nissa, Steward of Elements ----------------------------------------------------------




// --- Rottenmouth Viper -------------------------------------------------------------------

#[test]
fn rottenmouth_viper_opponents_choose_in_turn_order_then_act_together() {
    cr!("101.4", "101.4a");
    ruling!(
        "Rottenmouth Viper",
        "In a multiplayer game, each opponent in turn order makes all of their choices, then all of the actions occur simultaneously."
    );
    supported("Rottenmouth Viper");
    let mut t = TestGame::new(3);
    let card1 = t.hand(P1, "Hill Giant");
    let card2 = t.hand(P2, "Hill Giant");
    t.answer_choose(P1, &[obj(card1)]);
    t.answer_choose(P2, &[obj(card2)]);
    // When P2 chooses, P1's chosen card is still in P1's hand.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.player(P1).hand.len(),
    );
    let from = t.asked().len();
    t.enter(P0, "Rottenmouth Viper");
    t.resolve_all();
    let order: Vec<PlayerId> = asked_since(&t, from)
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order.first(), Some(&P1), "{order:?}");
    assert!(order.contains(&P2));
    assert_eq!(seen.lock().unwrap().first(), Some(&1));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P2, "Hill Giant"));
    assert_eq!((t.life(P1), t.life(P2)), (20, 20));
}

// --- Hallar, the Firefletcher ------------------------------------------------------------

#[test]
fn hallar_in_two_headed_giant_costs_the_opposing_team_two_per_counter() {
    cr!("810.9", "120.3a");
    ruling!(
        "Hallar, the Firefletcher",
        "In a Two-Headed Giant game, Hallar's last ability causes the opposing team to lose 2 life for each +1/+1 counter on it."
    );
    supported("Hallar, the Firefletcher");
    let mut t = two_headed_giant();
    with_counters(&mut t, P0, "Hallar, the Firefletcher", 1);
    let before = t.life(P2);
    t.set_step(P0, Step::PrecombatMain);
    // Sergeant-at-Arms kicked ({2}{W} more).
    t.lands(P0, "Plains", 3);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    cast_new(&mut t, P0, "Sergeant-at-Arms", &[]);
    t.resolve_all();
    // Two counters: 2 damage to each of P2 and P3, 4 from the team's life.
    assert_eq!(before - t.life(P2), 4);
    assert_eq!(t.life(P2), t.life(P3));
}

// --- Shaile, Dean of Radiance ------------------------------------------------------------

#[test]
fn shaile_counts_a_creature_that_entered_under_your_control_now_controlled_by_another() {
    cr!("603.10a", "122.6");
    ruling!(
        "Shaile, Dean of Radiance // Embrose, Dean of Shadow",
        "If a creature entered the battlefield under your control this turn but is controlled by a different player when Shaile’s activated ability resolves, it will still get a +1/+1 counter."
    );
    supported("Shaile, Dean of Radiance // Embrose, Dean of Shadow");
    supported("Ray of Command");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let shaile = t.battlefield(P0, "Shaile, Dean of Radiance // Embrose, Dean of Shadow");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    cast_resolve(&mut t, P1, "Ray of Command", &[obj(bears)]);
    assert_eq!(controller(&mut t, bears), P1);
    t.activate(P0, shaile, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
}

// --- Afiya Grove -------------------------------------------------------------------------


// --- Exuberant Wolfbear ------------------------------------------------------------------

#[test]
fn exuberant_wolfbear_choice_is_made_on_resolution() {
    cr!("608.2c", "603.3d");
    ruling!(
        "Exuberant Wolfbear",
        "You choose whether the Human’s base power and toughness change as the ability resolves, not as the ability is put onto the stack."
    );
    supported("Exuberant Wolfbear");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let wb = t.battlefield(P0, "Exuberant Wolfbear");
        let human = t.battlefield(P0, "Elite Vanguard");
        t.answer_targets(P0, &[obj(human)]);
        let from = t.asked().len();
        attack_with(&mut t, &[(wb, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 1);
        assert!(!was_asked(&t, P0, from, |d| matches!(d, Decision::YesNo { .. })));
        // Pumped in response: the values as it resolves.
        t.answer_yes(P0, yes);
        cast_resolve(&mut t, P0, "Giant Growth", &[obj(wb)]);
        assert_eq!(t.pt(human), if yes { (7, 7) } else { (2, 1) });
    }
}
