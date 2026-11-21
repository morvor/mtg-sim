//! Rulings batch P042 — redirection effects (CR 614.9: Harm's Way, Carom, Martyrdom,
//! Palisade Giant) and prevention effects that deal damage of their own when they prevent
//! (Divine Deflection, Refraction Trap; CR 615.5).

use crate::r_p042_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Harm's Way ------------------------------------------------------------------------------

/// P0 casts Harm's Way targeting `to`, choosing `source` as it resolves.
fn harms_way(t: &mut TestGame, to: Entity, source: ObjectId) {
    supported("Harm's Way");
    t.answer_choose(P0, &[obj(source)]);
    cast_resolve(t, P0, "Harm's Way", &[to]);
}

#[test]
fn harms_way_target_on_cast_source_on_resolution_any_of_your_recipients() {
    cr!("614.9", "609.7a", "601.2c");
    ruling!(
        "Harm's Way",
        "As you cast Harm’s Way, you target that the redirected damage will be dealt to. As Harm’s Way resolves, you choose a source of damage. You never choose the original recipient of the damage; Harm’s Way will apply to whatever the chosen source tries to deal damage to, as long as it’s you or a permanent you control."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    lands_for_cost(&mut t, P0, "Harm's Way");
    let card = t.hand(P0, "Harm's Way");
    t.answer_targets(P0, &[obj(wurm)]);
    let spell = t.cast_with(P0, card, &[]).unwrap();
    // The target is chosen as it's cast; the source isn't chosen yet.
    assert_eq!(crate::r_s25_common::targets_of(&t, spell), vec![obj(wurm)]);
    let chose_before = t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. }));
    assert!(!chose_before, "no source chosen while casting");
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    // The Giant's damage to P0's Bears (a permanent P0 controls) is redirected: 1 of the
    // 2 to the Bears, then the remaining 1 to P0.
    damage(&mut t, giant, obj(bears), 1);
    assert_eq!(damage_marked(&t, bears), 0);
    assert_eq!(damage_marked(&t, wurm), 1);
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 18, "1 of 3 redirected");
    assert_eq!(damage_marked(&t, wurm), 2);
}

#[test]
fn harms_way_redirects_to_a_target_that_became_illegal_but_not_one_that_cant_be_dealt_damage() {
    cr!("614.9");
    ruling!(
        "Harm's Way",
        "After Harm’s Way resolves, it no longer checks to see if the target is a legal target. However, if that permanent or player can’t be dealt damage at the time the chosen source would deal damage (perhaps because the creature is no longer on the battlefield or is no longer a creature, or because the player is no longer in the game), the damage can’t be redirected. It’s dealt to the original recipient."
    );
    supported("Gods Willing");
    // The target gains protection from white (Harm's Way's color) after it resolves: still
    // redirected (the Giant is red, so its damage isn't prevented).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    harms_way(&mut t, obj(bears), giant);
    t.set_step(P1, Step::PrecombatMain);
    crate::r_p125_common::choose_color(&mut t, P1, mtg_engine::types::Color::White);
    cast_resolve(&mut t, P1, "Gods Willing", &[obj(bears)]);
    damage(&mut t, giant, pl(P0), 2);
    assert_eq!(t.life(P0), 20);
    assert!(!t.on_battlefield(bears), "2 damage killed the Bears");
    // The target left the battlefield: the damage is dealt to P0.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    harms_way(&mut t, obj(bears), giant);
    exile_now(&mut t, bears);
    damage(&mut t, giant, pl(P0), 2);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn harms_way_doesnt_affect_damage_already_dealt() {
    cr!("614.1", "614.9");
    ruling!(
        "Harm's Way",
        "Harm’s Way has no effect on damage that’s already been dealt."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P0, "Craw Wurm");
    damage(&mut t, giant, obj(bears), 2);
    harms_way(&mut t, obj(wurm), giant);
    assert_eq!(damage_marked(&t, bears), 2, "already dealt: stays");
    assert_eq!(damage_marked(&t, wurm), 0);
    damage(&mut t, giant, obj(bears), 2);
    assert_eq!(damage_marked(&t, bears), 2);
    assert_eq!(damage_marked(&t, wurm), 2);
}

// --- Carom, Martyrdom, Palisade Giant ---------------------------------------------------------

#[test]
fn carom_draws_on_resolution_and_redirects_only_while_both_are_there() {
    cr!("614.9", "608.2c");
    ruling!(
        "Carom",
        "Carom’s controller draws a card when Carom resolves, not when the damage is redirected."
    );
    ruling!(
        "Carom",
        "If either target has left the battlefield before the damage would be dealt, that damage isn’t redirected."
    );
    supported("Carom");
    // 0: both stay; 1: the first target leaves; 2: the second target leaves.
    for leaves in 0..3 {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let from = t.battlefield(P0, "Craw Wurm");
        let to = t.battlefield(P1, "Grizzly Bears");
        let hand = t.hand_size(P0);
        cast_resolve(&mut t, P0, "Carom", &[obj(from), obj(to)]);
        assert_eq!(t.hand_size(P0), hand + 1, "drew as it resolved");
        assert!(damage_events(&t).is_empty());
        match leaves {
            1 => exile_now(&mut t, from),
            2 => exile_now(&mut t, to),
            _ => {}
        }
        if leaves == 1 {
            // Nothing to redirect from; the Bears isn't dealt damage by anything.
            assert_eq!(damage_marked(&t, to), 0);
            continue;
        }
        damage(&mut t, giant, obj(from), 2);
        if leaves == 0 {
            assert_eq!((damage_marked(&t, from), damage_marked(&t, to)), (1, 1));
        } else {
            assert_eq!(damage_marked(&t, from), 2, "not redirected");
        }
        assert_eq!(t.hand_size(P0), hand + 1, "no draw on redirection");
    }
}

#[test]
fn martyrdom_redirecting_from_the_creature_to_itself() {
    cr!("614.9");
    ruling!(
        "Martyrdom",
        "Can be used to redirect damage from that creature to itself. This has no particularly useful effect."
    );
    supported("Martyrdom");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Martyrdom", &[obj(wurm)]);
    activate_resolve(&mut t, P0, wurm, 0, &[obj(wurm)]);
    damage(&mut t, giant, obj(wurm), 3);
    assert_eq!(damage_marked(&t, wurm), 3);
    assert_eq!(damage_events(&t), vec![(giant, obj(wurm), 3)]);
}

#[test]
fn palisade_giant_redirected_combat_damage_is_still_combat_damage() {
    cr!("614.9", "510.2");
    ruling!(
        "Palisade Giant",
        "Applying this redirection effect doesn’t change whether the damage is combat damage."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Palisade Giant");
    let attacker = t.battlefield(P1, "Hill Giant");
    to_beginning_of_combat(&mut t, P1);
    t.attack(&[(attacker, pl(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(combat_damage(&t, attacker, obj(giant)), 3);
    assert_eq!(damage_marked(&t, giant), 3);
    // Noncombat damage stays noncombat.
    damage(&mut t, attacker, pl(P0), 1);
    assert_eq!(
        damage_by(&t, attacker),
        vec![(obj(giant), 3, true), (obj(giant), 1, false)]
    );
}

// --- Divine Deflection and Refraction Trap ----------------------------------------------------

/// P0 casts Divine Deflection with X = `x` targeting `to`; it resolves.
fn deflection(t: &mut TestGame, x: i64, to: Entity) -> ObjectId {
    supported("Divine Deflection");
    t.lands(P0, "Plains", 1 + x as usize);
    let card = t.hand(P0, "Divine Deflection");
    let spell = t.cast(P0, card).target(to).x(x).go();
    t.resolve_all();
    spell
}

#[test]
fn divine_deflection_targets_on_cast_and_prevents_damage_to_you_and_your_permanents() {
    cr!("615.5", "601.2c", "615.7");
    ruling!(
        "Divine Deflection",
        "Divine Deflection’s only target is the permanent or player it may deal damage to. You choose that target as you cast Divine Deflection, not at the time it prevents damage."
    );
    ruling!(
        "Divine Deflection",
        "Divine Deflection can prevent damage that would be dealt to you, one or more creatures you control, and/or one or more planeswalkers you control."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pw = t.battlefield(P0, "Gideon Jura");
    supported("Divine Deflection");
    t.lands(P0, "Plains", 4);
    let card = t.hand(P0, "Divine Deflection");
    let spell = t.cast(P0, card).target(pl(P1)).x(3).go();
    assert_eq!(crate::r_s25_common::targets_of(&t, spell), vec![pl(P1)]);
    t.resolve_all();
    damage(&mut t, giant, obj(bears), 1);
    damage(&mut t, giant, obj(pw), 1);
    damage(&mut t, giant, pl(P0), 1);
    assert_eq!(damage_marked(&t, bears), 0);
    assert_eq!(t.counters(pw, "loyalty"), 6);
    assert_eq!(t.life(P0), 20);
    assert_eq!(prevented_events(&t).len(), 3);
    assert_eq!(t.life(P1), 17, "1 damage to P1 each time");
    // Used up.
    damage(&mut t, giant, pl(P0), 1);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn divine_deflection_is_the_source_of_its_noncombat_damage() {
    cr!("615.5", "120.2b", "702.2b", "702.15b");
    ruling!(
        "Divine Deflection",
        "Divine Deflection’s effect is not a redirection effect. If it prevents damage, Divine Deflection (not the original source) deals damage to the targeted permanent or player as part of that prevention effect. Divine Deflection is the source of the new damage, so the characteristics of the original source (such as its color, or whether it had lifelink or deathtouch) don’t affect this damage. The new damage is not combat damage, even if the prevented damage was."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let knight = t.battlefield(P1, "Vampire Nighthawk");
    let spell = deflection(&mut t, 2, obj(wurm));
    to_beginning_of_combat(&mut t, P1);
    t.attack(&[(knight, pl(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(prevented_events(&t), vec![(knight, pl(P0), 2)]);
    assert_eq!(damage_by(&t, spell), vec![(obj(wurm), 2, false)]);
    assert!(t.on_battlefield(wurm), "no deathtouch");
    assert_eq!(damage_marked(&t, wurm), 2);
    assert_eq!(t.life(P1), 20, "no lifelink");
}

#[test]
fn divine_deflection_still_prevents_if_its_target_is_gone() {
    cr!("615.5", "609.3");
    ruling!(
        "Divine Deflection",
        "If Divine Deflection can’t deal damage to the targeted permanent or player (because the creature is no longer on the battlefield, or is no longer a creature, or the player is no longer in the game, for example), it will still prevent damage. It just won’t deal any damage itself."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = deflection(&mut t, 2, obj(bears));
    exile_now(&mut t, bears);
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 19);
    assert_eq!(prevented_events(&t), vec![(giant, pl(P0), 2)]);
    assert!(damage_by(&t, spell).is_empty());
}

#[test]
fn divine_deflection_deals_its_damage_right_after_the_excess_before_the_spell_continues() {
    cr!("615.5", "608.2c");
    ruling!(
        "Divine Deflection",
        "If Divine Deflection prevents damage, excess damage (if any) dealt by that source is dealt at the same time. Immediately afterward, as part of that same prevention effect, Divine Deflection deals its damage. This happens before state-based actions are checked, and before the spell or ability that caused damage to be dealt resumes its resolution."
    );
    supported("Lightning Helix");
    // X = 2; P1's Lightning Helix deals 3 to P0 (1 gets through) and P1 gains 3.
    let mut t = TestGame::new(2);
    let spell = deflection(&mut t, 2, pl(P1));
    t.set_step(P1, Step::PrecombatMain);
    let helix = cast_resolve(&mut t, P1, "Lightning Helix", &[pl(P0)]);
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 21);
    let order: Vec<&str> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage { source, .. } if *source == helix => Some("helix damage"),
                Event::Damage { source, .. } if *source == spell => Some("deflection damage"),
                Event::LifeGained { player, .. } if *player == P1 => Some("helix life"),
                _ => None,
            })
            .collect();
    assert_eq!(
        order,
        vec!["helix damage", "deflection damage", "helix life"]
    );
}

#[test]
fn divine_deflection_you_choose_which_damage_to_prevent_as_it_is_dealt() {
    cr!("615.7");
    ruling!(
        "Divine Deflection",
        "Divine Deflection can prevent damage from multiple sources dealing damage simultaneously (such as during combat). If the chosen value for X won’t prevent all the damage, you choose which sources to prevent damage from."
    );
    ruling!(
        "Divine Deflection",
        "If damage is dealt to multiple permanents you control, or is dealt to you and at least one permanent you control, you choose which of that damage to prevent if the chosen value for X won’t prevent all the damage."
    );
    interp!("shield-divided-among-recipients");
    // X = 2. Two sources each deal 2 to P0 at once: P0 orders, so the Bears' damage is
    // prevented, then the Giant's.
    for first in [1usize, 0] {
        let mut t = TestGame::new(2);
        deflection(&mut t, 2, pl(P1));
        let giant = t.battlefield(P1, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer(
            P0,
            DecisionKind::Order,
            Answer::Indices(if first == 1 { vec![1, 0] } else { vec![0, 1] }),
        );
        damage_at_once(&mut t, &[(giant, pl(P0), 2), (bears, pl(P0), 2)], true);
        let prevented_from = if first == 1 { bears } else { giant };
        assert_eq!(prevented_events(&t), vec![(prevented_from, pl(P0), 2)]);
        assert_eq!(t.life(P0), 18);
    }
    // One source deals 2 to P0 and 2 to P0's Wurm at once: as the damage is dealt, P0
    // divides the 2 prevention between them (asked then, not earlier).
    for (to_p0, to_wurm) in [(2i64, 0i64), (0, 2), (1, 1)] {
        let mut t = TestGame::new(2);
        deflection(&mut t, 2, pl(P1));
        let giant = t.battlefield(P1, "Hill Giant");
        let wurm = t.battlefield(P0, "Craw Wurm");
        let from = t.asked().len();
        divide(&mut t, P0, &[to_p0, to_wurm]);
        damage_at_once(&mut t, &[(giant, pl(P0), 2), (giant, obj(wurm), 2)], false);
        let asked: Vec<_> = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::Divide { .. }))
            .cloned()
            .collect();
        assert_eq!(asked.len(), 1, "P0 chose as the damage was dealt");
        assert_eq!(
            (t.life(P0), damage_marked(&t, wurm)),
            (18 + to_p0 as i32, 2 - to_wurm as u32)
        );
        assert_eq!(t.life(P1), 18, "2 damage prevented in all");
    }
}

#[test]
fn refraction_trap_deals_no_damage_if_it_prevented_none() {
    cr!("615.5", "616.1", "615.12");
    ruling!(
        "Refraction Trap",
        "If Refraction Trap doesn’t prevent any damage (perhaps because a different prevention effect is applied to the damage the source would deal, or because the damage is unpreventable), Refraction Trap won’t deal any damage itself."
    );
    supported("Refraction Trap");
    supported("Banefire");
    /// P0 casts Refraction Trap at P1, choosing `source`.
    fn trap(t: &mut TestGame, source: ObjectId) -> ObjectId {
        t.answer_choose(P0, &[obj(source)]);
        cast_resolve(t, P0, "Refraction Trap", &[pl(P1)])
    }
    // Another prevention effect applied first: Pay No Heed on the same source.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = trap(&mut t, giant);
    t.answer_choose(P0, &[obj(giant)]);
    cast_resolve(&mut t, P0, "Pay No Heed", &[]);
    apply_first(&mut t, P0, "Pay No Heed");
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 20);
    assert_eq!(
        prevented_events(&t).len(),
        1,
        "prevented once, by Pay No Heed"
    );
    assert!(damage_by(&t, spell).is_empty());
    assert_eq!(t.life(P1), 20);
    // Unpreventable damage: Banefire with X = 5.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 6);
    let fire = t.hand(P1, "Banefire");
    let fire = t.cast(P1, fire).target(pl(P0)).x(5).go();
    let spell = trap(&mut t, fire);
    t.resolve_all();
    assert_eq!(t.life(P0), 15);
    assert!(damage_by(&t, spell).is_empty());
    assert_eq!(t.life(P1), 20);
}
