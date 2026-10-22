//! Rulings batch S30 — damage and life totals (CR 119, 120): damage that can't be
//! prevented to a player whose life total can't change, commander damage that doesn't
//! change a life total, damage below 0 life, losing life from damage or payment, and
//! regeneration when damage can't be prevented.

use crate::r_s01_common::supported;
use crate::r_s25_common::cast_new;
use crate::r_s30_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Makes `id` a commander (the designation of its card, CR 903.3).
fn make_commander(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let owner = t.obj(id).owner;
    let name = t.obj(id).card.as_ref().unwrap().name.clone();
    t.g.players[owner.idx()].commander_names.push(name);
}

#[test]
fn unpreventable_damage_to_a_protected_player_still_has_its_other_effects() {
    cr!("702.16e", "615.12", "119.8", "702.15b", "702.90b");
    ruling!(
        "Teferi's Protection",
        "Protection from everything will usually prevent damage if it would be dealt to you, but some damage can't be prevented. In this case, because your life total also can't change, that damage has any other effects that it may have aside from causing you to lose that much life (such as effects from lifelink or infect) and triggers and effects can see that damage was dealt even though your life total didn't change."
    );
    supported("Teferi's Protection");
    supported("Unstable Footing");
    let mut t = TestGame::new(2);
    let nighthawk = t.battlefield(P1, "Vampire Nighthawk");
    let elf = t.battlefield(P1, "Glistener Elf");
    // P0: "Until your next turn, your life total can't change and you gain protection from
    // everything."
    cast_new(&mut t, P0, "Teferi's Protection", &[]);
    t.resolve_all();
    // P1's turn: "Damage can't be prevented this turn." Then P1 attacks P0.
    t.advance_to(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Unstable Footing", &[]);
    t.resolve_all();
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(
        &[(nighthawk, Entity::Player(P0)), (elf, Entity::Player(P0))],
        &[],
    );
    // The damage was dealt (and seen as dealt); P0's life total didn't change, but
    // lifelink and infect had their effects.
    let dealt: u32 = damage_events(&t)
        .iter()
        .filter(|(_, to, _, combat)| *to == Entity::Player(P0) && *combat)
        .map(|(_, _, n, _)| *n)
        .sum();
    assert_eq!(dealt, 3);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 22);
    assert_eq!(t.g.player(P0).counter(counters::POISON), 1);
    // Without Unstable Footing, the damage is prevented: no lifelink, no poison.
    let mut t = TestGame::new(2);
    let nighthawk = t.battlefield(P1, "Vampire Nighthawk");
    let elf = t.battlefield(P1, "Glistener Elf");
    cast_new(&mut t, P0, "Teferi's Protection", &[]);
    t.resolve_all();
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(
        &[(nighthawk, Entity::Player(P0)), (elf, Entity::Player(P0))],
        &[],
    );
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.g.player(P0).counter(counters::POISON), 0);
}

#[test]
fn commander_damage_is_tracked_even_when_the_life_total_doesnt_change() {
    cr!("903.10a", "614.1a");
    ruling!(
        "Angel's Grace",
        "In a Commander game, combat damage you're dealt by a commander is still tracked, even if it doesn't change your life total."
    );
    supported("Angel's Grace");
    let mut t = crate::r_s13_common::commander_game();
    let giant = t.battlefield(P1, "Hill Giant");
    make_commander(&mut t, giant);
    set_life(&mut t, P0, 1);
    t.set_step(P1, Step::PrecombatMain);
    // "Until end of turn, damage that would reduce your life total to less than 1 reduces
    // it to 1 instead."
    cast_new(&mut t, P0, "Angel's Grace", &[]);
    t.resolve_all();
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 1);
    assert!(!t.has_lost(P0));
    assert_eq!(
        t.g.player(P0).commander_damage.get("Hill Giant").copied(),
        Some(3)
    );
}

#[test]
fn damage_to_a_player_below_1_life_reduces_it_further() {
    cr!("119.3", "120.3a", "614.1a");
    ruling!(
        "Angel of Grace",
        "If you have less than 1 life and somehow haven't lost the game, damage dealt to you reduces your life total further below 0 (as normal)."
    );
    supported("Angel of Grace");
    let mut t = TestGame::new(2);
    // Platinum Angel: "You can't lose the game and your opponents can't win the game."
    t.battlefield(P0, "Platinum Angel");
    set_life(&mut t, P0, 3);
    // "When this creature enters, until end of turn, damage that would reduce your life
    // total to less than 1 reduces it to 1 instead."
    t.enter(P0, "Angel of Grace");
    t.resolve_all();
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 1);
    // With less than 1 life, damage reduces it further.
    set_life(&mut t, P0, -2);
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), -4);
    assert!(!t.has_lost(P0));
}

#[test]
fn damage_and_paying_life_both_make_an_opponent_lose_life() {
    cr!("120.3a", "119.4", "119.3");
    ruling!(
        "Exquisite Blood",
        "Damage dealt to an opponent usually causes that opponent to lose life. An opponent paying life also causes loss of life."
    );
    supported("Exquisite Blood");
    let mut t = TestGame::new(2);
    // "Whenever an opponent loses life, you gain that much life."
    t.battlefield(P0, "Exquisite Blood");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    // Greed: "{B}, Pay 2 life: Draw a card."
    let greed = t.battlefield(P1, "Greed");
    t.lands(P1, "Swamp", 1);
    t.activate(P1, greed, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (24, 16));
}

#[test]
fn a_creature_dealt_lethal_damage_can_regenerate_when_damage_cant_be_prevented() {
    cr!("701.19a", "614.8", "615.12");
    ruling!(
        "Leyline of Punishment",
        "If a creature is dealt lethal damage, it can still regenerate. If it does, the damage marked on it will be removed from it."
    );
    supported("Leyline of Punishment");
    let mut t = TestGame::new(2);
    // "Players can't gain life. Damage can't be prevented."
    t.battlefield(P1, "Leyline of Punishment");
    // River Boa: "{G}: Regenerate River Boa."
    let boa = t.battlefield(P0, "River Boa");
    t.lands(P0, "Forest", 1);
    t.activate(P0, boa, 0, &[]).unwrap();
    t.resolve_all();
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Object(boa)]);
    t.resolve_all();
    assert!(t.on_battlefield(boa));
    assert_eq!(t.obj_now(boa).damage, 0);
    assert!(t.obj_now(boa).tapped);
}
