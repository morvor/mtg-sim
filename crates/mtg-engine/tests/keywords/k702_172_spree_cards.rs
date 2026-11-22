//! CR 702.172 Spree: spells whose modes each add a cost (CR 702.172a), with modes of many
//! kinds.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use mtg_engine::decision::{Agent, Answer, Decision, PassiveAgent};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn rush_of_dread_halves_rounded_up() {
    cr!("702.172a", "107.1a");
    assert_supported("Rush of Dread");
    // Rush of Dread: {1}{B}{B} sorcery, spree, "+ {1} — Target opponent sacrifices half the
    // creatures they control of their choice, rounded up." "+ {2} — Target opponent
    // discards half the cards in their hand, rounded up." "+ {2} — Target opponent loses
    // half their life, rounded up."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
        t.hand(P1, "Forest");
    }
    t.g.players[P1.idx()].life = 15;
    let rush = t.hand(P0, "Rush of Dread");
    add_mana(&mut t, P0, ManaType::B, 8);
    t.cast(P0, rush)
        .modes(&[0, 1, 2])
        .targets(&[Entity::Player(P1), Entity::Player(P1), Entity::Player(P1)])
        .go();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    // Three creatures: two sacrificed; three cards: two discarded; 15 life: 8 lost.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.life(P1), 7);
}

#[test]
fn smugglers_surprise_returns_milled_creature_and_land_cards() {
    cr!("702.172a", "701.17a");
    assert_supported("Smuggler's Surprise");
    // Smuggler's Surprise: {G} instant, spree, "+ {2} — Mill four cards. You may put up to
    // two creature and/or land cards from among the milled cards into your hand."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // A creature card already in the graveyard can't be chosen.
    t.graveyard(P0, "Craw Wurm");
    t.library_top(P0, "Lightning Bolt");
    let forest = t.library_top(P0, "Forest");
    t.library_top(P0, "Divination");
    let giant = t.library_top(P0, "Hill Giant");
    let spell = t.hand(P0, "Smuggler's Surprise");
    add_mana(&mut t, P0, ManaType::G, 3);
    t.cast(P0, spell).modes(&[0]).go();
    // The milled cards are new objects (CR 400.7): P0 takes as many of the offered ones
    // as it may.
    take_all_offered(&mut t, P0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    // The player chose up to two among the milled creature and land cards.
    let offered = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                candidates, max, ..
            } => Some((candidates, max)),
            _ => None,
        })
        .expect("chose among the milled cards");
    // The milled Hill Giant and Forest, as they were in the graveyard.
    let mut got: Vec<(String, bool)> = offered
        .0
        .iter()
        .filter_map(|e| e.object())
        .map(|o| {
            let ob = t.g.obj(o);
            (
                ob.chars.name.to_string(),
                ob.zone == mtg_engine::object::Zone::Graveyard(P0),
            )
        })
        .collect();
    got.sort();
    let want = vec![
        ("Forest".to_string(), true),
        ("Hill Giant".to_string(), true),
    ];
    assert_eq!((got, offered.1), (want, 2));
    for c in [giant, forest] {
        assert_eq!(t.zone(t.g.current(c)), mtg_engine::object::Zone::Hand(P0));
    }
    // Both went to the hand; the other milled cards stay in the graveyard.
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_hand(P0, "Hill Giant") && t.in_hand(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Divination") && t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
}

/// Makes `p` choose as many of the offered entities as it may whenever it's asked to choose
/// entities (other decisions are still the scripted agent's).
fn take_all_offered(t: &mut TestGame, p: PlayerId) {
    struct TakeAll(Box<dyn Agent>);
    impl Agent for TakeAll {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            let answer = self.0.decide(g, p, d);
            match d {
                Decision::ChooseEntities {
                    candidates, max, ..
                } => Answer::Entities(candidates.iter().take(*max as usize).copied().collect()),
                _ => answer,
            }
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[p.idx()], Box::new(PassiveAgent));
    agents[p.idx()] = Box::new(TakeAll(inner));
}

#[test]
fn final_showdown_chooses_a_creature_as_it_resolves() {
    cr!("702.172a", "115.10");
    assert_supported("Final Showdown");
    // Final Showdown: {W} instant, spree, "+ {1} — All creatures lose all abilities until
    // end of turn." "+ {1} — Choose a creature you control. It gains indestructible until
    // end of turn." "+ {3}{W}{W} — Destroy all creatures."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Serra Angel");
    let spell = t.hand(P0, "Final Showdown");
    add_mana(&mut t, P0, ManaType::W, 7);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let cast = t.cast(P0, spell).modes(&[1, 2]).go();
    assert_eq!(pool(&t, P0), 0);
    // The creature isn't a target: nothing is chosen as the spell is cast, only as it
    // resolves.
    let choices = |t: &TestGame| {
        t.asked()
            .into_iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. }))
            .count()
    };
    let targets: usize = t
        .obj(cast)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|c| c.targets.iter())
        .map(Vec::len)
        .sum();
    assert_eq!((targets, choices(&t)), (0, 0));
    t.resolve_all();
    assert_eq!(choices(&t), 1);
    // Only the chosen Bears survive.
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Serra Angel"));
}

#[test]
fn getaway_glamer_destroys_only_the_most_powerful_creature() {
    cr!("702.172a");
    assert_supported("Getaway Glamer");
    // Getaway Glamer: {W} instant, spree, "+ {2} — Destroy target creature if no other
    // creature has greater power."
    for (target, destroyed) in [("Hill Giant", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let giant = t.battlefield(P1, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let spell = t.hand(P0, "Getaway Glamer");
        add_mana(&mut t, P0, ManaType::W, 3);
        let tgt = if target == "Hill Giant" { giant } else { bears };
        t.cast(P0, spell).modes(&[1]).target(tgt).go();
        t.resolve_all();
        assert_eq!(!t.on_battlefield(tgt), destroyed, "{target}");
    }
}
