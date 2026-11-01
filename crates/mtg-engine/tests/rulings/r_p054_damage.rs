//! Rulings batch P054 — "whenever this creature is dealt damage" triggers (enrage) and
//! other damage-driven abilities: one trigger per damage event whatever the amount, even
//! for lethal damage (CR 120.3, 603.2, 603.10a, 510.2), "that much" counting all the
//! damage dealt, trample assignment to such a creature (CR 702.19c), and Equipment that
//! reacts to the damage the equipped creature deals.

use crate::r_p054_common::*;
use crate::r_s01_common::with_subtype;
use crate::r_s06_common::{attach_new, damage, give_control};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn library(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.library_top(p, "Island");
    }
}

#[test]
fn illusory_ambusher_draws_all_the_damage_dealt_even_beyond_its_toughness() {
    cr!("120.3e", "603.2", "603.10a");
    ruling!(
        "Illusory Ambusher",
        "Creatures may be dealt damage greater than their toughness. For example, if a source deals 5 damage to Illusory Ambusher, you'll draw five cards."
    );
    let mut t = TestGame::new(2);
    library(&mut t, P0, 10);
    let amb = t.battlefield(P0, "Illusory Ambusher");
    let src = t.battlefield(P1, "Craw Wurm");
    let hand = t.hand_size(P0);
    damage(&mut t, src, 5, amb);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Illusory Ambusher"));
    assert_eq!(t.hand_size(P0), hand + 5);
}

#[test]
fn a_trampler_blocked_by_illusory_ambusher_may_assign_it_anything_from_one_to_all() {
    cr!("702.19b", "510.1c");
    ruling!(
        "Illusory Ambusher",
        "If Illusory Ambusher blocks a creature with trample, that creature's controller can assign any amount of damage from 1 to that creature's power to Illusory Ambusher. Excess damage doesn't have to be assigned to the defending player."
    );
    for (to_amb, to_player) in [(1u32, 5u32), (6, 0), (3, 3)] {
        let mut t = TestGame::new(2);
        library(&mut t, P1, 10);
        let maw = t.battlefield(P0, "Colossal Dreadmaw");
        let amb = t.battlefield(P1, "Illusory Ambusher");
        let hand = t.hand_size(P1);
        crate::r_s01_common::attack_with(&mut t, &[(maw, pl(P1))]);
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(amb, maw)]),
        );
        t.answer(
            P0,
            DecisionKind::Damage,
            Answer::Numbers(vec![to_amb as i64, to_player as i64]),
        );
        t.advance_to(P0, Step::EndOfCombat);
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - to_player as i32, "{to_amb}/{to_player}");
        assert_eq!(
            t.hand_size(P1),
            hand + to_amb as usize,
            "{to_amb}/{to_player}"
        );
    }
}

#[test]
fn dealt_damage_triggers_once_per_event_not_per_point_or_per_source() {
    cr!("603.2", "510.2", "120.3");
    ruling!(
        "Dromad Purebred",
        "Each time Dromad Purebred is dealt damage, including from multiple sources at once, you gain 1 life. You don’t gain life equal to the damage dealt."
    );
    ruling!(
        "High Priest of Penance",
        "High Priest of Penance's ability triggers once for each instance of damage dealt to it, not once for each 1 damage. The ability will trigger even if High Priest of Penance is dealt lethal damage."
    );
    // Dromad Purebred, blocked by two creatures at once: 1 life.
    let mut t = TestGame::new(2);
    let dromad = t.battlefield(P0, "Dromad Purebred");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(dromad, pl(P1))], &[(a, dromad), (b, dromad)]);
    t.resolve_all();
    assert_eq!(t.obj_now(dromad).damage, 4);
    assert_eq!(t.life(P0), 21);
    // Two separate instances: 2 life.
    let shock = t.battlefield(P1, "Prodigal Pyromancer");
    damage(&mut t, shock, 1, dromad);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // High Priest of Penance dealt 3 (lethal): one trigger, which still resolves.
    let mut t = TestGame::new(2);
    let hp = t.battlefield(P0, "High Priest of Penance");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let other = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[o(other)]);
    t.answer_yes(P0, true);
    damage(&mut t, wurm, 3, hp);
    assert!(t.in_graveyard(P0, "High Priest of Penance"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(other));
    assert!(t.on_battlefield(wurm));
}

#[test]
fn wall_of_essence_gains_all_the_combat_damage_even_if_lethal() {
    cr!("603.10a", "510.2", "120.3");
    ruling!(
        "Wall of Essence",
        "Wall of Essence’s triggered ability will trigger even if it’s dealt lethal damage. For example, if it’s dealt 7 combat damage, its ability will trigger and you’ll gain 7 life."
    );
    let mut t = TestGame::new(2);
    let big = t.battlefield(P0, "Craw Wurm");
    crate::r_p017_common::animate(&mut t, big, 7);
    let wall = t.battlefield(P1, "Wall of Essence");
    t.attack(&[(big, pl(P1))], &[(wall, big)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Wall of Essence"));
    assert_eq!(t.life(P1), 27);
    // Noncombat damage doesn't trigger it.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Essence");
    let src = t.battlefield(P0, "Prodigal Pyromancer");
    damage(&mut t, src, 2, wall);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn wall_of_resistance_gets_one_counter_a_turn() {
    cr!("603.4", "513.1a");
    ruling!(
        "Wall of Resistance",
        "It gets only one counter a turn, not one per point of damage."
    );
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Resistance");
    let src = t.battlefield(P1, "Prodigal Pyromancer");
    damage(&mut t, src, 1, wall);
    damage(&mut t, src, 1, wall);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(wall, "+0/+1"), 1);
    // Not dealt damage on the next turn: no counter.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(wall, "+0/+1"), 1);
}

#[test]
fn broodhatch_nantuko_makes_all_the_tokens_or_none() {
    cr!("603.5", "608.2d");
    ruling!(
        "Broodhatch Nantuko",
        "You can choose to put zero tokens or all the tokens onto the battlefield. You can't choose a number in between."
    );
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Craw Wurm");
        let bn = t.battlefield(P0, "Broodhatch Nantuko");
        let src = t.battlefield(P1, "Craw Wurm");
        let mark = t.asked().len();
        t.answer_yes(P0, yes);
        damage(&mut t, src, 3, bn);
        t.resolve_all();
        let insects = with_subtype(&t, P0, "Insect").len();
        assert_eq!(insects, if yes { 3 } else { 0 });
        let asked = &t.asked()[mark..];
        assert!(asked
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
        assert!(!asked
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseNumber { .. })));
    }
}

#[test]
fn unscythe_counts_noncombat_damage_and_needs_the_card_in_the_graveyard() {
    cr!("603.10a", "608.2c", "111.7");
    ruling!(
        "Unscythe, Killer of Kings",
        "Unscythe’s second ability cares about any damage, not just combat damage."
    );
    ruling!(
        "Unscythe, Killer of Kings",
        "If Unscythe’s second ability triggers, but the creature that was put into a graveyard is somehow removed from the graveyard in response (or it was a token), it won’t be able to be exiled with Unscythe’s ability. You won’t get a Zombie token."
    );
    // Noncombat damage from the equipped creature: the creature is exiled, a Zombie made.
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    attach_new(&mut t, P0, "Unscythe, Killer of Kings", pyro);
    t.battlefield(P1, "Llanowar Elves");
    let elves = t.named_on_battlefield("Llanowar Elves")[0];
    t.answer_yes(P0, true);
    damage(&mut t, pyro, 1, elves);
    t.resolve_all();
    assert!(t.in_exile("Llanowar Elves"));
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
    // Removed from the graveyard in response: no exile, no Zombie.
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    attach_new(&mut t, P0, "Unscythe, Killer of Kings", pyro);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_yes(P0, true);
    damage(&mut t, pyro, 1, elves);
    assert_eq!(t.stack_len(), 1);
    let card = t.g.current(elves);
    move_to(&mut t, card, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.in_hand(P1, "Llanowar Elves"));
    assert!(with_subtype(&t, P0, "Zombie").is_empty());
    // A token: no Zombie.
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    attach_new(&mut t, P0, "Unscythe, Killer of Kings", pyro);
    let tok = crate::r_s02_common::create_token(&mut t, P1, "Goblin");
    t.answer_yes(P0, true);
    damage(&mut t, pyro, 1, tok);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Zombie").is_empty());
}

#[test]
fn sword_of_kaldra_exiles_the_creature_the_equipped_creature_damaged() {
    cr!("603.2", "510.2");
    ruling!(
        "Sword of Kaldra",
        "You exile the creature that gets damaged."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Sword of Kaldra", bear);
    // An indestructible blocker survives the damage and is exiled by the trigger.
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.attack(&[(bear, pl(P1))], &[(myr, bear)]);
    t.resolve_all();
    assert!(t.in_exile("Darksteel Myr"));
    assert!(t.on_battlefield(bear));
}

#[test]
fn stitchers_graft_cant_sacrifice_a_permanent_you_no_longer_control() {
    cr!("701.21a", "603.2");
    ruling!(
        "Stitcher's Graft",
        "If Stitcher’s Graft’s last triggered ability triggers, but you don’t control the permanent it became unattached from at the time that ability resolves (perhaps because another player has somehow gained control of it), you won’t be able to sacrifice it."
    );
    for stolen in [false, true] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        let other = t.battlefield(P0, "Hill Giant");
        let graft = attach_new(&mut t, P0, "Stitcher's Graft", bear);
        pool(&mut t, P0);
        t.activate(P0, graft, 0, &[o(other)]).expect("equip");
        t.resolve(); // the equip ability: the unattach trigger goes on the stack
        assert_eq!(t.stack_len(), 1);
        if stolen {
            give_control(&mut t, bear, P1);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(bear), stolen, "stolen: {stolen}");
    }
}
