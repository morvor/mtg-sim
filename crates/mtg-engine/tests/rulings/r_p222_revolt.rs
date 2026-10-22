//! Rulings batch P222 — revolt (an ability word, CR 207.2c) and energy (CR 107.14):
//! Aether Revolt, Decommission, and the other "whenever you get one or more {E}" cards.

use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s03_common::{in_hand_with_mana, respond};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn energy(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::ENERGY)
}

/// `p` casts the real card `name` from hand with the lands to pay for it.
fn cast(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let c = in_hand_with_mana(t, p, name);
    t.g.turn.priority = Some(p);
    t.cast_with(p, c, targets)
        .unwrap_or_else(|e| panic!("casting {name}: {e:?}"))
}

/// Makes a permanent leave the battlefield under P0's control this turn (enables revolt).
fn revolt(t: &mut TestGame) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(t, bears);
}

/// The damage events dealt to `to` since event index `from` of this turn: (source, amount).
fn damage_to(t: &TestGame, to: Entity) -> Vec<(ObjectId, u32)> {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                ..
            } if *target == to => Some((*source, *amount)),
            _ => None,
        })
        .collect()
}

#[test]
fn aether_revolt_triggers_once_for_energy_gotten_at_once() {
    cr!("603.2", "122.1", "107.14");
    ruling!("Aether Revolt", "If you get multiple {E} at once, Aether Revolt's last ability will trigger only once. When the triggered ability resolves, it will deal damage to the target equal to the amount of {E} you gained.");
    supported("Aether Revolt");
    supported("Attune with Aether");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Forest");
    t.battlefield(P0, "Aether Revolt");
    cast(&mut t, P0, "Attune with Aether", &[]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(energy(&t, P0), 2);
    assert_eq!(t.stack_len(), 1, "one trigger");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn aether_revolt_sees_the_energy_gotten_even_if_it_is_then_spent() {
    cr!("603.2", "608.2c", "107.14");
    ruling!("Aether Revolt", "If an effect instructs you to get one or more {E} and then allows you to spend {E}, Aether Revolt's last ability will see the amount of {E} you got. It doesn't matter how much {E} you spend after that.");
    supported("Aether Revolt");
    // A spell that gets {E}{E}{E} and then may spend two of them.
    let spend = custom_card(
        "Spend Test",
        "Sorcery",
        "{U}",
        None,
        "You get {E}{E}{E}. Then you may pay {E}{E}. If you do, draw a card.",
    );
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Aether Revolt");
        let c = t.custom(P0, spend.clone(), Zone::Hand(P0));
        t.lands(P0, "Island", 1);
        t.answer_yes(P0, pay);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let hand = t.hand_size(P0);
        t.cast(P0, c).go();
        t.resolve();
        assert_eq!(energy(&t, P0), if pay { 1 } else { 3 });
        assert_eq!(t.hand_size(P0), hand - 1 + pay as usize);
        t.resolve_all();
        assert_eq!(t.life(P1), 17, "pay {pay}");
    }
}

#[test]
fn other_get_energy_triggers() {
    cr!("603.2", "107.14", "122.1");
    supported("Fabrication Module");
    supported("Territorial Gorger");
    supported("Brotherhood Scribe");
    supported("Attune with Aether");
    // Fabrication Module: one +1/+1 counter for getting {E}{E} at once; its own ability
    // gets {E}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Forest");
    let module = t.battlefield(P0, "Fabrication Module");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[bears.into()]);
    cast(&mut t, P0, "Attune with Aether", &[]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[bears.into()]);
    t.activate(P0, module, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(energy(&t, P0), 3);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    // Territorial Gorger gets +2/+2 once.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Forest");
    let gorger = t.battlefield(P0, "Territorial Gorger");
    cast(&mut t, P0, "Attune with Aether", &[]);
    t.resolve_all();
    assert_eq!(t.pt(gorger), (4, 4));
    // An opponent getting energy doesn't trigger it.
    t.set_step(P1, Step::PrecombatMain);
    t.library_top(P1, "Forest");
    cast(&mut t, P1, "Attune with Aether", &[]);
    t.resolve_all();
    assert_eq!(energy(&t, P1), 2);
    assert_eq!(t.pt(gorger), (4, 4));
    // Brotherhood Scribe: only during your turn. Its metalcraft ability gets {E}.
    for your_turn in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(if your_turn { P0 } else { P1 }, Step::PrecombatMain);
        let scribe = t.battlefield(P0, "Brotherhood Scribe");
        for _ in 0..2 {
            t.battlefield(P0, "Ornithopter");
        }
        // Metalcraft: not with only two artifacts.
        assert!(!crate::r_s02_common::can_activate(&mut t, P0, scribe));
        t.battlefield(P0, "Ornithopter");
        t.activate(P0, scribe, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(energy(&t, P0), 1);
        let pt = if your_turn { (2, 4) } else { (1, 3) };
        assert_eq!(t.pt(scribe), pt, "your turn {your_turn}");
    }
}

#[test]
fn aether_revolt_applies_for_the_rest_of_the_turn_while_it_is_on_the_battlefield() {
    cr!("614.1a", "611.3a");
    ruling!("Aether Revolt", "Once a permanent you control leaves the battlefield, Aether Revolt's revolt ability will apply to noncombat damage dealt by sources you control until the turn ends or Aether Revolt leaves the battlefield, whichever happens first. It doesn't matter what happens to the card or token that left the battlefield afterward.");
    supported("Aether Revolt");
    // Not before a permanent left.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ar = t.battlefield(P0, "Aether Revolt");
    cast(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // A token left (and ceased to exist): +2 for the rest of the turn.
    let token = create_token(&mut t, P0, "Soldier");
    destroy(&mut t, token);
    cast(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // Also to a permanent an opponent controls (not to P0's own).
    let giant = t.battlefield(P1, "Colossal Dreadmaw");
    cast(&mut t, P0, "Shock", &[giant.into()]);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 4);
    let mine = t.battlefield(P0, "Colossal Dreadmaw");
    cast(&mut t, P0, "Shock", &[mine.into()]);
    t.resolve_all();
    assert_eq!(t.obj_now(mine).damage, 2);
    // Aether Revolt leaves: no more.
    destroy(&mut t, ar);
    cast(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    // Ends with the turn.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Aether Revolt");
    revolt(&mut t);
    t.advance_to(P1, Step::Upkeep);
    cast(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn aether_revolt_extra_damage_is_dealt_by_the_original_source() {
    cr!("614.1a", "120.2b");
    ruling!("Aether Revolt", "The additional damage is dealt by the original source of the damage, not by Aether Revolt.");
    supported("Aether Revolt");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ar = t.battlefield(P0, "Aether Revolt");
    revolt(&mut t);
    let shock = cast(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    let dealt = damage_to(&t, Entity::Player(P1));
    assert_eq!(dealt, vec![(shock, 4)]);
    assert!(dealt.iter().all(|(s, _)| *s != ar));
}

fn prevention_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(d, true)
}

fn revolt_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(d, false)
}

fn pick_replacement(d: &Decision, prevent: bool) -> Option<Answer> {
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| o.starts_with("Aether Revolt") != prevent)
            .map(Answer::Index),
        _ => None,
    }
}

#[test]
fn aether_revolt_and_prevention_the_damaged_player_orders_them() {
    cr!("616.1", "616.1e", "615.1a");
    ruling!("Aether Revolt", "If another effect modifies how much damage a source would deal to an opponent or a permanent they control, including preventing some of it, the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Aether Revolt's effect no longer applies.");
    supported("Aether Revolt");
    supported("Healing Salve");
    for prevent_first in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Aether Revolt");
        revolt(&mut t);
        // P1 prevents the next 3 damage that would be dealt to them this turn.
        let salve = in_hand_with_mana(&mut t, P1, "Healing Salve");
        t.cast(P1, salve).modes(&[1]).target(P1).go();
        t.resolve_all();
        respond(
            &mut t,
            P1,
            if prevent_first {
                prevention_first
            } else {
                revolt_first
            },
        );
        let from = t.asked().len();
        cast(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        t.resolve_all();
        // P1 (the player being dealt damage) chose the order.
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseReplacement { .. })));
        // Prevention first: all 2 prevented (1 of the shield left), nothing for Aether
        // Revolt to add. Aether Revolt first: 4, 3 of it prevented.
        assert_eq!(t.life(P1), if prevent_first { 20 } else { 19 });
    }
}

#[test]
fn decommission_can_enable_its_own_revolt() {
    cr!("608.2c");
    ruling!("Decommission", "If Decommission destroys a permanent you control, it will enable its own revolt ability and you'll gain 3 life.");
    supported("Decommission");
    for mine in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let owner = if mine { P0 } else { P1 };
        let thopter = t.battlefield(owner, "Ornithopter");
        cast(&mut t, P0, "Decommission", &[thopter.into()]);
        t.resolve_all();
        assert!(t.in_graveyard(owner, "Ornithopter"));
        assert_eq!(t.life(P0), if mine { 23 } else { 20 }, "mine {mine}");
    }
}
