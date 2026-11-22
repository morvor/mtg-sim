//! Rulings batch P046 — the targets of cost-reduced spells: spells that target spells
//! (Bolt Bend, Not of This World; CR 115.7, 115.9), "becomes the target" triggers (CR
//! 603.2c, 603.3), fight spells with two targets (CR 608.2b, 701.14), and spells that
//! can't be countered by ward (CR 702.21a, 101.2).

use crate::r_p046_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 casts Forked Bolt (a sorcery) in its main phase with the given targets (2 damage
/// divided among them).
fn p1_forked_bolt(t: &mut TestGame, targets: &[Entity]) -> ObjectId {
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let s = t.hand(P1, "Forked Bolt");
    t.g.players[1].mana_pool.add_type(ManaType::R, 1);
    t.cast(P1, s).targets(targets).go()
}

/// P1 casts Seeds of Strength targeting `c` three times.
fn p1_seeds_thrice(t: &mut TestGame, c: ObjectId) -> ObjectId {
    supported("Seeds of Strength");
    let s = t.hand(P1, "Seeds of Strength");
    t.g.players[1].mana_pool.add_type(ManaType::G, 1);
    t.g.players[1].mana_pool.add_type(ManaType::W, 1);
    t.cast(P1, s).target(c).target(c).target(c).go()
}

/// Whether P0 can cast `name` targeting `spell` (with plenty of mana).
fn can_target(t: &mut TestGame, name: &str, spell: ObjectId) -> bool {
    let s = t.hand(P0, name);
    let ok = cost_to_cast(t, P0, s, &[Entity::Object(spell)]).is_some();
    if !ok {
        assert_eq!(t.zone(s), Zone::Hand(P0));
    }
    ok
}

// ---------------------------------------------------------------------------------------
// Bolt Bend: "Change the target of target spell or ability with a single target."
// ---------------------------------------------------------------------------------------

#[test]
fn bolt_bend_cant_target_a_spell_with_several_targets() {
    cr!("115.9a");
    ruling!(
        "Bolt Bend",
        "If a spell or ability targets multiple things, you can't target it with Bolt Bend, even if all but one of those targets have become illegal."
    );
    supported("Bolt Bend");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let fb = p1_forked_bolt(&mut t, &[Entity::Player(P0), Entity::Object(b)]);
    assert!(!can_target(&mut t, "Bolt Bend", fb));
    // The Bears leave: one legal target left, but still two targets.
    bounce(&mut t, b);
    assert!(!can_target(&mut t, "Bolt Bend", fb));
    // A Forked Bolt with one target can be targeted.
    let mut t = TestGame::new(2);
    let fb = p1_forked_bolt(&mut t, &[Entity::Player(P0)]);
    assert!(can_target(&mut t, "Bolt Bend", fb));
}

#[test]
fn bolt_bend_cant_target_a_spell_targeting_one_object_several_times() {
    cr!("115.9a", "115.3");
    ruling!(
        "Bolt Bend",
        "If a spell or ability targets the same player or object multiple times, you can't target it with Bolt Bend."
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P1, "Grizzly Bears");
    let seeds = p1_seeds_thrice(&mut t, b);
    assert!(!can_target(&mut t, "Bolt Bend", seeds));
}

/// P1's Shock targets `shock_target`; P0 (with Craw Wurm, so Bolt Bend costs {R}) casts
/// Bolt Bend targeting the Shock. Returns the Shock.
fn bolt_bend_a_shock(t: &mut TestGame, shock_target: Entity) -> ObjectId {
    t.battlefield(P0, "Craw Wurm");
    let shock = p1_shocks(t, shock_target);
    let s = t.hand(P0, "Bolt Bend");
    t.g.players[0].mana_pool.add_type(ManaType::R, 1);
    t.cast(P0, s).target(shock).go();
    shock
}

#[test]
fn bolt_bend_chooses_the_new_target_as_it_resolves() {
    cr!("115.7a", "608.2c");
    ruling!(
        "Bolt Bend",
        "The single target that the target spell or ability targets doesn't have to be a creature you control with power 4 or greater."
    );
    ruling!(
        "Bolt Bend",
        "You don't choose the new target for the spell or ability until Bolt Bend resolves."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    bolt_bend_a_shock(&mut t, Entity::Player(P0));
    let targets_chosen = |t: &TestGame| {
        asked_since(t, from)
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. }))
            .count()
    };
    // Only Bolt Bend's own target so far.
    assert_eq!(targets_chosen(&t), 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(targets_chosen(&t), 2);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
}

#[test]
fn bolt_bend_must_change_the_target_to_a_legal_one_if_possible() {
    cr!("115.7a");
    ruling!(
        "Bolt Bend",
        "You must change the target if possible. However, you can't change the target to an illegal target. If there are no legal targets to choose from, the target isn't changed. It doesn't matter if the original target has somehow become illegal itself."
    );
    // With no answer given, the target still changes (to another legal one).
    let mut t = TestGame::new(2);
    bolt_bend_a_shock(&mut t, Entity::Player(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Shock"));
    // P1's Unsummon targets P0's Bears; P0's only other creature, Gladecover Scout, has
    // hexproof (an illegal target for Unsummon): the target isn't changed to the Scout.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let scout = t.battlefield(P0, "Gladecover Scout");
    let unsummon = t.hand(P1, "Unsummon");
    t.g.players[1].mana_pool.add_type(ManaType::U, 1);
    t.cast(P1, unsummon).target(bears).go();
    let bb = t.hand(P0, "Bolt Bend");
    assert_eq!(cost_to_cast(&mut t, P0, bb, &[Entity::Object(unsummon)]), Some(4));
    t.answer_targets(P0, &[Entity::Object(scout)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    assert!(t.on_battlefield(scout));
    // The original target became illegal (the Bears left): the target is still changed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    bolt_bend_a_shock(&mut t, Entity::Object(bears));
    bounce(&mut t, bears);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------------------
// Not of This World: "Counter target spell or ability that targets a permanent you
// control."
// ---------------------------------------------------------------------------------------

#[test]
fn not_of_this_world_needs_a_permanent_you_control_among_the_targets() {
    cr!("115.9b", "601.2c");
    ruling!(
        "Not of This World",
        "If a spell or ability targets you, but not a permanent you control, you can't cast Not of This World targeting it."
    );
    supported("Not of This World");
    let mut t = TestGame::new(2);
    let shock = p1_shocks(&mut t, Entity::Player(P0));
    assert!(!can_target(&mut t, "Not of This World", shock));
    ruling!(
        "Not of This World",
        "The spell or ability you target with Not of This World may have any number of targets, as long as one of them is a permanent you control."
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let fb = p1_forked_bolt(&mut t, &[Entity::Player(P0), Entity::Object(b)]);
    assert!(can_target(&mut t, "Not of This World", fb));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Forked Bolt"));
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(b));
}

#[test]
fn not_of_this_world_ignores_whether_the_targets_are_still_legal() {
    cr!("115.9b");
    ruling!(
        "Not of This World",
        "If, in response, another spell or ability causes that creature to gain shroud, then you can still cast Not of This World targeting Forked Bolt."
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let fb = p1_forked_bolt(&mut t, &[Entity::Player(P0), Entity::Object(b)]);
    // Ranger's Guile: the Bears gain hexproof (an illegal target for P1's Forked Bolt).
    let guile = t.hand(P0, "Ranger's Guile");
    t.g.players[0].mana_pool.add_type(ManaType::G, 1);
    t.cast(P0, guile).target(b).go();
    t.resolve();
    assert!(can_target(&mut t, "Not of This World", fb));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Forked Bolt"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn not_of_this_world_checks_who_controls_the_targets_now() {
    cr!("115.9b", "608.2b");
    ruling!(
        "Not of This World",
        "If, in response, another spell or ability causes that creature to no longer be under your control, then you can't cast Not of This World targeting Forked Bolt."
    );
    supported("Ray of Command");
    // Ray of Command (P1): P1 gains control of P0's Bears in response.
    let steal = |t: &mut TestGame, b: ObjectId| {
        let r = t.hand(P1, "Ray of Command");
        t.g.players[1].mana_pool.add_type(ManaType::U, 4);
        t.cast(P1, r).target(b).go();
        t.resolve();
        assert_eq!(t.obj_now(b).controller, P1);
    };
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let fb = p1_forked_bolt(&mut t, &[Entity::Player(P0), Entity::Object(b)]);
    steal(&mut t, b);
    assert!(!can_target(&mut t, "Not of This World", fb));
    // Cast first, then the Bears change control: Not of This World doesn't resolve.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let fb = p1_forked_bolt(&mut t, &[Entity::Player(P0), Entity::Object(b)]);
    let notw = t.hand(P0, "Not of This World");
    assert!(cost_to_cast(&mut t, P0, notw, &[Entity::Object(fb)]).is_some());
    steal(&mut t, b);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Not of This World"));
    // Forked Bolt resolved.
    assert!(t.life(P0) < 20 || t.obj_now(b).damage > 0);
}

// ---------------------------------------------------------------------------------------
// Cactarantula: "Whenever this creature becomes the target of a spell or ability an
// opponent controls, you may draw a card." Cindervines: "Whenever an opponent casts a
// noncreature spell, this enchantment deals 1 damage to that player."
// ---------------------------------------------------------------------------------------

/// P0 counters the spell `spell` with Counterspell.
fn counter(t: &mut TestGame, spell: ObjectId) {
    let c = t.hand(P0, "Counterspell");
    t.g.players[0].mana_pool.add_type(ManaType::U, 2);
    t.cast(P0, c).target(spell).go();
}

#[test]
fn cactarantulas_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Cactarantula",
        "Cactarantula’s last ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Cactarantula");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cactarantula");
    let shock = p1_shocks(&mut t, Entity::Object(c));
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), shock);
    counter(&mut t, shock);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.obj_now(c).damage, 0);
}

#[test]
fn cactarantula_triggers_once_for_a_spell_targeting_it_several_times() {
    cr!("603.2c", "115.3");
    ruling!(
        "Cactarantula",
        "If a spell or ability targets Cactarantula more than once, its last ability still triggers only once."
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cactarantula");
    p1_seeds_thrice(&mut t, c);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.pt(c), (9, 8));
}

#[test]
fn cindervines_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Cindervines",
        "Cindervines's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Cindervines");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cindervines");
    let shock = p1_shocks(&mut t, Entity::Player(P0));
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), shock);
    counter(&mut t, shock);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 19));
}

#[test]
fn cindervines_can_target_itself() {
    cr!("602.2b", "608.2b");
    ruling!(
        "Cindervines",
        "Cindervines can be the target of its own activated ability. The ability won't resolve since it won't have a legal target, and no player will be dealt damage"
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cindervines");
    t.g.players[0].mana_pool.add_type(ManaType::C, 1);
    t.activate(P0, c, 0, &[Entity::Object(c)]).unwrap();
    assert!(t.in_graveyard(P0, "Cindervines"));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

// ---------------------------------------------------------------------------------------
// Savage Stomp: "Put a +1/+1 counter on target creature you control. Then that creature
// fights target creature you don't control." Mage Duel: "Target creature you control
// gets +1/+2 until end of turn. Then it fights target creature you don't control."
// ---------------------------------------------------------------------------------------

#[test]
fn fight_spells_need_both_targets() {
    cr!("601.2c", "115.1");
    ruling!(
        "Savage Stomp",
        "You can't cast Savage Stomp unless you choose both a creature you control and a creature you don't control as targets."
    );
    ruling!(
        "Mage Duel",
        "You can’t cast Mage Duel unless you choose both a creature you control and a creature you don’t control as targets."
    );
    for name in ["Savage Stomp", "Mage Duel"] {
        supported(name);
        // Only a creature P0 controls.
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let s = t.hand(P0, name);
        assert_eq!(cost_to_cast(&mut t, P0, s, &[Entity::Object(mine)]), None, "{name}");
        // Only one P0 doesn't control.
        let mut t = TestGame::new(2);
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let s = t.hand(P0, name);
        assert_eq!(cost_to_cast(&mut t, P0, s, &[Entity::Object(theirs)]), None, "{name}");
        // Both.
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let s = t.hand(P0, name);
        let targets = [Entity::Object(mine), Entity::Object(theirs)];
        assert!(cost_to_cast(&mut t, P0, s, &targets).is_some(), "{name}");
    }
}

/// P0's Savage Stomp targets P0's Craw Wurm and P1's Hill Giant; `gone` leaves in
/// response. Returns (wurm, giant).
fn savage_stomp_with(gone: impl FnOnce(ObjectId, ObjectId) -> ObjectId) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let s = t.hand(P0, "Savage Stomp");
    cost_to_cast(&mut t, P0, s, &[Entity::Object(wurm), Entity::Object(giant)]).unwrap();
    let g = gone(wurm, giant);
    bounce(&mut t, g);
    t.resolve_all();
    (t, wurm, giant)
}

#[test]
fn savage_stomp_with_an_illegal_target() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Savage Stomp",
        "If either target is an illegal target as Savage Stomp resolves, neither creature will deal or be dealt damage."
    );
    ruling!(
        "Savage Stomp",
        "If the creature you control is an illegal target as Savage Stomp tries to resolve, you won't put a +1/+1 counter on it. If that creature is a legal target but the other creature isn't, you'll still put the counter on the creature you control."
    );
    // The Giant left: the Wurm gets its counter and isn't dealt damage.
    let (t, wurm, giant) = savage_stomp_with(|_, g| g);
    assert_eq!(t.counters(wurm, "+1/+1"), 1);
    assert_eq!(t.obj_now(wurm).damage, 0);
    assert_eq!(t.zone(giant), Zone::Hand(P1));
    // The Wurm left: no counter on anything, the Giant isn't dealt damage.
    let (t, wurm, giant) = savage_stomp_with(|w, _| w);
    assert_eq!(t.zone(wurm), Zone::Hand(P0));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.counters(giant, "+1/+1"), 0);
}

#[test]
fn tentative_connection_can_target_your_own_untapped_creature() {
    cr!("115.1");
    ruling!(
        "Tentative Connection",
        "Tentative Connection can target any creature, even one that you already control or that is already untapped."
    );
    supported("Tentative Connection");
    let mut t = TestGame::new(2);
    let mine = t.battlefield_sick(P0, "Grizzly Bears");
    let s = t.hand(P0, "Tentative Connection");
    assert!(cost_to_cast(&mut t, P0, s, &[Entity::Object(mine)]).is_some());
    t.resolve_all();
    assert_eq!(t.obj_now(mine).controller, P0);
    assert!(!t.obj_now(mine).tapped);
    assert!(t
        .obj_now(mine)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn suspicious_detonation_isnt_countered_by_ward() {
    cr!("702.21a", "101.2");
    ruling!(
        "Suspicious Detonation",
        "If you target a creature or planeswalker with ward, you may still pay the ward cost, but Suspicious Detonation won't be countered even if you don't."
    );
    supported("Suspicious Detonation");
    supported("Tomakul Honor Guard");
    let mut t = TestGame::new(2);
    // Tomakul Honor Guard: 3/1 with ward {2}.
    let guard = t.battlefield(P1, "Tomakul Honor Guard");
    let s = t.hand(P0, "Suspicious Detonation");
    t.answer_yes(P0, false);
    assert!(cost_to_cast(&mut t, P0, s, &[Entity::Object(guard)]).is_some());
    t.settle();
    // The ward trigger is on the stack above the spell.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Tomakul Honor Guard"));
    assert!(t.in_graveyard(P0, "Suspicious Detonation"));
}
