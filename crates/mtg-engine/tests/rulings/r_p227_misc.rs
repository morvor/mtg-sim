//! Rulings batch P227 — vanishing (Chronozoa), warp (Astelli Reclaimer's "amount of mana
//! spent"), wither (CR 702.80), and creatures: Hardy Veteran, Foundry Inspector, Steal
//! Artifact.

use crate::r_s01_common::*;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s29_common::put_counters;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn next_upkeep(t: &mut TestGame, p: PlayerId) {
    let other = if p == P0 { P1 } else { P0 };
    if t.g.turn.active == p {
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

/// P0's Chronozoa with one time counter left.
fn chronozoa_with_one_counter(t: &mut TestGame) -> ObjectId {
    supported("Chronozoa");
    let c = t.enter(P0, "Chronozoa");
    assert_eq!(t.counters(c, "time"), 3);
    t.g.objects[c.0 as usize].counters.clear();
    t.g.dirty = true;
    put_counters(t, c, "time", 1);
    assert_eq!(t.counters(c, "time"), 1);
    c
}

#[test]
fn chronozoa_checks_counters_however_it_died() {
    cr!("702.63a", "603.10a", "603.4");
    ruling!(
        "Chronozoa",
        "The ability checks to see if Chronozoa had no time counters on it at the time it was put into a graveyard from the battlefield."
    );
    // The last counter is removed; with the sacrifice ability on the stack, Chronozoa is
    // destroyed: it had no time counters, so two token copies are created.
    let mut t = TestGame::new(2);
    let c = chronozoa_with_one_counter(&mut t);
    next_upkeep(&mut t, P0);
    t.resolve();
    assert_eq!(t.counters(c, "time"), 0);
    assert!(t.on_battlefield(c));
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, c);
    assert!(t.in_graveyard(P0, "Chronozoa"));
    t.resolve_all();
    let copies = tokens(&t, P0);
    assert_eq!(copies.len(), 2);
    for tok in copies {
        assert_eq!(t.obj_now(tok).chars.name, "Chronozoa");
    }
    // Destroyed while it still has a time counter: no copies.
    let mut t = TestGame::new(2);
    let c = chronozoa_with_one_counter(&mut t);
    destroy(&mut t, c);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Chronozoa"));
    assert!(tokens(&t, P0).is_empty());
}

/// P0 casts Astelli Reclaimer (normally, or with warp, with or without Helm of
/// Awakening, "Spells cost {1} less to cast."), with Mind Stone (MV 2) and Jayemdae Tome
/// (MV 4) in their graveyard; returns the Reclaimer's enters-trigger target candidates.
fn reclaimer_candidates(warp: bool, helm: bool) -> (TestGame, Vec<Entity>) {
    supported("Astelli Reclaimer");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let stone = t.graveyard(P0, "Mind Stone");
    let tome = t.graveyard(P0, "Jayemdae Tome");
    if helm {
        t.battlefield(P1, "Helm of Awakening");
    }
    let paid = match (warp, helm) {
        (false, false) => 5,
        (false, true) => 4,
        (true, false) => 3,
        (true, true) => 2,
    };
    t.lands(P0, "Plains", paid);
    let r = t.hand(P0, "Astelli Reclaimer");
    let from = t.asked().len();
    let b = t.cast(P0, r);
    let b = if warp {
        b.method(CastMethod::Keyword(KeywordKind::Warp))
    } else {
        b
    };
    b.go();
    assert_eq!(tapped_lands(&t, P0), paid);
    t.resolve_all();
    let cands: Vec<Entity> = target_candidates(&t, P0, from)
        .into_iter()
        .flatten()
        .filter(|e| [Entity::Object(stone), Entity::Object(tome)].contains(e))
        .collect();
    (t, cands)
}

#[test]
fn astelli_reclaimer_x_is_the_mana_actually_spent() {
    cr!("601.2h", "702.185a", "601.2f");
    ruling!(
        "Astelli Reclaimer",
        "Astelli Reclaimer’s enters ability cares about the mana you actually paid to cast Astelli Reclaimer, not its mana cost."
    );
    // {3}{W}{W}: X is 5.
    let (_, c) = reclaimer_candidates(false, false);
    assert_eq!(c.len(), 2);
    // Warp {2}{W}: X is 3, so the Tome (MV 4) can't be chosen.
    let (t, c) = reclaimer_candidates(true, false);
    assert_eq!(c.len(), 1);
    assert_eq!(t.obj_now(c[0].object().unwrap()).chars.name, "Mind Stone");
    // A cost reduction counts: {4} spent normally (X is 4), {2} with warp (X is 2).
    let (_, c) = reclaimer_candidates(false, true);
    assert_eq!(c.len(), 2);
    let (t, c) = reclaimer_candidates(true, true);
    assert_eq!(c.len(), 1);
    assert_eq!(t.obj_now(c[0].object().unwrap()).chars.name, "Mind Stone");
}

/// The -1/-1 counters on `id`.
fn minus(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, "-1/-1")
}

/// P0's `attacker` (a creature with wither) attacks and P1's Colossal Dreadmaw (6/6)
/// blocks it.
fn blocked_by_dreadmaw(t: &mut TestGame, attacker: ObjectId) -> ObjectId {
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    block_and_finish(t, P1, &[(dreadmaw, attacker)]);
    dreadmaw
}

#[test]
fn hateflayer_wither_applies_to_combat_and_ability_damage() {
    cr!("702.80a", "120.3d", "702.80c");
    ruling!(
        "Hateflayer",
        "Wither applies to any damage Hateflayer deals to a creature, which includes both its combat damage and damage from its activated ability."
    );
    supported("Hateflayer");
    // Combat damage: five -1/-1 counters on the Dreadmaw, no damage marked.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hateflayer");
    let d = blocked_by_dreadmaw(&mut t, h);
    assert_eq!(minus(&t, d), 5);
    assert_eq!(t.obj_now(d).damage, 0);
    assert_eq!(t.pt(d), (1, 1));
    // The activated ability ({2}{R}, {Q}: damage equal to its power).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let h = t.battlefield(P0, "Hateflayer");
    t.g.objects[h.0 as usize].tapped = true;
    let d = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 3);
    t.activate(P0, h, 0, &[Entity::Object(d)]).expect("Hateflayer");
    t.resolve_all();
    assert_eq!(minus(&t, d), 5);
    assert_eq!(t.obj_now(d).damage, 0);
    // To a player it's ordinary damage.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let h = t.battlefield(P0, "Hateflayer");
    t.g.objects[h.0 as usize].tapped = true;
    t.lands(P0, "Mountain", 3);
    t.activate(P0, h, 0, &[Entity::Player(P1)])
        .expect("Hateflayer");
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn village_pillagers_wither_applies_to_its_enters_damage_too() {
    cr!("702.80a", "120.3d");
    ruling!(
        "Village Pillagers",
        "Wither applies to any damage dealt to creatures by Village Pillagers."
    );
    supported("Village Pillagers");
    // Its enters ability: a -1/-1 counter on each creature P1 controls.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    let p = t.enter(P0, "Village Pillagers");
    t.resolve_all();
    assert_eq!(minus(&t, bears), 1);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.pt(bears), (1, 1));
    assert_eq!(minus(&t, mine), 0);
    // Combat damage.
    t.g.objects[p.0 as usize].summoning_sick = false;
    let d = blocked_by_dreadmaw(&mut t, p);
    assert_eq!(minus(&t, d), 5);
    assert_eq!(t.obj_now(d).damage, 0);
}

#[test]
fn spinerock_tyrant_wither_applies_to_noncombat_damage_too() {
    cr!("702.80a", "120.3d", "701.14a");
    ruling!(
        "Spinerock Tyrant",
        "Any damage dealt to creatures by a source with wither, whether it's combat damage or noncombat damage, is dealt in the form of -1/-1 counters."
    );
    // Its wither compiles (its copy trigger isn't supported).
    let c = mtg_engine::card::card("Spinerock Tyrant");
    assert!(c
        .unsupported_text()
        .iter()
        .all(|u| u.starts_with("Whenever you cast an instant or sorcery spell")));
    // Noncombat: it fights P1's Ancient Brontodon (9/9) via Prey Upon.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tyrant = t.battlefield(P0, "Spinerock Tyrant");
    let bronto = t.battlefield(P1, "Ancient Brontodon");
    give_mana_for(&mut t, P0, "Prey Upon");
    let prey = t.hand(P0, "Prey Upon");
    t.cast(P0, prey)
        .targets(&[Entity::Object(tyrant)])
        .targets(&[Entity::Object(bronto)])
        .go();
    t.resolve_all();
    assert_eq!(minus(&t, bronto), 6);
    assert_eq!(t.obj_now(bronto).damage, 0);
    assert_eq!(t.pt(bronto), (3, 3));
    // Combat: it blocks the Brontodon.
    let mut t = TestGame::new(2);
    let tyrant = t.battlefield(P0, "Spinerock Tyrant");
    let bronto = t.battlefield(P1, "Ancient Brontodon");
    t.set_step(P1, Step::BeginningOfCombat);
    crate::r_s03_common::to_blockers(
        &mut t,
        &[(bronto, Entity::Player(P0))],
        &[(tyrant, bronto)],
    );
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(minus(&t, bronto), 6);
    assert_eq!(t.obj_now(bronto).damage, 0);
}

#[test]
fn hardy_veteran_damage_wears_off_before_its_bonus() {
    cr!("514.2", "611.3a");
    ruling!(
        "Hardy Veteran",
        "Hardy Veteran gets +0/+2 for the entire duration of your turn. If it’s dealt damage or gets -X/-X until end of turn, those will wear off before your turn is over."
    );
    supported("Hardy Veteran");
    // 3 damage on P0's turn (2/4): it survives into P1's turn as an undamaged 2/2.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let v = t.battlefield(P0, "Hardy Veteran");
    assert_eq!(t.pt(v), (2, 4));
    give_mana_for(&mut t, P0, "Lightning Bolt");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Object(v)).go();
    t.resolve_all();
    assert!(t.on_battlefield(v));
    assert_eq!(t.obj_now(v).damage, 3);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(v));
    assert_eq!(t.obj_now(v).damage, 0);
    assert_eq!(t.pt(v), (2, 2));
    // -2/-2 until end of turn (Disfigure) on P0's turn: 0/2, then a 2/2.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let v = t.battlefield(P0, "Hardy Veteran");
    give_mana_for(&mut t, P0, "Disfigure");
    let d = t.hand(P0, "Disfigure");
    t.cast(P0, d).target(Entity::Object(v)).go();
    t.resolve_all();
    assert_eq!(t.pt(v), (0, 2));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(v));
    assert_eq!(t.pt(v), (2, 2));
}

#[test]
fn foundry_inspector_reduces_after_x_is_chosen() {
    cr!("601.2b", "601.2f", "107.3a");
    ruling!(
        "Foundry Inspector",
        "If an artifact spell has {X} in its mana cost, choose the value for X first, and then reduce the cost by {1}."
    );
    supported("Foundry Inspector");
    supported("Walking Ballista");
    // Walking Ballista ({X}{X}) with X = 2 costs {3}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Foundry Inspector");
    t.lands(P0, "Wastes", 3);
    let b = t.hand(P0, "Walking Ballista");
    t.cast(P0, b).x(2).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    let ballista = t.named_on_battlefield("Walking Ballista")[0];
    assert_eq!(t.counters(ballista, "+1/+1"), 2);
}

#[test]
fn foundry_inspector_cant_be_removed_while_the_cost_is_determined() {
    cr!("601.2", "601.2f", "601.2i");
    ruling!(
        "Foundry Inspector",
        "Once a player has announced an artifact spell, no player may take actions to try to remove Foundry Inspector from the battlefield before that spell's cost is locked in."
    );
    supported("Foundry Inspector");
    // P0 casts Jayemdae Tome ({4}) for {3}: nobody is asked anything (no priority)
    // between the announcement and the payment.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let inspector = t.battlefield(P0, "Foundry Inspector");
    t.lands(P0, "Wastes", 3);
    let tome = t.hand(P0, "Jayemdae Tome");
    let from = t.asked().len();
    let spell = t.cast(P0, tome).go();
    assert!(asked_since(&t, from).iter().all(|(p, d)| *p == P0
        && !matches!(d, mtg_engine::decision::Decision::Priority { .. })));
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_eq!(tapped_lands(&t, P0), 3);
    // Removing the Inspector afterwards doesn't change the cost that was paid.
    destroy(&mut t, inspector);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Jayemdae Tome").len(), 1);
}

#[test]
fn steal_artifact_can_enchant_an_artifact_creature() {
    cr!("303.4a", "702.5a", "613.1b");
    ruling!("Steal Artifact", "Can be used on artifact creatures.");
    supported("Steal Artifact");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let thopter = t.battlefield(P1, "Ornithopter");
    give_mana_for(&mut t, P0, "Steal Artifact");
    let s = t.hand(P0, "Steal Artifact");
    t.cast(P0, s).target(Entity::Object(thopter)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(thopter).controller, P0);
    assert!(t.obj_now(thopter).chars.is_creature());
}
