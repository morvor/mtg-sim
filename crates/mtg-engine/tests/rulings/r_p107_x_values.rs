//! Rulings batch P107 — "where X is ..." in activated abilities: X is determined once, as
//! the ability resolves (CR 608.2h), except where the ability says "as you activate this
//! ability", in which case the number of targets is fixed as it's activated (CR 601.2c,
//! 602.2b).

use crate::r_p107_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts `n` Bobbleheads under `p`'s control besides `first` (Charisma Bobbleheads, whose
/// own abilities don't matter here) and returns them.
fn more_bobbleheads(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.battlefield(p, "Charisma Bobblehead")).collect()
}

#[test]
fn bobblehead_x_counts_bobbleheads_as_the_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Charisma Bobblehead",
        "The value of X is calculated only once, as Charisma Bobblehead’s last ability resolves."
    );
    ruling!(
        "Intelligence Bobblehead",
        "The value of X is determined only once, as Intelligence Bobblehead’s last ability resolves."
    );
    ruling!(
        "Strength Bobblehead",
        "The value of X is determined only once, as Strength Bobblehead’s last ability resolves."
    );
    // Each ability is activated with three Bobbleheads; one is destroyed in response, so
    // X is 2. Afterwards another Bobblehead arrives: nothing changes.
    for (name, needle, cost) in [
        ("Charisma Bobblehead", "Soldier", 4),
        ("Intelligence Bobblehead", "Draw X", 5),
        ("Strength Bobblehead", "+1/+1 counters", 3),
    ] {
        let mut t = TestGame::new(2);
        supported(name);
        let bob = t.battlefield(P0, name);
        let others = more_bobbleheads(&mut t, P0, 2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        mana(&mut t, P0, ManaType::C, cost);
        let soldiers = tokens_with(&t, P0, "Soldier");
        let hand = t.hand_size(P0);
        let targets: Vec<Entity> = if name == "Strength Bobblehead" {
            vec![obj(bear)]
        } else {
            vec![]
        };
        act(&mut t, P0, bob, needle, &targets).unwrap();
        destroy(&mut t, others[0]);
        t.resolve_all();
        let got = match name {
            "Charisma Bobblehead" => (tokens_with(&t, P0, "Soldier") - soldiers) as u32,
            "Intelligence Bobblehead" => (t.hand_size(P0) - hand) as u32,
            _ => t.counters(bear, counters::PLUS1),
        };
        assert_eq!(got, 2, "{name}");
        t.battlefield(P0, "Charisma Bobblehead");
        t.g.recompute();
        let after = match name {
            "Charisma Bobblehead" => (tokens_with(&t, P0, "Soldier") - soldiers) as u32,
            "Intelligence Bobblehead" => (t.hand_size(P0) - hand) as u32,
            _ => t.counters(bear, counters::PLUS1),
        };
        assert_eq!(after, 2, "{name}");
    }
}

#[test]
fn perception_bobblehead_looks_at_bobbleheads_counted_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Perception Bobblehead",
        "The value of X is determined only once, as Perception Bobblehead’s last ability resolves."
    );
    // Three Bobbleheads at activation, two at resolution: two cards are looked at (the
    // rest go to the bottom, so the third card from the top stays on top).
    let mut t = TestGame::new(2);
    let bob = t.battlefield(P0, "Perception Bobblehead");
    let others = more_bobbleheads(&mut t, P0, 2);
    let lib = stack_library(&mut t, P0, &["Plains", "Island", "Swamp"]);
    mana(&mut t, P0, ManaType::C, 3);
    act(&mut t, P0, bob, "Look at the top X", &[]).unwrap();
    destroy(&mut t, others[0]);
    t.resolve_all();
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(top, lib[2]);
}

#[test]
fn tezzeret_x_is_fixed_as_the_ability_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Tezzeret the Schemer",
        "The value of X for Tezzeret's second ability is determined only as the ability resolves."
    );
    // −2: Target creature gets +X/-X, X = artifacts you control. Two artifacts at
    // resolution (one destroyed in response); a third later doesn't change it.
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret the Schemer");
    let a = t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    let giant = t.battlefield(P1, "Hill Giant");
    act(&mut t, P0, tez, "+X/-X", &[obj(giant)]).unwrap();
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 1));
    t.battlefield(P0, "Ornithopter");
    t.g.recompute();
    assert_eq!(t.pt(giant), (5, 1));
    t.battlefield(P0, "Ornithopter");
    t.settle();
    assert!(t.on_battlefield(giant));
}

#[test]
fn wickersmiths_tools_counts_charge_counters_once_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Wickersmith's Tools",
        "The value of X is calculated only once, as Wickersmith's Tools's last ability resolves."
    );
    // The ability sacrifices the Tools as a cost, so X uses its last known information:
    // the counters it had as it left the battlefield.
    let mut t = TestGame::new(2);
    let tools = t.battlefield(P0, "Wickersmith's Tools");
    put_counters(&mut t, tools, counters::CHARGE, 3);
    mana(&mut t, P0, ManaType::C, 5);
    act(&mut t, P0, tools, "Scarecrow", &[]).unwrap();
    t.resolve_all();
    let crows = with_subtype(&t, P0, "Scarecrow");
    assert_eq!(crows.len(), 3);
    assert!(crows.iter().all(|c| t.obj_now(*c).tapped));
}

#[test]
fn all_fates_scroll_counts_differently_named_lands_on_resolution() {
    cr!("608.2h");
    ruling!(
        "All-Fates Scroll",
        "The value of X is determined only once, as All-Fates Scroll’s last ability resolves."
    );
    // Plains, Plains, Island, Swamp: three different names, but the Swamp is destroyed
    // in response, so X is 2.
    let mut t = TestGame::new(2);
    let scroll = t.battlefield(P0, "All-Fates Scroll");
    t.battlefield(P0, "Plains");
    t.battlefield(P0, "Plains");
    t.battlefield(P0, "Island");
    let swamp = t.battlefield(P0, "Swamp");
    mana(&mut t, P0, ManaType::C, 7);
    let hand = t.hand_size(P0);
    act(&mut t, P0, scroll, "Draw X", &[]).unwrap();
    destroy(&mut t, swamp);
    t.resolve_all();
    assert_eq!(t.hand_size(P0) - hand, 2);
}

#[test]
fn agility_and_endurance_bobbleheads_fix_the_number_of_targets_on_activation() {
    cr!("601.2c", "602.2b");
    ruling!(
        "Agility Bobblehead",
        "You choose how many targets Agility Bobblehead’s last ability has as you activate the ability."
    );
    ruling!(
        "Endurance Bobblehead",
        "You choose how many targets Endurance Bobblehead’s last ability has as you activate the ability."
    );
    // Two Bobbleheads: up to two targets. Destroying one in response doesn't matter: both
    // targets are affected.
    for (name, needle, cost) in [
        ("Agility Bobblehead", "haste", 3),
        ("Endurance Bobblehead", "indestructible", 3),
    ] {
        let mut t = TestGame::new(2);
        let bob = t.battlefield(P0, name);
        let other = t.battlefield(P0, "Charisma Bobblehead");
        let b1 = t.battlefield_sick(P0, "Grizzly Bears");
        let b2 = t.battlefield_sick(P0, "Grizzly Bears");
        mana(&mut t, P0, ManaType::C, cost);
        t.answer_targets(P0, &[obj(b1), obj(b2)]);
        activate_containing(&mut t, P0, bob, needle).unwrap();
        destroy(&mut t, other);
        t.resolve_all();
        let kw = if name == "Agility Bobblehead" {
            mtg_engine::keywords::KeywordKind::Haste
        } else {
            mtg_engine::keywords::KeywordKind::Indestructible
        };
        assert!(has_kw(&t, b1, kw), "{name}");
        assert!(has_kw(&t, b2, kw), "{name}");
    }
}

#[test]
fn bobbleheads_can_choose_fewer_targets_than_x() {
    cr!("601.2c");
    // With three Bobbleheads, Endurance Bobblehead's ability may still target just one
    // creature.
    let mut t = TestGame::new(2);
    let bob = t.battlefield(P0, "Endurance Bobblehead");
    more_bobbleheads(&mut t, P0, 2);
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 3);
    t.answer_targets(P0, &[obj(b1)]);
    activate_containing(&mut t, P0, bob, "indestructible").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(b1), (3, 2));
    assert_eq!(t.pt(b2), (2, 2));
}
