//! Rulings batch P202 — boast (CR 702.142) and bolster (CR 701.39).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::add_mana;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The uid of `source`'s activated ability whose text starts with `prefix`.
fn ability_uid(t: &TestGame, source: ObjectId, prefix: &str) -> u64 {
    t.g.obj(source)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with(prefix))
        .map(|a| a.uid)
        .unwrap_or_else(|| panic!("no activated ability starting with {prefix:?}"))
}

/// P0 activates `source`'s boast ability (with priority).
fn boast(t: &mut TestGame, source: ObjectId) {
    t.g.recompute();
    let uid = ability_uid(t, source, "Boast");
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, source, uid).unwrap();
    t.g.flush_events();
}

#[test]
fn arni_uses_the_actual_power_of_the_other_creature_once() {
    cr!("702.142a", "613.4b", "608.2h");
    ruling!(
        "Arni Brokenbrow",
        "Arni Brokenbrow's new base power is set to the 1 plus the actual power of the other creature you control with the highest power, not that creature's base power. If that creature's power changes after the boast ability has resolved, Arni's power is unaffected."
    );
    supported("Arni Brokenbrow");
    let mut t = TestGame::new(2);
    let arni = t.battlefield(P0, "Arni Brokenbrow");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 2);
    t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    attack_with(&mut t, &[(arni, Entity::Player(P1))]);
    add_mana(&mut t, P0, ManaType::R, 1);
    t.answer_yes(P0, true);
    boast(&mut t, arni);
    t.resolve_all();
    // 1 + 4 (the Bears' actual power, not its base power 2); toughness unchanged.
    assert_eq!(t.pt(arni), (5, 3));
    // The Bears' power changes later: Arni is unaffected.
    t.g.objects[bears.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 0);
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(arni), (5, 3));
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(arni), (3, 3));
}

#[test]
fn arnis_greatest_power_is_determined_and_chosen_as_the_ability_resolves() {
    cr!("702.142a", "608.2h", "608.2d");
    ruling!(
        "Arni Brokenbrow",
        "The greatest power among other creatures you control is determined as the boast ability resolves. You choose at that time whether you'd like to change Arni Brokenbrow's base power. If you control no other creatures at that time, you can change Arni's base power to 1 until end of turn."
    );
    // A creature arriving in response counts.
    let mut t = TestGame::new(2);
    let arni = t.battlefield(P0, "Arni Brokenbrow");
    attack_with(&mut t, &[(arni, Entity::Player(P1))]);
    add_mana(&mut t, P0, ManaType::R, 1);
    let from = t.asked().len();
    boast(&mut t, arni);
    // Nothing was chosen yet.
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(arni), (4, 3));

    // Declining leaves it unchanged.
    let mut t = TestGame::new(2);
    let arni = t.battlefield(P0, "Arni Brokenbrow");
    t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(arni, Entity::Player(P1))]);
    add_mana(&mut t, P0, ManaType::R, 1);
    boast(&mut t, arni);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.pt(arni), (3, 3));

    // With no other creatures, its base power can become 1.
    let mut t = TestGame::new(2);
    let arni = t.battlefield(P0, "Arni Brokenbrow");
    attack_with(&mut t, &[(arni, Entity::Player(P1))]);
    add_mana(&mut t, P0, ManaType::R, 1);
    boast(&mut t, arni);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(arni), (1, 3));
}

#[test]
fn eradicator_valkyries_opponents_choose_in_turn_order_then_sacrifice_together() {
    cr!("702.142a", "101.4", "701.21a");
    ruling!(
        "Eradicator Valkyrie",
        "As the boast ability resolves, first the next opponent in turn order chooses a creature or planeswalker they control. Then each other opponent in turn order chooses a creature or planeswalker they control, knowing the choices made before their choice. Then all chosen creatures and planeswalkers are sacrificed at the same time."
    );
    supported("Eradicator Valkyrie");
    let mut t = TestGame::new(3);
    let valk = t.battlefield(P0, "Eradicator Valkyrie");
    let fodder = t.battlefield(P0, "Llanowar Elves");
    let p1a = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let p2a = t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P2, "Hill Giant");
    attack_with(&mut t, &[(valk, Entity::Player(P1))]);
    add_mana(&mut t, P0, ManaType::B, 2);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    boast(&mut t, valk);
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    t.answer_choose(P1, &[Entity::Object(p1a)]);
    t.answer_choose(P2, &[Entity::Object(p2a)]);
    // When P2 chooses, P1's choice was made but nothing was sacrificed yet.
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g: &Game| {
            g.permanents()
                .filter(|o| o.chars.name == "Grizzly Bears")
                .count()
        },
    );
    let from = t.asked().len();
    t.resolve_all();
    let order: Vec<PlayerId> = asked_since(&t, from)
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P1, P2]);
    assert_eq!(seen.lock().unwrap().clone(), vec![2]);
    assert!(!t.on_battlefield(p1a) && !t.on_battlefield(p2a));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
}

#[test]
fn tuskeri_firewalkers_card_is_exiled_face_up_and_playable_only_this_turn() {
    cr!("702.142a", "406.3");
    ruling!(
        "Tuskeri Firewalker",
        "The card you exile with the boast ability is exiled face up."
    );
    ruling!(
        "Tuskeri Firewalker",
        "If you don't play the card by the end of the turn, it will remain exiled. Boasting again on a future turn won't let you play that card."
    );
    supported("Tuskeri Firewalker");
    let mut t = TestGame::new(2);
    let tusk = t.battlefield(P0, "Tuskeri Firewalker");
    t.lands(P0, "Forest", 3);
    t.library_top(P0, "Grizzly Bears");
    attack_with(&mut t, &[(tusk, Entity::Player(P1))]);
    boast(&mut t, tusk);
    t.resolve_all();
    let bears = t.g.exile.last().copied().unwrap();
    assert_eq!(t.obj(bears).chars.name.as_str(), "Grizzly Bears");
    assert!(!t.obj(bears).face_down);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    // Not played this turn. On a later turn, boasting again exiles another card; the
    // Bears stay exiled and can't be played.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.library_top(P0, "Llanowar Elves");
    attack_with(&mut t, &[(tusk, Entity::Player(P1))]);
    boast(&mut t, tusk);
    t.resolve_all();
    let elves = t.g.exile.last().copied().unwrap();
    assert_eq!(t.obj(elves).chars.name.as_str(), "Llanowar Elves");
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, elves, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
}

#[test]
fn dragonkin_berserkers_reduced_boast_cost_is_locked_in_once_paid() {
    cr!("702.142a", "602.2b", "601.2f", "601.2h");
    ruling!(
        "Dragonkin Berserker",
        "Once you announce that you're activating a boast ability, no player may take actions until the ability has been paid for. Notably, opponents can't try to change the activation cost by removing your Dragons."
    );
    supported("Dragonkin Berserker");
    let mut t = TestGame::new(2);
    let zerk = t.battlefield(P0, "Dragonkin Berserker");
    let d1 = t.battlefield(P0, "Shivan Dragon");
    t.battlefield(P0, "Shivan Dragon");
    t.lands(P0, "Mountain", 3);
    attack_with(&mut t, &[(zerk, Entity::Player(P1))]);
    let uid = ability_uid(&t, zerk, "Boast");
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Activate {
            source: zerk,
            ability: uid,
        }),
    );
    let from = t.asked().len();
    let ok = t.g.run_until(1_000, |g| {
        g.stack.len() == 1 && g.turn.priority == Some(P1)
    });
    assert!(ok);
    // P1 couldn't act before the ability was on the stack and paid for: {4}{R} less {2}.
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::Priority { .. })));
    assert_eq!(tapped_lands(&t, P0), 3);
    // Removing a Dragon now changes nothing; the ability resolves.
    destroy(&mut t, d1);
    t.resolve_all();
    assert_eq!(
        tokens(&t, P0)
            .iter()
            .filter(|id| t.obj(**id).chars.has_subtype("Dragon"))
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------------
// Bolster
// ---------------------------------------------------------------------------------

#[test]
fn abzan_advantage_bolsters_after_the_enchantment_is_gone() {
    cr!("701.39a", "608.2c");
    ruling!(
        "Abzan Advantage",
        "The enchantment will have already left the battlefield when you bolster 1."
    );
    supported("Abzan Advantage");
    let mut t = TestGame::new(2);
    // Eidolon of Blossoms (2/2), an enchantment creature, has the least toughness, but
    // it's sacrificed first: the Hill Giant gets the counter.
    let eidolon = t.battlefield(P0, "Eidolon of Blossoms");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 2);
    let c = t.hand(P0, "Abzan Advantage");
    t.cast(P0, c).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(eidolon));
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
}

#[test]
fn pinion_feast_needs_a_flying_target() {
    cr!("701.39a", "601.2c");
    ruling!(
        "Pinion Feast",
        "You must choose a creature with flying in order to cast Pinion Feast. You can’t cast it without a legal target just to bolster."
    );
    supported("Pinion Feast");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 5);
    let c = t.hand(P0, "Pinion Feast");
    assert!(!can_cast(&mut t, P0, c, CastMethod::Normal));
    let bird = t.battlefield(P1, "Wind Drake");
    assert!(can_cast(&mut t, P0, c, CastMethod::Normal));
    t.cast(P0, c).target(bird).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bird));
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn the_great_aerie_bolsters_without_targeting_choosing_on_resolution() {
    cr!("701.39a", "115.1");
    ruling!(
        "The Great Aerie",
        "Bolster itself doesn’t target any creature, though some spells and abilities that bolster may have other effects that target creatures."
    );
    ruling!(
        "The Great Aerie",
        "You determine which creature to put counter(s) on as the spell or ability that instructs you to bolster resolves. That could be the creature with the bolster ability, if it’s still under your control and has the least toughness. If there’s a tie, you choose which creature gets the counter(s)."
    );
    // (Its chaos ability doesn't compile; its bolster trigger does.)
    let mut t = crate::r_s19_common::planechase_game(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    crate::r_s19_common::start_planar_deck(&mut t, P0, &["The Great Aerie"]);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "bolster 3"), 1);
    // No target was chosen for it.
    assert!(target_candidates(&t, P0, from).is_empty());
    // A smaller creature arrives before it resolves; it gets the counters.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(t.counters(elves, counters::PLUS1), 3);
    assert_eq!(
        t.counters(a, counters::PLUS1) + t.counters(b, counters::PLUS1),
        0
    );
    // A tie between the two Bears: the player chooses.
    destroy(&mut t, elves);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.resolve_all();
    assert_eq!(t.counters(b, counters::PLUS1), 3);
    assert_eq!(t.counters(a, counters::PLUS1), 0);
}
