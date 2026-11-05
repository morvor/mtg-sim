//! Rulings batch P043 — creatures that prevent damage to themselves: by removing +1/+1
//! counters (Oathsworn Knight, Ugin's Conjurant, Rock Hydra; CR 615.5, 615.12), Diamond
//! Weapon's "Immune" (CR 510.1c), Nomads en-Kor's redirection (CR 614.9); and X in the
//! costs of Ugin's Conjurant, Rock Hydra and Slumbering Trudge (CR 107.3, 601.2f).

use crate::r_p001_common::mv;
use crate::r_p007_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn hit(t: &mut TestGame, src: ObjectId, to: Entity, n: u32) {
    t.g.deal_damage(src, to, n, false);
    t.g.flush_events();
    t.settle();
}

fn with_counters(t: &mut TestGame, name: &str, n: u32) -> ObjectId {
    let id = t.battlefield(P0, name);
    if n > 0 {
        t.g.add_counters(obj(id), "+1/+1", n, None);
    }
    t.g.recompute();
    id
}

/// P0 casts Skullcrack at P1: damage can't be prevented this turn.
fn skullcrack(t: &mut TestGame) {
    cast_new(t, P0, "Skullcrack", &[pl(P1)]);
    t.resolve_all();
}

#[test]
fn diamond_weapon_lethal_damage_ignores_its_prevention() {
    cr!("510.1c", "702.19b", "615.1");
    ruling!(
        "Diamond Weapon",
        "Diamond Weapon's prevention effect isn't considered while assigning combat damage from a creature with trample that's being blocked by Diamond Weapon. For example, if it blocks a 12/12 creature with trample, that creature's controller must assign at least 8 of that creature's combat damage to Diamond Weapon, and the remainder can be assigned to the defending player, planeswalker, or battle."
    );
    supported("Diamond Weapon");
    for (answer, life) in [(vec![8, 4], 16), (vec![1, 11], 16)] {
        let mut t = TestGame::new(2);
        let ghalta = t.battlefield(P0, "Ghalta, Primal Hunger");
        let weapon = t.battlefield(P1, "Diamond Weapon");
        t.set_step(P0, Step::PrecombatMain);
        attack_with(&mut t, &[(ghalta, pl(P1))]);
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(weapon, ghalta)]),
        );
        t.answer(P0, DecisionKind::Damage, Answer::Numbers(answer.clone()));
        let from = t.asked().len();
        t.advance_to(P0, Step::EndOfCombat);
        let lethal: Vec<Vec<u32>> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::AssignCombatDamage { lethal, .. } => Some(lethal.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(lethal, vec![vec![8]]);
        // At least 8 to the Weapon (an assignment of 1 isn't accepted); its damage is
        // prevented.
        assert_eq!(t.life(P1), life, "{answer:?}");
        assert_eq!(dmg(&t, weapon), 0);
    }
}

#[test]
fn oathsworn_knight_attacks_only_if_able_without_paying() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Oathsworn Knight",
        "If Oathsworn Knight can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Oathsworn Knight");
    for case in ["able", "tapped", "sick", "cost"] {
        let mut t = TestGame::new(2);
        let knight = if case == "sick" {
            t.battlefield_sick(P0, "Oathsworn Knight")
        } else {
            t.battlefield(P0, "Oathsworn Knight")
        };
        t.g.add_counters(obj(knight), "+1/+1", 4, None);
        if case == "tapped" {
            t.g.objects[knight.0 as usize].tapped = true;
        }
        if case == "cost" {
            t.battlefield(P1, "Ghostly Prison");
            t.lands(P0, "Wastes", 2);
        }
        t.set_step(P0, Step::BeginningOfCombat);
        t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
        let turn = t.g.turn.number;
        t.g.run_until(10_000, |g| {
            g.turn.step == Step::DeclareAttackers && g.turn.priority.is_some()
                || g.turn.number != turn
        });
        let attacking = t.g.is_attacking(knight);
        assert_eq!(attacking, case == "able", "{case}");
    }
}

#[test]
fn oathsworn_knight_unpreventable_and_simultaneous_damage() {
    cr!("615.12", "615.5", "615.7");
    ruling!(
        "Oathsworn Knight",
        "If damage that can't be prevented is dealt to Oathsworn Knight, one counter is still removed from it."
    );
    ruling!(
        "Oathsworn Knight",
        "If multiple sources would deal damage to Oathsworn Knight simultaneously (for example, multiple creatures blocking it), its last ability applies once. All of that damage is prevented, and only one counter is removed."
    );
    let mut t = TestGame::new(2);
    let knight = with_counters(&mut t, "Oathsworn Knight", 4);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage_batch(vec![(a, obj(knight), 2), (b, obj(knight), 3)], true);
    t.g.flush_events();
    t.settle();
    assert_eq!((t.counters(knight, "+1/+1"), dmg(&t, knight)), (3, 0));
    skullcrack(&mut t);
    hit(&mut t, a, obj(knight), 1);
    assert_eq!((t.counters(knight, "+1/+1"), dmg(&t, knight)), (2, 1));
    assert!(t.on_battlefield(knight));
}

#[test]
fn ugins_conjurant_prevents_all_of_it_removing_the_counters_it_has() {
    cr!("615.5", "613.4c");
    ruling!(
        "Ugin's Conjurant",
        "If Ugin’s Conjurant would be dealt more damage than it has counters, you remove all of the +1/+1 counters it has on it and prevent all of that damage. If another effect is raising its toughness, it will survive until the next time it’s dealt lethal damage."
    );
    supported("Ugin's Conjurant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let conj = with_counters(&mut t, "Ugin's Conjurant", 2);
    let giant = t.battlefield(P1, "Craw Wurm");
    hit(&mut t, giant, obj(conj), 5);
    assert_eq!((t.counters(conj, "+1/+1"), dmg(&t, conj)), (0, 0));
    assert!(t.on_battlefield(conj));
    assert_eq!(t.pt(conj), (1, 1));
    hit(&mut t, giant, obj(conj), 1);
    assert!(!t.on_battlefield(conj), "lethal damage the next time");
}

#[test]
fn ugins_conjurant_enters_with_x_counters_whatever_was_paid() {
    cr!("107.3a", "601.2f", "122.6");
    ruling!(
        "Ugin's Conjurant",
        "To cast Ugin’s Conjurant, first choose the value for X you’d like, then apply any cost increases, then any cost reductions. The number of counters it enters with may be different than the amount of mana you pay if an effect (such as that of Ugin, the Ineffable) causes that you pay an amount of mana other than {X}."
    );
    // Ugin, the Ineffable: "Colorless spells you cast cost {2} less to cast." X = 3 for 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ugin, the Ineffable");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Ugin's Conjurant");
    t.cast(P0, card).x(3).go();
    t.resolve_all();
    let conj = t.named_on_battlefield("Ugin's Conjurant")[0];
    assert_eq!(t.counters(conj, "+1/+1"), 3);
    // With Lodestone Golem too ("Nonartifact spells cost {1} more"): X = 3 for 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ugin, the Ineffable");
    t.battlefield(P1, "Lodestone Golem");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Ugin's Conjurant");
    assert!(
        t.cast(P0, card).x(3).try_go().is_err(),
        "1 mana isn't enough"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ugin, the Ineffable");
    t.battlefield(P1, "Lodestone Golem");
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Ugin's Conjurant");
    t.cast(P0, card).x(3).go();
    t.resolve_all();
    let conj = t.named_on_battlefield("Ugin's Conjurant")[0];
    assert_eq!(t.counters(conj, "+1/+1"), 3);
}

#[test]
fn rock_hydra_counters_and_x() {
    cr!("615.12", "107.3f", "107.3g", "202.3e", "615.7");
    ruling!(
        "Rock Hydra",
        "If damage that would be dealt to Rock Hydra can't be prevented, you still remove a +1/+1 counter from it for each 1 damage dealt."
    );
    ruling!(
        "Rock Hydra",
        "If put onto the battlefield without casting it from your hand, X will be zero."
    );
    ruling!(
        "Rock Hydra",
        "Once on the battlefield, the X is considered to be zero when calculating its mana cost."
    );
    ruling!(
        "Rock Hydra",
        "You can activate the activated ability that prevents damage any time, even if no source seems likely to deal damage to you in the near future. It will prevent the next 1 damage that would be dealt this turn, even if that turns out to be much later in the turn."
    );
    supported("Rock Hydra");
    // Unpreventable: 2 damage dealt, 2 counters removed.
    let mut t = TestGame::new(2);
    let hydra = with_counters(&mut t, "Rock Hydra", 4);
    let giant = t.battlefield(P1, "Hill Giant");
    skullcrack(&mut t);
    hit(&mut t, giant, obj(hydra), 1);
    assert_eq!((t.counters(hydra, "+1/+1"), dmg(&t, hydra)), (3, 1));
    assert_eq!(mv(&t, hydra), 2);
    // Put onto the battlefield: no counters, so it dies.
    let mut t = TestGame::new(2);
    let hydra = t.enter(P0, "Rock Hydra");
    t.settle();
    assert!(!t.on_battlefield(hydra));
    assert!(t.in_graveyard(P0, "Rock Hydra"));
    // Cast with X = 3: 3 counters, mana value 2 on the battlefield.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Rock Hydra");
    t.cast(P0, card).x(3).go();
    t.resolve_all();
    let hydra = t.named_on_battlefield("Rock Hydra")[0];
    assert_eq!(t.counters(hydra, "+1/+1"), 3);
    assert_eq!(mv(&t, hydra), 2);
    // {R} in the upkeep (no counters; Glorious Anthem keeps it alive): the 1 damage in the
    // end step is prevented.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let hydra = t.battlefield(P0, "Rock Hydra");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::Upkeep);
    t.lands(P0, "Mountain", 1);
    t.activate(P0, hydra, 0, &[]).expect("{R}: prevent");
    t.resolve_all();
    t.advance_to(P0, Step::End);
    hit(&mut t, giant, obj(hydra), 1);
    assert!(t.on_battlefield(hydra));
    assert_eq!(dmg(&t, hydra), 0);
    hit(&mut t, giant, obj(hydra), 1);
    assert!(!t.on_battlefield(hydra), "only the next 1");
}

#[test]
fn nomads_en_kor_can_redirect_more_than_the_creatures_toughness() {
    cr!("614.9", "704.5g");
    ruling!(
        "Nomads en-Kor",
        "It is possible to redirect more damage to a creature than that creature's toughness."
    );
    let mut t = TestGame::new(2);
    let nomads = t.battlefield(P0, "Nomads en-Kor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    for _ in 0..3 {
        t.activate(P0, nomads, 0, &[obj(bears)]).unwrap();
        t.resolve_all();
    }
    t.g.deal_damage(giant, obj(nomads), 3, false);
    assert_eq!(t.obj_now(bears).damage, 3, "3 damage on the 2/2");
    t.settle();
    assert!(t.on_battlefield(nomads));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn slumbering_trudge_x_is_0_unless_cast_with_x() {
    cr!("107.3f", "107.3m", "122.1d");
    ruling!(
        "Slumbering Trudge",
        "If Slumbering Trudge enters the battlefield without being cast, X is 0. The same is true if an effect lets you cast Slumbering Trudge without paying its mana cost."
    );
    supported("Slumbering Trudge");
    // Enters without being cast: three stun counters, tapped.
    let mut t = TestGame::new(2);
    let trudge = t.enter(P0, "Slumbering Trudge");
    assert_eq!(t.counters(trudge, "stun"), 3);
    assert!(t.obj_now(trudge).tapped);
    // Cast for free with Omniscience: X is 0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let card = t.hand(P0, "Slumbering Trudge");
    t.cast(P0, card).method(CastMethod::Free).go();
    t.resolve_all();
    let trudge = t.named_on_battlefield("Slumbering Trudge")[0];
    assert_eq!(t.counters(trudge, "stun"), 3);
    assert!(t.obj_now(trudge).tapped);
    // Cast with X = 3: none, untapped.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Slumbering Trudge");
    t.cast(P0, card).x(3).go();
    t.resolve_all();
    let trudge = t.named_on_battlefield("Slumbering Trudge")[0];
    assert_eq!(t.counters(trudge, "stun"), 0);
    assert!(!t.obj_now(trudge).tapped);
}

#[test]
fn leonardo_cost_reductions_apply_after_increases_and_keep_its_mana_value() {
    cr!("601.2f", "202.3", "702.41a");
    ruling!(
        "Leonardo, Worldly Warrior",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Leonardo's first ability). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Leonardo, Worldly Warrior");
    // {7}{W}, plus {1} (Lodestone Golem), minus {3} (three creatures): six mana.
    for (lands, ok) in [(5, false), (6, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Lodestone Golem");
        for _ in 0..3 {
            t.battlefield(P0, "Grizzly Bears");
        }
        t.lands(P0, "Plains", lands);
        let card = t.hand(P0, "Leonardo, Worldly Warrior");
        let r = t.cast(P0, card).try_go();
        assert_eq!(r.is_ok(), ok, "{lands} lands");
        if let Ok(spell) = r {
            assert_eq!(mv(&t, spell), 8);
        }
    }
}
