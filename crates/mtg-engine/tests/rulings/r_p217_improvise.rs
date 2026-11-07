//! Rulings batch P217 — improvise (CR 702.126): tapping artifacts pays only for generic
//! mana in the spell's total cost, and doesn't change its mana cost or mana value.

use crate::r_s01_common::supported;
use crate::r_s04_common::untapped_lands;
use crate::r_s08_common::mana_value;
use crate::r_s21_common::castable;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn artifacts(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.battlefield(p, "Ornithopter")).collect()
}

fn tapped(t: &TestGame, ids: &[ObjectId]) -> usize {
    ids.iter().filter(|id| t.obj_now(**id).tapped).count()
}

fn tap_for_improvise(t: &mut TestGame, p: PlayerId, ids: &[ObjectId]) {
    t.answer_choose(p, &ids.iter().map(|a| Entity::Object(*a)).collect::<Vec<_>>());
}

#[test]
fn improvise_cant_pay_for_metallic_rebukes_blue() {
    cr!("702.126a", "601.2h");
    ruling!(
        "Metallic Rebuke",
        "Improvise can't pay for {U} in Metallic Rebuke's total cost."
    );
    supported("Metallic Rebuke");
    // Three artifacts and no blue mana: Metallic Rebuke ({2}{U}) can't be cast.
    let mut t = TestGame::new(2);
    let orn = artifacts(&mut t, P0, 3);
    let rebuke = t.hand(P0, "Metallic Rebuke");
    assert!(!castable(&mut t, P0, rebuke));
    // With an Island, two artifacts pay the {2}.
    t.lands(P0, "Island", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    assert!(castable(&mut t, P0, rebuke));
    tap_for_improvise(&mut t, P0, &orn[..2]);
    t.cast(P0, rebuke).target(spell).go();
    assert_eq!(tapped(&t, &orn), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

/// The spell `name` costs `generic` generic mana plus `colored` (lands of `land`): with
/// `generic + colored.len()` artifacts and no lands it can't be cast; with `generic`
/// artifacts and the lands it can, and its mana value on the stack is unchanged.
fn not_below_colored(name: &str, land: &str, generic: usize, colored: usize, mv: i64) {
    supported(name);
    let mut t = TestGame::new(2);
    let orn = artifacts(&mut t, P0, generic + colored);
    let card = t.hand(P0, name);
    assert!(!castable(&mut t, P0, card));
    t.lands(P0, land, colored);
    tap_for_improvise(&mut t, P0, &orn[..generic]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P0, card).go();
    assert_eq!(tapped(&t, &orn), generic);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_eq!(mana_value(&t, spell), mv);
    assert_eq!(t.obj(spell).chars.mana_cost, card_cost(name));
}

fn card_cost(name: &str) -> Option<mtg_engine::mana::ManaCost> {
    mtg_engine::card::card(name).front().chars.mana_cost.clone()
}

#[test]
fn improvise_doesnt_change_bottle_cap_blasts_cost_or_reduce_it_below_red() {
    cr!("702.126a", "702.126b", "202.3", "601.2f");
    ruling!(
        "Bottle-Cap Blast",
        "Improvise doesn't change Bottle-Cap Blast's mana cost or mana value, and it can't reduce Bottle-Cap Blast's cost below {R}."
    );
    // {4}{R}.
    not_below_colored("Bottle-Cap Blast", "Mountain", 4, 1, 5);
}

#[test]
fn improvise_doesnt_change_synth_infiltrators_cost_or_reduce_it_below_uu() {
    cr!("702.126a", "702.126b", "202.3", "601.2f");
    ruling!(
        "Synth Infiltrator",
        "Improvise doesn’t change Synth Infiltrator’s mana cost or mana value, and it can’t reduce Synth Infiltrator’s cost below {U}{U}."
    );
    // {3}{U}{U}.
    not_below_colored("Synth Infiltrator", "Island", 3, 2, 5);
}

#[test]
fn with_x_choose_x_first_then_tap_artifacts_for_the_total_cost() {
    cr!("702.126a", "601.2b", "601.2f", "107.3a");
    ruling!(
        "Battle at the Bridge",
        "When using improvise to cast a spell with {X} in its mana cost, first choose the value for X. That choice, plus any cost increases or decreases, will determine the spell’s total cost. Then you can tap artifacts you control to help pay that cost."
    );
    supported("Battle at the Bridge");
    // Battle at the Bridge ({X}{B}) with X = 3: the total cost is {3}{B}. Two artifacts
    // tapped leave {1}{B} to pay: a Swamp and a Wastes.
    let mut t = TestGame::new(2);
    let orn = artifacts(&mut t, P0, 2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let battle = t.hand(P0, "Battle at the Bridge");
    tap_for_improvise(&mut t, P0, &orn);
    t.cast(P0, battle).x(3).target(giant).go();
    assert_eq!(tapped(&t, &orn), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
    assert_eq!(t.life(P0), 23);
    // With only the two artifacts and a Swamp, X = 3 can't be paid.
    let mut t = TestGame::new(2);
    let orn = artifacts(&mut t, P0, 2);
    t.lands(P0, "Swamp", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let battle = t.hand(P0, "Battle at the Bridge");
    tap_for_improvise(&mut t, P0, &orn);
    assert!(t.cast(P0, battle).x(3).target(giant).try_go().is_err());
    assert_eq!(t.zone(battle), Zone::Hand(P0));
}
