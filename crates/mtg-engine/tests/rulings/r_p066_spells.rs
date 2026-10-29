//! Rulings batch P066 — "Giant Growth" spells: untap spells on untapped creatures, spells
//! whose single target became illegal (CR 608.2b), spells with several targets of which
//! some became illegal, the same object chosen for two "target" words (CR 115.3), divided
//! counters (CR 601.2d), charms (CR 700.2), and assorted effects.

use crate::r_p066_common::*;
use crate::r_s01_common::{attack_with, supported, with_subtype};
use crate::r_s02_common::destroy;
use crate::r_s04_common::spell_targets;
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s29_common::{choose_modes, divide};
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn has_kw(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(kw)
}

/// Casts the Adventure half of `name` from P0's hand with the given targets.
fn cast_adventure(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Forest", 4);
    let c = t.hand(P0, name);
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.cast(P0, c).method(CastMethod::Half(1)).go()
}

// ---------------------------------------------------------------------------
// "Untap target creature" spells on an untapped creature
// ---------------------------------------------------------------------------

#[test]
fn untap_spells_can_target_an_untapped_creature() {
    cr!("115.1a", "608.2c");
    ruling!(
        "Aim High",
        "Aim High can target a creature that’s already untapped. It still gets +2/+2 and gains reach until end of turn."
    );
    ruling!(
        "Burst of Strength",
        "Burst of Strength can target and put a +1/+1 counter on a creature that’s already untapped."
    );
    ruling!(
        "Dragonscale Boon",
        "Dragonscale Boon can target any creature, including one that’s already untapped."
    );
    ruling!("Savage Surge", "Savage Surge can target an untapped creature.");
    ruling!(
        "Spidery Grasp",
        "Spidery Grasp can target a creature that’s already untapped. It will still get +2/+4 and gain reach."
    );
    ruling!(
        "Stony Strength",
        "Stony Strength can target any creature you control, even one that's already untapped."
    );
    ruling!(
        "Sudden Spinnerets",
        "Sudden Spinnerets can target a creature that’s already untapped. It will still get +1/+3 and a reach counter."
    );
    ruling!(
        "Ornamental Courage",
        "You can cast Ornamental Courage targeting an untapped creature. It will still get +1/+3 until end of turn."
    );
    ruling!(
        "Vines of the Recluse",
        "You can target an untapped creature with Vines of the Recluse. It will still get the other bonuses."
    );
    // (card, P/T of a Grizzly Bears afterwards, +1/+1 counters, keyword gained)
    let cases: &[(&str, (i32, i32), u32, Option<KeywordKind>)] = &[
        ("Aim High", (4, 4), 0, Some(KeywordKind::Reach)),
        ("Burst of Strength", (3, 3), 1, None),
        ("Dragonscale Boon", (4, 4), 2, None),
        ("Savage Surge", (4, 4), 0, None),
        ("Spidery Grasp", (4, 6), 0, Some(KeywordKind::Reach)),
        ("Stony Strength", (3, 3), 1, None),
        ("Sudden Spinnerets", (3, 5), 0, Some(KeywordKind::Reach)),
        ("Ornamental Courage", (3, 5), 0, None),
        ("Vines of the Recluse", (3, 4), 0, Some(KeywordKind::Reach)),
    ];
    for (name, pt, plus, kw) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        assert!(!tapped(&t, bears));
        cast_new(&mut t, P0, name, &[o(bears)]);
        t.resolve_all();
        assert!(!tapped(&t, bears), "{name}");
        assert_eq!(t.pt(bears), *pt, "{name}");
        assert_eq!(t.counters(bears, counters::PLUS1), *plus, "{name}");
        if let Some(k) = kw {
            assert!(has_kw(&t, bears, *k), "{name}");
        }
    }
    // Sudden Spinnerets' reach is a counter: it stays after the turn ends.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Sudden Spinnerets", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "reach"), 1);
    t.advance_to(P1, Step::Upkeep);
    assert!(has_kw(&t, bears, KeywordKind::Reach));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn adventure_untap_spells_can_target_an_untapped_creature() {
    cr!("115.1a", "715.3a");
    ruling!(
        "Dread Linnorm // Scale Deflection",
        "Scale Deflection can target an untapped creature. If the target creature is untapped as Scale Deflection resolves, you will still put two +1/+1 counters on it and it will still gain hexproof."
    );
    ruling!(
        "Silverflame Squire // On Alert",
        "On Alert can target a creature that's already untapped. It will still get +2/+2."
    );
    supported("Dread Linnorm // Scale Deflection");
    supported("Silverflame Squire // On Alert");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_adventure(&mut t, "Dread Linnorm // Scale Deflection", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert!(has_kw(&t, bears, KeywordKind::Hexproof));
    assert!(!tapped(&t, bears));
    let card = t.g.find_in_zone(Zone::Exile, "Dread Linnorm");
    assert_eq!(card.len(), 1, "the Adventure wasn't exiled");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_adventure(&mut t, "Silverflame Squire // On Alert", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(!tapped(&t, bears));
}

#[test]
fn stony_strength_targets_only_a_creature_you_control() {
    cr!("115.1a");
    ruling!(
        "Stony Strength",
        "Stony Strength can target any creature you control, even one that's already untapped."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(mine);
    let cands = spell_targets(&mut t, P0, "Stony Strength");
    assert!(cands.contains(&o(mine)));
    assert!(!cands.contains(&o(theirs)));
}

#[test]
fn colossal_heroics_can_target_untapped_creatures() {
    cr!("115.1a", "702.92a");
    ruling!(
        "Colossal Heroics",
        "Colossal Heroics can target any creatures, not just ones that are tapped."
    );
    supported("Colossal Heroics");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.g.tap(a);
    // Strive: {2}{G} plus {1}{G} for the second target.
    t.lands(P0, "Forest", 5);
    let c = t.hand(P0, "Colossal Heroics");
    t.cast(P0, c).targets(&[o(a), o(b)]).go();
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.pt(b), (5, 5));
    assert!(!tapped(&t, a));
    assert!(!tapped(&t, b));
}

#[test]
fn silk_net_lasts_until_end_of_turn() {
    cr!("514.2", "611.2a");
    ruling!(
        "Silk Net",
        "Both the +1/+1 and reach only last until end of turn."
    );
    supported("Silk Net");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Silk Net", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(has_kw(&t, bears, KeywordKind::Reach));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!has_kw(&t, bears, KeywordKind::Reach));
}

// ---------------------------------------------------------------------------
// A single target that became illegal: the spell doesn't resolve
// ---------------------------------------------------------------------------

/// P0 casts `name` targeting P0's `target`; `kill` makes it illegal in response; the
/// spell doesn't resolve (it goes to the graveyard, CR 608.2b).
fn fizzles(t: &mut TestGame, name: &str, target: ObjectId, kill: impl FnOnce(&mut TestGame)) {
    supported(name);
    let s = cast_new(t, P0, name, &[o(target)]);
    kill(t);
    t.resolve_all();
    assert!(!t.g.stack.iter().any(|x| *x == s));
    assert!(t.in_graveyard(P0, name), "{name}");
}

#[test]
fn spells_with_an_illegal_only_target_dont_resolve() {
    cr!("608.2b");
    ruling!(
        "Take Heart",
        "If the target creature becomes an illegal target for Take Heart, the spell doesn’t resolve. You don’t gain life."
    );
    ruling!(
        "Aggressive Urge",
        "If the target creature is an illegal target by the time Aggressive Urge tries to resolve, the spell doesn’t resolve. You won’t draw a card."
    );
    ruling!(
        "Desperate Lunge",
        "If the target creature is an illegal target by the time Desperate Lunge tries to resolve, the spell doesn't resolve. You don't gain 2 life."
    );
    ruling!(
        "Moment of Triumph",
        "If the target creature is an illegal target by the time Moment of Triumph tries to resolve, it won't resolve and none of its effects will happen. You won't gain 2 life."
    );
    ruling!(
        "Wildsize",
        "If the target creature is an illegal target by the time Wildsize tries to resolve, the spell doesn't resolve. You don't draw a card."
    );
    for name in [
        "Take Heart",
        "Aggressive Urge",
        "Desperate Lunge",
        "Moment of Triumph",
        "Wildsize",
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        // An attacking creature for Take Heart to count.
        let other = t.battlefield(P0, "Hill Giant");
        if name == "Take Heart" {
            attack_with(&mut t, &[(other, Entity::Player(P1))]);
        }
        let hand = t.hand_size(P0);
        fizzles(&mut t, name, bears, |t| destroy(t, bears));
        assert_eq!(t.life(P0), 20, "{name}");
        assert_eq!(t.hand_size(P0), hand, "{name}");
        assert_eq!(t.pt(other), (3, 3), "{name}");
    }
    // Control: they do all that with a legal target.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    cast_new(&mut t, P0, "Take Heart", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Wildsize", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn echoing_courage_with_an_illegal_target_pumps_nothing() {
    cr!("608.2b");
    ruling!(
        "Echoing Courage",
        "If the target creature is an illegal target by the time Echoing Courage tries to resolve, the spell doesn't resolve. No creatures get +2/+2."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    fizzles(&mut t, "Echoing Courage", a, |t| destroy(t, a));
    assert_eq!(t.pt(b), (2, 2));
}

/// Gives `id` protection from green until end of turn with Gods Willing (P0 chooses
/// green), resolving it.
fn protect_from_green(t: &mut TestGame, id: ObjectId) {
    supported("Gods Willing");
    crate::r_s35_common::pick_options(t, P0, &["green"]);
    cast_new(t, P0, "Gods Willing", &[o(id)]);
    t.resolve();
    assert!(
        !spell_targets(t, P1, "Giant Growth").contains(&o(id)),
        "no protection from green"
    );
}

#[test]
fn echoing_courage_pumps_a_same_named_creature_it_doesnt_target() {
    cr!("115.1", "702.16b");
    ruling!(
        "Echoing Courage",
        "Echoing Courage has only one target. Other creatures with that name are not targeted. For example, a creature with protection from green will still get +2/+2 if it has the same name as the target creature."
    );
    supported("Echoing Courage");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    protect_from_green(&mut t, b);
    let s = cast_new(&mut t, P0, "Echoing Courage", &[o(a)]);
    assert_eq!(crate::r_s25_common::targets_of(&t, s), vec![o(a)]);
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.pt(b), (4, 4));
    assert_eq!(t.pt(other), (3, 3));
}

#[test]
fn barge_in_with_an_illegal_target_gives_no_trample() {
    cr!("608.2b");
    ruling!(
        "Barge In",
        "If the target attacking creature is an illegal target by the time Barge In tries to resolve, the spell won't resolve. Your attacking non-Human creatures won't gain trample."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    fizzles(&mut t, "Barge In", a, |t| destroy(t, a));
    assert!(!has_kw(&t, b, KeywordKind::Trample));
    // With a legal target, the other attacker gains trample.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    cast_new(&mut t, P0, "Barge In", &[o(a)]);
    t.resolve_all();
    assert!(has_kw(&t, b, KeywordKind::Trample));
}

#[test]
fn joint_assault_with_an_illegal_target_doesnt_pump_the_paired_creature() {
    cr!("608.2b", "702.95a");
    ruling!(
        "Joint Assault",
        "Joint Assault targets only one creature. If that creature is an illegal target when Joint Assault tries to resolve, it won’t resolve and none of its effects will happen. No creature will get +2/+2."
    );
    supported("Silverblade Paladin");
    // Grizzly Bears paired with Silverblade Paladin.
    let pair = |t: &mut TestGame| {
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[o(bears)]);
        t.answer_choose(P0, &[o(bears)]);
        let p = t.enter(P0, "Silverblade Paladin");
        t.resolve_all();
        assert_eq!(t.obj_now(p).paired_with, Some(bears), "not paired");
        (bears, p)
    };
    let mut t = TestGame::new(2);
    let (bears, p) = pair(&mut t);
    let s = cast_new(&mut t, P0, "Joint Assault", &[o(bears)]);
    protect_from_green(&mut t, bears);
    assert_eq!(t.obj_now(p).paired_with, Some(bears));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Joint Assault"));
    assert!(!t.g.stack.contains(&s));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(p), (2, 2));
    // Legal: both get +2/+2.
    let mut t = TestGame::new(2);
    let (bears, p) = pair(&mut t);
    cast_new(&mut t, P0, "Joint Assault", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(p), (4, 4));
}

// ---------------------------------------------------------------------------
// Several targets, some illegal
// ---------------------------------------------------------------------------

#[test]
fn synchronized_strike_with_one_illegal_target_affects_the_other() {
    cr!("608.2b");
    ruling!(
        "Synchronized Strike",
        "If Synchronized Strike targets two creatures and one becomes an illegal target before it resolves, the remaining target creature will still be untapped and get +2/+2."
    );
    supported("Synchronized Strike");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.g.tap(a);
    t.g.tap(b);
    lands_for_cost(&mut t, P0, "Synchronized Strike");
    let c = t.hand(P0, "Synchronized Strike");
    t.cast(P0, c).targets(&[o(a), o(b)]).go();
    destroy(&mut t, a);
    t.resolve_all();
    assert!(!tapped(&t, b));
    assert_eq!(t.pt(b), (5, 5));
}

#[test]
fn synchronized_strike_takes_zero_one_or_two_different_targets() {
    cr!("115.3", "601.2c");
    ruling!(
        "Synchronized Strike",
        "You can cast Synchronized Strike with two, one, or zero targets."
    );
    ruling!(
        "Synchronized Strike",
        "You can’t target the same creature twice to have it get +4/+4."
    );
    supported("Synchronized Strike");
    for n in 0..=2usize {
        let mut t = TestGame::new(2);
        let cs: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
        for c in &cs {
            t.g.tap(*c);
        }
        lands_for_cost(&mut t, P0, "Synchronized Strike");
        let s = t.hand(P0, "Synchronized Strike");
        let targets: Vec<Entity> = cs[..n].iter().map(|x| o(*x)).collect();
        t.cast(P0, s).targets(&targets).go();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Synchronized Strike"), "{n}");
        for (i, c) in cs.iter().enumerate() {
            assert_eq!(!tapped(&t, *c), i < n, "{n} targets");
            assert_eq!(t.pt(*c), if i < n { (4, 4) } else { (2, 2) });
        }
    }
    // The same creature twice: not a legal choice of targets (one slot, distinct
    // targets).
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Synchronized Strike");
    let s = t.hand(P0, "Synchronized Strike");
    t.answer_targets(P0, &[o(a), o(a)]);
    let _ = t.g.cast_spell(P0, s, CastMethod::Normal);
    t.resolve_all();
    assert!(t.pt(a).0 <= 4, "it got +4/+4");
}

#[test]
fn leeching_bite_with_one_illegal_target_affects_the_other() {
    cr!("608.2b");
    ruling!(
        "Leeching Bite",
        "If only one of the targeted creatures is still a legal target when Leeching Bite tries to resolve, that creature will still be affected, but the illegal target won’t be."
    );
    supported("Leeching Bite");
    // The first target illegal (it left): the second gets -1/-1.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Leeching Bite", &[o(a), o(b)]);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.pt(b), (2, 2));
    // The second target illegal (protection from green, still on the battlefield): it
    // doesn't get -1/-1; the first gets +1/+1.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Leeching Bite", &[o(a), o(b)]);
    protect_from_green(&mut t, b);
    t.resolve_all();
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(b), (3, 3));
}

#[test]
fn press_the_advantage_with_one_illegal_target_affects_the_other() {
    cr!("608.2b");
    ruling!(
        "Press the Advantage",
        "If you cast Press the Advantage with two targets, and only one of them is still a legal target as Press the Advantage resolves, that target will get +2/+2 and gain trample. Press the Advantage will have no effect on the illegal target."
    );
    supported("Press the Advantage");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Press the Advantage");
    let s = t.hand(P0, "Press the Advantage");
    t.cast(P0, s).targets(&[o(a), o(b)]).go();
    protect_from_green(&mut t, b);
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 4));
    assert!(has_kw(&t, a, KeywordKind::Trample));
    assert_eq!(t.pt(b), (3, 3));
    assert!(!has_kw(&t, b, KeywordKind::Trample));
}

#[test]
fn defend_the_celestus_distribution_is_chosen_on_cast_and_kept() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Defend the Celestus",
        "You decide how to distribute the counters as you cast Defend the Celestus. You must distribute at least one counter to each creature you target."
    );
    ruling!(
        "Defend the Celestus",
        "If some, but not all, of the target creatures are illegal targets as Defend the Celestus resolves, the original distribution of counters remains the same. Counters that were to be put on the now-illegal targets are lost."
    );
    supported("Defend the Celestus");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Defend the Celestus");
    let s = t.hand(P0, "Defend the Celestus");
    // Two counters on the Bears, one on the Giant.
    let from = t.asked().len();
    divide(&mut t, P0, &[2, 1]);
    t.cast(P0, s).targets(&[o(a), o(b)]).go();
    // The division was asked as it was cast.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Divide { .. })));
    assert_eq!(t.stack_len(), 1);
    // The Giant (1 counter) becomes illegal: the Bears still gets only its 2.
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert_eq!(t.counters(c, counters::PLUS1), 0);
    // A division with a zero for one target isn't accepted.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Defend the Celestus");
    let s = t.hand(P0, "Defend the Celestus");
    divide(&mut t, P0, &[3, 0]);
    t.cast(P0, s).targets(&[o(a), o(b)]).go();
    t.resolve_all();
    assert!(t.counters(b, counters::PLUS1) >= 1, "a target got no counter");
    assert_eq!(
        t.counters(a, counters::PLUS1) + t.counters(b, counters::PLUS1),
        3
    );
}

// ---------------------------------------------------------------------------
// The same object for two "target" words (CR 115.3)
// ---------------------------------------------------------------------------

#[test]
fn common_bond_can_target_the_same_creature_twice() {
    cr!("115.3");
    ruling!(
        "Common Bond",
        "You may choose the same creature for both targets. You may also choose two different creatures. This is because the word “target” is used twice."
    );
    supported("Common Bond");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Common Bond", &[o(a), o(a)]);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Common Bond", &[o(a), o(b)]);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

#[test]
fn river_heralds_boon_can_target_one_merfolk_twice() {
    cr!("115.3");
    ruling!(
        "River Heralds' Boon",
        "You can choose one Merfolk creature as both targets for River Heralds’ Boon. You can also choose two different Merfolk creatures."
    );
    supported("River Heralds' Boon");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Merfolk of the Pearl Trident");
    cast_new(&mut t, P0, "River Heralds' Boon", &[o(m), o(m)]);
    t.resolve_all();
    assert_eq!(t.counters(m, counters::PLUS1), 2);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let n = t.battlefield(P0, "Merfolk of the Pearl Trident");
    cast_new(&mut t, P0, "River Heralds' Boon", &[o(m), o(n)]);
    t.resolve_all();
    assert_eq!(t.counters(m, counters::PLUS1), 1);
    assert_eq!(t.counters(n, counters::PLUS1), 1);
    // The second target must be a Merfolk.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "River Heralds' Boon");
    let s = t.hand(P0, "River Heralds' Boon");
    let from = t.asked().len();
    t.answer_targets(P0, &[o(m)]);
    t.answer_targets(P0, &[o(m)]);
    t.cast(P0, s).go();
    let slots = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(slots[0].contains(&o(bears)));
    assert!(!slots[1].contains(&o(bears)));
}

// ---------------------------------------------------------------------------
// Charms
// ---------------------------------------------------------------------------

#[test]
fn a_charm_is_only_its_chosen_mode_on_the_stack() {
    cr!("700.2a", "601.2b", "601.2c");
    ruling!(
        "Esper Charm",
        "While the spell is on the stack, treat it as though its only text is the chosen mode. The other two modes are treated as though they don’t exist. You don’t choose targets for those modes."
    );
    ruling!(
        "Grixis Charm",
        "While the spell is on the stack, treat it as though its only text is the chosen mode. The other two modes are treated as though they don’t exist. You don’t choose targets for those modes."
    );
    ruling!(
        "Jund Charm",
        "While the spell is on the stack, treat it as though its only text is the chosen mode. The other two modes are treated as though they don’t exist. You don’t choose targets for those modes."
    );
    // (charm, untargeted mode)
    for (name, mode) in [("Esper Charm", 1), ("Grixis Charm", 2), ("Jund Charm", 1)] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Glorious Anthem");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.graveyard(P1, "Lightning Bolt");
        lands_for_cost(&mut t, P0, name);
        let c = t.hand(P0, name);
        let from = t.asked().len();
        choose_modes(&mut t, P0, &[mode]);
        let s = t.cast(P0, c).go();
        assert!(
            !t.asked()[from..]
                .iter()
                .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })),
            "{name}: targets chosen for other modes"
        );
        assert!(crate::r_s25_common::targets_of(&t, s).is_empty(), "{name}");
        t.resolve_all();
        // Only the chosen mode happened.
        assert!(t.on_battlefield(bears) || name == "Jund Charm", "{name}");
        assert!(t.in_graveyard(P1, "Lightning Bolt"), "{name}");
        assert_eq!(t.g.find_in_zone(Zone::Battlefield, "Glorious Anthem").len(), 1);
    }
}

#[test]
fn a_charm_mode_needs_legal_targets() {
    cr!("700.2a", "601.2c");
    ruling!(
        "Esper Charm",
        "You can choose a mode only if you can choose legal targets for that mode. If you can’t choose legal targets for any of the modes, you can’t cast the spell."
    );
    ruling!(
        "Grixis Charm",
        "You can choose a mode only if you can choose legal targets for that mode. If you can’t choose legal targets for any of the modes, you can’t cast the spell."
    );
    ruling!(
        "Jund Charm",
        "You can choose a mode only if you can choose legal targets for that mode. If you can’t choose legal targets for any of the modes, you can’t cast the spell."
    );
    // Esper Charm with no enchantment on the battlefield: "Destroy target enchantment"
    // can't be chosen; choosing it falls back to a legal mode.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Esper Charm");
    let c = t.hand(P0, "Esper Charm");
    let from = t.asked().len();
    choose_modes(&mut t, P0, &[0]);
    let s = t.cast(P0, c).go();
    let offered: Vec<Vec<usize>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert!(!offered[0].contains(&0), "the enchantment mode was available");
    assert!(offered[0].contains(&1));
    assert_ne!(crate::r_s26_common::modes_on_stack(&t, s), vec![0]);
    // Grixis Charm with a creature on the battlefield: all modes available. Every charm
    // here has an untargeted mode, so it can always be cast; a charm whose every mode
    // targets can't be cast with no legal targets (Bant Charm, no artifacts, creatures
    // or instants).
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Bant Charm");
    let c = t.hand(P0, "Bant Charm");
    assert!(!crate::r_s21_common::castable(&mut t, P0, c));
    t.battlefield(P1, "Grizzly Bears");
    assert!(crate::r_s21_common::castable(&mut t, P0, c));
}

// ---------------------------------------------------------------------------
// Assorted
// ---------------------------------------------------------------------------

#[test]
fn take_heart_with_no_attackers_still_pumps() {
    cr!("608.2h");
    ruling!(
        "Take Heart",
        "You can cast Take Heart even if you control no attacking creatures, perhaps because it’s not even your turn. The target creature gets +2/+2 even though you don’t gain life."
    );
    supported("Take Heart");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Take Heart", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn confront_the_unknown_counts_the_clue_it_just_made() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Confront the Unknown",
        "No player may take any action in between the two steps of Confront the Unknown, so the target creature will get at least +1/+1."
    );
    supported("Confront the Unknown");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Confront the Unknown", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    // With two Clues already: +3/+3.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s02_common::create_token(&mut t, P0, "Clue");
    crate::r_s02_common::create_token(&mut t, P0, "Clue");
    let s = cast_new(&mut t, P0, "Confront the Unknown", &[o(bears)]);
    // Nobody gets priority during the resolution: resolve the spell alone.
    assert_eq!(t.stack.last(), Some(&s));
    t.g.resolve_top();
    assert_eq!(with_subtype(&t, P0, "Clue").len(), 3);
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn swell_of_growth_land_isnt_a_land_play() {
    cr!("305.2", "305.4");
    ruling!(
        "Swell of Growth",
        "Putting a land onto the battlefield this way is not the same as playing a land. Swell of Growth doesn’t affect how many lands you can play from your hand during your turn."
    );
    supported("Swell of Growth");
    // After Swell of Growth's land, a land can still be played.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let f = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(f)]);
    cast_new(&mut t, P0, "Swell of Growth", &[o(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(f)));
    let m = t.hand(P0, "Mountain");
    assert!(t.play_land(P0, m).is_ok());
    // After playing a land, Swell of Growth can still put one onto the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let m = t.hand(P0, "Mountain");
    t.play_land(P0, m).unwrap();
    let f = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(f)]);
    cast_new(&mut t, P0, "Swell of Growth", &[o(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(f)));
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn tawnoss_tinkering_on_an_old_artifact_or_land_can_attack_and_tap() {
    cr!("302.6", "508.1a");
    ruling!(
        "Tawnos's Tinkering",
        "If an artifact or land that becomes a creature this way has been on the battlefield since the beginning of your upkeep (even if it wasn't a creature at the time), it can attack during your turn and its abilities may be activated."
    );
    ruling!(
        "Tawnos's Tinkering",
        "Tawnos's Tinkering doesn't grant haste, so if a noncreature permanent that entered the battlefield since the beginning of your upkeep becomes a creature this way, it can't attack until your next turn. Any activated abilities that it has with a {T} symbol in their cost also can't be activated until your next upkeep step. Notably, this means a land that entered the battlefield after your last upkeep can't be tapped for mana if it becomes a creature this way."
    );
    supported("Tawnos's Tinkering");
    for old in [true, false] {
        let mut t = TestGame::new(2);
        let land = if old {
            t.battlefield(P0, "Forest")
        } else {
            t.battlefield_sick(P0, "Forest")
        };
        t.lands(P0, "Forest", 4);
        let tink = t.hand(P0, "Tawnos's Tinkering");
        t.answer_targets(P0, &[o(land)]);
        // Pay with the other Forests.
        t.cast(P0, tink).go();
        t.resolve_all();
        assert!(t.obj_now(land).is(CardType::Creature), "old: {old}");
        assert_eq!(t.pt(land), (2, 2));
        assert_eq!(
            crate::r_s20_common::tap_for_mana(&mut t, P0, land, "Add {G}"),
            old,
            "old: {old}"
        );
        t.g.untap(land);
        t.set_step(P0, Step::BeginningOfCombat);
        assert_eq!(crate::r_s02_common::can_attack(&mut t, land), old, "old: {old}");
    }
}
