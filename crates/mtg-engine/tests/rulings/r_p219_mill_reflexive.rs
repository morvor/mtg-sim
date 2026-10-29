//! Rulings batch P219 — mill (CR 701.17) with reflexive triggered abilities ("mill N
//! cards. When you do, ...": the target is chosen only when the reflexive ability
//! triggers, CR 603.12), casting milled cards, and the order of "mill, then discard"
//! instructions in multiplayer games.

use crate::r_p016_common::stack_targets;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::{in_hand_with_mana, respond};
use crate::r_s05_common::enter;
use crate::r_s07_common::cast_methods;
use crate::r_s11_common::{empty_library, triggered_from};
use crate::r_s19_common::add_lore;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::cell::Cell;

thread_local! {
    /// The card name [`target_named`] targets.
    static TARGET: Cell<&'static str> = const { Cell::new("") };
}

/// Targets the candidate named [`TARGET`] (an object that may not exist yet when the
/// answer is scripted).
fn target_named(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::ChooseTargets { candidates, .. } = d else {
        return None;
    };
    let want = TARGET.with(|w| w.get());
    let pick = candidates
        .iter()
        .find(|e| e.object().is_some_and(|o| g.obj(o).chars.name == want))?;
    Some(Answer::Entities(vec![*pick]))
}

/// The number of target choices asked so far.
fn target_choices(t: &TestGame) -> usize {
    t.asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count()
}

/// The triggered ability on top of the stack.
fn top(t: &TestGame) -> ObjectId {
    *t.g.stack.last().expect("empty stack")
}

#[test]
fn the_reflexive_target_is_chosen_after_milling() {
    cr!("603.12", "603.3d", "701.17a");
    ruling!(
        "Wick's Patrol",
        "You don't choose a target for Wick's Patrol's ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you mill three cards this way. You choose a target for this ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    ruling!(
        "Yathan Roadwatcher",
        "You don’t choose a target for Yathan Roadwatcher’s ability at the time it triggers. Rather, a second “reflexive” ability triggers when you mill four cards this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    ruling!(
        "The Last Ronin",
        "You don't choose a target for The Last Ronin's second chapter ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you mill four cards this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    ruling!(
        "Sigil of Myrkul",
        "Sigil of Myrkul's triggered ability triggers and goes on the stack without a target. After you mill a card, if there are four or more creature cards in your graveyard, the reflexive triggered ability will trigger. You choose the target for that ability at that time."
    );
    ruling!(
        "Flitting Guerrilla",
        "The triggered ability of Flitting Guerrilla triggers and goes on the stack without a target. If you exile Flitting Guerrilla, the second \"reflexive\" triggered ability will trigger. You'll choose the target creature or battle card for that second ability at that time. You may choose a card you just milled as the target or a card that was already in your graveyard."
    );
    for card in [
        "Wick's Patrol",
        "Yathan Roadwatcher",
        "The Last Ronin",
        "Sigil of Myrkul",
        "Flitting Guerrilla",
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        // The target: P1's Hill Giant (Wick's Patrol, -X/-X where X is the greatest
        // mana value in P0's graveyard), or a creature card P0 mills (the others).
        let giant = t.battlefield(P1, "Hill Giant");
        TARGET.with(|w| w.set(if card == "Wick's Patrol" { "Hill Giant" } else { "Gray Ogre" }));
        respond(&mut t, P0, target_named);
        stack_library(&mut t, P0, &["Gray Ogre", "Forest", "Island", "Swamp"]);
        let source = match card {
            "Yathan Roadwatcher" => {
                let c = in_hand_with_mana(&mut t, P0, card);
                t.cast(P0, c).go();
                t.resolve();
                t.named_on_battlefield(card)[0]
            }
            "The Last Ronin" => {
                let saga = t.battlefield(P0, card);
                add_lore(&mut t, saga, 1);
                t.resolve_all();
                add_lore(&mut t, saga, 1);
                saga
            }
            "Sigil of Myrkul" => {
                // Three creature cards already there; the milled Gray Ogre is the fourth.
                for _ in 0..3 {
                    t.graveyard(P0, "Grizzly Bears");
                }
                t.battlefield(P0, "Gray Ogre");
                let sigil = t.battlefield(P0, card);
                t.advance_to(P0, Step::BeginningOfCombat);
                TARGET.with(|w| w.set("Gray Ogre"));
                sigil
            }
            "Flitting Guerrilla" => {
                let g = t.battlefield(P0, card);
                t.answer_yes(P0, true);
                destroy(&mut t, g);
                g
            }
            _ => enter(&mut t, P0, card),
        };
        t.settle();
        // The trigger is on the stack without a target, and none was asked for.
        assert_eq!(t.stack_len(), 1, "{card}");
        assert!(stack_targets(&t, top(&t)).is_empty(), "{card}");
        let asked = target_choices(&t);
        t.resolve();
        // After milling, the reflexive ability triggered and its target was chosen.
        assert!(t.in_graveyard(P0, "Gray Ogre"), "{card}");
        assert_eq!(t.stack_len(), 1, "{card}");
        assert_eq!(target_choices(&t), asked + 1, "{card}");
        let target = stack_targets(&t, top(&t));
        assert_eq!(target.len(), 1, "{card}");
        let named = t.g.obj(target[0].object().unwrap()).chars.name.clone();
        assert_eq!(named, TARGET.with(|w| w.get()), "{card}");
        t.resolve_all();
        match card {
            "Wick's Patrol" => assert!(!t.on_battlefield(giant)),
            "Sigil of Myrkul" => {
                let ogre = t.named_on_battlefield("Gray Ogre")[0];
                assert_eq!(t.counters(ogre, "+1/+1"), 1);
            }
            "Flitting Guerrilla" => {
                assert!(t.in_exile(card));
                let top = t.g.library_top(P0).unwrap();
                assert_eq!(t.g.obj(top).chars.name, "Gray Ogre");
            }
            "The Last Ronin" => assert!(t.in_hand(P0, "Gray Ogre")),
            _ => assert_eq!(t.named_on_battlefield("Gray Ogre").len(), 1, "{card}"),
        }
        let _ = source;
    }
}

#[test]
fn wicks_patrol_doesnt_reflex_if_it_mills_fewer_than_three() {
    cr!("603.12", "701.17b");
    ruling!(
        "Wick's Patrol",
        "If your library has two or fewer cards in it when Wick's Patrol's ability resolves, you'll mill as many cards as you can, but the reflexive ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Craw Wurm", "Forest"]);
    let patrol = enter(&mut t, P0, "Wick's Patrol");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(triggered_from(&t, patrol), 1);
    assert!(t.on_battlefield(giant));
}

#[test]
fn summons_of_saruman_casts_during_resolution_only() {
    cr!("608.2g", "701.17a");
    ruling!(
        "Summons of Saruman",
        "If you want to cast one of the milled cards, you cast it as part of the resolution of Summons of Saruman. You can't wait to cast the spell later in the turn. Timing permissions based on the card's type are ignored."
    );
    supported("Summons of Saruman");
    supported("Lava Spike");
    // X = 1: Lava Spike (a sorcery, cast while Summons is still on the stack).
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Lava Spike"]);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let summons = t.hand(P0, "Summons of Saruman");
    fn first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
        let Decision::ChooseEntities { candidates, .. } = d else {
            return None;
        };
        Some(Answer::Entities(candidates.iter().take(1).copied().collect()))
    }
    respond(&mut t, P0, first);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, summons).x(1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Declining: it can't be cast later.
    let mut t = TestGame::new(2);
    let spike = stack_library(&mut t, P0, &["Lava Spike"])[0];
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    let summons = t.hand(P0, "Summons of Saruman");
    t.answer_choose(P0, &[]);
    t.cast(P0, summons).x(1).go();
    t.resolve_all();
    let spike = t.g.current(spike);
    assert_eq!(t.zone(spike), Zone::Graveyard(P0));
    assert!(cast_methods(&mut t, P0, spike).is_empty());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn locke_permission_is_unused_by_casting_with_another_permission() {
    cr!("601.3", "702.34a");
    ruling!(
        "Locke, Treasure Hunter",
        "If you cast a spell from among the milled cards using another permission, Locke's effect doesn't apply. You can cast another spell from among the milled cards using the permission granted by Locke's last ability."
    );
    supported("Locke, Treasure Hunter");
    supported("Think Twice");
    let mut t = TestGame::new(2);
    let locke = t.battlefield(P0, "Locke, Treasure Hunter");
    let twice = t.library_top(P0, "Think Twice");
    let shock = t.library_top(P1, "Shock");
    attack_with(&mut t, &[(locke, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Island", 3);
    // Think Twice for its flashback cost (its own permission).
    let twice = t.g.current(twice);
    let flashback = CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback);
    assert!(cast_methods(&mut t, P0, twice).contains(&flashback));
    t.cast(P0, twice).method(flashback).go();
    t.resolve_all();
    assert!(t.in_exile("Think Twice"));
    // Shock, with Locke's permission.
    t.lands(P0, "Mountain", 1);
    let shock = t.g.current(shock);
    let methods = cast_methods(&mut t, P0, shock);
    assert!(!methods.is_empty());
    t.cast(P0, shock)
        .method(methods[0].clone())
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 2 - 2);
}

#[test]
fn singularity_rupture_targets_are_optional_but_all_illegal_means_nothing() {
    cr!("608.2b", "115.1a", "800.4a");
    ruling!(
        "Singularity Rupture",
        "You don’t have to choose any targets for Singularity Rupture. However, if you do and those targets are all illegal as Singularity Rupture tries to resolve, it won’t resolve and none of its effects will happen. No creatures will be destroyed, and no one will mill any cards."
    );
    supported("Singularity Rupture");
    // No targets: the creatures are destroyed.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rupture = in_hand_with_mana(&mut t, P0, "Singularity Rupture");
    t.cast(P0, rupture).targets(&[]).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // Its only target left the game: nothing happens.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rupture = in_hand_with_mana(&mut t, P0, "Singularity Rupture");
    t.cast(P0, rupture).targets(&[Entity::Player(P2)]).go();
    t.g.player_loses(P2);
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Singularity Rupture"));
}

/// Hands of `p` and the library sizes of every player, at a decision.
fn hands_and_libraries(g: &mtg_engine::game::Game) -> (Vec<usize>, Vec<usize>) {
    (
        g.players.iter().map(|p| p.hand.len()).collect(),
        g.players.iter().map(|p| p.library.len()).collect(),
    )
}

#[test]
fn invasion_of_amonkhet_mills_then_opponents_discard_together_then_you_draw() {
    cr!("101.4", "701.9a", "701.17a");
    ruling!(
        "Invasion of Amonkhet // Lazotep Convert",
        "To resolve Invasion of Amonkhet’s enters-the-battlefield ability, first each player mills three cards. Then the next opponent in turn order (or, if it’s an opponent’s turn, the opponent whose turn it is) chooses a card in hand and sets it aside without revealing it. Then each other opponent in turn order does the same. Finally, all chosen cards are revealed and discarded at the same time. Finally, you draw a card."
    );
    supported("Invasion of Amonkhet // Lazotep Convert");
    let mut t = TestGame::new(3);
    for p in [P1, P2] {
        t.hand(p, "Grizzly Bears");
        t.hand(p, "Forest");
    }
    let is_choice = |d: &Decision| matches!(d, Decision::ChooseEntities { .. });
    let seen1 = watch(&mut t, P1, is_choice, hands_and_libraries);
    let seen2 = watch(&mut t, P2, is_choice, hands_and_libraries);
    let from = t.asked().len();
    enter(&mut t, P0, "Invasion of Amonkhet // Lazotep Convert");
    t.resolve_all();
    let seen1 = seen1.lock().unwrap().clone();
    let seen2 = seen2.lock().unwrap().clone();
    assert_eq!(seen1.len(), 1);
    assert_eq!(seen2.len(), 1);
    // Everyone milled three first; P0 hasn't drawn yet.
    assert_eq!(seen1[0].1, vec![27, 27, 27]);
    assert_eq!(seen1[0].0[0], 0);
    // P2 chooses while P1's chosen card is still in P1's hand: they're discarded together.
    assert_eq!(seen2[0].0, vec![0, 2, 2]);
    // P1 chose first.
    let choosers: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(p, d)| is_choice(d) && *p != P0)
        .map(|(p, _)| p)
        .collect();
    assert_eq!(choosers, vec![P1, P2]);
    assert_eq!((t.hand_size(P1), t.hand_size(P2)), (1, 1));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn the_true_scriptures_opponents_discard_together_then_mill() {
    cr!("101.4", "701.9a", "714.2b");
    ruling!(
        "Sheoldred // The True Scriptures",
        "To resolve chapter II of The True Scriptures in a multiplayer game, the next opponent in turn order (or, if it’s an opponent’s turn, the opponent whose turn it is) chooses three cards in hand. Any opponent who has fewer than three cards in hand chooses all cards they have in hand. They set the chosen cards aside without revealing them. Then each other opponent in turn order does the same. Next, all chosen cards are revealed and discarded at the same time. Then, each opponent mills three cards."
    );
    supported("Sheoldred // The True Scriptures");
    let mut t = TestGame::new(3);
    for name in ["Grizzly Bears", "Forest", "Island", "Shock"] {
        t.hand(P1, name);
    }
    t.hand(P2, "Grizzly Bears");
    t.hand(P2, "Forest");
    let is_choice = |d: &Decision| matches!(d, Decision::ChooseEntities { .. });
    let seen1 = watch(&mut t, P1, is_choice, hands_and_libraries);
    let saga = crate::r_s17_common::enter_transformed(&mut t, P0, "Sheoldred // The True Scriptures");
    t.resolve_all();
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    // P1 chose with nothing milled yet.
    let seen1 = seen1.lock().unwrap().clone();
    assert_eq!(seen1.len(), 1);
    assert_eq!(seen1[0].1, vec![30, 30, 30]);
    assert_eq!(t.hand_size(P1), 1);
    // P2 had fewer than three: both discarded.
    assert_eq!(t.hand_size(P2), 0);
    assert_eq!((t.library_size(P1), t.library_size(P2)), (27, 27));
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn tale_of_tamiyo_keeps_milling_until_no_shared_type_or_no_cards() {
    cr!("714.2b", "701.17b", "205.2a");
    ruling!(
        "The Tale of Tamiyo",
        "When the first, second, or third chapter ability of The Tale of Tamiyo resolves, the ability will continue until you mill a set of two cards that share no card types or you mill one or fewer cards. You can't just pause the story whenever you want to. (And why would you? It's quite the story.)"
    );
    // Every chapter compiles.
    assert!(card("The Tale of Tamiyo").unsupported_text().is_empty());
    // Two lands (share a type): draw, repeat; Bears and Shock (no shared type): stop.
    let mut t = TestGame::new(2);
    stack_library(
        &mut t,
        P0,
        &["Forest", "Island", "Swamp", "Grizzly Bears", "Shock", "Mountain"],
    );
    enter(&mut t, P0, "The Tale of Tamiyo");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 4);
    assert!(t.in_hand(P0, "Swamp"));
    assert_eq!(t.hand_size(P0), 1);
    // Running out of cards stops it.
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Forest", "Island", "Swamp", "Mountain"]);
    enter(&mut t, P0, "The Tale of Tamiyo");
    t.resolve_all();
    // Forest+Island: draw Swamp; then only the Mountain is milled.
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.library_size(P0), 0);
    assert!(t.in_hand(P0, "Swamp"));
}
