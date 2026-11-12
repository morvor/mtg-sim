//! CR 615 audit: prevention effects against damage that can't be prevented. Such damage
//! is dealt; a shield applied to it isn't reduced or used up, so it still prevents later
//! damage (615.12, 609.7b); and "can't be prevented" doesn't stop redirection, which isn't
//! prevention (614.9).

use crate::r614_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_cop_shield_isnt_used_up_by_unpreventable_damage_and_prevents_the_next() {
    cr!("615.12", "609.7b", "615.8");
    supported("Circle of Protection: Red");
    supported("Prodigal Pyromancer");
    supported("Leyline of Punishment");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let leyline = t.battlefield(P0, "Leyline of Punishment");
    let cop = t.battlefield(P1, "Circle of Protection: Red");
    t.lands(P1, "Plains", 1);
    t.answer_choose(P1, &[obj(pyro)]);
    t.activate(P1, cop, 0, &[]).expect("activate the Circle");
    t.resolve_all();
    // Damage can't be prevented: the Pyromancer's 1 damage is dealt, and the shield isn't
    // used up.
    t.activate(P0, pyro, 0, &[Entity::Player(P1)])
        .expect("ping");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Without the Leyline, the same shield prevents the next damage from that source.
    cast_resolve(&mut t, P1, "Disenchant", &[obj(leyline)]);
    t.g.untap(pyro);
    t.activate(P0, pyro, 0, &[Entity::Player(P1)])
        .expect("ping again");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // ... and then it's gone.
    t.g.untap(pyro);
    t.activate(P0, pyro, 0, &[Entity::Player(P1)])
        .expect("ping a third time");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_healing_salve_shield_isnt_reduced_by_banefire_and_prevents_a_later_bolt() {
    cr!("615.12", "615.7");
    supported("Healing Salve");
    supported("Banefire");
    let mut t = TestGame::new(2);
    // Healing Salve's second mode: prevent the next 3 damage to P1 this turn.
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.answer_targets(P1, &[Entity::Player(P1)]);
    t.cast(P1, salve).modes(&[1]).go();
    t.resolve_all();
    // Banefire with X = 5: its damage can't be prevented.
    t.lands(P0, "Mountain", 6);
    let bf = t.hand(P0, "Banefire");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bf).x(5).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // The shield is still 3.
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 15);
    cast_resolve(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn unpreventable_damage_is_still_redirected_by_palisade_giant() {
    cr!("615.12", "614.9");
    supported("Skullcrack");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Palisade Giant");
    // Skullcrack: damage can't be prevented this turn, and 3 damage to target player.
    cast_resolve(&mut t, P0, "Skullcrack", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage_on(&t, giant), 3);
}

#[test]
fn protection_doesnt_prevent_damage_that_cant_be_prevented() {
    cr!("615.12", "702.16e");
    supported("Kor Firewalker");
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P1, "Kor Firewalker");
    t.battlefield(P0, "Leyline of Punishment");
    cast_resolve(&mut t, P0, "Pyroclasm", &[]);
    assert!(!t.on_battlefield(walker));
    assert!(t.in_graveyard(P1, "Kor Firewalker"));
}

#[test]
fn a_prevention_choice_is_offered_for_unpreventable_damage_but_prevents_none() {
    cr!("615.12", "615.12a", "616.1");
    supported("Urza's Armor");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Urza's Armor");
    t.battlefield(P1, "Urza's Armor");
    t.battlefield(P0, "Leyline of Punishment");
    t.answer(P1, DecisionKind::Replacement, Answer::Index(0));
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    // Each Armor is applied once, preventing nothing (no endless loop).
    assert_eq!(t.life(P1), 17);
}

/// P0 controls Vigor and Grizzly Bears; P1 controls Furnace of Rath and Bolts the Bears;
/// P0's answers are `picks`.
fn bolt_bears_under_vigor_and_furnace(picks: &[usize]) -> TestGame {
    supported("Vigor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vigor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Furnace of Rath");
    pick_replacements(&mut t, P0, picks);
    cast_resolve(&mut t, P1, "Lightning Bolt", &[obj(bears)]);
    t
}

#[test]
fn vigor_counts_the_damage_it_prevents_after_furnace_doubles_it() {
    cr!("615.5", "616.1", "616.1e", "616.1f");
    let probe = bolt_bears_under_vigor_and_furnace(&[]);
    let (who, opts) = replacement_choices(&probe)[0].clone();
    assert_eq!(who, P0);
    let bears = |t: &TestGame| t.named_on_battlefield("Grizzly Bears")[0];
    // Doubled first, then all 6 prevented: six counters.
    let t = bolt_bears_under_vigor_and_furnace(&[option_index(&opts, "Furnace of Rath")]);
    assert_eq!(plus_counters(&t, bears(&t)), 6);
    assert_eq!(damage_on(&t, bears(&t)), 0);
    // Prevented first: 3 counters, and nothing is left for the Furnace to double.
    let t = bolt_bears_under_vigor_and_furnace(&[option_index(&opts, "Vigor")]);
    assert_eq!(plus_counters(&t, bears(&t)), 3);
    assert_eq!(replacement_choices(&t).len(), 1);
}

/// Both players control Gisela, Blade of Goldnight; P0 Bolts P1, whose answers are `picks`.
fn bolt_between_giselas(picks: &[usize]) -> TestGame {
    supported("Gisela, Blade of Goldnight");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gisela, Blade of Goldnight");
    t.battlefield(P1, "Gisela, Blade of Goldnight");
    pick_replacements(&mut t, P1, picks);
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t
}

#[test]
fn halving_before_doubling_is_the_damaged_players_choice() {
    cr!("616.1", "616.1e", "615.1a");
    let probe = bolt_between_giselas(&[]);
    let (who, opts) = replacement_choices(&probe)[0].clone();
    assert_eq!(who, P1);
    // Prevent half of 3 rounded up (2), then double the 1 left: 2.
    let t = bolt_between_giselas(&[option_index(&opts, "prevent half")]);
    assert_eq!(t.life(P1), 18);
    // Double to 6, then prevent 3: 3.
    let t = bolt_between_giselas(&[option_index(&opts, "double")]);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn questing_beast_makes_only_combat_damage_unpreventable() {
    cr!("615.12", "120.2a", "120.2b");
    supported("Questing Beast");
    supported("Holy Day");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Questing Beast");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    // P1 casts Holy Day and Healing Salve (prevent the next 3 damage to P1).
    cast_resolve(&mut t, P1, "Holy Day", &[]);
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.answer_targets(P1, &[Entity::Player(P1)]);
    t.cast(P1, salve).modes(&[1]).go();
    t.resolve_all();
    // The Pyromancer's noncombat damage is prevented by the shield.
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).expect("ping");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // The Bears' combat damage can't be prevented: neither Holy Day nor what's left of
    // the shield prevents it, and the shield isn't reduced.
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    cast_resolve(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 18, "2 of the shield's 2 remaining prevent the Shock");
}

#[test]
fn a_cop_shield_on_a_sacrificed_mogg_fanatic_prevents_its_damage() {
    cr!("609.7a", "615.8");
    supported("Mogg Fanatic");
    let mut t = TestGame::new(2);
    let fanatic = t.battlefield(P0, "Mogg Fanatic");
    let cop = t.battlefield(P1, "Circle of Protection: Red");
    t.lands(P1, "Plains", 1);
    t.answer_choose(P1, &[obj(fanatic)]);
    t.activate(P1, cop, 0, &[]).expect("activate the Circle");
    t.resolve_all();
    // The Fanatic is sacrificed to pay for its ability; it deals the damage as it last
    // existed, and it's still the source the shield was created for.
    t.activate(P0, fanatic, 0, &[Entity::Player(P1)])
        .expect("sacrifice the Fanatic");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mogg Fanatic"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_shield_on_a_creature_doesnt_protect_it_after_it_returns_as_a_new_object() {
    cr!("615.7", "400.7");
    supported("Cloudshift");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let salve = t.hand(P0, "Healing Salve");
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, salve).modes(&[1]).go();
    t.resolve_all();
    cast_resolve(&mut t, P0, "Cloudshift", &[obj(bears)]);
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    cast_resolve(&mut t, P1, "Shock", &[obj(bears)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn damage_pariah_redirects_to_a_protected_creature_is_then_prevented() {
    cr!("616.2", "614.9", "702.16e");
    supported("Pariah");
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P0, "Kor Firewalker");
    let pariah = t.battlefield(P0, "Pariah");
    t.g.attach(pariah, obj(walker));
    // (P0 declines the Firewalker's "you may gain 1 life" for the red spell.)
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    // Redirected to the Firewalker, the damage is from a red source: protection from red
    // now applies to it.
    assert_eq!(t.life(P0), 20);
    assert_eq!(damage_on(&t, walker), 0);
    assert!(t.on_battlefield(walker));
}
