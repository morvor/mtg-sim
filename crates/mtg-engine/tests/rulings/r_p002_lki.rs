//! Rulings batch P002 — a creature's power or toughness turned into life, cards, counters
//! or a token: last known information (CR 608.2h, 603.10a), illegal targets (CR 608.2b),
//! negative power (CR 107.1b), characteristic-defining abilities in every zone (CR 604.3),
//! and "dies" versus "destroyed" (CR 700.4, 614.6).

use crate::r_p002_common::*;
use crate::r_p003_common::*;
use crate::r_s01_common::{supported, tokens};
use crate::r_s06_common::{activate_containing, attach_new, give_control};
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts a real card into `p`'s graveyard.
fn grave(t: &mut TestGame, p: PlayerId, name: &str) {
    t.graveyard(p, name);
}

// ---------------------------------------------------------------------------------------
// Bishop of Binding: "When this creature enters, exile target creature an opponent
// controls until this creature leaves the battlefield. Whenever this creature attacks,
// target Vampire gets +X/+X until end of turn, where X is the power of the exiled card."
// ---------------------------------------------------------------------------------------

/// P0's Bishop enters exiling `victim` (P1's); its trigger resolves.
fn bishop_exiling(t: &mut TestGame, victim: ObjectId) -> ObjectId {
    supported("Bishop of Binding");
    t.answer_targets(P0, &[obj(victim)]);
    let b = enter(t, P0, "Bishop of Binding");
    t.resolve_all();
    assert!(!t.on_battlefield(victim));
    t.g.objects[b.0 as usize].summoning_sick = false;
    b
}

/// P0 attacks P1 with the Bishop, its trigger targeting `vampire`; the trigger is left on
/// the stack.
fn bishop_attacks(t: &mut TestGame, bishop: ObjectId, vampire: ObjectId) {
    t.answer_targets(P0, &[obj(vampire)]);
    crate::r_s01_common::attack_with(t, &[(t.g.current(bishop), pl(P1))]);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn bishop_of_binding_leaving_before_its_enter_trigger_resolves_exiles_nothing() {
    cr!("610.3c");
    ruling!(
        "Bishop of Binding",
        "If Bishop of Binding leaves the battlefield before its first triggered ability tries to resolve, the target creature won’t be exiled."
    );
    let mut t = TestGame::new(2);
    let bears = add(&mut t, P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let b = enter(&mut t, P0, "Bishop of Binding");
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, b);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Control: the Bishop staying, the creature is exiled.
    let bears2 = add(&mut t, P1, "Grizzly Bears");
    bishop_exiling(&mut t, bears2);
    assert_eq!(exiled(&t, "Grizzly Bears"), 1);
}

#[test]
fn bishop_of_binding_uses_a_vehicle_s_printed_power_and_x_is_locked_in() {
    cr!("208.3", "608.2h", "611.2c");
    ruling!(
        "Bishop of Binding",
        "If a Vehicle is exiled with Bishop of Binding, use its printed power to determine the value of X."
    );
    ruling!(
        "Bishop of Binding",
        "The value of X is determined only as Bishop of Binding’s second ability resolves. It won’t change if the card leaves exile later in the turn."
    );
    supported("Smuggler's Copter");
    let mut t = TestGame::new(2);
    let copter = add(&mut t, P1, "Smuggler's Copter");
    let crew = add(&mut t, P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_choose(P1, &[obj(crew)]);
    activate_containing(&mut t, P1, copter, "Crew").expect("crew");
    t.resolve_all();
    assert!(is_creature(&t, copter));
    t.set_step(P0, Step::PrecombatMain);
    let b = bishop_exiling(&mut t, copter);
    let hawk = add(&mut t, P0, "Vampire Nighthawk");
    bishop_attacks(&mut t, b, hawk);
    t.resolve_all();
    assert_eq!(t.pt(hawk), (5, 6), "+3/+3 from the Copter's printed power");
    // The Bishop leaves: the Copter returns, the bonus stays.
    kill(&mut t, b);
    assert_eq!(t.named_on_battlefield("Smuggler's Copter").len(), 1);
    assert_eq!(t.pt(hawk), (5, 6));
}

#[test]
fn bishop_of_binding_x_is_0_with_no_exiled_card_or_one_without_power() {
    cr!("208.3", "610.3a", "608.2h");
    ruling!(
        "Bishop of Binding",
        "If there is no exiled card as Bishop of Binding’s last ability resolves, most likely because Bishop of Binding left the battlefield while that ability was on the stack, X is 0."
    );
    // The Bishop dies with its attack trigger on the stack: Grizzly Bears returns, X = 0.
    let mut t = TestGame::new(2);
    let bears = add(&mut t, P1, "Grizzly Bears");
    let b = bishop_exiling(&mut t, bears);
    let hawk = add(&mut t, P0, "Vampire Nighthawk");
    bishop_attacks(&mut t, b, hawk);
    kill(&mut t, b);
    assert!(t.named_on_battlefield("Grizzly Bears").len() == 1);
    t.resolve_all();
    assert_eq!(t.pt(hawk), (2, 3));
    // Control: with the Bears still exiled, +2/+2.
    let mut t = TestGame::new(2);
    let bears = add(&mut t, P1, "Grizzly Bears");
    let b = bishop_exiling(&mut t, bears);
    let hawk = add(&mut t, P0, "Vampire Nighthawk");
    bishop_attacks(&mut t, b, hawk);
    t.resolve_all();
    assert_eq!(t.pt(hawk), (4, 5));
    // An animated Mutavault (a land card, no power) exiled: X = 0.
    supported("Mutavault");
    let mut t = TestGame::new(2);
    let vault = add(&mut t, P1, "Mutavault");
    t.set_step(P1, Step::PrecombatMain);
    activate_resolve(&mut t, P1, vault, "becomes", "1", &[]);
    assert!(is_creature(&t, vault));
    t.set_step(P0, Step::PrecombatMain);
    let b = bishop_exiling(&mut t, vault);
    let hawk = add(&mut t, P0, "Vampire Nighthawk");
    bishop_attacks(&mut t, b, hawk);
    t.resolve_all();
    assert_eq!(t.pt(hawk), (2, 3));
}

// ---------------------------------------------------------------------------------------
// Characteristic-defining abilities: Talara's Bane, Essence Backlash.
// ---------------------------------------------------------------------------------------

#[test]
fn talara_s_bane_and_essence_backlash_use_star_abilities_off_the_battlefield() {
    cr!("604.3", "208.2a");
    ruling!(
        "Talara's Bane",
        "If the creature card has “*” in its power or toughness, the ability that defines “*” works in all zones."
    );
    ruling!(
        "Essence Backlash",
        "If the creature’s power is * and it has an ability defining its power, use that ability to determine its power."
    );
    supported("Talara's Bane");
    supported("Essence Backlash");
    // Graveyards: an instant and a sorcery. Tarmogoyf in P1's hand is a 2/3.
    let mut t = TestGame::new(2);
    grave(&mut t, P0, "Lightning Bolt");
    grave(&mut t, P1, "Pyroclasm");
    let goyf = t.hand(P1, "Tarmogoyf");
    t.answer_choose(P0, &[obj(goyf)]);
    cast_resolve(&mut t, P0, "Talara's Bane", "1B", &[pl(P1)]);
    assert_eq!(t.life(P0), 23);
    assert!(t.in_graveyard(P1, "Tarmogoyf"));
    // Tarmogoyf on the stack (instant, sorcery: power 2) is countered: 2 damage.
    let mut t = TestGame::new(2);
    grave(&mut t, P0, "Lightning Bolt");
    grave(&mut t, P1, "Pyroclasm");
    t.set_step(P1, Step::PrecombatMain);
    let goyf = cast(&mut t, P1, "Tarmogoyf", "1G", &[]);
    assert_eq!(t.obj_now(goyf).chars.power, Some(2));
    cast_resolve(&mut t, P0, "Essence Backlash", "2UR", &[obj(goyf)]);
    assert!(t.in_graveyard(P1, "Tarmogoyf"));
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------------------
// Vanish into Memory: "Exile target creature. You draw cards equal to that creature's
// power. At the beginning of your next upkeep, return that card to the battlefield under
// its owner's control. If you do, discard cards equal to that creature's toughness."
// ---------------------------------------------------------------------------------------

#[test]
fn vanish_into_memory_a_returned_vehicle_or_a_token_discards_nothing() {
    cr!("208.3", "111.8", "603.7");
    ruling!(
        "Vanish into Memory",
        "If the creature exiled with Vanish into Memory isn't a creature card (for example, a crewed Vehicle), that card will still be returned to the battlefield, but not as a creature."
    );
    ruling!(
        "Vanish into Memory",
        "If the creature exiled with Vanish into Memory never returns to the battlefield (because it was a token creature, for example), you don't discard any cards."
    );
    supported("Vanish into Memory");
    // A crewed Copter (3/3): draw 3; it returns as a noncreature Vehicle; discard none.
    let mut t = TestGame::new(2);
    let copter = add(&mut t, P1, "Smuggler's Copter");
    let crew = add(&mut t, P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_choose(P1, &[obj(crew)]);
    activate_containing(&mut t, P1, copter, "Crew").expect("crew");
    t.resolve_all();
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P1, "Vanish into Memory", "2WU", &[obj(copter)]);
    // (P1 cast it: P1 draws.)
    let p1_hand = t.hand_size(P1);
    assert_eq!(t.hand_size(P0), hand);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let back = t.named_on_battlefield("Smuggler's Copter");
    assert_eq!(back.len(), 1);
    assert!(!is_creature(&t, back[0]));
    assert_eq!(t.hand_size(P1), p1_hand, "no cards discarded");
    // Control: a returned Hill Giant (3/3) makes its caster discard 3.
    let mut t = TestGame::new(2);
    let giant = add(&mut t, P1, "Hill Giant");
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Vanish into Memory", "2WU", &[obj(giant)]);
    assert_eq!(t.hand_size(P0), hand + 3);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.hand_size(P0), hand, "3 drawn, 3 discarded");
    // A token: drawn for, never returns, nothing discarded.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Raise the Alarm", "1W", &[]);
    let soldier = tokens(&t, P1)[0];
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Vanish into Memory", "2WU", &[obj(soldier)]);
    assert_eq!(t.hand_size(P0), hand + 1);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(tokens(&t, P1).len(), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------------
// Negative power: Phthisis, Infernal Reckoning, Impetuous Protege.
// ---------------------------------------------------------------------------------------

#[test]
fn phthisis_adds_negative_power_and_a_negative_total_does_nothing() {
    cr!("107.1b", "608.2h");
    ruling!(
        "Phthisis",
        "If the creature's power is negative, you still add its power and toughness to determine how much life is lost. If that total is negative, its controller doesn't gain or lose any life."
    );
    supported("Phthisis");
    supported("Pin to the Earth");
    // Pin to the Earth (-6/-0): Shivan Dragon is -1/5 (lose 4); Grizzly Bears -4/2 (none).
    for (name, loss) in [("Shivan Dragon", 4), ("Grizzly Bears", 0), ("Hill Giant", 0)] {
        let mut t = TestGame::new(2);
        let c = add(&mut t, P1, name);
        attach_new(&mut t, P0, "Pin to the Earth", obj(c));
        cast_resolve(&mut t, P0, "Phthisis", "3BBBB", &[obj(c)]);
        assert!(!t.on_battlefield(c));
        assert_eq!(t.life(P1), 20 - loss, "{name}");
    }
}

#[test]
fn phthisis_illegal_target_no_loss_indestructible_still_loses() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Phthisis",
        "If the target creature is an illegal target by the time Phthisis tries to resolve, the spell doesn't resolve. No player loses life. If the target is legal but not destroyed (most likely because it has indestructible), its controller does lose life."
    );
    let mut t = TestGame::new(2);
    let giant = add(&mut t, P1, "Hill Giant");
    cast(&mut t, P0, "Phthisis", "3BBBB", &[obj(giant)]);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Darksteel Myr (0/1, indestructible) pumped to 3/4: P1 loses 7.
    let myr = add(&mut t, P1, "Darksteel Myr");
    t.set_step(P1, Step::PrecombatMain);
    giant_growth(&mut t, myr);
    t.set_step(P0, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Phthisis", "3BBBB", &[obj(myr)]);
    assert!(t.on_battlefield(myr));
    assert_eq!(t.life(P1), 13);
}

#[test]
fn infernal_reckoning_negative_power_gains_nothing_and_uses_last_known_power() {
    cr!("107.1b", "608.2h", "119.3");
    ruling!(
        "Infernal Reckoning",
        "If the creature’s power is negative, you don’t lose or gain life."
    );
    ruling!(
        "Infernal Reckoning",
        "The amount of life gained is equal to the power of the creature as it last existed on the battlefield."
    );
    supported("Infernal Reckoning");
    // Ornithopter (0/2) under Pin to the Earth: -6/2.
    let mut t = TestGame::new(2);
    let thopter = add(&mut t, P1, "Ornithopter");
    attach_new(&mut t, P0, "Pin to the Earth", obj(thopter));
    assert_eq!(t.pt(thopter).0, -6);
    cast_resolve(&mut t, P0, "Infernal Reckoning", "B", &[obj(thopter)]);
    assert!(!t.on_battlefield(thopter));
    assert_eq!(t.life(P0), 20);
    // Pumped to 3/5 by Giant Growth: gain 3 (the exiled card is a 0/2).
    let thopter = add(&mut t, P1, "Ornithopter");
    t.set_step(P1, Step::PrecombatMain);
    giant_growth(&mut t, thopter);
    t.set_step(P0, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Infernal Reckoning", "B", &[obj(thopter)]);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn impetuous_protege_x_is_0_for_negative_or_no_tapped_creatures_and_is_locked_in() {
    cr!("107.1b", "608.2h", "611.2c");
    ruling!(
        "Impetuous Protege",
        "If the greatest power among tapped creatures your opponents control is negative, X is considered to be 0. The same is true if your opponents control no tapped creatures."
    );
    ruling!(
        "Impetuous Protege",
        "The value of X is determined only as Impetuous Protege’s last ability resolves."
    );
    supported("Impetuous Protege");
    // A tapped -4/2 Bears (Pin to the Earth) and an untapped Hill Giant: X = 0.
    let mut t = TestGame::new(2);
    let protege = add(&mut t, P0, "Impetuous Protege");
    let bears = add(&mut t, P1, "Grizzly Bears");
    add(&mut t, P1, "Hill Giant");
    attach_new(&mut t, P0, "Pin to the Earth", obj(bears));
    t.g.tap(bears);
    crate::r_s01_common::attack_with(&mut t, &[(protege, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(protege), (0, 4));
    // No tapped creatures: X = 0.
    let mut t = TestGame::new(2);
    let protege = add(&mut t, P0, "Impetuous Protege");
    add(&mut t, P1, "Hill Giant");
    crate::r_s01_common::attack_with(&mut t, &[(protege, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(protege), (0, 4));
    // A tapped Hill Giant: +3/+0, unchanged when the Giant then grows or dies.
    let mut t = TestGame::new(2);
    let protege = add(&mut t, P0, "Impetuous Protege");
    let giant = add(&mut t, P1, "Hill Giant");
    t.g.tap(giant);
    crate::r_s01_common::attack_with(&mut t, &[(protege, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(protege), (3, 4));
    giant_growth(&mut t, giant);
    assert_eq!(t.pt(protege), (3, 4));
    kill(&mut t, giant);
    assert_eq!(t.pt(protege), (3, 4));
}

// ---------------------------------------------------------------------------------------
// Illegal targets: Vraska's Stoneglare, Brightmare, Garruk, Sin.
// ---------------------------------------------------------------------------------------

#[test]
fn illegal_targets_mean_no_life_gain() {
    cr!("608.2b");
    ruling!(
        "Vraska's Stoneglare",
        "If the target creature becomes an illegal target for Vraska’s Stoneglare, the spell doesn’t resolve. You don’t gain life or search for Vraska, Regal Gorgon."
    );
    ruling!(
        "Brightmare",
        "If the target creature is an illegal target by the time Brightmare's ability tries to resolve, the ability won't resolve. You won't gain any life."
    );
    ruling!(
        "Garruk, Apex Predator",
        "If you activate the third ability, and the target creature is an illegal target when the ability tries to resolve, it won’t resolve and none of its effects will happen. You won’t gain any life."
    );
    supported("Vraska's Stoneglare");
    supported("Brightmare");
    supported("Garruk, Apex Predator");
    // Vraska's Stoneglare: no life, no search (no choice offered).
    let mut t = TestGame::new(2);
    t.library_top(P0, "Vraska, Regal Gorgon");
    let giant = add(&mut t, P1, "Hill Giant");
    cast(&mut t, P0, "Vraska's Stoneglare", "4BG", &[obj(giant)]);
    kill(&mut t, giant);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(!t.in_hand(P0, "Vraska, Regal Gorgon"));
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| matches!(d, Decision::Priority { .. })));
    // Control: legal, gain 3 and may search.
    let giant = add(&mut t, P1, "Hill Giant");
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Vraska's Stoneglare", "4BG", &[obj(giant)]);
    assert_eq!(t.life(P0), 23);
    // Brightmare: the target leaves; no life. Control: gain 3.
    let mut t = TestGame::new(2);
    let giant = add(&mut t, P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    enter(&mut t, P0, "Brightmare");
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    let giant = add(&mut t, P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    enter(&mut t, P0, "Brightmare");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert!(t.obj_now(giant).tapped);
    // Garruk's −3: the target leaves; no life. Control: gain 3.
    let mut t = TestGame::new(2);
    let garruk = add(&mut t, P0, "Garruk, Apex Predator");
    let giant = add(&mut t, P1, "Hill Giant");
    activate(&mut t, P0, garruk, "You gain life", "", &[obj(giant)]);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    let mut t = TestGame::new(2);
    let garruk = add(&mut t, P0, "Garruk, Apex Predator");
    let giant = add(&mut t, P1, "Hill Giant");
    activate_resolve(&mut t, P0, garruk, "You gain life", "", &[obj(giant)]);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn vraska_s_stoneglare_uses_last_known_or_current_toughness() {
    cr!("608.2h", "702.12b");
    ruling!(
        "Vraska's Stoneglare",
        "The amount of life gained is equal to the destroyed creature’s toughness as it last existed on the battlefield. If it wasn’t destroyed (most likely because it has indestructible), you gain life equal to its toughness as it currently exists on the battlefield."
    );
    // Grizzly Bears pumped to 5/5, destroyed: gain 5.
    let mut t = TestGame::new(2);
    let bears = add(&mut t, P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    giant_growth(&mut t, bears);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Vraska's Stoneglare", "4BG", &[obj(bears)]);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P0), 25);
    // Darksteel Myr pumped to 3/4 survives: gain 4.
    let myr = add(&mut t, P1, "Darksteel Myr");
    t.set_step(P1, Step::PrecombatMain);
    giant_growth(&mut t, myr);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Vraska's Stoneglare", "4BG", &[obj(myr)]);
    assert!(t.on_battlefield(myr));
    assert_eq!(t.life(P0), 29);
}

// ---------------------------------------------------------------------------------------
// "Destroyed" versus "dies": Noxious Gearhulk, Kalitas, Violet Pall.
// ---------------------------------------------------------------------------------------

#[test]
fn noxious_gearhulk_needs_a_destruction_not_a_graveyard() {
    cr!("701.8a", "702.12b", "614.6");
    ruling!(
        "Noxious Gearhulk",
        "If the target creature has indestructible, it isn't destroyed this way and you won't gain life. If it is destroyed but put into a zone other than a graveyard, you'll gain life."
    );
    supported("Noxious Gearhulk");
    supported("Rest in Peace");
    let mut t = TestGame::new(2);
    let myr = add(&mut t, P1, "Darksteel Myr");
    t.answer_targets(P0, &[obj(myr)]);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Noxious Gearhulk");
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.life(P0), 20);
    // Rest in Peace exiles the destroyed Hill Giant: still gain 3.
    let mut t = TestGame::new(2);
    add(&mut t, P1, "Rest in Peace");
    let giant = add(&mut t, P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Noxious Gearhulk");
    t.resolve_all();
    assert_eq!(exiled(&t, "Hill Giant"), 1);
    assert_eq!(t.life(P0), 23);
}

/// P0's Kalitas activates targeting `victim`, everything resolves.
fn kalitas_destroys(t: &mut TestGame, victim: ObjectId) {
    supported("Kalitas, Bloodchief of Ghet");
    let k = t.named_on_battlefield("Kalitas, Bloodchief of Ghet");
    let k = match k.first() {
        Some(k) => *k,
        None => add(t, P0, "Kalitas, Bloodchief of Ghet"),
    };
    t.g.objects[k.0 as usize].tapped = false;
    activate_resolve(t, P0, k, "Destroy", "BBB", &[obj(victim)]);
}

/// P0's creature tokens other than Kalitas's Vampires... (all P0's tokens).
fn vampires(t: &TestGame) -> Vec<ObjectId> {
    tokens(t, P0)
        .into_iter()
        .filter(|v| t.obj_now(*v).chars.has_subtype("Vampire"))
        .collect()
}

#[test]
fn kalitas_no_token_unless_the_creature_dies() {
    cr!("700.4", "702.12b", "701.19a", "614.6");
    ruling!(
        "Kalitas, Bloodchief of Ghet",
        "If the targeted creature has indestructible or regenerates, you won’t get a Vampire token. Similarly, if the targeted creature is destroyed but a replacement effect moves it to a different zone"
    );
    let mut t = TestGame::new(2);
    let myr = add(&mut t, P1, "Darksteel Myr");
    kalitas_destroys(&mut t, myr);
    assert!(t.on_battlefield(myr));
    assert_eq!(vampires(&t).len(), 0);
    // Drudge Skeletons regenerates.
    let skel = add(&mut t, P1, "Drudge Skeletons");
    t.set_step(P1, Step::PrecombatMain);
    activate_resolve(&mut t, P1, skel, "Regenerate", "B", &[]);
    t.set_step(P0, Step::PrecombatMain);
    kalitas_destroys(&mut t, skel);
    assert!(t.on_battlefield(skel));
    assert_eq!(vampires(&t).len(), 0);
    // Rest in Peace exiles it.
    add(&mut t, P1, "Rest in Peace");
    let giant = add(&mut t, P1, "Hill Giant");
    kalitas_destroys(&mut t, giant);
    assert_eq!(exiled(&t, "Hill Giant"), 1);
    assert_eq!(vampires(&t).len(), 0);
}

#[test]
fn kalitas_s_token_has_the_last_known_power_and_toughness_only_and_is_yours() {
    cr!("608.2h", "111.2");
    ruling!(
        "Kalitas, Bloodchief of Ghet",
        "The power and toughness of the token are based on the destroyed creature’s last existence on the battlefield. The token doesn’t have any of the destroyed creature’s other characteristics."
    );
    ruling!(
        "Kalitas, Bloodchief of Ghet",
        "You get the token, not the player who controlled the creature."
    );
    ruling!(
        "Violet Pall",
        "You get the token, not the player who controlled the creature."
    );
    supported("Violet Pall");
    // Serra Angel (4/4 flying, vigilance) pumped to 7/7.
    let mut t = TestGame::new(2);
    let angel = add(&mut t, P1, "Serra Angel");
    t.set_step(P1, Step::PrecombatMain);
    giant_growth(&mut t, angel);
    t.set_step(P0, Step::PrecombatMain);
    kalitas_destroys(&mut t, angel);
    assert!(t.in_graveyard(P1, "Serra Angel"));
    let v = vampires(&t);
    assert_eq!(v.len(), 1);
    assert_eq!(t.obj_now(v[0]).controller, P0);
    assert_eq!(t.pt(v[0]), (7, 7));
    let o = t.obj_now(v[0]);
    assert!(!o.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert!(!o.has_keyword(mtg_engine::keywords::KeywordKind::Vigilance));
    assert!(!o.chars.has_subtype("Angel"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
    assert_eq!(tokens(&t, P1).len(), 0);
    // Violet Pall: P0 gets the Faerie.
    let giant = add(&mut t, P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Violet Pall", "4B", &[obj(giant)]);
    assert!(!t.on_battlefield(giant));
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(tokens(&t, P1).len(), 0);
}

// ---------------------------------------------------------------------------------------
// Last known information: Banewasp Affliction, Kresh, Abattoir Ghoul.
// ---------------------------------------------------------------------------------------

#[test]
fn banewasp_affliction_the_last_controller_loses_its_last_known_toughness() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Banewasp Affliction",
        "The player who loses life is the player who controlled the creature when it was put into a graveyard. This may not be the player whose graveyard it was put into. That player loses life equal to the creature’s toughness as it last existed on the battlefield."
    );
    supported("Banewasp Affliction");
    // P1's Grizzly Bears, controlled by P0 and pumped to 5/5, enchanted by P1's Banewasp.
    let mut t = TestGame::new(2);
    let bears = add(&mut t, P1, "Grizzly Bears");
    give_control(&mut t, bears, P0);
    attach_new(&mut t, P1, "Banewasp Affliction", obj(bears));
    giant_growth(&mut t, bears);
    kill(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 15);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn kresh_uses_the_last_known_power() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Kresh the Bloodbraided",
        "X is equal to the power of the creature as it last existed on the battlefield."
    );
    supported("Kresh the Bloodbraided");
    let mut t = TestGame::new(2);
    let kresh = add(&mut t, P0, "Kresh the Bloodbraided");
    let bears = add(&mut t, P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    giant_growth(&mut t, bears);
    t.answer_yes(P0, true);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(kresh, counters::PLUS1), 5);
}

#[test]
fn abattoir_ghoul_uses_the_last_known_toughness() {
    cr!("603.10a", "510.4", "704.5g");
    ruling!(
        "Abattoir Ghoul",
        "You'll gain life equal to the creature's last known toughness before it died. For example, if Abattoir Ghoul deals 3 first-strike damage"
    );
    supported("Abattoir Ghoul");
    // Ghoul (3/2 first strike) blocked by Colossal Dreadmaw (6/6): 3 first-strike damage;
    // then Grasp of Darkness (-4/-4) makes it a 2/2 with 3 damage: it dies, gain 2.
    let mut t = TestGame::new(2);
    let ghoul = add(&mut t, P0, "Abattoir Ghoul");
    let maw = add(&mut t, P1, "Colossal Dreadmaw");
    crate::r_s01_common::attack_with(&mut t, &[(ghoul, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(maw, ghoul)]),
    );
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(dmg(&t, maw), 3);
    cast_resolve(&mut t, P0, "Grasp of Darkness", "BB", &[obj(maw)]);
    assert!(!t.on_battlefield(maw));
    assert_eq!(t.life(P0), 22);
    assert!(t.on_battlefield(ghoul));
}
