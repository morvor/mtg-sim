//! Rulings batch P018 — "whenever [this] deals damage" triggers: any damage, not only
//! combat damage (CR 120.1, 603.2); once per damage event however many things are dealt
//! damage (CR 510.2, 603.2c); damage dealt at the same time as lethal damage (CR 603.10a,
//! 704.3); a triggered ability isn't lifelink (CR 702.15b); redirection and prevention
//! (CR 614.9, 615); delayed triggers following an object (CR 603.7c, 400.7); untapping
//! in other players' untap steps (CR 502.3); and permanent cards with a subtype (CR 205.3,
//! 308.3).

use crate::r_p018_common::*;
use crate::r_p212_common::move_eq;
use crate::r_s01_common::{attack_with, stack_library, supported, tokens};
use crate::r_s02_common::target_candidates;
use crate::r_s03_common::choice_candidates;
use crate::r_s04_common::{add_mana, on_stack};
use crate::r_s06_common::{attach_new, damage};
use mtg_engine::ability::{Effect, Sel, Value};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `source` deals `n` damage to `to` (a player or permanent) as a resolving effect would
/// have it do (noncombat damage), then settles.
fn deal(t: &mut TestGame, source: ObjectId, n: i32, to: Entity) {
    let source = t.g.current(source);
    let mut ctx = mtg_engine::eval::Ctx::new(Some(source), t.obj(source).controller);
    ctx.targets = vec![vec![to]];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::This,
            amount: Value::Const(n),
            to: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    plenty_of_mana(t, p);
    let card = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, card).go();
    empty_pool(t, p);
    t.resolve_all();
}

/// Puts one card of each of `types` (an instant, a sorcery, a land, an enchantment) into
/// P0's graveyard.
fn graveyard_types(t: &mut TestGame, n: usize) {
    for name in ["Shock", "Stone Rain", "Forest", "Glorious Anthem"]
        .into_iter()
        .take(n)
    {
        t.graveyard(P0, name);
    }
}

// ---------------------------------------------------------------------------
// Any damage, not only combat damage.
// ---------------------------------------------------------------------------

#[test]
fn deals_damage_triggers_on_noncombat_damage_too() {
    cr!("120.1", "603.2", "120.3");
    ruling!(
        "Kjeldoran Gargoyle",
        "Kjeldoran Gargoyle’s triggered ability triggers when it deals any damage, not only combat damage."
    );
    ruling!(
        "Odylic Wraith",
        "Odylic Wraith’s ability triggers when it deals any kind of damage to a player, not just when it deals combat damage."
    );
    ruling!(
        "Warren Instigator",
        "Warren Instigator’s ability triggers when it deals any kind of damage to an opponent, not just combat damage."
    );
    ruling!(
        "Wei Night Raiders",
        "Wei Night Raiders’s second ability triggers whenever it deals damage to an opponent for any reason. It’s not limited to combat damage."
    );
    ruling!(
        "Witherscale Wurm",
        "Witherscale Wurm’s second ability triggers when it deals any damage to an opponent, not just combat damage."
    );
    ruling!(
        "Greatbow Doyen",
        "The ability will trigger whether the damage is combat damage or not."
    );
    ruling!(
        "Mephidross Vampire",
        "The triggered ability Mephidross Vampire grants to each creature you control triggers on any damage dealt to any creature, not just combat damage."
    );
    // (card, whether the damage is to a creature, the outcome after the trigger resolves)
    type Check = fn(&mut TestGame, ObjectId, ObjectId) -> bool;
    let cases: [(&str, bool, Check); 7] = [
        ("Kjeldoran Gargoyle", false, |t, _, _| t.life(P0) == 21),
        ("Odylic Wraith", false, |t, _, _| t.hand_size(P1) == 0),
        ("Warren Instigator", false, |t, _, _| {
            !t.named_on_battlefield("Raging Goblin").is_empty()
        }),
        ("Wei Night Raiders", false, |t, _, _| t.hand_size(P1) == 0),
        ("Witherscale Wurm", false, |t, src, _| {
            t.counters(src, counters::MINUS1) == 0
        }),
        ("Greatbow Doyen", true, |t, _, _| t.life(P1) == 19),
        ("Mephidross Vampire", true, |t, src, _| {
            t.counters(src, counters::PLUS1) == 1
        }),
    ];
    for (name, to_creature, check) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let source = t.battlefield(P0, name);
        let goblin = t.hand(P0, "Raging Goblin");
        t.hand(P1, "Mind Rot");
        if name == "Witherscale Wurm" {
            put_counters(&mut t, source, counters::MINUS1, 2);
        }
        let victim = t.battlefield(P1, "Craw Wurm");
        let to = if to_creature {
            Entity::Object(victim)
        } else {
            Entity::Player(P1)
        };
        if name == "Warren Instigator" {
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[Entity::Object(goblin)]);
        }
        deal(&mut t, source, 1, to);
        assert_eq!(t.stack_len(), 1, "{name}: the ability triggered");
        t.resolve_all();
        assert!(check(&mut t, source, victim), "{name}");
    }
}

#[test]
fn greatbow_doyen_ignores_damage_from_an_archer_spell() {
    cr!("109.2", "603.2");
    ruling!(
        "Greatbow Doyen",
        "The second ability triggers only when an Archer permanent you control deals damage to a creature. It won’t trigger when an Archer spell you control deals damage."
    );
    supported("Greatbow Doyen");
    let volley = crate::r_s01_common::custom_card(
        "Archer's Volley",
        "Kindred Instant — Elf Archer",
        "{R}",
        None,
        "Archer's Volley deals 2 damage to target creature.",
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Greatbow Doyen");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.custom(P0, volley, Zone::Hand(P0));
    add_mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, spell).target(giant).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// Ohabi Caleria: "Untap all Archers you control during each other player's untap step."
// ---------------------------------------------------------------------------

#[test]
fn ohabi_caleria_untaps_archers_in_each_other_players_untap_step() {
    cr!("502.3");
    ruling!(
        "Ohabi Caleria",
        "All your Archers untap during each other player’s untap step. You have no choice about what untaps."
    );
    ruling!(
        "Ohabi Caleria",
        "Effects that say an Archer you control “doesn’t untap during your untap step” or “doesn’t untap during its controller’s untap step” won’t prevent Ohabi Caleria from causing you to untap it during other players’ untap steps."
    );
    supported("Ohabi Caleria");
    supported("Claustrophobia");
    let mut t = TestGame::new(3);
    let ohabi = t.battlefield(P0, "Ohabi Caleria");
    let archer = t.battlefield(P0, "Elvish Archers");
    let held = t.battlefield(P0, "Elvish Archers");
    let elf = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P1, "Claustrophobia", held);
    for c in [ohabi, archer, held, elf] {
        t.g.tap(c);
    }
    let from = t.asked().len();
    t.advance_to(P1, Step::Upkeep);
    // Every Archer untaps (Claustrophobia's held one too), without a choice; the
    // non-Archer doesn't.
    for c in [ohabi, archer, held] {
        assert!(!t.obj_now(c).tapped);
    }
    assert!(t.obj_now(elf).tapped);
    assert!(crate::r_s01_common::asked_since(&t, from)
        .iter()
        .all(|(p, d)| *p != P0 || matches!(d, mtg_engine::decision::Decision::Priority { .. })));
    // Each other player's: P2's too.
    t.g.tap(archer);
    t.advance_to(P2, Step::Upkeep);
    assert!(!t.obj_now(archer).tapped);
}

// ---------------------------------------------------------------------------
// Once per damage event.
// ---------------------------------------------------------------------------

#[test]
fn damage_to_several_things_at_once_triggers_once() {
    cr!("510.2", "603.2c");
    ruling!(
        "Briarbridge Patrol",
        "Briarbridge Patrol's first ability will only trigger once per combat damage step, regardless of how many creatures it deals damage to."
    );
    ruling!(
        "Angel of Deliverance",
        "If Angel of Deliverance deals damage to multiple creatures, players, or planeswalkers at once (for example, if it's blocked by two smaller creatures) its last ability triggers only once."
    );
    supported("Briarbridge Patrol");
    supported("Angel of Deliverance");
    let mut t = TestGame::new(2);
    let patrol = t.battlefield(P0, "Briarbridge Patrol");
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    t.attack(&[(patrol, Entity::Player(P1))], &[(a, patrol), (b, patrol)]);
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(tokens(&t, P0).len(), 1, "one Clue");
    // Angel of Deliverance with delirium, blocked by two reach creatures: one trigger.
    let mut t = TestGame::new(2);
    graveyard_types(&mut t, 4);
    let angel = t.battlefield(P0, "Angel of Deliverance");
    let a = t.battlefield(P1, "Giant Spider");
    let b = t.battlefield(P1, "Giant Spider");
    t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(a, angel), (b, angel)]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(on_stack(&t, "exile target creature"), 1);
}

// ---------------------------------------------------------------------------
// Angel of Deliverance: "Delirium — Whenever this creature deals damage, if there are
// four or more card types among cards in your graveyard, exile target creature an
// opponent controls."
// ---------------------------------------------------------------------------

#[test]
fn angel_of_deliverance_triggers_when_it_dies_to_the_same_damage() {
    cr!("603.2", "701.14a", "704.5g");
    ruling!(
        "Angel of Deliverance",
        "If Angel of Deliverance deals damage and is dealt lethal damage at the same time, its ability will trigger."
    );
    supported("Prey Upon");
    // The Angel fights a Craw Wurm: each deals 6 damage to the other at the same time.
    let mut t = TestGame::new(2);
    graveyard_types(&mut t, 4);
    let angel = t.battlefield(P0, "Angel of Deliverance");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    plenty_of_mana(&mut t, P0);
    let prey = t.hand(P0, "Prey Upon");
    t.answer_targets(P0, &[Entity::Object(angel)]);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.cast(P0, prey).go();
    empty_pool(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve();
    assert!(!t.on_battlefield(angel) && !t.on_battlefield(wurm));
    assert_eq!(on_stack(&t, "exile target creature"), 1);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Exile);
}

#[test]
fn angel_of_deliverance_cant_target_the_creature_it_killed() {
    cr!("704.5g", "603.3d", "115.1");
    ruling!(
        "Angel of Deliverance",
        "If Angel of Deliverance deals lethal damage to a creature, that creature can't be targeted by Angel of Deliverance's triggered ability. It's already in its owner's graveyard."
    );
    supported("Giant Spider");
    let mut t = TestGame::new(2);
    graveyard_types(&mut t, 4);
    let angel = t.battlefield(P0, "Angel of Deliverance");
    let spider = t.battlefield(P1, "Giant Spider");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(spider, angel)]),
    );
    let from = t.asked().len();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert!(t.in_graveyard(P1, "Giant Spider"));
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered, vec![vec![Entity::Object(giant)]]);
}

#[test]
fn angel_of_deliverance_doesnt_count_a_creature_dying_at_the_same_time() {
    cr!("603.4", "603.2", "704.3");
    ruling!(
        "Angel of Deliverance",
        "If you have three noncreature card types among cards in your graveyard and Angel of Deliverance deals damage at the same time another creature you control is dealt lethal damage, the ability won't trigger. The creature doesn't die until after Angel of Deliverance's ability checks to see if it triggers."
    );
    let mut t = TestGame::new(2);
    graveyard_types(&mut t, 3);
    let angel = t.battlefield(P0, "Angel of Deliverance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(
        &mut t,
        &[(angel, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(giant, bears)]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 14);
    assert_eq!(on_stack(&t, "exile target creature"), 0);
    // With the creature card now there, the next damage triggers it.
    t.advance_to(P0, Step::EndOfCombat);
    let angel_now = t.g.current(angel);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    deal(&mut t, angel_now, 1, Entity::Player(P1));
    assert_eq!(on_stack(&t, "exile target creature"), 1);
}

// ---------------------------------------------------------------------------
// Lowland Basilisk: "Whenever this creature deals damage to a creature, destroy that
// creature at end of combat."
// ---------------------------------------------------------------------------

#[test]
fn lowland_basilisk_doesnt_destroy_a_creature_that_left_and_came_back() {
    cr!("400.7", "603.7c");
    ruling!(
        "Lowland Basilisk",
        "If a creature leaves the battlefield before the end of combat, then the Lowland Basilisk effect on it will end."
    );
    supported("Lowland Basilisk");
    supported("Cloudshift");
    let mut t = TestGame::new(2);
    let basilisk = t.battlefield(P0, "Lowland Basilisk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(basilisk, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(bears, basilisk)]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
    // P1 blinks the Bears: a new object.
    cast_resolve(&mut t, P1, "Cloudshift", &[Entity::Object(bears)]);
    let new = t.g.current(bears);
    assert_ne!(new, bears);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.on_battlefield(new));
}

#[test]
fn lowland_basilisk_outside_combat_destroys_at_end_of_the_next_combat() {
    cr!("603.7a", "511.2");
    ruling!(
        "Lowland Basilisk",
        "If it damages a creature outside of combat, its effect will destroy the creature at the end of the next combat."
    );
    supported("Prey Upon");
    let mut t = TestGame::new(2);
    let basilisk = t.battlefield(P0, "Lowland Basilisk");
    let giant = t.battlefield(P1, "Grizzly Bears");
    // After P0's combat this turn: the next combat is P1's.
    t.set_step(P0, Step::PostcombatMain);
    cast_resolve(
        &mut t,
        P0,
        "Prey Upon",
        &[Entity::Object(basilisk), Entity::Object(giant)],
    );
    assert!(t.on_battlefield(basilisk) && t.on_battlefield(giant));
    // Through the cleanup step (damage wears off) into P1's turn.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.on_battlefield(giant));
    // P1's combat (nobody attacks) ends: the Giant is destroyed.
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

// ---------------------------------------------------------------------------
// Shackles of Treachery: "Gain control of target creature until end of turn. Untap that
// creature. Until end of turn, it gains haste and "Whenever this creature deals damage,
// destroy target Equipment attached to it.""
// ---------------------------------------------------------------------------

#[test]
fn shackles_of_treachery_targets_any_creature() {
    cr!("115.1", "601.2c");
    ruling!(
        "Shackles of Treachery",
        "Shackles of Treachery can target any creature, including one that’s already untapped or one that doesn’t have any Equipment attached to it."
    );
    supported("Shackles of Treachery");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(
        &mut t,
        P0,
        "Shackles of Treachery",
        &[Entity::Object(bears)],
    );
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn shackles_of_treachery_destroys_equipment_whoever_controls_it() {
    cr!("603.3d", "608.2b", "120.1");
    ruling!(
        "Shackles of Treachery",
        "If that ability triggers, you must choose a legal target if one is available. It doesn’t matter who controls the Equipment attached to the creature. If that Equipment is no longer attached to the creature as the ability tries to resolve, the ability won’t resolve and the Equipment won’t be destroyed."
    );
    ruling!(
        "Shackles of Treachery",
        "The triggered ability gained by the creature will trigger when that creature deals any damage, not just combat damage."
    );
    supported("Bonesplitter");
    // P1's Bears wear P1's Bonesplitter; P0 shackles them; the Bears deal noncombat
    // damage: the Bonesplitter is the only legal target, and it's destroyed.
    for unattach in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let other = t.battlefield(P1, "Hill Giant");
        let splitter = t.battlefield(P1, "Bonesplitter");
        assert!(t.g.attach(splitter, Entity::Object(bears)));
        cast_resolve(
            &mut t,
            P0,
            "Shackles of Treachery",
            &[Entity::Object(bears)],
        );
        let from = t.asked().len();
        deal(&mut t, bears, 2, Entity::Player(P1));
        assert_eq!(t.stack_len(), 1);
        let offered = target_candidates(&t, P0, from);
        assert_eq!(offered, vec![vec![Entity::Object(splitter)]]);
        if unattach {
            move_eq(&mut t, splitter, other);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(splitter), unattach);
    }
}

// ---------------------------------------------------------------------------
// Discard triggers.
// ---------------------------------------------------------------------------

#[test]
fn skullknocker_ogre_discards_the_only_card_and_draws() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Skullknocker Ogre",
        "If that player has one card in hand, it's discarded at random (even though that's not very random). The player draws a card."
    );
    supported("Skullknocker Ogre");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Skullknocker Ogre");
    let card = t.hand(P1, "Hill Giant");
    t.attack(&[(ogre, Entity::Player(P1))], &[]);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.zone(card), Zone::Graveyard(P1));
    assert_eq!(t.hand_size(P1), 1);
}

// ---------------------------------------------------------------------------
// Aura and spell-granted "you gain that much life".
// ---------------------------------------------------------------------------

#[test]
fn soul_link_triggers_twice_when_the_creature_damages_itself() {
    cr!("603.2", "120.3");
    ruling!(
        "Soul Link",
        "If the creature damages itself, both abilities trigger."
    );
    supported("Soul Link");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Soul Link", giant);
    damage(&mut t, giant, 2, giant);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn venomous_fangs_needs_two_regeneration_shields_against_lethal_damage() {
    cr!("701.19a", "704.5g", "603.2");
    ruling!(
        "Venomous Fangs",
        "If the damaged creature takes both lethal damage and is destroyed by this effect, you have to regenerate the creature twice to keep it alive."
    );
    supported("Venomous Fangs");
    supported("Regenerate");
    for shields in [1, 2] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        attach_new(&mut t, P0, "Venomous Fangs", giant);
        let bears = t.battlefield(P1, "Grizzly Bears");
        for _ in 0..shields {
            cast_resolve(&mut t, P1, "Regenerate", &[Entity::Object(bears)]);
        }
        damage(&mut t, giant, 3, bears);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), shields == 2, "{shields} shields");
    }
}

#[test]
fn kjeldoran_gargoyles_lifegain_is_too_late_against_simultaneous_lethal_damage() {
    cr!("702.15b", "704.5a", "603.3");
    ruling!(
        "Kjeldoran Gargoyle",
        "Unlike lifelink, Kjeldoran Gargoyle’s last ability is a triggered ability that goes on the stack and may be responded to. Notably, if Kjeldoran Gargoyle deals damage at the same time you’re dealt enough damage to reduce your life total to 0 or less, you’ll lose the game before you can gain any life."
    );
    supported("White Knight");
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 2;
    let gargoyle = t.battlefield(P0, "Kjeldoran Gargoyle");
    let knight = t.battlefield(P1, "White Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(knight, Entity::Player(P0)), (bears, Entity::Player(P0))],
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(gargoyle, bears)]),
    );
    t.g.run_until(10_000, |g| g.players[0].has_lost || g.is_over());
    assert!(t.has_lost(P0));
}

#[test]
fn stir_the_prides_lifegain_isnt_lifelink() {
    cr!("702.15b", "603.2", "113.2c");
    ruling!(
        "Stir the Pride",
        "Notably, the ability granted by the second mode isn’t lifelink. It’s a triggered ability that triggers whenever the creature with that ability deals damage. If that creature also has lifelink, you’ll gain life as a result of that damage and also when the triggered ability resolves. Multiple instances of this ability trigger separately."
    );
    supported("Stir the Pride");
    supported("Child of Night");
    let mut t = TestGame::new(2);
    let child = t.battlefield(P0, "Child of Night");
    for _ in 0..2 {
        plenty_of_mana(&mut t, P0);
        let stir = t.hand(P0, "Stir the Pride");
        t.cast(P0, stir).modes(&[1]).go();
        empty_pool(&mut t, P0);
        t.resolve_all();
    }
    deal(&mut t, child, 2, Entity::Player(P1));
    // Lifelink: 2 at once; two triggers on the stack.
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn stir_the_pride_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Stir the Pride",
        "Only creatures you control when Stir the Pride resolves will get the bonuses."
    );
    let mut t = TestGame::new(2);
    let before = t.battlefield(P0, "Grizzly Bears");
    plenty_of_mana(&mut t, P0);
    let stir = t.hand(P0, "Stir the Pride");
    t.cast(P0, stir).kicked(true).modes(&[0, 1]).go();
    empty_pool(&mut t, P0);
    t.resolve_all();
    let after = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(before), (4, 4));
    assert_eq!(t.pt(after), (2, 2));
    deal(&mut t, after, 2, Entity::Player(P1));
    assert_eq!(t.stack_len(), 0);
    deal(&mut t, before, 4, Entity::Player(P1));
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn armadillo_cloak_isnt_lifelink_and_stacks() {
    cr!("702.15b", "603.2", "510.2");
    ruling!(
        "Armadillo Cloak",
        "While Armadillo Cloak’s last ability is similar to lifelink, it isn’t lifelink—it’s a normal triggered ability. If a creature you control with lifelink wears an Armadillo Cloak and deals damage, first you’ll gain that much life for lifelink, and then Armadillo Cloak’s triggered ability is put onto the stack. This also means that multiple instances of Armadillo Cloak’s triggered ability do trigger separately."
    );
    supported("Armadillo Cloak");
    let mut t = TestGame::new(2);
    let child = t.battlefield(P0, "Child of Night");
    attach_new(&mut t, P0, "Armadillo Cloak", child);
    attach_new(&mut t, P0, "Armadillo Cloak", child);
    assert_eq!(t.pt(child), (6, 5));
    attack_with(&mut t, &[(child, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 26, "lifelink first");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 38);
}

#[test]
fn vampiric_link_gains_all_the_damage_dealt() {
    cr!("120.3", "510.1c");
    ruling!(
        "Vampiric Link",
        "You gain life for all damage which is not prevented, regardless of what the creature damages (player or another creature) or the toughness of the blocking creature. If you put this on a 5/5 and it is blocked by a 1/1, you still gain 5 life."
    );
    supported("Vampiric Link");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    attach_new(&mut t, P0, "Vampiric Link", wurm);
    let elf = t.battlefield(P1, "Llanowar Elves");
    t.attack(&[(wurm, Entity::Player(P1))], &[(elf, wurm)]);
    assert!(!t.on_battlefield(elf));
    assert_eq!(t.life(P0), 26);
}

#[test]
fn witherscale_wurm_doesnt_give_itself_wither() {
    cr!("702.80a", "120.3e");
    ruling!(
        "Witherscale Wurm",
        "Witherscale Wurm’s first ability does *not* give Witherscale Wurm wither."
    );
    supported("Witherscale Wurm");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Witherscale Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(giant, wurm)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert!(t
        .obj_now(giant)
        .has_keyword(mtg_engine::keywords::KeywordKind::Wither));
    assert!(!t
        .obj_now(wurm)
        .has_keyword(mtg_engine::keywords::KeywordKind::Wither));
    t.advance_to(P0, Step::EndOfCombat);
    // The Giant's 3 damage (wither) are -1/-1 counters on the Wurm.
    assert_eq!(t.counters(wurm, counters::MINUS1), 3);
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

// ---------------------------------------------------------------------------
// Prevention and redirection.
// ---------------------------------------------------------------------------

#[test]
fn avenging_druid_does_nothing_if_all_its_damage_is_prevented() {
    cr!("615.1", "603.2", "305.4");
    ruling!(
        "Avenging Druid",
        "The ability does not work if all damage is prevented."
    );
    ruling!(
        "Avenging Druid",
        "Putting a land onto the battlefield does not count as playing a land."
    );
    supported("Avenging Druid");
    supported("Holy Day");
    for prevent in [true, false] {
        let mut t = TestGame::new(2);
        let druid = t.battlefield(P0, "Avenging Druid");
        let cards = stack_library(&mut t, P0, &["Shock", "Forest"]);
        attack_with(&mut t, &[(druid, Entity::Player(P1))]);
        if prevent {
            cast_resolve(&mut t, P1, "Holy Day", &[]);
        }
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), if prevent { 20 } else { 19 });
        assert_eq!(t.on_battlefield(cards[1]), !prevent);
        assert_eq!(t.zone(cards[0]) == Zone::Graveyard(P0), !prevent);
        assert_eq!(t.g.players[P0.idx()].lands_played_this_turn, 0);
    }
}

#[test]
fn hypnotic_specter_and_redirected_or_prevented_damage() {
    cr!("614.9", "615.1", "603.2");
    ruling!(
        "Hypnotic Specter",
        "The ability triggers even if the Specter’s damage is being redirected to an opponent. It does not trigger if damage that would have been dealt to the opponent is redirected to a nonopponent player or a creature, or if the damage is prevented."
    );
    supported("Hypnotic Specter");
    supported("Jade Monolith");
    supported("Pariah");
    // (setup, whether it triggers)
    type Setup = fn(&mut TestGame, ObjectId, ObjectId) -> Entity;
    let cases: [(&str, Setup, bool); 4] = [
        // P1's Jade Monolith redirects the Specter's damage to P1's Bears to P1.
        (
            "to an opponent",
            |t, specter, bears| {
                let mono = t.battlefield(P1, "Jade Monolith");
                add_mana(t, P1, ManaType::C, 1);
                t.answer_choose(P1, &[Entity::Object(specter)]);
                t.activate(P1, mono, 0, &[Entity::Object(bears)]).unwrap();
                t.resolve();
                Entity::Object(bears)
            },
            true,
        ),
        // P0's own Jade Monolith redirects it to P0.
        (
            "to a nonopponent",
            |t, specter, bears| {
                let mono = t.battlefield(P0, "Jade Monolith");
                add_mana(t, P0, ManaType::C, 1);
                t.answer_choose(P0, &[Entity::Object(specter)]);
                t.activate(P0, mono, 0, &[Entity::Object(bears)]).unwrap();
                t.resolve();
                Entity::Object(bears)
            },
            false,
        ),
        // Pariah on P1's Bears: damage to P1 is dealt to the Bears.
        (
            "to a creature",
            |t, _, bears| {
                attach_new(t, P1, "Pariah", bears);
                Entity::Player(P1)
            },
            false,
        ),
        (
            "prevented",
            |t, _, _| {
                cast_resolve(t, P1, "Healing Salve", &[Entity::Player(P1)]);
                Entity::Player(P1)
            },
            false,
        ),
    ];
    for (what, setup, triggers) in cases {
        let mut t = TestGame::new(2);
        let specter = t.battlefield(P0, "Hypnotic Specter");
        let bears = t.battlefield(P1, "Craw Wurm");
        t.hand(P1, "Hill Giant");
        if what == "prevented" {
            t.answer(
                P1,
                DecisionKind::Modes,
                mtg_engine::decision::Answer::Indices(vec![1]),
            );
        }
        let to = setup(&mut t, specter, bears);
        deal(&mut t, specter, 2, to);
        assert_eq!(t.stack_len(), usize::from(triggers), "{what}");
        t.resolve_all();
        assert_eq!(t.hand_size(P1), usize::from(!triggers), "{what}");
    }
}

// ---------------------------------------------------------------------------
// Tapping and holding creatures.
// ---------------------------------------------------------------------------

#[test]
fn thalakos_dreamsower_can_target_a_tapped_creature() {
    cr!("115.1", "701.26a");
    ruling!(
        "Thalakos Dreamsower",
        "Can be used on a creature that is already tapped."
    );
    supported("Thalakos Dreamsower");
    let mut t = TestGame::new(2);
    let sower = t.battlefield(P0, "Thalakos Dreamsower");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    // Tapped (as it would be after attacking) when its ability resolves.
    t.g.tap(sower);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    deal(&mut t, sower, 1, Entity::Player(P1));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(
        t.obj_now(giant).tapped,
        "held while the Dreamsower is tapped"
    );
}

#[test]
fn somnophore_can_hold_several_creatures_and_the_same_one_twice() {
    cr!("603.3d", "611.2b");
    ruling!(
        "Somnophore",
        "It can end up holding more than one creature tapped."
    );
    ruling!(
        "Somnophore",
        "You can choose the same creature more than once (for no additional effect)."
    );
    supported("Somnophore");
    let mut t = TestGame::new(2);
    let somnophore = t.battlefield(P0, "Somnophore");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    for target in [a, b, a] {
        t.answer_targets(P0, &[Entity::Object(target)]);
        deal(&mut t, somnophore, 1, Entity::Player(P1));
        t.resolve_all();
    }
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    // Somnophore leaves: both untap normally.
    crate::r_s02_common::destroy(&mut t, somnophore);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(a).tapped && !t.obj_now(b).tapped);
}

// ---------------------------------------------------------------------------
// Searching and putting permanent cards.
// ---------------------------------------------------------------------------

#[test]
fn hunting_cheetah_finds_any_card_with_the_forest_land_type() {
    cr!("205.3i", "701.23a");
    ruling!(
        "Hunting Cheetah",
        "Hunting Cheetah's ability allows you to search your library for any card with the land type Forest, not just a card with the name Forest."
    );
    supported("Hunting Cheetah");
    let mut t = TestGame::new(2);
    let cheetah = t.battlefield(P0, "Hunting Cheetah");
    let pool = t.library_top(P0, "Breeding Pool");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(pool)]);
    let from = t.asked().len();
    deal(&mut t, cheetah, 2, Entity::Player(P1));
    t.resolve_all();
    let offered = choice_candidates(&t, from, "");
    assert!(offered.iter().any(|c| c.contains(&Entity::Object(pool))));
    assert_eq!(t.zone(pool), Zone::Hand(P0));
}

#[test]
fn goblin_lackey_can_put_a_kindred_goblin_card() {
    cr!("110.4", "205.3m");
    ruling!(
        "Goblin Lackey",
        "A Goblin permanent card is an artifact, creature, land, and/or enchantment card with the creature type \"Goblin\". Usually such cards will be creatures in addition to any other types, but they may be kindred instead. For example, you could put Boggart Shenanigans onto the battlefield with this effect."
    );
    supported("Goblin Lackey");
    let mut t = TestGame::new(2);
    let lackey = t.battlefield(P0, "Goblin Lackey");
    let shenanigans = t.hand(P0, "Boggart Shenanigans");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(shenanigans)]);
    deal(&mut t, lackey, 1, Entity::Player(P1));
    t.resolve_all();
    assert!(t.on_battlefield(shenanigans));
}
