//! Rulings batch P076 — cards that make creatures unblockable: triggers that resolve
//! before the spell that caused them, effects that outlast their source, "up to one"
//! targets, protection from players and from creatures, and the "can't be blocked" rule
//! effect that isn't an ability.

use crate::r_p076_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::Modification;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Answers each target choice with the first candidate that isn't the ability's source.
fn target_other(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseTargets {
            source, candidates, ..
        } => candidates
            .iter()
            .find(|c| **c != Entity::Object(*source))
            .map(|c| Answer::Entities(vec![*c])),
        _ => None,
    }
}

#[test]
fn two_phase_dolphins_can_target_each_other() {
    cr!("115.1", "603.3d");
    ruling!(
        "Phase Dolphin",
        "If two Phase Dolphins attack at the same time, their abilities can target each other."
    );
    supported("Phase Dolphin");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Phase Dolphin");
    let b = t.battlefield(P0, "Phase Dolphin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s03_common::respond(&mut t, P0, target_other);
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(bears, a)]));
    assert!(!legal_blocks(&mut t, P1, &[(bears, b)]));
}

#[test]
fn phase_dolphins_effect_outlasts_it() {
    cr!("611.2a");
    ruling!(
        "Phase Dolphin",
        "The target creature can’t be blocked this turn even if Phase Dolphin leaves the battlefield."
    );
    supported("Phase Dolphin");
    let mut t = TestGame::new(2);
    let dolphin = t.battlefield(P0, "Phase Dolphin");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    attack_with(
        &mut t,
        &[(dolphin, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    t.resolve_all();
    crate::r_s02_common::destroy(&mut t, dolphin);
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
}

#[test]
fn bria_trigger_resolves_first_and_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Bria, Riptide Rogue",
        "Bria’s last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Bria, Riptide Rogue");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Bria, Riptide Rogue");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let spell = cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(giant)]);
    t.settle();
    // Bria's trigger and the prowess triggers are above the spell.
    assert_eq!(t.g.stack.first().copied(), Some(spell));
    assert!(t.stack_len() >= 2);
    t.g.counter(spell, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Giant Growth"));
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
}

#[test]
fn ardbert_trigger_resolves_first_and_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Ardbert, Warrior of Darkness",
        "Each of Ardbert's abilities resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Ardbert, Warrior of Darkness");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ardbert = t.battlefield(P0, "Ardbert, Warrior of Darkness");
    // A white spell: the trigger resolves while the spell waits.
    let spell = cast_new(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve();
    assert_eq!(t.counters(ardbert, "+1/+1"), 1);
    assert!(crate::r_s06_common::has_kw(
        &t,
        ardbert,
        mtg_engine::keywords::KeywordKind::Vigilance
    ));
    assert_eq!(t.g.stack.last().copied(), Some(spell));
    t.resolve_all();
    // A white spell countered before the trigger resolves.
    let spell = cast_new(&mut t, P0, "Raise the Alarm", &[]);
    t.settle();
    t.g.counter(spell, None);
    t.resolve_all();
    assert_eq!(t.counters(ardbert, "+1/+1"), 2);
}

#[test]
fn aragorn_with_no_target_or_an_illegal_target() {
    cr!("608.2b", "115.1");
    ruling!(
        "Aragorn, King of Gondor",
        "When Aragorn, King of Gondor's last ability triggers, you can choose not to target a creature, probably because you're the monarch and just want creatures to be unable to block this turn. However, if you do choose a target, and that target is illegal at the time the ability tries to resolve, the ability won't resolve and none of its effects will happen. Creatures will still be able to block this turn, even if you're the monarch."
    );
    supported("Aragorn, King of Gondor");
    use mtg_engine::designations::become_monarch;
    // No target: as the monarch, no creature can block.
    let mut t = TestGame::new(2);
    let aragorn = t.battlefield(P0, "Aragorn, King of Gondor");
    let giant = t.battlefield(P1, "Hill Giant");
    become_monarch(&mut t.g, P0);
    t.answer_targets(P0, &[]);
    attack_with(&mut t, &[(aragorn, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(giant, aragorn)]));
    // A target that becomes illegal: nothing happens.
    let mut t = TestGame::new(2);
    let aragorn = t.battlefield(P0, "Aragorn, King of Gondor");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    become_monarch(&mut t.g, P0);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(&mut t, &[(aragorn, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert!(legal_blocks(&mut t, P1, &[(giant, aragorn)]));
}

#[test]
fn distortion_strike_works_even_if_the_creature_loses_all_abilities() {
    cr!("613.1f", "509.1b");
    ruling!(
        "Distortion Strike",
        "Distortion Strike doesn’t grant an ability to the targeted creature. Rather, it affects the game rules and states something that’s now true about that creature. The creature can’t be blocked even if it loses all abilities."
    );
    supported("Distortion Strike");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Distortion Strike", &[Entity::Object(giant)]);
    t.resolve_all();
    crate::r_s26_common::modify_until_eot(&mut t, giant, vec![Modification::RemoveAllAbilities]);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
}

#[test]
fn midnight_pathlighter_triggers_twice_for_double_strike() {
    cr!("702.4b", "510.4");
    ruling!(
        "Midnight Pathlighter",
        "Unblocked creatures with double strike deal damage twice, which will cause Midnight Pathlighter's last ability to trigger twice."
    );
    supported("Midnight Pathlighter");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        (*mtg_engine::card::card("Lost Mine of Phandelver")).clone(),
        mtg_engine::object::Zone::Outside(P0),
    );
    t.battlefield(P0, "Midnight Pathlighter");
    // Fencing Ace: 1/1 double strike.
    let ace = t.battlefield(P0, "Fencing Ace");
    attack_with(&mut t, &[(ace, Entity::Player(P1))]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Two ventures: past the first room.
    let (_, room) = mtg_engine::dungeons::marker(&t.g, P0).expect("in a dungeon");
    assert!(room > 0, "ventured only once");
}

#[test]
fn silent_assassin_at_the_next_end_of_combat_step() {
    cr!("603.7", "511.1", "508.8");
    ruling!(
        "Silent Assassin",
        "If you activate the ability during the end of combat step, the creature will be destroyed at the beginning of the next end of combat step, even if that step occurs during a different turn."
    );
    supported("Silent Assassin");
    let mut t = TestGame::new(2);
    let assassin = t.battlefield(P1, "Silent Assassin");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[(wall, giant)]);
    assert_eq!(t.g.turn.step, Step::EndOfCombat);
    mana(&mut t, P1, ManaType::B, 1);
    mana(&mut t, P1, ManaType::C, 3);
    t.activate(P1, assassin, 0, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    assert!(t.on_battlefield(wall), "destroyed this turn");
    // P1's turn: no attackers, but the end of combat step still happens.
    t.advance_to(P1, Step::PostcombatMain);
    assert!(!t.on_battlefield(wall));
    assert!(t.in_graveyard(P1, "Wall of Stone"));
}

#[test]
fn key_to_the_city_without_a_target() {
    cr!("115.1", "601.2c");
    ruling!(
        "Key to the City",
        "You can activate Key to the City's first ability without targeting any creature."
    );
    supported("Key to the City");
    let mut t = TestGame::new(2);
    let key = t.battlefield(P0, "Key to the City");
    t.battlefield(P0, "Hill Giant");
    t.hand(P0, "Island");
    t.answer_targets(P0, &[]);
    t.activate(P0, key, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(key).tapped);
    assert!(t.in_graveyard(P0, "Island"));
}

#[test]
fn key_to_the_city_pays_only_once_per_untap() {
    cr!("603.3", "118.12");
    ruling!(
        "Key to the City",
        "You can pay {2} only once each time Key to the City's triggered ability resolves. You can't pay more to draw additional cards."
    );
    supported("Key to the City");
    let mut t = TestGame::new(2);
    let key = t.battlefield(P0, "Key to the City");
    t.g.tap(key);
    mana(&mut t, P0, ManaType::C, 6);
    let hand = hand_count(&t, P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.g.untap(key);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(hand_count(&t, P0), hand + 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 4);
}

/// Orders P0's triggers with Key to the City's untap trigger first (bottom of the stack).
fn key_bottom(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    order_key(d, true)
}

/// Orders P0's triggers with Key to the City's untap trigger last (top of the stack).
fn key_top(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    order_key(d, false)
}

fn order_key(d: &Decision, first: bool) -> Option<Answer> {
    match d {
        Decision::Order { items, .. } => {
            let i = items.iter().position(|x| x.contains("untapped"))?;
            let mut v: Vec<usize> = (0..items.len()).filter(|&j| j != i).collect();
            if first {
                v.insert(0, i);
            } else {
                v.push(i);
            }
            Some(Answer::Indices(v))
        }
        _ => None,
    }
}

#[test]
fn key_to_the_city_untap_trigger_waits_for_upkeep_triggers() {
    cr!("502.4", "503.1a", "603.3b");
    ruling!(
        "Key to the City",
        "Key to the City's last ability triggers during your untap step, but it's put onto the stack at the same time as abilities that trigger at the beginning of your upkeep step. Even though Key to the City's ability triggered first, you may order it before or after other abilities you control that are put onto the stack at this time."
    );
    supported("Key to the City");
    for key_first in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let key = t.battlefield(P0, "Key to the City");
        // Phyrexian Arena: "At the beginning of your upkeep, you draw a card and you
        // lose 1 life."
        t.battlefield(P0, "Phyrexian Arena");
        t.g.tap(key);
        let order_from = t.asked().len();
        crate::r_s03_common::respond(
            &mut t,
            P0,
            if key_first { key_bottom } else { key_top },
        );
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(t.g.turn.step, Step::Upkeep);
        assert_eq!(t.stack_len(), 2, "both triggers wait for the upkeep");
        assert!(t.asked()[order_from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
        let top = *t.g.stack.last().unwrap();
        let top_text = match &t.g.obj(top).stack.as_ref().unwrap().kind {
            mtg_engine::object::StackKind::Triggered { ability, .. } => ability.text.clone(),
            _ => String::new(),
        };
        // The first in the order is put on the stack first (it resolves last).
        assert_eq!(top_text.contains("untapped"), !key_first, "{top_text}");
    }
}

#[test]
fn detective_of_the_month_keeps_the_citys_blessing() {
    cr!("702.131b", "702.131c");
    ruling!(
        "Detective of the Month",
        "Once you have the city’s blessing, you have it for the rest of the game, even if you lose control of some or all your permanents. The city’s blessing isn’t a permanent itself and can’t be removed by any effect."
    );
    supported("Detective of the Month");
    let mut t = TestGame::new(2);
    let det = t.battlefield(P0, "Detective of the Month");
    let lands = t.lands(P0, "Island", 9);
    t.g.recompute();
    t.settle();
    assert!(t.g.player(P0).has_citys_blessing);
    for l in lands {
        crate::r_s02_common::destroy(&mut t, l);
    }
    assert!(t.g.player(P0).has_citys_blessing);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(det, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, det)]));
}

#[test]
fn jace_triggers_after_the_second_draw_of_a_multi_draw() {
    cr!("603.2", "603.3d", "121.2");
    ruling!(
        "Jace, Arcane Strategist",
        "If an effect instructs you to draw multiple cards, Jace’s first ability triggers after you draw whichever is the second one for the turn (if any). You choose a target for the ability after you’ve drawn all of the cards."
    );
    supported("Jace, Arcane Strategist");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Jace, Arcane Strategist");
    let giant = t.battlefield(P0, "Hill Giant");
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.player(P0).hand.len(),
    );
    t.answer_targets(P0, &[Entity::Object(giant)]);
    // Divination: draw two cards (the first and second this turn).
    cast_new(&mut t, P0, "Divination", &[]);
    let hand = hand_count(&t, P0);
    t.resolve_all();
    assert_eq!(hand_count(&t, P0), hand + 2);
    assert_eq!(*seen.lock().unwrap(), vec![hand + 2]);
    assert_eq!(t.counters(giant, "+1/+1"), 1);
}

#[test]
fn jace_triggers_once_per_turn_even_if_it_missed_the_first_draw() {
    cr!("603.2", "603.2c");
    ruling!(
        "Jace, Arcane Strategist",
        "Jace’s first ability can trigger only once each turn. It doesn’t matter whether Jace was on the battlefield when the first card was drawn."
    );
    supported("Jace, Arcane Strategist");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.draw_cards(P0, 1);
    t.battlefield(P0, "Jace, Arcane Strategist");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(giant, "+1/+1"), 1);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn jaces_ultimate_lasts_after_jace_leaves() {
    cr!("611.2a");
    ruling!(
        "Jace, Arcane Strategist",
        "Once Jace’s last ability has resolved, its effect applies even if Jace has left the battlefield."
    );
    supported("Jace, Arcane Strategist");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let jace = t.battlefield(P0, "Jace, Arcane Strategist");
    crate::r_s29_common::put_counters(&mut t, jace, "loyalty", 3);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, jace, 1, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(jace));
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
}

#[test]
fn senator_peacock_artifact_is_a_clue() {
    cr!("205.3g", "613.1d");
    ruling!(
        "Senator Peacock",
        "If Senator Peacock somehow becomes an artifact, it will also be a Clue."
    );
    supported("Senator Peacock");
    let mut t = TestGame::new(2);
    let peacock = t.battlefield(P0, "Senator Peacock");
    assert!(!t.obj_now(peacock).chars.has_subtype("Clue"));
    crate::r_s26_common::modify_until_eot(
        &mut t,
        peacock,
        vec![Modification::AddTypes(vec![CardType::Artifact])],
    );
    assert!(t.obj_now(peacock).chars.has_subtype("Clue"));
}

#[test]
fn reverse_the_polarity_switch_applies_last() {
    cr!("613.4d", "613.7");
    ruling!(
        "Reverse the Polarity",
        "Effects that switch a creature's power and toughness apply after all other effects, regardless of when those effects began to apply. For instance, if you switch a 2/4 creature's power and toughness and then give it +2/+0 later in the turn, it's a 4/4 creature, not a 6/2 creature."
    );
    supported("Reverse the Polarity");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Horned Turtle: 1/4.
    let turtle = t.battlefield(P0, "Horned Turtle");
    crate::r_s29_common::choose_modes(&mut t, P0, &[1]);
    cast_new(&mut t, P0, "Reverse the Polarity", &[]);
    t.resolve_all();
    assert_eq!(t.pt(turtle), (4, 1));
    use mtg_engine::ability::Value;
    crate::r_s26_common::modify_until_eot(
        &mut t,
        turtle,
        vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
    );
    assert_eq!(t.pt(turtle), (4, 3));
}

#[test]
fn inversion_behemoth_switch_applies_last() {
    cr!("613.4d", "613.7");
    ruling!(
        "Inversion Behemoth",
        "Effects that switch a creature's power and toughness apply after all other effects, regardless of when those effects began to apply. For instance, if you switch a 2/4 creature's power and toughness and then give it +2/+0 later in the turn, it's a 4/4 creature, not a 6/2 creature."
    );
    supported("Inversion Behemoth");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Inversion Behemoth");
    let turtle = t.battlefield(P0, "Horned Turtle");
    t.answer_targets(P0, &[Entity::Object(turtle)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(turtle), (4, 1));
    use mtg_engine::ability::Value;
    crate::r_s26_common::modify_until_eot(
        &mut t,
        turtle,
        vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
    );
    assert_eq!(t.pt(turtle), (4, 3));
}
