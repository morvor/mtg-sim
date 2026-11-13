//! Rulings batch P079 — spells and abilities aimed at attacking or blocking creatures:
//! illegal targets on resolution (CR 608.2b), divided damage with some targets illegal
//! (CR 601.2d, 608.2b), what an ability that "doesn't resolve" doesn't do, the source of
//! damage (CR 120.3, 609.7), and triggers from a card in the graveyard (CR 603.6, 603.10).

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::damage_on;
use crate::r_s08_common::is_tapped;
use crate::r_s21_common::go_to;
use mtg_engine::combat::remove_from_combat;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn catti_brie_with_no_counters_still_targets() {
    cr!("602.2b", "115.1b", "603.2");
    ruling!(
        "Catti-brie of Mithral Hall",
        "If Catti-brie of Mithral Hall has no counters, you can still activate the last ability. This won't deal any damage, but it will count as targeting something"
    );
    supported("Frost Titan");
    // P0's Catti-brie (no counters) attacks and is blocked by P1's Frost Titan ("whenever
    // this creature becomes the target of a spell or ability an opponent controls, counter
    // that spell or ability unless its controller pays {2}").
    let mut t = TestGame::new(2);
    let cb = t.battlefield(P0, "Catti-brie of Mithral Hall");
    let titan = t.battlefield(P1, "Frost Titan");
    t.lands(P0, "Wastes", 1);
    to_blockers(&mut t, &[(cb, Entity::Player(P1))], &[(titan, cb)]);
    assert_eq!(t.counters(cb, "+1/+1"), 0);
    t.answer_targets(P0, &[obj(titan)]);
    activate_containing(&mut t, P0, cb, "Remove all").expect("can activate with no counters");
    t.settle();
    assert_eq!(
        t.stack_len(),
        2,
        "Frost Titan's ability triggered on being targeted"
    );
    t.resolve_all();
    assert_eq!(damage_on(&t, titan), 0);
}

#[test]
fn durkwood_tracker_needs_both_itself_and_its_target() {
    cr!("608.2b", "608.2h");
    ruling!(
        "Durkwood Tracker",
        "If Durkwood Tracker has left the battlefield before the ability resolves, the ability doesn’t do anything. If the targeted creature has left the battlefield before the ability resolves, the ability doesn’t resolve."
    );
    supported("Durkwood Tracker");
    for tracker_leaves in [true, false] {
        let mut t = TestGame::new(2);
        let tracker = t.battlefield(P1, "Durkwood Tracker");
        t.lands(P1, "Forest", 2);
        let giant = t.battlefield(P0, "Hill Giant");
        let other = t.battlefield(P0, "Grizzly Bears");
        attack_with(
            &mut t,
            &[(giant, Entity::Player(P1)), (other, Entity::Player(P1))],
        );
        t.answer_targets(P1, &[obj(giant)]);
        activate_containing(&mut t, P1, tracker, "damage").unwrap();
        if tracker_leaves {
            crate::r_s02_common::destroy(&mut t, tracker);
        } else {
            crate::r_s02_common::destroy(&mut t, giant);
        }
        t.resolve_all();
        if tracker_leaves {
            assert_eq!(damage_on(&t, giant), 0, "no damage to the target");
        } else {
            assert_eq!(damage_on(&t, tracker), 0, "no damage to the Tracker");
            assert!(t.on_battlefield(tracker));
        }
    }
    // Both there: 4 to the Giant, 3 back.
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P1, "Durkwood Tracker");
    t.lands(P1, "Forest", 2);
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    t.answer_targets(P1, &[obj(giant)]);
    activate_containing(&mut t, P1, tracker, "damage").unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(tracker), "4/3 dealt 3");
}

/// P1's Arbalest Elite targets P0's attacking Hill Giant (P0's turn). Returns the game,
/// the Elite and the Giant, with the ability on the stack.
fn arbalest() -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let elite = t.battlefield(P1, "Arbalest Elite");
    t.lands(P1, "Plains", 1);
    t.lands(P1, "Wastes", 2);
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    t.answer_targets(P1, &[obj(giant)]);
    activate_containing(&mut t, P1, elite, "damage").unwrap();
    (t, elite, giant)
}

#[test]
fn arbalest_elite_untaps_if_its_ability_doesnt_resolve() {
    cr!("608.2b", "502.3");
    ruling!(
        "Arbalest Elite",
        "If the creature is an illegal target when Arbalest Elite's ability tries to resolve, the ability won't resolve and none of its effects will happen. Arbalest Elite will untap during its controller's next untap step."
    );
    supported("Arbalest Elite");
    for illegal in [true, false] {
        let (mut t, elite, giant) = arbalest();
        if illegal {
            remove_from_combat(&mut t.g, giant);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(giant), illegal);
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(is_tapped(&t, elite), !illegal, "illegal target: {illegal}");
    }
}

#[test]
fn arbalest_elite_doesnt_untap_only_applies_to_that_untap_step() {
    cr!("502.3", "611.2a");
    ruling!(
        "Arbalest Elite",
        "If another player controls Arbalest Elite during your next untap step, the part of the effect that keeps it from untapping does nothing. It won't try to apply during a future untap step."
    );
    // P0 gains control of the Elite before P1's next untap step and gives it back during
    // P1's turn: it untaps in P1's following untap step.
    let (mut t, elite, _) = arbalest();
    t.resolve_all();
    crate::r_s06_common::give_control(&mut t, elite, P0);
    t.advance_to(P1, Step::Upkeep);
    assert!(
        is_tapped(&t, elite),
        "P0's permanent doesn't untap in P1's untap step"
    );
    crate::r_s06_common::give_control(&mut t, elite, P1);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_tapped(&t, elite), "untapped in P1's next untap step");
}

#[test]
fn spirit_flare_does_nothing_if_either_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Spirit Flare",
        "If either target is illegal when the spell resolves, it will do no damage."
    );
    supported("Spirit Flare");
    // P1 taps its Hill Giant to deal 3 to P0's attacking Siege Mastodon. Before Spirit
    // Flare resolves, the Giant becomes tapped or the Mastodon leaves combat.
    for case in 0..3 {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let m = t.battlefield(P0, "Siege Mastodon");
        attack_with(&mut t, &[(m, Entity::Player(P1))]);
        cast_card(&mut t, P1, "Spirit Flare", &[obj(giant), obj(m)]).unwrap();
        match case {
            0 => {
                t.g.tap(giant);
            }
            1 => remove_from_combat(&mut t.g, m),
            _ => {}
        }
        t.resolve_all();
        assert_eq!(
            damage_on(&t, m),
            if case == 2 { 3 } else { 0 },
            "case {case}"
        );
    }
}

/// P1 casts the real card `name` ("deals N damage divided as you choose among ... target
/// attacking or blocking creatures") at P0's attacking 4/5 Siege Mastodons with the given
/// division; before it resolves, the attackers at `leave` are removed from combat.
/// Returns the damage on each and whether the spell resolved.
fn divided(name: &str, division: &[u32], leave: &[usize]) -> (Vec<u32>, bool) {
    let mut t = TestGame::new(2);
    let ms: Vec<ObjectId> = division
        .iter()
        .map(|_| t.battlefield(P0, "Siege Mastodon"))
        .collect();
    attack_with(
        &mut t,
        &ms.iter()
            .map(|m| (*m, Entity::Player(P1)))
            .collect::<Vec<_>>(),
    );
    crate::r_s01_common::give_mana_for(&mut t, P1, name);
    let card = t.hand(P1, name);
    t.answer(
        P1,
        DecisionKind::Divide,
        Answer::Numbers(division.iter().map(|n| *n as i64).collect()),
    );
    let targets: Vec<Entity> = ms.iter().map(|m| obj(*m)).collect();
    let spell = t.cast(P1, card).targets(&targets).go();
    for i in leave {
        remove_from_combat(&mut t.g, ms[*i]);
    }
    t.resolve_all();
    let resolved = crate::r_s07_common::resolved(&t, spell);
    (ms.iter().map(|m| damage_on(&t, *m)).collect(), resolved)
}

#[test]
fn divided_damage_keeps_the_original_division_when_some_targets_are_illegal() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Deft Dismissal",
        "If some (but not all) of the targets become illegal, the original division of damage still applies, but no damage is dealt to illegal targets. If all targets become illegal, Deft Dismissal doesn’t resolve."
    );
    ruling!(
        "Roil's Retribution",
        "If some, but not all, of the targets become illegal by the time Roil’s Retribution tries to resolve, the original division of damage still applies, but no damage is dealt to illegal targets."
    );
    supported("Deft Dismissal");
    supported("Roil's Retribution");
    assert_eq!(divided("Deft Dismissal", &[2, 1], &[0]), (vec![0, 1], true));
    assert_eq!(
        divided("Deft Dismissal", &[1, 1, 1], &[]),
        (vec![1, 1, 1], true)
    );
    assert_eq!(
        divided("Deft Dismissal", &[2, 1], &[0, 1]),
        (vec![0, 0], false)
    );
    assert_eq!(
        divided("Roil's Retribution", &[3, 2], &[0]),
        (vec![0, 2], true)
    );
    assert_eq!(
        divided("Roil's Retribution", &[4, 1], &[0, 1]),
        (vec![0, 0], false)
    );
}

#[test]
fn boros_fury_shield_is_the_source_of_the_damage_to_the_controller() {
    cr!("120.3", "609.7", "702.15b");
    ruling!(
        "Boros Fury-Shield",
        "If red mana was spent, Boros Fury-Shield, not the target creature, deals the damage to the creature’s controller."
    );
    supported("Boros Fury-Shield");
    // P0's Vampire Nighthawk (2/3 lifelink) attacks; P1 casts Fury-Shield with {R} spent:
    // P0 loses 2 and gains nothing (the creature isn't the source).
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    attack_with(&mut t, &[(hawk, Entity::Player(P1))]);
    t.lands(P1, "Plains", 1);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", 1);
    let shield = t.hand(P1, "Boros Fury-Shield");
    t.cast(P1, shield).target(hawk).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "its combat damage was prevented");
    assert_eq!(t.life(P0), 18);
}

/// P1 has Vengeful Pharaoh in the graveyard and P0 attacks with the given attacks (all
/// unblocked); the Pharaoh triggers target P0's creatures in order.
fn pharaoh(t: &mut TestGame, attacks: &[(ObjectId, Entity)]) -> ObjectId {
    let ph = t.graveyard(P1, "Vengeful Pharaoh");
    to_blockers(t, attacks, &[]);
    for (a, _) in attacks {
        t.answer_targets(P1, &[obj(*a)]);
    }
    ph
}

#[test]
fn vengeful_pharaoh_triggers_once_per_damage_event_recipient() {
    cr!("603.2c", "510.2", "603.6", "400.7");
    ruling!(
        "Vengeful Pharaoh",
        "If multiple creatures deal combat damage to you simultaneously, Vengeful Pharaoh will only trigger once."
    );
    ruling!(
        "Vengeful Pharaoh",
        "If multiple creatures deal combat damage to you and to a planeswalker you control simultaneously, Vengeful Pharaoh will trigger twice."
    );
    supported("Vengeful Pharaoh");
    // Two attackers at P1: one trigger.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    pharaoh(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    go_to(&mut t, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(
        [t.on_battlefield(a), t.on_battlefield(b)]
            .iter()
            .filter(|x| !**x)
            .count(),
        1
    );
    // One at P1 and one at P1's planeswalker: two triggers. The first puts the Pharaoh on
    // top of P1's library; it's then put back into the graveyard, but the second trigger
    // does nothing: that's a new object.
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P1, "Jace Beleren");
    crate::r_s13_common::add(&mut t, pw, "loyalty", 10);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let ph = pharaoh(&mut t, &[(a, Entity::Player(P1)), (b, obj(pw))]);
    go_to(&mut t, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.zone(ph), Zone::Library(P1));
    let first = [a, b]
        .into_iter()
        .find(|x| !t.on_battlefield(*x))
        .expect("one destroyed");
    let second = if first == a { b } else { a };
    t.g.mill(P1, 1);
    t.settle();
    assert!(t.in_graveyard(P1, "Vengeful Pharaoh"));
    t.resolve_all();
    assert!(t.on_battlefield(second), "the second trigger did nothing");
    assert!(t.in_graveyard(P1, "Vengeful Pharaoh"));
}

#[test]
fn vengeful_pharaoh_does_nothing_once_it_left_the_graveyard_or_its_target_is_illegal() {
    cr!("603.4", "608.2b");
    ruling!(
        "Vengeful Pharaoh",
        "If Vengeful Pharaoh is no longer in your graveyard when the triggered ability would resolve, the triggered ability won't do anything."
    );
    ruling!(
        "Vengeful Pharaoh",
        "If the attacking creature is an illegal target when the triggered ability tries to resolve, it won't resolve and none of its effects will happen. Vengeful Pharaoh will remain in your graveyard."
    );
    // Exiled with the trigger on the stack: the attacker survives.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let ph = pharaoh(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_s05_common::move_to(&mut t, ph, Zone::Exile);
    t.resolve_all();
    assert!(t.on_battlefield(a));
    // The attacker left combat: the Pharaoh stays in the graveyard.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    pharaoh(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::CombatDamage);
    t.settle();
    remove_from_combat(&mut t.g, a);
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(t.in_graveyard(P1, "Vengeful Pharaoh"));
}

#[test]
fn shimian_night_stalker_needs_an_attacking_creature_and_redirects_any_damage() {
    cr!("614.9", "115.1b");
    ruling!(
        "Shimian Night Stalker",
        "Can redirect damage from combat and from abilities, but the creature must be attacking before you can use this card’s ability."
    );
    supported("Shimian Night Stalker");
    supported("Valakut Invoker");
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P1, "Shimian Night Stalker");
    t.lands(P1, "Swamp", 1);
    let invoker = t.battlefield(P0, "Valakut Invoker");
    t.lands(P0, "Wastes", 8);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(
        !crate::r_s02_common::can_activate(&mut t, P1, stalker),
        "no attacking creature to target yet"
    );
    attack_with(&mut t, &[(invoker, Entity::Player(P1))]);
    t.answer_targets(P1, &[obj(invoker)]);
    activate_containing(&mut t, P1, stalker, "damage").unwrap();
    t.resolve_all();
    // Its ability's damage to P1 is redirected, and so is its combat damage.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, invoker, "damage").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage_on(&t, stalker), 3);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(stalker), "3 + 2 damage to a 4/4");
}

#[test]
fn aetherspouts_owner_orders_cards_put_on_top_together() {
    cr!("401.4", "101.4");
    ruling!(
        "Aetherspouts",
        "If an effect puts two or more cards on the top or bottom of a library at the same time, the owner of those cards may arrange them in any order."
    );
    supported("Aetherspouts");
    // P0's Bears and Giant both go on top; P0 orders them (the card put there last ends
    // up on top), both ways.
    for order in [[0usize, 1], [1, 0]] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        let from = t.asked().len();
        cast_card(&mut t, P1, "Aetherspouts", &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.zone(a), Zone::Library(P0));
        assert_eq!(t.zone(b), Zone::Library(P0));
        let asked: Vec<_> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
            .cloned()
            .collect();
        assert!(
            asked
                .iter()
                .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })),
            "the owner orders them: {asked:?}"
        );
        // The card put there last is on top.
        let lib = &t.g.player(P0).library;
        let name = |id: &ObjectId| t.obj_now(*id).chars.name.to_string();
        let top_two = [name(&lib[lib.len() - 1]), name(&lib[lib.len() - 2])];
        let expected = if order == [0, 1] {
            ["Hill Giant", "Grizzly Bears"]
        } else {
            ["Grizzly Bears", "Hill Giant"]
        };
        assert_eq!(top_two, expected, "{order:?}");
    }
}

#[test]
fn angel_of_the_dire_hour_put_onto_the_battlefield_wasnt_cast() {
    cr!("603.4", "601.1a");
    ruling!(
        "Angel of the Dire Hour",
        "If a spell or ability instructs you to put Angel of the Dire Hour onto the battlefield from your hand, that means you won’t have cast it"
    );
    supported("Angel of the Dire Hour");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    let angel = t.hand(P1, "Angel of the Dire Hour");
    let new = crate::r_s17_common::put_onto_battlefield(&mut t, P1, angel, false);
    assert!(new.is_some());
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(crate::r_s10_common::attacking(&t, a));
}
