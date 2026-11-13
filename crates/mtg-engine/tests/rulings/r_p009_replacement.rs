//! Rulings batch P009 — burn with replacement and prevention riders: "if it would die this
//! turn, exile it instead" (CR 614.1a, 700.4), effects that don't depend on the damage
//! being dealt (CR 608.2c, 615), "damage can't be prevented" (CR 614.16), and "players
//! can't gain life" against setting a life total (CR 119.5, 119.7).

use crate::r_p009_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, attach_new, damage};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 casts Healing Salve: prevent the next 3 damage that would be dealt to `to`.
fn salve(t: &mut TestGame, to: Entity) {
    t.lands(P1, "Plains", 1);
    let s = t.hand(P1, "Healing Salve");
    t.g.turn.priority = Some(P1);
    t.cast(P1, s).modes(&[1]).target(to).go();
    t.resolve();
}

#[test]
fn would_die_this_turn_exiles_it_for_any_reason() {
    cr!("614.1a", "700.4");
    ruling!(
        "Chandra, Awakened Inferno",
        "The replacement effect of Chandra's last ability will exile the target creature or planeswalker if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Fanged Flames",
        "The replacement effect of Fanged Flames will exile the target creature or planeswalker if it would die this turn for any reason, not just due to lethal damage."
    );
    supported("Chandra, Awakened Inferno");
    supported("Fanged Flames");
    // Chandra −1 on a 6/6, then it's destroyed: exiled.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Awakened Inferno");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    t.answer_targets(P0, &[obj(dreadmaw)]);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    activate_containing(&mut t, P0, chandra, "exile it instead").unwrap();
    t.resolve_all();
    assert_eq!(dmg(&t, dreadmaw), 1);
    kill(&mut t, dreadmaw);
    assert_eq!(t.zone(dreadmaw), mtg_engine::object::Zone::Exile);
    // Fanged Flames on a 6/6, then it's destroyed: exiled.
    let mut t = TestGame::new(2);
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Fanged Flames", &[obj(dreadmaw)]);
    t.resolve_all();
    assert_eq!(dmg(&t, dreadmaw), 4);
    kill(&mut t, dreadmaw);
    assert_eq!(t.zone(dreadmaw), mtg_engine::object::Zone::Exile);
}

#[test]
fn brutal_expulsion_exiles_even_if_its_damage_is_prevented() {
    cr!("614.1a", "608.2c", "615.1");
    ruling!(
        "Brutal Expulsion",
        "The second mode will exile the target creature or planeswalker if it would be put into the graveyard this turn for any reason, not just due to lethal damage. The exile effect applies to that permanent even if Brutal Expulsion deals no damage to it (due to a prevention effect)"
    );
    supported("Brutal Expulsion");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P1, "Inviolability", obj(bears));
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 3);
    let be = t.hand(P0, "Brutal Expulsion");
    t.cast(P0, be).modes(&[1]).target(bears).go();
    t.resolve_all();
    assert_eq!(dmg(&t, bears), 0);
    kill(&mut t, bears);
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Exile);
}

#[test]
fn discard_happens_even_if_the_damage_is_prevented() {
    cr!("608.2c", "615.1");
    ruling!(
        "Rakdos's Return",
        "The target opponent will discard X cards even if some or all of the damage dealt by Rakdos’s Return is prevented or redirected."
    );
    ruling!(
        "Blightning",
        "The targeted player will discard two cards even if some or all of Blightning's damage is prevented or redirected."
    );
    supported("Rakdos's Return");
    supported("Blightning");
    // Blightning: 3 damage prevented, two cards discarded.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    cast_new(&mut t, P0, "Blightning", &[pl(P1)]);
    salve(&mut t, pl(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), 1);
    // Rakdos's Return, X = 2: damage prevented, two cards discarded.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 3);
    let rr = t.hand(P0, "Rakdos's Return");
    t.cast(P0, rr).x(2).target(pl(P1)).go();
    salve(&mut t, pl(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn unstable_footing_makes_all_damage_unpreventable() {
    cr!("614.16", "702.16e");
    ruling!(
        "Unstable Footing",
        "Unstable Footing's effect causes damage from all sources to be unpreventable, not just damage from Unstable Footing itself."
    );
    supported("Unstable Footing");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let corpse = t.battlefield(P0, "Walking Corpse");
    let uf = t.hand(P0, "Unstable Footing");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, uf).kicked(false).go();
    t.resolve_all();
    damage(&mut t, corpse, 1, knight);
    assert_eq!(dmg(&t, knight), 1);
}

#[test]
fn sunspine_lynx_stops_setting_a_life_total_higher() {
    cr!("119.5", "119.7");
    ruling!(
        "Sunspine Lynx",
        "If an effect says to set a player’s life total to a number that’s higher than the player’s current life total while Sunspine Lynx is on the battlefield, the player’s life total doesn’t change."
    );
    supported("Sunspine Lynx");
    supported("Blessed Wind");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sunspine Lynx");
    t.g.players[0].life = 10;
    cast_new(&mut t, P0, "Blessed Wind", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 10);
    // Without the Lynx, the life total becomes 20.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 10;
    cast_new(&mut t, P0, "Blessed Wind", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}
