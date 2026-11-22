//! Rulings batch P097 — impulse draw that lasts (until the end of your next turn, or
//! during a turn you attacked): the cards are exiled face up (CR 406.3), are played with
//! the normal timing rules, land plays and costs (CR 305.1, 305.2, 307.1, 601.2f), the
//! permission doesn't depend on the source staying around (CR 611.2a), unplayed cards stay
//! exiled, and nothing is exiled if the spell's only target is gone (CR 608.2b). Also
//! Demonic Rising's intervening "if" (CR 603.4).

use crate::r_p097_common::*;
use crate::r_s06_common::activate_containing;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Sets up `case` exiling the cards `top` (top first) from P0's library, ending in P0's
/// main phase with the stack empty and no land played this turn. Returns the game and
/// the exiled cards.
fn impulse(case: &str, top: &[&str]) -> (TestGame, Vec<ObjectId>) {
    supported(case);
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, top);
    match case {
        "Wrenn's Resolve" | "Reckless Impulse" => {
            pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 1)]);
            let s = t.hand(P0, case);
            t.cast(P0, s).go();
        }
        "Season of the Bold" => {
            // {P}{P} — Exile the top two cards of your library. Until the end of your
            // next turn, you may play them.
            pool(&mut t, P0, &[(ManaType::R, 2), (ManaType::C, 3)]);
            let s = t.hand(P0, case);
            t.cast(P0, s).modes(&[1]).go();
        }
        "Ignite the Future" => {
            pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 3)]);
            let s = t.hand(P0, case);
            t.cast(P0, s).go();
        }
        "Commune with Lava" => {
            let x = top.len() as u32;
            pool(&mut t, P0, &[(ManaType::R, 2), (ManaType::C, x)]);
            let s = t.hand(P0, case);
            t.cast(P0, s).x(x as i64).go();
        }
        "Escape to the Wilds" => {
            pool(
                &mut t,
                P0,
                &[(ManaType::R, 1), (ManaType::G, 1), (ManaType::C, 3)],
            );
            let s = t.hand(P0, case);
            t.cast(P0, s).go();
        }
        "Showdown of the Skalds" => {
            pool(
                &mut t,
                P0,
                &[(ManaType::R, 1), (ManaType::W, 1), (ManaType::C, 2)],
            );
            let s = t.hand(P0, case);
            t.cast(P0, s).go();
        }
        "Outlaws' Fury" => {
            // Rowdy Crew is a Pirate, an outlaw.
            t.battlefield(P0, "Rowdy Crew");
            pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 2)]);
            let s = t.hand(P0, case);
            t.cast(P0, s).go();
        }
        "Painter's Studio // Defaced Gallery" => {
            pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 2)]);
            let s = t.hand(P0, case);
            t.cast(P0, s).method(CastMethod::Half(0)).go();
        }
        "Captive Weird // Compleated Conjurer" => {
            let weird = t.battlefield(P0, case);
            pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 3)]);
            activate_containing(&mut t, P0, weird, "Transform").expect("transform");
        }
        "Fire Giant's Fury" => {
            // Hill Giant becomes 5/5: five cards are exiled.
            let giant = t.battlefield(P0, "Hill Giant");
            pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 1)]);
            let s = t.hand(P0, case);
            t.cast(P0, s).target(giant).go();
            t.resolve_all();
            t.attack(&[(giant, Entity::Player(P1))], &[]);
            t.resolve_all();
            t.advance_to(P0, Step::PostcombatMain);
            assert_eq!(t.life(P1), 15);
        }
        "Goblin Researcher" => {
            let r = t.enter(P0, case);
            t.resolve_all();
            t.g.objects[r.0 as usize].summoning_sick = false;
            t.attack(&[(r, Entity::Player(P1))], &[]);
            t.resolve_all();
            t.advance_to(P0, Step::PostcombatMain);
        }
        _ => panic!("no setup for {case}"),
    }
    t.resolve_all();
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile, "{case}: {}", t.dump_log());
    }
    (t, cards)
}

#[test]
fn unplayed_cards_stay_in_exile() {
    cr!("611.2a", "406.3");
    ruling!(
        "Wrenn's Resolve",
        "Any cards you don't play will remain in exile."
    );
    ruling!(
        "Showdown of the Skalds",
        "Any cards you don’t play by the end of your next turn will remain exiled."
    );
    ruling!(
        "Fire Giant's Fury",
        "Any exiled cards that you don't play by the end of your next turn remain exiled."
    );
    ruling!(
        "Ignite the Future",
        "If you don't play some or all of the exiled cards, those cards remain in exile."
    );
    for case in [
        "Wrenn's Resolve",
        "Showdown of the Skalds",
        "Fire Giant's Fury",
        "Ignite the Future",
    ] {
        let (mut t, cards) = impulse(case, &["Forest", "Island"]);
        stay_exiled_after_next_turn(&mut t, &cards);
    }
}

#[test]
fn the_cards_are_exiled_face_up() {
    cr!("406.3");
    ruling!(
        "Goblin Researcher",
        "The card Goblin Researcher exiles is exiled face up."
    );
    ruling!(
        "Fire Giant's Fury",
        "The cards exiled by the triggered ability created by Fire Giant's Fury are exiled face up."
    );
    ruling!(
        "Showdown of the Skalds",
        "The cards you exile because of the chapter I ability are exiled face up."
    );
    for case in [
        "Goblin Researcher",
        "Fire Giant's Fury",
        "Showdown of the Skalds",
    ] {
        let (t, cards) = impulse(case, &["Grizzly Bears"]);
        assert!(!t.obj_now(cards[0]).face_down, "{case}");
    }
}

#[test]
fn an_exiled_land_needs_a_land_play_and_main_phase_timing() {
    cr!("305.1", "305.2", "116.2a");
    ruling!(
        "Commune with Lava",
        "Under normal circumstances, you can play a land card exiled with Commune with Lava only if you haven't played a land yet that turn."
    );
    ruling!(
        "Showdown of the Skalds",
        "Under normal circumstances, you can play a land from among the exiled cards only if you haven’t played a land yet that turn."
    );
    ruling!(
        "Captive Weird // Compleated Conjurer",
        "Unless an effect allows you to play additional lands that turn, you can play a land card exiled with Compleated Conjurer only if you haven’t played a land yet that turn."
    );
    ruling!(
        "Fire Giant's Fury",
        "Unless an effect allows you to play additional lands that turn, you can play land cards exiled this way only if you haven't played a land yet that turn."
    );
    ruling!(
        "Outlaws' Fury",
        "You pay all costs and follow all normal timing rules for a card played from exile this way. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    ruling!(
        "Painter's Studio // Defaced Gallery",
        "You pay all costs and follow all timing rules for cards played with the permission granted by Painter's Studio. For example, if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty."
    );
    ruling!(
        "Goblin Researcher",
        "If you exile a land card with Goblin Researcher's enters-the-battlefield ability, you may play that land only if you have any available land plays. Normally, this means you can play the land only if you haven't played a land yet that turn."
    );
    ruling!(
        "Season of the Bold",
        "You pay all costs and follow all timing rules for cards played with the permission granted by the second mode. For example, if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty."
    );
    for case in [
        "Season of the Bold",
        "Commune with Lava",
        "Showdown of the Skalds",
        "Captive Weird // Compleated Conjurer",
        "Fire Giant's Fury",
        "Outlaws' Fury",
        "Painter's Studio // Defaced Gallery",
        "Goblin Researcher",
        "Ignite the Future",
    ] {
        let (mut t, cards) = impulse(case, &["Forest"]);
        check_land_timing(&mut t, cards[0]);
    }
}

#[test]
fn an_exiled_spell_follows_its_timing_rules_and_costs() {
    cr!("307.1", "307.5", "601.2f", "601.2h");
    ruling!(
        "Escape to the Wilds",
        "Escape to the Wilds doesn't change when you can play the exiled cards. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    ruling!(
        "Goblin Researcher",
        "Goblin Researcher's effect doesn't change when you can play the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    ruling!(
        "Showdown of the Skalds",
        "Playing a card this way follows the normal rules for playing the card. You must pay its costs, and you must follow all applicable timing rules. For example, if one of the cards is a sorcery card, you can cast it only during your main phase while the stack is empty."
    );
    ruling!(
        "Fire Giant's Fury",
        "Playing the exiled cards follows the normal rules for playing those cards. You must pay their costs, if any, and you must follow all applicable timing rules. For example, if one of the cards is a sorcery card, you can cast that card by paying its mana cost only during your main phase while the stack is empty."
    );
    ruling!(
        "Wrenn's Resolve",
        "The cards you play from exile follow the usual timing restrictions, and you must pay any costs for spells you cast."
    );
    ruling!(
        "Reckless Impulse",
        "You must still pay all costs to cast those spells and follow normal casting permissions. For example, if one of the exiled cards is a sorcery, this effect doesn't allow you to cast it during an opponent's turn."
    );
    ruling!(
        "Ignite the Future",
        "Ignite the Future doesn't change when you can play the exiled cards. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    for case in [
        "Escape to the Wilds",
        "Goblin Researcher",
        "Showdown of the Skalds",
        "Fire Giant's Fury",
        "Wrenn's Resolve",
        "Reckless Impulse",
        "Ignite the Future",
    ] {
        let (mut t, cards) = impulse(case, &["Divination"]);
        check_sorcery_timing(&mut t, cards[0], &[(ManaType::U, 1), (ManaType::C, 2)]);
    }
    // Escape to the Wilds: with its additional land play, two lands (one exiled).
    let (mut t, cards) = impulse("Escape to the Wilds", &["Forest", "Mountain"]);
    check_land_timing_with_extra_play(&mut t, cards[0], cards[1]);
}

/// With one additional land play this turn: `a` and `b`, exiled lands, may both be played
/// (in a main phase with the stack empty), then no more lands.
fn check_land_timing_with_extra_play(t: &mut TestGame, a: ObjectId, b: ObjectId) {
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!playable(t, a));
    t.g.combat = None;
    t.set_step(P0, Step::PrecombatMain);
    let a = t.g.current(a);
    t.play_land(P0, a).expect("first land");
    let b = t.g.current(b);
    assert!(playable(t, b));
    t.play_land(P0, b).expect("second land");
    let forest = t.hand(P0, "Forest");
    assert!(t.play_land(P0, forest).is_err());
}

#[test]
fn a_creature_card_exiled_by_compleated_conjurer_is_cast_at_sorcery_speed() {
    cr!("307.1", "302.1", "601.2f");
    ruling!(
        "Captive Weird // Compleated Conjurer",
        "Playing the card exiled with Compleated Conjurer follows the normal rules for playing that card. You must pay its costs, and you must follow all applicable timing rules. For example, if the card is a creature card, you can cast that card by paying its mana cost only during your main phase while the stack is empty."
    );
    let (mut t, cards) = impulse("Captive Weird // Compleated Conjurer", &["Grizzly Bears"]);
    check_sorcery_timing(&mut t, cards[0], &[(ManaType::G, 1), (ManaType::C, 1)]);
}

#[test]
fn escape_to_the_wilds_additional_land_doesnt_have_to_be_exiled() {
    cr!("305.2");
    ruling!(
        "Escape to the Wilds",
        "The additional land that you play doesn't have to be from among the exiled cards."
    );
    let (mut t, cards) = impulse("Escape to the Wilds", &["Island"]);
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Forest");
    t.play_land(P0, a).expect("first land");
    t.play_land(P0, b).expect("second land from hand too");
    assert!(!playable(&mut t, cards[0]));
}

#[test]
fn the_card_may_be_played_after_the_source_leaves_or_changes_control() {
    cr!("611.2a", "611.2b");
    ruling!(
        "Captive Weird // Compleated Conjurer",
        "You may play the card exiled with Compleated Conjurer even if Compleated Conjurer leaves the battlefield or you lose control of it."
    );
    ruling!(
        "Fire Giant's Fury",
        "You may play those cards that turn even if the Giant leaves the battlefield or another player gains control of it."
    );
    for (case, source, leaves) in [
        ("Captive Weird // Compleated Conjurer", "Compleated Conjurer", true),
        ("Captive Weird // Compleated Conjurer", "Compleated Conjurer", false),
        ("Fire Giant's Fury", "Hill Giant", true),
        ("Fire Giant's Fury", "Hill Giant", false),
    ] {
        let (mut t, cards) = impulse(case, &["Forest"]);
        let src = t.named_on_battlefield(source)[0];
        if leaves {
            t.g.destroy(src, None);
            t.settle();
        } else {
            crate::r_s06_common::give_control(&mut t, src, P1);
        }
        assert!(!t.on_battlefield(src) || t.obj_now(src).controller == P1);
        assert!(playable(&mut t, cards[0]), "{case} leaves={leaves}");
        let f = t.g.current(cards[0]);
        t.play_land(P0, f).expect("play the exiled land");
    }
}

#[test]
fn nothing_is_exiled_if_the_only_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Blazing Crescendo",
        "If the target creature is an illegal target by the time the spell tries to resolve (most likely because it has left the battlefield in response), the spell will not resolve. No card will be exiled."
    );
    ruling!(
        "Suspend Aggression",
        "If the target permanent is an illegal target as Suspend Aggression tries to resolve, it won't resolve and none of its effects will happen. No cards will be exiled."
    );
    ruling!(
        "Campus Renovation",
        "You don't have to choose a target artifact or enchantment card to cast Campus Renovation. However, if you do, and that card is an illegal target at the time Campus Renovation tries to resolve, it won't resolve and none of its effects will happen. You won't exile any cards from your library."
    );
    for (case, mana) in [
        ("Blazing Crescendo", vec![(ManaType::R, 1), (ManaType::C, 1)]),
        (
            "Suspend Aggression",
            vec![(ManaType::R, 1), (ManaType::W, 1), (ManaType::C, 1)],
        ),
        (
            "Campus Renovation",
            vec![(ManaType::R, 1), (ManaType::W, 1), (ManaType::C, 3)],
        ),
    ] {
        supported(case);
        for remove in [false, true] {
            let mut t = TestGame::new(2);
            let top = t.library_top(P0, "Forest");
            let target = if case == "Campus Renovation" {
                t.graveyard(P0, "Sol Ring")
            } else {
                t.battlefield(P0, "Grizzly Bears")
            };
            pool(&mut t, P0, &mana);
            let s = t.hand(P0, case);
            t.cast(P0, s).target(target).go();
            if remove {
                let to = if case == "Campus Renovation" {
                    Zone::Exile
                } else {
                    Zone::Hand(P0)
                };
                t.g.move_object(target, to, events::MoveCause::Effect, None);
            }
            t.resolve_all();
            assert_eq!(
                t.zone(top) == Zone::Exile,
                !remove,
                "{case} remove={remove}"
            );
            assert_eq!(t.in_graveyard(P0, case), true, "{case}: still goes away");
        }
    }
    // Campus Renovation without a target exiles the cards.
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Forest");
    pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::W, 1), (ManaType::C, 3)]);
    let s = t.hand(P0, "Campus Renovation");
    t.cast(P0, s).targets(&[]).go();
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Exile);
}

#[test]
fn ignite_the_future_free_only_when_cast_from_a_graveyard() {
    cr!("118.9", "118.9a", "702.34a");
    ruling!(
        "Ignite the Future",
        "If you cast Ignite the Future from a zone other than a graveyard, you must pay the mana cost or an alternative cost for each spell cast this way. If you cast Ignite the Future from a graveyard, you may choose to cast each spell for its mana cost, an alternative cost, or without paying its mana cost."
    );
    for from_graveyard in [false, true] {
        let mut t = TestGame::new(2);
        let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Grizzly Bears", "Island"]);
        let ignite = if from_graveyard {
            pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 7)]);
            let s = t.graveyard(P0, "Ignite the Future");
            t.cast(P0, s)
                .method(CastMethod::Keyword(KeywordKind::Flashback))
                .go();
            s
        } else {
            pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 3)]);
            let s = t.hand(P0, "Ignite the Future");
            t.cast(P0, s).go();
            s
        };
        let _ = ignite;
        t.resolve_all();
        let free = t.cast(P0, t.g.current(cards[0])).method(CastMethod::Free).try_go();
        assert_eq!(free.is_ok(), from_graveyard);
        t.resolve_all();
        // The mana cost may always be paid.
        pool(&mut t, P0, &[(ManaType::G, 2)]);
        let c = t.g.current(cards[1]);
        t.cast(P0, c).go();
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            if from_graveyard { 2 } else { 1 }
        );
    }
}

#[test]
fn ignite_the_future_free_spells_still_pay_mandatory_additional_costs() {
    cr!("118.9", "118.8", "601.2b");
    ruling!(
        "Ignite the Future",
        "If you cast a card “without paying its mana cost,” you can't choose to cast it for any alternative costs and not pay that cost. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Momentous Fall, you must pay those to cast the card."
    );
    supported("Momentous Fall");
    for with_creature in [false, true] {
        let mut t = TestGame::new(2);
        let fall = t.library_top(P0, "Momentous Fall");
        pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 7)]);
        let s = t.graveyard(P0, "Ignite the Future");
        t.cast(P0, s)
            .method(CastMethod::Keyword(KeywordKind::Flashback))
            .go();
        t.resolve_all();
        let bears = with_creature.then(|| t.battlefield(P0, "Hill Giant"));
        let hand = t.hand_size(P0);
        if let Some(b) = bears {
            t.answer_choose(P0, &[Entity::Object(b)]);
        }
        let c = t.g.current(fall);
        let r = t.cast(P0, c).method(CastMethod::Free).try_go();
        assert_eq!(r.is_ok(), with_creature, "Momentous Fall needs its sacrifice");
        if let Some(b) = bears {
            t.resolve_all();
            assert!(!t.on_battlefield(b));
            // "draw cards equal to the sacrificed creature's power, then gain life equal
            // to its toughness": Hill Giant, 3/3.
            assert_eq!(t.hand_size(P0), hand + 3);
            assert_eq!(t.life(P0), 23);
        }
    }
}

#[test]
fn archaics_agony_with_deathtouch_deals_excess_beyond_one_damage() {
    cr!("120.10", "702.2c", "702.2b");
    ruling!(
        "Archaic's Agony",
        "In some unusual cases, Archaic's Agony may have deathtouch. Even 1 damage dealt to a creature from a source with deathtouch is considered lethal damage"
    );
    supported("Pestilent Spirit");
    for deathtouch in [false, true] {
        let mut t = TestGame::new(2);
        if deathtouch {
            // "Instant and sorcery spells you control have deathtouch."
            t.battlefield(P0, "Pestilent Spirit");
        }
        let cards = stack_library(&mut t, P0, &["Forest", "Island", "Swamp"]);
        let wurm = t.battlefield(P1, "Craw Wurm");
        // Three colors spent: 3 damage to a 6/4.
        pool(
            &mut t,
            P0,
            &[
                (ManaType::R, 1),
                (ManaType::G, 1),
                (ManaType::U, 1),
                (ManaType::C, 2),
            ],
        );
        let s = t.hand(P0, "Archaic's Agony");
        t.cast(P0, s).target(wurm).go();
        t.resolve_all();
        let exiled = cards.iter().filter(|c| t.zone(**c) == Zone::Exile).count();
        assert_eq!(exiled, if deathtouch { 2 } else { 0 });
        assert_eq!(t.on_battlefield(wurm), !deathtouch);
    }
}

#[test]
fn outlaws_fury_affects_only_creatures_there_when_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Outlaws' Fury",
        "Outlaws' Fury affects only creatures you control at the time it resolves."
    );
    supported("Outlaws' Fury");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 2)]);
    let s = t.hand(P0, "Outlaws' Fury");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    let later = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.pt(later), (2, 2));
    // A noncreature permanent that becomes a creature later doesn't get +2/+0 either.
    let copter = t.battlefield(P0, "Smuggler's Copter");
    crate::r_s04_common::crew(&mut t, P0, copter, &[bears]);
    t.resolve_all();
    assert!(t.obj_now(copter).is(CardType::Creature));
    assert_eq!(t.pt(copter), (3, 3));
}

/// Runs P0's end step with Demonic Rising and `before` creatures; `between` changes the
/// game after the trigger would be put on the stack and before it resolves. The number of
/// Demon tokens created.
fn demonic_rising(before: usize, between: impl FnOnce(&mut TestGame, &[ObjectId])) -> usize {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Demonic Rising");
    let cs: Vec<ObjectId> = (0..before)
        .map(|_| t.battlefield(P0, "Grizzly Bears"))
        .collect();
    t.advance_to(P0, Step::End);
    t.settle();
    between(&mut t, &cs);
    t.resolve_all();
    crate::r_s01_common::with_subtype(&t, P0, "Demon").len()
}

#[test]
fn demonic_rising_checks_exactly_one_creature_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Demonic Rising",
        "If you control zero creatures or more than one creature at the beginning of your end step, the ability won’t trigger. If you control zero creatures or more than one creature when the ability resolves, it will do nothing."
    );
    ruling!(
        "Demonic Rising",
        "Although you must control exactly one creature both when the ability would trigger and when it would resolve in order for the ability to create the Demon token, that creature doesn’t have to be the same one both times."
    );
    supported("Demonic Rising");
    assert_eq!(demonic_rising(1, |_, _| {}), 1);
    // Zero or two at the beginning of the end step: no trigger, even if it's one later.
    assert_eq!(
        demonic_rising(0, |t, _| {
            assert_eq!(t.stack_len(), 0);
            t.battlefield(P0, "Grizzly Bears");
        }),
        0
    );
    assert_eq!(
        demonic_rising(2, |t, cs| {
            assert_eq!(t.stack_len(), 0);
            t.g.destroy(cs[0], None);
        }),
        0
    );
    // One when it triggers, two or zero on resolution: nothing.
    assert_eq!(
        demonic_rising(1, |t, _| {
            assert_eq!(t.stack_len(), 1);
            t.battlefield(P0, "Grizzly Bears");
        }),
        0
    );
    assert_eq!(
        demonic_rising(1, |t, cs| {
            t.g.destroy(cs[0], None);
        }),
        0
    );
    // A different single creature on resolution: the Demon.
    assert_eq!(
        demonic_rising(1, |t, cs| {
            t.g.destroy(cs[0], None);
            t.settle();
            t.battlefield(P0, "Hill Giant");
        }),
        1
    );
}
