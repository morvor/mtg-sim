//! Rulings batch P084 — the rest of the graveyard-hate batch: casting from among cards
//! exiled with a permanent (CR 601.2, 601.3, 607.2a), "becomes tapped" triggers, costs
//! paid by sacrificing the source, putting tokens and melded permanents into a library,
//! lands that become creatures, abilities that don't target, chapter abilities that add
//! mana, who chooses, simultaneous choices in APNAP order, and special actions.

use crate::r_p084_common::*;
use crate::r_s01_common::{attack_with, watch};
use crate::r_s06_common::activate_containing;
use mtg_engine::card::CardDb;
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The names of the objects on the stack, bottom first.
fn stack_names(g: &Game) -> Vec<String> {
    g.stack
        .iter()
        .map(|id| g.obj(*id).chars.name.to_string())
        .collect()
}

/// The ways `p` could cast `card` (followed across zone changes) now.
fn cast_methods(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    crate::r_s07_common::cast_methods(t, p, card)
}

/// Lets the permanent attack and tap this turn.
fn unsick(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.obj_mut(id).summoning_sick = false;
}

#[test]
fn cemetery_illuminator_casts_a_face_sharing_a_type_with_the_exiled_card() {
    cr!("601.3", "601.3e", "712.11b", "712.11c");
    ruling!(
        "Cemetery Illuminator",
        "Cemetery Illuminator's ability allows you to cast a spell that shares a card type with the exiled card, even if the card on top of the library doesn't. For example, if you've exiled an artifact card, you could cast The Omenkeel from the top of your library, even though it's the back face of the modal double-faced card Cosima, God of the Voyage, which isn't an artifact."
    );
    supported("Cemetery Illuminator");
    const COSIMA: &str = "Cosima, God of the Voyage // The Omenkeel";
    // Bone Saw (a noncreature artifact card) is exiled as Cemetery Illuminator enters.
    let mut t = TestGame::new(2);
    let saw = t.graveyard(P1, "Bone Saw");
    let top = t.library_top(P0, COSIMA);
    t.answer_choose(P0, &[obj(saw)]);
    t.enter(P0, "Cemetery Illuminator");
    t.resolve_all();
    assert_eq!(t.zone(saw), Zone::Exile);
    mana(&mut t, P0, "1UU");
    let methods = cast_methods(&mut t, P0, top);
    // The Omenkeel (Legendary Artifact — Vehicle) can be cast; Cosima (a creature) can't.
    assert!(methods.contains(&CastMethod::Half(1)), "{methods:?}");
    assert!(!methods.contains(&CastMethod::Normal), "{methods:?}");
    t.cast(P0, top).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("The Omenkeel").len(), 1);
}

#[test]
fn casting_from_among_exiled_cards_needs_the_permanent_only_to_begin() {
    cr!("601.2", "601.3", "607.2a", "608.3");
    ruling!(
        "Dawnhand Dissident",
        "Dawnhand Dissident must be under your control in order for you to start casting a creature spell using its last ability, but it doesn't need to stay under your control throughout that process or be under your control as the spell resolves. If, for example, you sacrifice Dawnhand Dissident to pay an additional cost of the spell or it's destroyed in response, the spell will still resolve normally."
    );
    ruling!(
        "Intrepid Paleontologist",
        "Intrepid Paleontologist must be under your control in order to start casting a Dinosaur creature spell using its last ability, but it doesn't need to stay under your control throughout that process or be under your control as the spell resolves. If, for example, you sacrifice Intrepid Paleontologist to pay an additional cost of the spell or it's destroyed in response, the spell will still resolve normally."
    );
    // Dawnhand Dissident exiles P0's Grizzly Bears: Blight 2 puts two -1/-1 counters on a
    // Hill Giant, and with the other Hill Giant's, there are three counters to remove.
    // Gone before the casting begins, it gives no permission; gone once the spell is on
    // the stack (destroyed in response), the spell still resolves.
    supported("Dawnhand Dissident");
    for gone_first in [true, false] {
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, "Dawnhand Dissident");
        let bears = t.graveyard(P0, "Grizzly Bears");
        let g1 = t.battlefield(P0, "Hill Giant");
        let g2 = t.battlefield(P0, "Hill Giant");
        t.g.add_counters(obj(g2), "-1/-1", 1, None);
        t.answer_choose(P0, &[obj(g1)]);
        t.activate(P0, d, 1, &[obj(bears)]).unwrap();
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Exile);
        mana(&mut t, P0, "1G");
        assert!(!cast_methods(&mut t, P0, bears).is_empty());
        if gone_first {
            t.g.destroy(d, None);
            t.g.flush_events();
            t.settle();
            assert!(cast_methods(&mut t, P0, bears).is_empty());
            continue;
        }
        let spell = t.cast(P0, t.g.current(bears)).go();
        // Destroyed in response: the Bears still resolve.
        t.g.destroy(d, None);
        t.g.flush_events();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Dawnhand Dissident"));
        assert_eq!(t.zone(spell), Zone::Battlefield, "{}", t.dump_log());
    }
    // Intrepid Paleontologist exiles P0's Colossal Dreadmaw (a Dinosaur).
    supported("Intrepid Paleontologist");
    for gone_first in [true, false] {
        let mut t = TestGame::new(2);
        let ip = t.battlefield(P0, "Intrepid Paleontologist");
        let maw = t.graveyard(P0, "Colossal Dreadmaw");
        activate(&mut t, P0, ip, "Exile target card", "2", &[obj(maw)]);
        t.resolve_all();
        assert_eq!(t.zone(maw), Zone::Exile);
        mana(&mut t, P0, "4GG");
        assert!(!cast_methods(&mut t, P0, maw).is_empty());
        if gone_first {
            t.g.destroy(ip, None);
            t.g.flush_events();
            t.settle();
            assert!(cast_methods(&mut t, P0, maw).is_empty());
            continue;
        }
        let spell = t.cast(P0, t.g.current(maw)).go();
        t.g.destroy(ip, None);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.zone(spell), Zone::Battlefield, "{}", t.dump_log());
        assert_eq!(t.counters(spell, "finality"), 1);
    }
}

#[test]
fn immersturm_predator_triggers_only_when_it_becomes_tapped() {
    cr!("603.2e", "701.26a", "508.1f");
    ruling!(
        "Immersturm Predator",
        "For the triggered ability to trigger, Immersturm Predator has to actually change from untapped to tapped. If an effect attempts to tap it while it is already tapped, the ability won't trigger."
    );
    ruling!(
        "Immersturm Predator",
        "Immersturm Predator's triggered ability doesn't allow you to tap Immersturm Predator. You have to find some other way to tap it. Let us suggest attacking."
    );
    ruling!(
        "Immersturm Predator",
        "You can activate Immersturm Predator's last ability even if it's already tapped."
    );
    supported("Immersturm Predator");
    // Its trigger is no way to tap it: alone and untapped, nothing of it can be activated.
    let mut t = TestGame::new(2);
    let ip = t.battlefield(P0, "Immersturm Predator");
    let card = t.graveyard(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    assert!(!t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == ip)));
    // Attacking taps it: the trigger exiles a card and puts a counter on it.
    t.answer_targets(P0, &[obj(card)]);
    attack_with(&mut t, &[(ip, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    assert_eq!(t.counters(ip, "+1/+1"), 1);
    // Already tapped: its last ability ("Sacrifice another creature: This creature gains
    // indestructible until end of turn. Tap it.") can still be activated; tapping it again
    // doesn't trigger.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[obj(elves)]);
    activate_containing(&mut t, P0, ip, "Sacrifice another").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert!(t.obj_now(ip).has_keyword(KeywordKind::Indestructible));
    assert_eq!(t.counters(ip, "+1/+1"), 1);
    // Untapped, the same ability taps it: that triggers.
    let mut t = TestGame::new(2);
    let ip = t.battlefield(P0, "Immersturm Predator");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bolt = t.graveyard(P1, "Lightning Bolt");
    t.answer_choose(P0, &[obj(elves)]);
    t.answer_targets(P0, &[obj(bolt)]);
    activate_containing(&mut t, P0, ip, "Sacrifice another").unwrap();
    t.resolve_all();
    assert!(t.obj_now(ip).tapped);
    assert_eq!(t.counters(ip, "+1/+1"), 1);
    // The trigger itself doesn't tap it.
    assert!(t.in_exile("Lightning Bolt"));
}

#[test]
fn general_kudro_sacrifices_itself_and_ignores_human_planeswalkers() {
    cr!("602.2b", "118.3", "205.3m");
    ruling!(
        "General Kudro of Drannith",
        "General Kudro can be sacrificed to pay the cost of its own last ability. How noble."
    );
    ruling!(
        "General Kudro of Drannith",
        "Only creatures are Humans. Planeswalkers such as Lukka and Narset who represent human characters won't cause General Kudro's middle ability to trigger."
    );
    supported("General Kudro of Drannith");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "General Kudro of Drannith");
    let human = t.battlefield(P0, "Elite Vanguard");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(obj(giant), "+1/+1", 1, None);
    t.answer_choose(P0, &[obj(k), obj(human)]);
    activate(&mut t, P0, k, "Destroy target creature", "2", &[obj(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "General Kudro of Drannith"));
    assert!(t.in_graveyard(P0, "Elite Vanguard"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Narset, Parter of Veils entering doesn't trigger Kudro; a Human creature does.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "General Kudro of Drannith");
    t.graveyard(P1, "Lightning Bolt");
    t.enter(P0, "Narset, Parter of Veils");
    t.settle();
    assert_eq!(
        crate::r_s01_common::triggers_on_stack(&t, "exile target card"),
        0
    );
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    t.enter(P0, "Elite Vanguard");
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
}

#[test]
fn endless_detour_puts_a_token_into_the_library_then_it_ceases_to_exist() {
    cr!("111.7", "704.5d");
    ruling!(
        "Endless Detour",
        "If Endless Detour targets a token, that token is put into the library, then ceases to exist."
    );
    let mut t = TestGame::new(2);
    let tok = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    let lib = t.library_size(P1);
    let spell = cast_modes(&mut t, P0, "Endless Detour", "GWU", &[], &[&[obj(tok)]]);
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Graveyard(P0));
    assert!(!t.g.is_live(t.g.current(tok)));
    assert_eq!(t.library_size(P1), lib);
    assert!(tokens(&t, P1).is_empty());
}

#[test]
fn a_melded_permanent_goes_all_on_top_or_all_on_the_bottom() {
    cr!("712.21a", "712.21c", "401.4");
    ruling!(
        "Endless Detour",
        "If multiple cards are put into the library this way (such as when the spell targets a melded permanent), that permanent's owner puts all the cards on top or all the cards on the bottom. They put them in whatever order they wish, and do not need to reveal the order."
    );
    ruling!(
        "Uncharted Voyage",
        "If multiple cards are put into the library this way (such as when the spell targets a melded permanent), that permanent's owner puts all the cards on top or all the cards on the bottom. They put them in whatever order they wish, and do not need to reveal the order."
    );
    for (spell, cost) in [("Endless Detour", "GWU"), ("Uncharted Voyage", "3U")] {
        for top in [true, false] {
            let mut t = TestGame::new(2);
            let rats = t.exile(P1, "Graf Rats");
            let scav = t.exile(P1, "Midnight Scavengers");
            let host = CardDb::global().get("Chittering Host").unwrap();
            let m = mtg_engine::merge::meld(&mut t.g, rats, scav, host, P1).unwrap();
            t.settle();
            // P1 (the owner) chooses top or bottom.
            t.answer(P1, DecisionKind::YesNo, Answer::Bool(top));
            t.answer(P1, DecisionKind::Option, Answer::Index(usize::from(!top)));
            cast_modes(&mut t, P0, spell, cost, &[], &[&[obj(m)]]);
            t.resolve_all();
            let lib = &t.g.player(P1).library;
            // Libraries are stored bottom first.
            let ids: Vec<ObjectId> = if top {
                lib.iter().rev().take(2).copied().collect()
            } else {
                lib.iter().take(2).copied().collect()
            };
            let ends: Vec<String> = ids
                .iter()
                .map(|id| t.g.obj(*id).chars.name.to_string())
                .collect();
            let mut ends = ends;
            ends.sort();
            assert_eq!(
                ends,
                vec!["Graf Rats".to_string(), "Midnight Scavengers".to_string()],
                "{spell} top={top}: {}",
                t.dump_log()
            );
        }
    }
}

#[test]
fn hive_of_the_eye_tyrant_ignores_lands_entering_with_it() {
    cr!("614.12", "614.1d");
    ruling!(
        "Hive of the Eye Tyrant",
        "If Hive of the Eye Tyrant enters the battlefield at the same time as one or more other lands, it doesn't take those lands into consideration when determining how many other lands you control."
    );
    supported("Manabond");
    // Manabond: "At the beginning of your end step, you may reveal your hand and put all
    // land cards from it onto the battlefield. If you do, discard your hand." With one
    // other land before, Hive enters untapped with two Mountains; with two, tapped.
    for already in [1usize, 2] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Manabond");
        t.lands(P0, "Swamp", already);
        let hive = t.hand(P0, "Hive of the Eye Tyrant");
        let m1 = t.hand(P0, "Mountain");
        let m2 = t.hand(P0, "Mountain");
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        for c in [hive, m1, m2] {
            assert!(t.on_battlefield(c), "already={already}");
        }
        assert_eq!(t.obj_now(hive).tapped, already >= 2, "already={already}");
    }
}

#[test]
fn hive_of_the_eye_tyrant_animated_twice_has_two_attack_triggers() {
    cr!("613.1f", "603.2", "113.2c");
    ruling!(
        "Hive of the Eye Tyrant",
        "If you activate Hive of the Eye Tyrant's last ability multiple times, it gains multiple instances of the triggered ability that exiles cards. For example, if you activate it twice and attack, you will choose two target cards to exile (or the same card twice, if you want to)."
    );
    supported("Hive of the Eye Tyrant");
    let mut t = TestGame::new(2);
    let hive = t.battlefield(P0, "Hive of the Eye Tyrant");
    for _ in 0..2 {
        activate(&mut t, P0, hive, "becomes a 3/3", "3B", &[]);
        t.resolve_all();
    }
    let a = t.graveyard(P1, "Lightning Bolt");
    let b = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(a)]);
    t.answer_targets(P0, &[obj(b)]);
    attack_with(&mut t, &[(hive, pl(P1))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
}

#[test]
fn hive_of_the_eye_tyrant_animated_the_turn_it_arrived_cant_attack_or_tap() {
    cr!("302.6", "508.1a");
    ruling!(
        "Hive of the Eye Tyrant",
        "If you turn Hive of the Eye Tyrant into a creature but haven't controlled it continuously since your most recent turn began, you won't be able to activate its mana ability or attack with it."
    );
    let mut t = TestGame::new(2);
    // As a land only, it taps for mana the turn it arrives.
    let mut t2 = TestGame::new(2);
    let h2 = t2.battlefield_sick(P0, "Hive of the Eye Tyrant");
    assert!(t2.activate(P0, h2, 0, &[]).is_ok());
    assert_eq!(t2.g.players[0].mana_pool.count(ManaType::B), 1);
    let hive = t.battlefield_sick(P0, "Hive of the Eye Tyrant");
    activate(&mut t, P0, hive, "becomes a 3/3", "3B", &[]);
    t.resolve_all();
    assert!(crate::r_s01_common::creatures(&t, P0).contains(&hive));
    t.g.recompute();
    let acts = t.g.legal_actions(P0);
    let mana_uid = t
        .obj_now(hive)
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains("Add {B}"))
        .map(|a| a.uid)
        .unwrap();
    assert!(!acts
        .iter()
        .any(|a| matches!(a, Action::Activate { source, ability } if *source == hive && *ability == mana_uid)));
    assert!(t.activate(P0, hive, 0, &[]).is_err());
    assert!(!crate::r_s09_common::legal_attack(
        &mut t,
        &[(hive, pl(P1))]
    ));
}

#[test]
fn dreams_of_steel_and_oil_exiles_from_the_graveyard_anyway() {
    cr!("608.2c", "609.3");
    ruling!(
        "Dreams of Steel and Oil",
        "If the opponent has no artifact or creature cards in their hand, you must still choose an artifact or creature card from their graveyard to exile."
    );
    supported("Dreams of Steel and Oil");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bears = t.graveyard(P1, "Grizzly Bears");
    cast_modes(
        &mut t,
        P0,
        "Dreams of Steel and Oil",
        "B",
        &[],
        &[&[pl(P1)]],
    );
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(bolt), Zone::Hand(P1));
}

#[test]
fn eater_of_the_dead_does_something_only_if_tapped_as_it_resolves() {
    cr!("608.2c", "602.2");
    ruling!(
        "Eater of the Dead",
        "It untaps during the resolution of its ability."
    );
    ruling!(
        "Eater of the Dead",
        "You can activate the ability whether or not Eater of the Dead is tapped. However, if Eater of the Dead is untapped when the ability resolves, the ability won't do anything."
    );
    supported("Eater of the Dead");
    // (tapped when activated, tapped as it resolves)
    for (at_activation, at_resolution) in
        [(true, true), (false, false), (true, false), (false, true)]
    {
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, "Eater of the Dead");
        let bears = t.graveyard(P1, "Grizzly Bears");
        t.g.obj_mut(e).tapped = at_activation;
        activate(
            &mut t,
            P0,
            e,
            "exile target creature card",
            "",
            &[obj(bears)],
        );
        t.settle();
        assert_eq!(t.stack_len(), 1);
        t.g.obj_mut(e).tapped = at_resolution;
        t.resolve_all();
        let did = at_resolution;
        assert_eq!(
            t.zone(bears) == Zone::Exile,
            did,
            "{at_activation} {at_resolution}"
        );
        assert!(!t.obj_now(e).tapped);
    }
}

#[test]
fn stonecloakers_return_doesnt_target_and_cant_be_responded_to() {
    cr!("603.3d", "608.2c", "115.1");
    ruling!(
        "Stonecloaker",
        "Stonecloaker's first triggered ability doesn't target any creature. Because of this, no player may take actions between the time you choose the creature to return and the time you return it."
    );
    supported("Stonecloaker");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P1, "Lightning Bolt");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| stack_names(g),
    );
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.enter(P0, "Stonecloaker");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    // The creature was chosen as the ability resolved (it was on the stack), with no
    // target chosen for it and no priority in between.
    let seen = seen.lock().unwrap().clone();
    assert!(
        seen.iter()
            .any(|s| s.iter().any(|n| n.starts_with("Stonecloaker"))),
        "{seen:?}"
    );
    let targets = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(
        targets.iter().all(|c| !c.contains(&obj(bears))),
        "{targets:?}"
    );
}

#[test]
fn esper_valigarmandas_mana_chapters_use_the_stack() {
    cr!("714.2b", "605.1b", "605.5a");
    ruling!(
        "Summon: Esper Valigarmanda",
        "Summon: Esper Valigarmanda's second, third, and fourth chapter abilities aren't mana abilities. They use the stack and can be responded to."
    );
    supported("Summon: Esper Valigarmanda");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Summon: Esper Valigarmanda");
    t.g.add_counters(obj(s), "lore", 1, None);
    t.g.flush_events();
    t.g.add_counters(obj(s), "lore", 1, None);
    t.g.flush_events();
    t.settle();
    // Chapter II is on the stack; no mana yet; P1 can respond.
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "Add {R}"), 1);
    assert!(t.g.players[0].mana_pool.is_empty());
    t.resolve_all();
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::R), 2);
}

/// King Narfi's Betrayal enters; chapter I mills four from each library (the top card of
/// P1's is `card`) and P0 exiles that card from P1's graveyard. Returns the Saga.
fn narfi(t: &mut TestGame, card: &str) -> (ObjectId, ObjectId) {
    supported("King Narfi's Betrayal");
    let c = t.library_top(P1, card);
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(c)]);
    let s = t.enter(P0, "King Narfi's Betrayal");
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Exile, "{}", t.dump_log());
    (s, c)
}

/// Adds a lore counter to the Saga and resolves the chapter ability.
fn next_chapter(t: &mut TestGame, s: ObjectId) {
    t.g.add_counters(obj(s), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
}

#[test]
fn king_narfis_betrayal_casts_only_its_own_cards_at_their_usual_time() {
    cr!("607.2a", "601.3", "307.1", "714.2b");
    ruling!(
        "King Narfi's Betrayal",
        "The \"cards exiled with King Narfi's Betrayal\" are only the cards exiled by that particular King Narfi's Betrayal, not any other Betrayals that you previously controlled."
    );
    ruling!(
        "King Narfi's Betrayal",
        "The chapter II/III ability doesn't change when you can cast the exiled cards. In most cases, this means during your main phase while the stack is empty, although flash may change this."
    );
    let mut t = TestGame::new(2);
    let (a, bears) = narfi(&mut t, "Grizzly Bears");
    let (b, giant) = narfi(&mut t, "Hill Giant");
    // Chapter II of the second Saga: its Hill Giant, not the first one's Grizzly Bears.
    next_chapter(&mut t, b);
    let _ = a;
    mana(&mut t, P0, "5");
    assert!(!cast_methods(&mut t, P0, giant).is_empty());
    assert!(cast_methods(&mut t, P0, bears).is_empty());
    // A creature: not with a spell on the stack, nor in combat.
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, "R");
    t.cast(P1, bolt).target(pl(P1)).go();
    assert!(cast_methods(&mut t, P0, giant).is_empty());
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(cast_methods(&mut t, P0, giant).is_empty());
    t.set_step(P0, Step::PostcombatMain);
    assert!(!cast_methods(&mut t, P0, giant).is_empty());
}

#[test]
fn dire_fleet_daredevils_card_keeps_its_timing() {
    cr!("601.3", "307.1", "611.2a");
    ruling!(
        "Dire Fleet Daredevil",
        "The effect of Dire Fleet Daredevil doesn't change when you can cast the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    let mut t = TestGame::new(2);
    let div = t.graveyard(P1, "Divination");
    t.answer_targets(P0, &[obj(div)]);
    t.enter(P0, "Dire Fleet Daredevil");
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Exile);
    mana(&mut t, P0, "3");
    assert!(!cast_methods(&mut t, P0, div).is_empty());
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, "R");
    t.cast(P1, bolt).target(pl(P1)).go();
    assert!(cast_methods(&mut t, P0, div).is_empty());
    t.resolve_all();
    t.set_step(P0, Step::EndOfCombat);
    assert!(cast_methods(&mut t, P0, div).is_empty());
}

#[test]
fn simple_graveyard_hate_who_chooses_and_who_gets_what() {
    cr!("608.2c", "111.2");
    ruling!(
        "Beckon Apparition",
        "The controller of Beckon Apparition gets the token."
    );
    ruling!(
        "Merrow Bonegnawer",
        "The targeted player chooses which card to exile."
    );
    // P1 casts Beckon Apparition on a card in P0's graveyard: P1 gets the Spirit.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_modes(&mut t, P1, "Beckon Apparition", "W", &[], &[&[obj(card)]]);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P1, "Spirit"), 1);
    assert_eq!(tokens_of(&t, P0, "Spirit"), 0);
    // Merrow Bonegnawer targeting P1: P1 picks the Hill Giant.
    supported("Merrow Bonegnawer");
    let mut t = TestGame::new(2);
    let mb = t.battlefield(P0, "Merrow Bonegnawer");
    t.graveyard(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(giant)]);
    activate(&mut t, P0, mb, "exiles a card", "", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Exile);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn watchers_of_the_dead_choices_are_made_in_apnap_order_then_exiled_together() {
    cr!("101.4", "608.2e");
    ruling!(
        "Watchers of the Dead",
        "When this ability resolves, first the player whose turn it is (if that player is an opponent) chooses two cards in their graveyard, then each other opponent in turn order does the same, then all of the other cards in their graveyards are exiled simultaneously. Each player will know the choices made by the players who chose before them."
    );
    supported("Watchers of the Dead");
    // Four players; P2 activates it on P1's turn: P1, then P3, then P0 choose.
    let mut t = TestGame::new(4);
    t.set_step(P1, Step::PrecombatMain);
    let w = t.battlefield(P2, "Watchers of the Dead");
    let mut kept = vec![];
    let mut order = Vec::new();
    for p in [P0, P1, P3] {
        let cards: Vec<ObjectId> = ["Grizzly Bears", "Hill Giant", "Lightning Bolt"]
            .iter()
            .map(|n| t.graveyard(p, n))
            .collect();
        t.answer_choose(p, &[obj(cards[0]), obj(cards[1])]);
        kept.extend_from_slice(&cards[..2]);
    }
    t.graveyard(P2, "Shock");
    for p in [P0, P1, P3] {
        let seen = watch(
            &mut t,
            p,
            |d| matches!(d, Decision::ChooseEntities { .. }),
            |g| (g.actions_taken, g.exile.len()),
        );
        order.push((p, seen));
    }
    activate_containing(&mut t, P2, w, "Each opponent chooses").unwrap();
    t.resolve_all();
    for k in &kept {
        assert!(matches!(t.zone(*k), Zone::Graveyard(_)));
    }
    assert_eq!(
        t.g.exile
            .iter()
            .filter(|o| t.g.obj(**o).chars.name == "Lightning Bolt")
            .count(),
        3
    );
    assert!(t.in_graveyard(P2, "Shock"));
    // Each chose once, nothing exiled until all had chosen, in the order P1, P3, P0.
    let mut when: Vec<(u64, PlayerId)> = vec![];
    for (p, seen) in &order {
        let s = seen.lock().unwrap().clone();
        assert_eq!(s.len(), 1, "{p:?}");
        assert_eq!(s[0].1, 1, "{p:?}: only Watchers itself is in exile");
        when.push((s[0].0, *p));
    }
    when.sort();
    let ps: Vec<PlayerId> = when.into_iter().map(|(_, p)| p).collect();
    assert_eq!(ps, vec![P1, P3, P0]);
}

#[test]
fn lost_in_thought_is_ignored_with_a_special_action() {
    cr!("116.2d", "116.1", "508.1c");
    ruling!(
        "Lost in Thought",
        "You can take the “exile three cards” action any time you have priority without using the stack."
    );
    supported("Lost in Thought");
    // Without the special action, the enchanted Bears can't attack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Lost in Thought", bears);
    t.set_step(P1, Step::BeginningOfCombat);
    unsick(&mut t, bears);
    assert!(!crate::r_s09_common::legal_attack(
        &mut t,
        &[(bears, pl(P0))]
    ));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Lost in Thought", bears);
    let cards: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P1, "Hill Giant")).collect();
    t.set_step(P1, Step::Upkeep);
    // With a spell on the stack, P1 still can.
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, "R");
    t.cast(P0, bolt).target(pl(P0)).go();
    t.g.turn.priority = Some(P1);
    t.g.recompute();
    let sa =
        t.g.legal_actions(P1)
            .into_iter()
            .find_map(|a| match a {
                Action::Special(s @ SpecialAction::Static { .. }) => Some(s),
                _ => None,
            })
            .expect("the special action is offered");
    let es: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.answer_choose(P1, &es);
    let stack = t.stack_len();
    t.g.take_action(P1, Action::Special(sa));
    t.g.flush_events();
    assert_eq!(t.stack_len(), stack, "it didn't use the stack");
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    t.resolve_all();
    // The Bears can attack this turn.
    t.set_step(P1, Step::BeginningOfCombat);
    unsick(&mut t, bears);
    assert!(crate::r_s09_common::legal_attack(
        &mut t,
        &[(bears, pl(P0))]
    ));
}

#[test]
fn cemetery_reaper_pumps_every_other_zombie_you_control() {
    cr!("613.4c", "611.3a");
    ruling!(
        "Cemetery Reaper",
        "Cemetery Reaper grants +1/+1 to all other Zombie creatures you control, not just the ones its ability puts onto the battlefield."
    );
    supported("Cemetery Reaper");
    let mut t = TestGame::new(2);
    let reaper = t.battlefield(P0, "Cemetery Reaper");
    let digger = t.battlefield(P0, "Gravedigger");
    let theirs = t.battlefield(P1, "Gravedigger");
    let bears = t.graveyard(P1, "Grizzly Bears");
    activate(
        &mut t,
        P0,
        reaper,
        "Exile target creature",
        "2B",
        &[obj(bears)],
    );
    t.resolve_all();
    let zombie = tokens(&t, P0)[0];
    assert_eq!(t.pt(zombie), (3, 3));
    assert_eq!(t.pt(digger), (3, 3));
    assert_eq!(t.pt(reaper), (2, 2));
    assert_eq!(t.pt(theirs), (2, 2));
}
