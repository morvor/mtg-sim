//! Rulings batch P043 — static prevention and redirection effects: "prevent all but 1"
//! (Temple Altisaur), "prevent 1 of that damage" (Hedron-Field Purists, Guardian Seraph,
//! Protection of the Hekma; CR 615.1a, 615.7), "is dealt to this creature instead"
//! (Palisade Giant, Protector of the Crown, Martyrs of Korlis; CR 614.9, 616.1), and
//! prevention from resolved spells and abilities (Pack Leader, Druid's Deliverance, The
//! Stasis Coffin).

use crate::r_p001_common::replacement_choices_since;
use crate::r_p007_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::destroy;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `src` deals `n` damage to `to` (combat damage if `combat`), then SBAs.
fn hit(t: &mut TestGame, src: ObjectId, to: Entity, n: u32, combat: bool) {
    t.g.deal_damage(src, to, n, combat);
    t.g.flush_events();
    t.settle();
}

/// Several sources deal damage at the same time.
fn hit_all(t: &mut TestGame, events: Vec<(ObjectId, Entity, u32)>, combat: bool) {
    t.g.deal_damage_batch(events, combat);
    t.g.flush_events();
    t.settle();
}

/// The damage events dealt this turn to `to`: (source, amount, combat).
fn dealt_to(t: &TestGame, to: Entity) -> Vec<(ObjectId, u32, bool)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                combat,
            } if *target == to => Some((*source, *amount, *combat)),
            _ => None,
        })
        .collect()
}

#[test]
fn temple_altisaur_leaves_1_damage_from_each_source() {
    cr!("615.1a", "616.1");
    ruling!(
        "Temple Altisaur",
        "If multiple sources would deal damage to another Dinosaur you control, all but 1 damage from each of those sources is prevented."
    );
    ruling!(
        "Temple Altisaur",
        "The effects of two Temple Altisaurs won't reduce the damage from one source below 1 damage."
    );
    supported("Temple Altisaur");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Temple Altisaur");
    let dino = t.battlefield(P0, "Ancient Brontodon");
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    hit_all(
        &mut t,
        vec![(giant, obj(dino), 3), (wurm, obj(dino), 6)],
        true,
    );
    assert_eq!(dmg(&t, dino), 2, "1 from each source");
    // Two Altisaurs: still 1.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Temple Altisaur");
    t.battlefield(P0, "Temple Altisaur");
    let giant = t.battlefield(P1, "Hill Giant");
    hit(&mut t, giant, obj(a), 3, false);
    assert_eq!(dmg(&t, a), 1);
}

#[test]
fn two_palisade_giants_take_all_of_one_sources_damage_on_one_of_them() {
    cr!("614.9", "616.1", "616.1f");
    ruling!(
        "Palisade Giant",
        "If you control more than one Palisade Giant, you choose which redirection effect to apply. You can’t divide damage dealt by one source. For example, if an attacking creature would deal 8 damage to you and you control two Palisade Giants, you may have that damage dealt to either of the Palisade Giants. You can’t have 4 damage dealt to each Giant or choose to have the 8 damage dealt to you."
    );
    let mut outcomes = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Palisade Giant");
        let b = t.battlefield(P0, "Palisade Giant");
        let src = t.battlefield(P1, "Craw Wurm");
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        hit(&mut t, src, pl(P0), 8, true);
        assert!(!replacement_choices_since(&t, P0, from).is_empty());
        assert_eq!(t.life(P0), 20);
        // (A Giant dealt 8 damage dies: count the damage events.)
        let to = |x: ObjectId| -> u32 { dealt_to(&t, obj(x)).iter().map(|d| d.1).sum() };
        let (da, db) = (to(a), to(b));
        assert!((da, db) == (8, 0) || (da, db) == (0, 8), "{da} {db}");
        assert!(dealt_to(&t, pl(P0)).is_empty());
        outcomes.push(da);
    }
    outcomes.sort();
    assert_eq!(outcomes, vec![0, 8], "either Giant");
}

#[test]
fn protector_of_the_crown_redirects_combat_damage_as_combat_damage_whoever_is_monarch() {
    cr!("614.9", "616.1", "725.1");
    ruling!(
        "Protector of the Crown",
        "Applying this redirection effect doesn't change whether the damage is combat damage."
    );
    ruling!(
        "Protector of the Crown",
        "Protector of the Crown's last ability doesn't depend on you being the monarch. It will create a redirection effect even if an opponent is the monarch."
    );
    ruling!(
        "Protector of the Crown",
        "If you control more than one Protector of the Crown, you choose which redirection effect to apply. You can't divide damage dealt by one source. For example, if an attacking creature would deal 6 damage to you and you control two Protectors of the Crown, you may have that damage dealt to either of the Protectors. You can't have 3 damage dealt to each one."
    );
    supported("Protector of the Crown");
    let mut t = TestGame::new(2);
    let prot = t.battlefield(P0, "Protector of the Crown");
    mtg_engine::designations::become_monarch(&mut t.g, P1);
    let src = t.battlefield(P1, "Hill Giant");
    hit(&mut t, src, pl(P0), 3, true);
    assert_eq!(t.life(P0), 20);
    assert_eq!(dealt_to(&t, obj(prot)), vec![(src, 3, true)]);
    // Two Protectors: all 6 to one of them.
    let mut outcomes = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Protector of the Crown");
        let b = t.battlefield(P0, "Protector of the Crown");
        let src = t.battlefield(P1, "Craw Wurm");
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        hit(&mut t, src, pl(P0), 4, true);
        assert_eq!(t.life(P0), 20);
        let (da, db) = (t.obj_now(a).damage, t.obj_now(b).damage);
        assert!((da, db) == (4, 0) || (da, db) == (0, 4), "{da} {db}");
        outcomes.push(da);
    }
    outcomes.sort();
    assert_eq!(outcomes, vec![0, 4]);
}

#[test]
fn martyrs_of_korlis_the_redirect_target_is_chosen_each_time() {
    cr!("614.9", "616.1");
    ruling!(
        "Martyrs of Korlis",
        "If you have multiple Martyrs of Korlis, you can decide which one receives the redirected damage each time artifact damage would be dealt to you."
    );
    supported("Martyrs of Korlis");
    let mut seen = std::collections::BTreeSet::new();
    for picks in [[0, 0], [0, 1], [1, 0], [1, 1]] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Martyrs of Korlis");
        let b = t.battlefield(P0, "Martyrs of Korlis");
        let golem = t.battlefield(P1, "Stone Golem");
        for (k, pick) in picks.iter().enumerate() {
            let from = t.asked().len();
            t.answer(P0, DecisionKind::Replacement, Answer::Index(*pick));
            hit(&mut t, golem, pl(P0), 1 + k as u32, false);
            assert_eq!(
                replacement_choices_since(&t, P0, from).len(),
                1,
                "asked each time"
            );
        }
        assert_eq!(t.life(P0), 20);
        seen.insert((t.obj_now(a).damage, t.obj_now(b).damage));
    }
    // Each hit to either: (1+2, 0), (1, 2), (2, 1), (0, 1+2).
    assert_eq!(seen.len(), 4, "{seen:?}");
    // A nonartifact source's damage isn't redirected.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Martyrs of Korlis");
    let giant = t.battlefield(P1, "Hill Giant");
    hit(&mut t, giant, pl(P0), 2, false);
    assert_eq!(t.life(P0), 18);
}

/// Hedron-Field Purists with `level` level counters, for P0.
fn purists(t: &mut TestGame, level: u32) -> ObjectId {
    let id = t.battlefield(P0, "Hedron-Field Purists");
    if level > 0 {
        t.g.add_counters(obj(id), "level", level, None);
    }
    t.g.recompute();
    id
}

#[test]
fn hedron_field_purists_prevent_1_or_2_from_each_source_any_damage() {
    cr!("711.2a", "711.2b", "615.1a", "615.7");
    ruling!(
        "Hedron-Field Purists",
        "The prevention effects prevent 1 or 2 damage, as applicable, from each source (regardless of who controls it) each time that source would deal damage to you or a creature you control. They prevent damage of any kind, not just combat damage."
    );
    ruling!(
        "Hedron-Field Purists",
        "Remember that Hedron-Field Purists never has both of these prevention effects at the same time."
    );
    ruling!(
        "Hedron-Field Purists",
        "The effects from multiple Hedron-Field Purists are cumulative."
    );
    ruling!(
        "Hedron-Field Purists",
        "The prevention effects don’t affect damage dealt directly to a planeswalker you control."
    );
    supported("Hedron-Field Purists");
    // Level 1: 1 of each source's damage, P0's own source too, noncombat too.
    let mut t = TestGame::new(2);
    purists(&mut t, 1);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let mine = t.battlefield(P0, "Hill Giant");
    let giant = t.battlefield(P1, "Hill Giant");
    let goblin = t.battlefield(P1, "Raging Goblin");
    hit(&mut t, mine, pl(P0), 3, false);
    assert_eq!(t.life(P0), 18, "own source");
    hit_all(
        &mut t,
        vec![(giant, obj(wurm), 3), (goblin, obj(wurm), 1)],
        true,
    );
    assert_eq!(dmg(&t, wurm), 2, "1 from each source");
    hit(&mut t, giant, obj(wurm), 1, false);
    assert_eq!(dmg(&t, wurm), 2, "each time");
    // Level 5: 2, not 1 + 2.
    let mut t = TestGame::new(2);
    purists(&mut t, 5);
    let giant = t.battlefield(P1, "Craw Wurm");
    hit(&mut t, giant, pl(P0), 6, false);
    assert_eq!(t.life(P0), 16);
    // Two Purists at level 1: 2 prevented.
    let mut t = TestGame::new(2);
    purists(&mut t, 1);
    purists(&mut t, 1);
    let giant = t.battlefield(P1, "Craw Wurm");
    hit(&mut t, giant, pl(P0), 6, false);
    assert_eq!(t.life(P0), 16);
    // A planeswalker dealt damage directly: nothing prevented.
    let mut t = TestGame::new(2);
    purists(&mut t, 5);
    let jace = pw(&mut t, P0, "Jace Beleren", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    hit(&mut t, giant, obj(jace), 3, false);
    assert_eq!(t.counters(jace, "loyalty"), 2);
}

#[test]
fn pack_leaders_prevention_outlasts_it() {
    cr!("611.2a", "615.1", "506.4");
    ruling!(
        "Pack Leader",
        "Once Pack Leader attacks, its effect prevents damage that would be dealt to Dogs you control even if Pack Leader leaves the battlefield."
    );
    supported("Pack Leader");
    let mut t = TestGame::new(2);
    let leader = t.battlefield(P0, "Pack Leader");
    let dog = t.battlefield(P0, "Alpine Watchdog");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(leader, pl(P1)), (dog, pl(P1))]);
    t.resolve_all();
    destroy(&mut t, leader);
    assert_eq!(t.pt(dog), (2, 2));
    block_and_finish(&mut t, P1, &[(giant, dog)]);
    assert!(t.on_battlefield(dog));
    assert_eq!(dmg(&t, dog), 0);
    assert_eq!(dmg(&t, giant), 2);
}

#[test]
fn guardian_seraph_prevents_1_of_each_opposing_sources_damage() {
    cr!("615.1a", "108.4a", "702.29a");
    ruling!(
        "Guardian Seraph",
        "This ability prevents 1 damage from each source an opponent controls each time that source would deal damage to you. It prevents 1 of any damage, not just combat damage."
    );
    ruling!(
        "Guardian Seraph",
        "The effects from multiple Guardian Seraphs are cumulative."
    );
    ruling!(
        "Guardian Seraph",
        "Spells and permanents have controllers, but cards that aren’t on the stack or the battlefield don’t. If a source without a controller (such as a cycled Jund Sojourners, for example) would deal damage to you, 1 of that damage is prevented if an opponent owns that source."
    );
    supported("Guardian Seraph");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian Seraph");
    let giant = t.battlefield(P1, "Hill Giant");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let mine = t.battlefield(P0, "Hill Giant");
    hit_all(&mut t, vec![(giant, pl(P0), 3), (goblin, pl(P0), 1)], true);
    assert_eq!(t.life(P0), 18, "1 from each source");
    hit(&mut t, giant, pl(P0), 3, false);
    assert_eq!(t.life(P0), 16, "noncombat too, each time");
    hit(&mut t, mine, pl(P0), 3, false);
    assert_eq!(t.life(P0), 13, "not P0's own source");
    // Two Seraphs.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian Seraph");
    t.battlefield(P0, "Guardian Seraph");
    let giant = t.battlefield(P1, "Hill Giant");
    hit(&mut t, giant, pl(P0), 3, false);
    assert_eq!(t.life(P0), 19);
    // P1 cycles Jund Sojourners: its 1 damage (from the card in P1's graveyard) is
    // prevented.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian Seraph");
    let soj = t.hand(P1, "Jund Sojourners");
    t.lands(P1, "Wastes", 2);
    t.lands(P1, "Mountain", 1);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[pl(P0)]);
    crate::r_s06_common::activate_containing(&mut t, P1, soj, "Cycling").expect("cycling");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Jund Sojourners"));
    let attempted =
        t.g.turn_events
            .iter()
            .any(|e| matches!(e, Event::DamagePrevented { .. }));
    assert!(attempted, "{}", t.dump_log());
    assert_eq!(t.life(P0), 20);
    // A card P0 owns in its graveyard: not prevented.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian Seraph");
    let soj = t.hand(P0, "Jund Sojourners");
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P0)]);
    crate::r_s06_common::activate_containing(&mut t, P0, soj, "Cycling").expect("cycling");
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn protection_of_the_hekma_prevents_1_from_each_source_to_you_only() {
    cr!("615.1a", "615.7");
    ruling!(
        "Protection of the Hekma",
        "If multiple sources would deal damage to you at once (for example, several attacking creatures), 1 damage from each of those sources is prevented."
    );
    ruling!(
        "Protection of the Hekma",
        "Damage that would be dealt to creatures you control is unaffected."
    );
    supported("Protection of the Hekma");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Protection of the Hekma");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    hit_all(
        &mut t,
        vec![(giant, pl(P0), 3), (goblin, pl(P0), 1), (bears, pl(P0), 2)],
        true,
    );
    assert_eq!(t.life(P0), 17);
    hit(&mut t, giant, obj(wurm), 3, false);
    assert_eq!(dmg(&t, wurm), 3);
}

#[test]
fn druids_deliverance_prevents_combat_damage_to_you_only() {
    cr!("615.1", "701.36a", "810.9");
    ruling!(
        "Druid's Deliverance",
        "Combat damage dealt to creatures or planeswalkers you control won't be prevented."
    );
    ruling!(
        "Druid's Deliverance",
        "You can cast Druid's Deliverance even if you don't control any creature tokens."
    );
    ruling!(
        "Druid's Deliverance",
        "In a Two-Headed Giant game, combat damage dealt to your teammate won't be prevented."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let jace = pw(&mut t, P0, "Jace Beleren", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    // No creature tokens: castable (populate does nothing).
    cast_new(&mut t, P0, "Druid's Deliverance", &[]);
    t.resolve_all();
    hit(&mut t, giant, pl(P0), 3, true);
    assert_eq!(t.life(P0), 20);
    hit(&mut t, giant, pl(P0), 3, false);
    assert_eq!(t.life(P0), 17, "noncombat damage isn't prevented");
    hit(&mut t, giant, obj(wurm), 3, true);
    assert_eq!(dmg(&t, wurm), 3);
    hit(&mut t, giant, obj(jace), 3, true);
    assert_eq!(t.counters(jace, "loyalty"), 2);
    // Two-Headed Giant: P1 is P0's teammate.
    let mut t = crate::r_p050_common::two_headed_giant();
    let giant = t.battlefield(P2, "Hill Giant");
    cast_new(&mut t, P0, "Druid's Deliverance", &[]);
    t.resolve_all();
    let life = t.life(P0);
    hit(&mut t, giant, pl(P0), 3, true);
    assert_eq!(t.life(P0), life);
    hit(&mut t, giant, pl(P1), 3, true);
    assert_eq!(
        t.life(P0),
        life - 3,
        "the teammate's combat damage isn't prevented"
    );
}

#[test]
fn the_stasis_coffin_protection_from_everything() {
    cr!("702.16b", "702.16c", "702.16e", "702.16j", "615.12");
    ruling!(
        "The Stasis Coffin",
        "If a player has protection from everything, it means three things: (1) All damage that would be dealt to that player is prevented. (2) Auras can't be attached to that player. (3) That player can't be the target of spells or abilities."
    );
    ruling!(
        "The Stasis Coffin",
        "If an effect says that damage can't be prevented, you can still take damage from untargeted sources, such as attacking creatures or spells that deal damage to each player."
    );
    supported("The Stasis Coffin");
    let mut t = TestGame::new(2);
    let coffin = t.battlefield(P0, "The Stasis Coffin");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, coffin, 0, &[]).expect("The Stasis Coffin");
    t.resolve_all();
    assert!(t.in_exile("The Stasis Coffin"));
    hit(&mut t, giant, pl(P0), 3, true);
    assert_eq!(t.life(P0), 20, "damage prevented");
    // Can't be targeted.
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    let from = t.asked().len();
    let _ = t.cast_with(P1, shock, &[pl(P0)]);
    let cands = crate::r_s02_common::target_candidates(&t, P1, from);
    assert!(
        !cands.is_empty()
            && cands
                .iter()
                .all(|c| !c.contains(&pl(P0)) && c.contains(&pl(P1)))
    );
    // An Aura with enchant player can't be attached.
    let curse = t.battlefield(P1, "Curse of the Pierced Heart");
    assert!(!t.g.attach(curse, pl(P0)), "Auras can't be attached");
    // Skullcrack (at P1): damage can't be prevented. Earthquake's damage to each player
    // is dealt to P0.
    let mut t = TestGame::new(2);
    let coffin = t.battlefield(P0, "The Stasis Coffin");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, coffin, 0, &[]).expect("The Stasis Coffin");
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Skullcrack", &[pl(P1)]);
    t.resolve_all();
    t.lands(P1, "Mountain", 3);
    let quake = t.hand(P1, "Earthquake");
    t.cast(P1, quake).x(2).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    let _ = Zone::Battlefield;
}
