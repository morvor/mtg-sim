//! CR 701.10: double; CR 701.11: triple.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn pump(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    run(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![id])),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
            duration: Duration::EndOfTurn,
        },
    );
}

#[test]
fn doubling_power_gives_plus_x_where_x_is_its_power_as_it_resolves() {
    cr!("701.10", "701.10a", "701.10b");
    supported("Unleash Fury");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let giant = t.battlefield(P0, "Hill Giant");
    pump(&mut t, giant, 1, 1);
    assert_eq!(t.pt(giant), (4, 4));
    let fury = t.hand(P0, "Unleash Fury");
    t.cast(P0, fury).target(giant).go();
    t.resolve();
    assert_eq!(t.pt(giant), (8, 4));
    // X was locked in: a later pump isn't doubled.
    pump(&mut t, giant, 1, 0);
    assert_eq!(t.pt(giant), (9, 4));
    // It modifies power rather than setting it: an effect setting its base power still
    // leaves the +4 (CR 613.4c).
    run(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![giant])),
            mods: vec![Modification::SetPT(Some(Value::c(1)), Some(Value::c(1)))],
            duration: Duration::EndOfTurn,
        },
    );
    assert_eq!(t.pt(giant), (1 + 1 + 4 + 1, 1 + 1));
}

#[test]
fn doubling_power_and_toughness_of_each_creature() {
    cr!("701.10b");
    supported("Double Trouble");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // "Double the power of each creature you control until end of turn."
    let s = t.hand(P0, "Double Trouble");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(giant), (6, 3));
    assert_eq!(t.pt(theirs), (2, 2));
    // "Double target creature's power and toughness" (custom card).
    let def = oracle_card(
        "Swell",
        "Instant",
        "{0}",
        None,
        "Double target creature's power and toughness until end of turn.",
    );
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, s).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (8, 4));
}

#[test]
fn doubling_a_negative_power_makes_it_more_negative() {
    cr!("701.10c");
    supported("Unleash Fury");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let giant = t.battlefield(P0, "Hill Giant");
    pump(&mut t, giant, -5, 0);
    assert_eq!(t.pt(giant), (-2, 3));
    let fury = t.hand(P0, "Unleash Fury");
    t.cast(P0, fury).target(giant).go();
    t.resolve();
    assert_eq!(t.pt(giant), (-4, 3));
    // One negative, one positive: -X/+Y.
    let def = oracle_card(
        "Swell",
        "Instant",
        "{0}",
        None,
        "Double target creature's power and toughness until end of turn.",
    );
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, s).target(giant).go();
    t.resolve();
    assert_eq!(t.pt(giant), (-8, 6));
}

#[test]
fn doubling_a_life_total() {
    cr!("701.10d");
    supported("Beacon of Immortality");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    t.g.players[0].life = 7;
    let beacon = t.hand(P0, "Beacon of Immortality");
    t.cast(P0, beacon).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.g.history.life_gained.get(&P0).copied(), Some(7));
    // A negative life total doubles by losing life.
    let mut t = TestGame::new(2);
    t.g.players[1].life = -3;
    let def = card("Beacon of Immortality");
    let effect = match &def.faces[0].chars.abilities[0].kind {
        AbilityKind::Spell(s) => s.body.effect.clone(),
        _ => unreachable!(),
    };
    run_targeted(&mut t, P0, None, effect, vec![vec![Entity::Player(P1)]]);
    assert_eq!(t.life(P1), -6);
}

#[test]
fn doubling_mana_adds_as_much_of_each_type() {
    cr!("701.10f");
    supported("Doubling Cube");
    let mut t = TestGame::new(2);
    t.g.players[0].mana_pool.add_type(ManaType::R, 2);
    t.g.players[0].mana_pool.add_type(ManaType::G, 1);
    let def = card("Doubling Cube");
    let effect = match &def.faces[0].chars.abilities[0].kind {
        AbilityKind::Activated(a) => a.body.effect.clone(),
        _ => unreachable!(),
    };
    run(&mut t, P0, None, effect);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 4);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 2);
}

#[test]
fn doubling_damage_is_a_replacement_effect() {
    cr!("701.10g");
    supported("Angrath's Marauders");
    supported("Shock");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angrath's Marauders");
    t.lands(P0, "Mountain", 3);
    let giant = t.battlefield(P1, "Hill Giant");
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(giant).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // An opponent's source isn't doubled.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 18);
    // Two doublers: each applies (CR 616.1): four times the damage.
    t.battlefield(P0, "Angrath's Marauders");
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 8);
}

#[test]
fn tripling_power_and_toughness() {
    cr!("701.11", "701.11a", "701.11b", "701.11c");
    let def = oracle_card(
        "Surge",
        "Instant",
        "{0}",
        None,
        "Triple target creature's power and toughness until end of turn.",
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let s = t.custom(P0, def.clone(), Zone::Hand(P0));
    t.cast(P0, s).target(giant).go();
    t.resolve();
    assert_eq!(t.pt(giant), (9, 9));
    // +6/+6, not "is 9/9": a later +1/+1 applies on top.
    pump(&mut t, giant, 1, 1);
    assert_eq!(t.pt(giant), (10, 10));
    // Negative power: -X where X is twice the difference.
    let bears = t.battlefield(P0, "Grizzly Bears");
    pump(&mut t, bears, -3, 0);
    assert_eq!(t.pt(bears), (-1, 2));
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, s).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (-3, 6));
}

#[test]
fn tripling_damage() {
    cr!("701.11");
    supported("Fiery Emancipation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fiery Emancipation");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 14);
}
