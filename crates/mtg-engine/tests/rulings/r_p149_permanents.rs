//! Rulings batch P149 — permanents' abilities: Codsworth's attaching and mana, Hunter's
//! Bow's damage, Kitesail Larcenist's partial targets, Ancient Adamantoise's redirection,
//! Vren's exile replacement, Tangletrove Kelp's Clues (layers), the locked-in sets of
//! Cactusfolk Sureshot and Mouth of the Storm, and Frostfist Strider's target.

use crate::r_p149_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Codsworth, Handy Helper -----------------------------------------------------------------

/// P0's Codsworth ("{T}: Attach target Aura or Equipment you control to target creature
/// you control. Activate only as a sorcery.") activates its last ability; it stays on the
/// stack.
fn codsworth_attaches(t: &mut TestGame, codsworth: ObjectId, what: ObjectId, to: ObjectId) {
    supported("Codsworth, Handy Helper");
    act_only(t, P0, codsworth, 1, &[obj(what), obj(to)]);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn codsworth_doesnt_move_what_cant_legally_be_attached() {
    cr!("701.3b", "702.16c");
    ruling!(
        "Codsworth, Handy Helper",
        "As Codsworth’s last ability resolves, if the target Aura or Equipment can’t legally be attached to the target creature (for example, because the creature has protection from enchantments or protection from artifacts), the target Aura or Equipment won’t move."
    );
    let mut t = TestGame::new(2);
    let codsworth = t.battlefield(P0, "Codsworth, Handy Helper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let rancor = attach_new(&mut t, P0, "Rancor", obj(bears));
    codsworth_attaches(&mut t, codsworth, rancor, giant);
    // In response, the Giant gains protection from green (Rancor's color): Rancor can't
    // enchant it (Codsworth, white, still targets it legally).
    crate::r_p125_common::choose_color(&mut t, P0, Color::Green);
    cast_at(&mut t, P0, "Gods Willing", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, rancor), Some(obj(bears)));
}

#[test]
fn codsworth_does_nothing_with_an_illegal_target_or_an_existing_attachment() {
    cr!("608.2b", "701.3b");
    ruling!(
        "Codsworth, Handy Helper",
        "If either target of Codsworth’s last ability is an illegal target as the ability resolves, the ability won’t do anything. If both targets are illegal, the ability won’t resolve. If the target Aura or Equipment is already attached to the target creature, nothing happens."
    );
    // The creature is gone: the Equipment stays where it is.
    let mut t = TestGame::new(2);
    let codsworth = t.battlefield(P0, "Codsworth, Handy Helper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let gear = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    codsworth_attaches(&mut t, codsworth, gear, giant);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(attached_to(&t, gear), Some(obj(bears)));
    // The Equipment is gone: nothing happens to the creature.
    let mut t = TestGame::new(2);
    let codsworth = t.battlefield(P0, "Codsworth, Handy Helper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let gear = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    codsworth_attaches(&mut t, codsworth, gear, giant);
    kill(&mut t, gear);
    t.resolve_all();
    assert_eq!(t.pt(giant), (3, 3));
    // Both gone: the ability doesn't resolve (it's removed from the stack).
    let mut t = TestGame::new(2);
    let codsworth = t.battlefield(P0, "Codsworth, Handy Helper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let gear = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    codsworth_attaches(&mut t, codsworth, gear, giant);
    kill(&mut t, gear);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    // Already attached there: nothing happens (it stays attached, with no new timestamp
    // of attachment).
    let mut t = TestGame::new(2);
    let codsworth = t.battlefield(P0, "Codsworth, Handy Helper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gear = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    let ts = t.obj_now(gear).timestamp;
    codsworth_attaches(&mut t, codsworth, gear, bears);
    t.resolve_all();
    assert_eq!(attached_to(&t, gear), Some(obj(bears)));
    assert_eq!(t.obj_now(gear).timestamp, ts);
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn codsworth_can_move_your_aura_from_an_opponents_creature() {
    cr!("115.1", "701.3a");
    ruling!(
        "Codsworth, Handy Helper",
        "The Aura or Equipment you target with Codsworth’s last ability doesn’t have to be attached to a creature you control when you target it. You just have to control the Aura or Equipment you’re targeting as well as the creature you’re targeting."
    );
    let mut t = TestGame::new(2);
    let codsworth = t.battlefield(P0, "Codsworth, Handy Helper");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    // P0's Rancor on P1's Bears.
    let rancor = attach_new(&mut t, P0, "Rancor", obj(theirs));
    assert_eq!(t.obj_now(rancor).controller, P0);
    codsworth_attaches(&mut t, codsworth, rancor, mine);
    t.resolve_all();
    assert_eq!(attached_to(&t, rancor), Some(obj(mine)));
    assert_eq!(t.pt(mine), (5, 3));
}

#[test]
fn codsworth_mana_cant_pay_for_equip_or_aura_abilities() {
    cr!("106.6", "702.6a");
    ruling!(
        "Codsworth, Handy Helper",
        "You can’t spend mana generated by Codsworth’s second ability to activate abilities of Auras or Equipment. This includes paying equip costs."
    );
    let mut t = TestGame::new(2);
    let codsworth = t.battlefield(P0, "Codsworth, Handy Helper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gear = t.battlefield(P0, "Bonesplitter");
    t.activate(P0, codsworth, 0, &[]).unwrap();
    assert_eq!(t.g.players[0].mana_pool.mana.len(), 2);
    // Equip {1}: not with Codsworth's mana.
    assert!(t.activate(P0, gear, 0, &[obj(bears)]).is_err());
    assert_eq!(t.g.players[0].mana_pool.mana.len(), 2);
    // An Aura spell: yes (Pacifism, {1}{W}).
    supported("Pacifism");
    let giant = t.battlefield(P1, "Hill Giant");
    let pacifism = t.hand(P0, "Pacifism");
    t.cast(P0, pacifism).target(obj(giant)).go();
    assert_eq!(t.g.players[0].mana_pool.mana.len(), 0);
}

// --- Hunter's Bow ----------------------------------------------------------------------------

#[test]
fn hunters_bow_no_damage_if_either_creature_is_an_illegal_target() {
    cr!("608.2b", "701.14");
    ruling!(
        "Hunter's Bow",
        "If either creature is an illegal target as Hunter's Bow's first ability tries to resolve, the creature you control won't deal damage."
    );
    supported("Hunter's Bow");
    for gone in [true, false] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        enter_targeting(&mut t, P0, "Hunter's Bow", &[obj(mine), obj(theirs)]);
        assert_eq!(t.stack_len(), 1);
        if gone {
            // The creature you control leaves: it deals no damage (last known information
            // isn't used for it).
            crate::r_p198_common::exile(&mut t, mine);
        }
        t.resolve_all();
        if gone {
            assert!(t.on_battlefield(theirs));
            assert_eq!(crate::r_p003_common::dmg(&t, theirs), 0);
        } else {
            assert!(!t.on_battlefield(theirs));
        }
    }
}

// --- Kitesail Larcenist ----------------------------------------------------------------------

#[test]
fn kitesail_larcenist_only_the_legal_targets_become_treasures() {
    cr!("608.2b", "115.3");
    ruling!(
        "Kitesail Larcenist",
        "If some of the targets, but not all, are illegal as the ability resolves, the illegal targets will be unaffected. Only the remaining legal targets will become Treasures."
    );
    supported("Kitesail Larcenist");
    let mut t = TestGame::new(3);
    let bears2 = t.battlefield(P1, "Grizzly Bears");
    let giant2 = t.battlefield(P2, "Hill Giant");
    // Its targets: P1's Bears, P2's Giant (P0 controls no other artifact or creature).
    t.answer_targets(P0, &[obj(bears2)]);
    t.answer_targets(P0, &[obj(giant2)]);
    t.enter(P0, "Kitesail Larcenist");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // The Giant leaves the battlefield and comes back as a new object: an illegal target.
    let giant3 = {
        crate::r_p198_common::exile(&mut t, giant2);
        let id = t.g.current(giant2);
        t.g.move_object(
            id,
            Zone::Battlefield,
            mtg_engine::events::MoveCause::Effect,
            Some(P2),
        )
        .unwrap()
    };
    t.resolve_all();
    assert!(t.obj_now(bears2).chars.has_subtype("Treasure"));
    assert!(!t.obj_now(giant3).chars.has_subtype("Treasure"));
    assert!(t.obj_now(giant3).is(CardType::Creature));
}

// --- Ancient Adamantoise ---------------------------------------------------------------------

#[test]
fn two_ancient_adamantoises_you_choose_one_for_all_the_damage() {
    cr!("616.1", "614.9");
    ruling!(
        "Ancient Adamantoise",
        "If you control more than one Ancient Adamantoise, you choose which redirection effect to apply. You can't divide damage dealt by a single source between them. For example, if an attacking creature would deal 8 damage to you and you control two Ancient Adamantoises, you'll choose which of those Ancient Adamantoises will take that damage. You can't have 4 damage be dealt to each Ancient Adamantoise, and you can't choose to take the damage yourself."
    );
    supported("Ancient Adamantoise");
    for pick in [0, 1] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Ancient Adamantoise");
        let b = t.battlefield(P0, "Ancient Adamantoise");
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.set_step(P1, Step::PrecombatMain);
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        t.g.deal_damage(wurm, pl(P0), 6, true);
        t.g.flush_events();
        t.settle();
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseReplacement { .. })));
        assert_eq!(t.life(P0), 20);
        let (da, db) = (
            crate::r_p003_common::dmg(&t, a),
            crate::r_p003_common::dmg(&t, b),
        );
        assert!((da, db) == (6, 0) || (da, db) == (0, 6), "{da} {db}");
    }
}

// --- Vren, the Relentless --------------------------------------------------------------------

#[test]
fn vren_exiles_opponents_creatures_instead_and_dies_triggers_dont_trigger() {
    cr!("614.1a", "700.4", "603.6c");
    ruling!(
        "Vren, the Relentless",
        "While Vren is on the battlefield, creatures your opponents control are exiled instead of dying, and abilities that trigger when a creature dies won't trigger."
    );
    supported("Vren, the Relentless");
    supported("Doomed Traveler");
    supported("Blood Artist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vren, the Relentless");
    t.battlefield(P0, "Blood Artist");
    let traveler = t.battlefield(P1, "Doomed Traveler");
    kill(&mut t, traveler);
    t.resolve_all();
    assert_eq!(t.zone(traveler), Zone::Exile);
    assert!(crate::r_s01_common::with_subtype(&t, P1, "Spirit").is_empty());
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn vren_leaving_with_the_creatures_still_exiles_them() {
    cr!("614.1a", "603.10a", "614.12");
    ruling!(
        "Vren, the Relentless",
        "If Vren would leave the battlefield at the same time as one or more creatures an opponent controls would die, those creatures will still be exiled."
    );
    supported("Wrath of God");
    let mut t = TestGame::new(2);
    let vren = t.battlefield(P0, "Vren, the Relentless");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    cast_at(&mut t, P0, "Wrath of God", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Vren, the Relentless"));
    assert_eq!(t.zone(vren), Zone::Graveyard(P0));
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!t.on_battlefield(token));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn vren_counts_exiled_creature_tokens() {
    cr!("111.1", "406.3");
    ruling!(
        "Vren, the Relentless",
        "Vren's last ability counts any creatures your opponents controlled that were exiled this turn, including creature tokens."
    );
    supported("Swords to Plowshares");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vren, the Relentless");
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // The token dies (exiled instead); the Bears are exiled by Swords to Plowshares.
    kill(&mut t, token);
    cast_at(&mut t, P0, "Swords to Plowshares", &[obj(bears)]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // Two Rats: the token counts though it has ceased to exist (CR 704.5d).
    assert_eq!(tokens(&t, P0).len(), 2);
    // A creature exiled and returned to the battlefield before the end step still counts.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vren, the Relentless");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_p198_common::exile(&mut t, bears);
    let id = t.g.current(bears);
    t.g.move_object(
        id,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P1),
    )
    .unwrap();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

// --- Tangletrove Kelp ------------------------------------------------------------------------

#[test]
fn tangletrove_kelp_clues_keep_their_abilities() {
    cr!("613.1d", "205.1b");
    ruling!(
        "Tangletrove Kelp",
        "Clues you control will keep any abilities they previously had while they’re creatures."
    );
    supported("Tangletrove Kelp");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tangletrove Kelp");
    let clue = crate::r_s02_common::create_token(&mut t, P0, "Clue");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj_now(clue).is(CardType::Creature));
    assert_eq!(t.pt(clue), (6, 6));
    // "{2}, Sacrifice this token: Draw a card."
    let hand = t.hand_size(P0);
    act(&mut t, P0, clue, 0, &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn tangletrove_kelp_overwrites_earlier_pt_setting_but_not_modifications() {
    cr!("613.4b", "613.4c", "613.7a");
    ruling!(
        "Tangletrove Kelp",
        "If a Clue you control is already a creature when Tangletrove Kelp’s second ability resolves, Tangletrove Kelp’s second ability will overwrite any previous effects that set that creature’s power and toughness to specific numbers. Effects that otherwise modify its power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Ensoul Artifact");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tangletrove Kelp");
    let clue = crate::r_s02_common::create_token(&mut t, P0, "Clue");
    // Ensoul Artifact: a 5/5 creature; Giant Growth: +3/+3; a +1/+1 counter.
    cast_at(&mut t, P0, "Ensoul Artifact", &[obj(clue)]);
    t.resolve_all();
    assert_eq!(t.pt(clue), (5, 5));
    cast_at(&mut t, P0, "Giant Growth", &[obj(clue)]);
    t.resolve_all();
    put(&mut t, clue, counters::PLUS1, 1);
    assert_eq!(t.pt(clue), (9, 9));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(clue), (6 + 3 + 1, 6 + 3 + 1));
}

// --- Locked-in sets --------------------------------------------------------------------------

#[test]
fn cactusfolk_sureshot_affects_only_the_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Cactusfolk Sureshot",
        "Cactusfolk Sureshot’s last ability affects only creatures you control at the time it resolves. Creatures you control with power 4 or greater you begin to control later in the turn won’t gain trample or haste."
    );
    supported("Cactusfolk Sureshot");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cactusfolk Sureshot");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let kw = |t: &TestGame, id, k| crate::r_s06_common::has_kw(t, id, k);
    assert!(kw(&t, wurm, mtg_engine::keywords::KeywordKind::Trample));
    let later = t.battlefield(P0, "Craw Wurm");
    let theirs = t.battlefield(P1, "Craw Wurm");
    crate::r_s06_common::give_control(&mut t, theirs, P0);
    for id in [later, theirs] {
        assert!(!kw(&t, id, mtg_engine::keywords::KeywordKind::Trample));
        assert!(!kw(&t, id, mtg_engine::keywords::KeywordKind::Haste));
    }
}

#[test]
fn mouth_of_the_storm_affects_only_creatures_opponents_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Mouth of the Storm",
        "Mouth of the Storm’s last ability affects only creatures your opponents control at the time it resolves. Creatures they begin to control after the ability resolves but before your next turn begins won’t get -3/-0."
    );
    supported("Mouth of the Storm");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    enter_targeting(&mut t, P0, "Mouth of the Storm", &[]);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (3, 4));
    let later = t.battlefield(P1, "Craw Wurm");
    assert_eq!(t.pt(later), (6, 4));
    t.set_step(P1, Step::PrecombatMain);
    assert_eq!(t.pt(wurm), (3, 4));
}

#[test]
fn frostfist_strider_can_target_a_tapped_creature() {
    cr!("115.1", "122.1d");
    ruling!(
        "Frostfist Strider",
        "You may choose a creature that is already tapped as the target of Frostfist Strider’s last ability."
    );
    supported("Frostfist Strider");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    enter_targeting(&mut t, P0, "Frostfist Strider", &[obj(giant)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(t.counters(giant, "stun"), 1);
}
