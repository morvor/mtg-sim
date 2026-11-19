//! Rulings batch P095 — replacement effects and statics of the "life payment" cards: Zur's
//! Weirding and Underrealm Lich replacing draws (CR 614.1a, 614.6), with dredge (CR 616.1),
//! Mornsong Aria's "can't" (CR 101.2), Biotransference and replacement effects that modify
//! how artifacts enter (CR 614.12), Infernal Darkness changing the type of mana (CR
//! 106.12b), Purgatory's trigger (CR 603.6c), Aether Storm (CR 601.2), and Transcendence
//! with an effect that says you can't lose (CR 104.4b).

use crate::r_p095_common::*;
use crate::r_s01_common::supported;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameResult;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Moves every card of `p`'s library to exile.
fn empty_library(t: &mut TestGame, p: PlayerId) {
    for id in t.g.player(p).library.clone() {
        t.g.move_object(id, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
    t.g.flush_events();
}

// ---------------------------------------------------------------------------------------
// Zur's Weirding
// ---------------------------------------------------------------------------------------

#[test]
fn zurs_weirding_is_a_replacement_effect_not_a_trigger() {
    cr!("614.1a", "614.6");
    ruling!(
        "Zur's Weirding",
        "This is a continuous replacement effect that modifies a draw."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zur's Weirding");
    t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.g.draw_cards(P0, 1);
    // Done as the draw would have happened: nothing uses the stack.
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.g.history.cards_drawn.get(&P0).copied().unwrap_or(0), 0);
}

#[test]
fn zurs_weirding_and_dredge_in_either_order() {
    cr!("616.1", "614.5", "702.52a");
    ruling!(
        "Zur's Weirding",
        "When using Dredge with this, you have two choices. You can immediately replace the draw with Dredge, or you can apply Zur’s Weirding’s replacement. If the latter, you first resolve Zur’s Weirding’s ability, then if you’re going to draw the card, decide whether or not to replace that draw with Dredge."
    );
    let replacement_options = |t: &TestGame, from: usize| -> Vec<Vec<String>> {
        t.asked()[from..]
            .iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseReplacement { options } if *p == P0 => Some(options.clone()),
                _ => None,
            })
            .collect()
    };
    let setup = || {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Zur's Weirding");
        t.graveyard(P0, "Golgari Thug");
        t
    };
    // The two replacement effects P0 chooses between (seen in a probe game).
    let mut probe = setup();
    probe.g.draw_cards(P0, 1);
    let opts = replacement_options(&probe, 0);
    assert_eq!(opts.len(), 1, "{opts:?}");
    let dredge = opts[0].iter().position(|o| o.contains("Dredge")).unwrap();
    let zur = opts[0].iter().position(|o| o.contains("Zur")).unwrap();
    // Dredge first: nobody is offered to pay life.
    let mut t = setup();
    let from = t.asked().len();
    let library = t.library_size(P0);
    t.answer(P0, DecisionKind::Replacement, Answer::Index(dredge));
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Golgari Thug"));
    assert_eq!(t.library_size(P0), library - 4);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })));
    // Zur's Weirding first: P1 declines to pay, then P0 may still dredge the draw.
    let mut t = setup();
    let from = t.asked().len();
    let library = t.library_size(P0);
    t.answer(P0, DecisionKind::Replacement, Answer::Index(zur));
    t.answer_yes(P1, false);
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })));
    assert!(t.in_hand(P0, "Golgari Thug"));
    assert_eq!(t.library_size(P0), library - 4);
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------------------
// Underrealm Lich
// ---------------------------------------------------------------------------------------

#[test]
fn underrealm_lich_with_few_or_no_cards_in_the_library() {
    cr!("614.6", "704.5b", "121.4");
    ruling!(
        "Underrealm Lich",
        "Underrealm Lich's first ability applies even if you have fewer than three cards in your library. If you have no cards in your library, you can't lose the game for trying to draw a card from an empty library."
    );
    supported("Underrealm Lich");
    // Two cards: look at both, one to the hand, the other to the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underrealm Lich");
    empty_library(&mut t, P0);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    let graveyard = t.graveyard_size(P0);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.graveyard_size(P0), graveyard + 1);
    // No cards: the draw is replaced, so P0 didn't try to draw from an empty library.
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(!t.has_lost(P0));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn underrealm_lich_can_gain_indestructible_while_tapped() {
    cr!("602.5", "701.26a", "702.12b");
    ruling!(
        "Underrealm Lich",
        "You can activate Underrealm Lich's last ability even if it's already tapped. It can't be tapped again, but it still gains indestructible."
    );
    let mut t = TestGame::new(2);
    let lich = t.battlefield(P0, "Underrealm Lich");
    t.g.tap(lich);
    t.activate(P0, lich, 0, &[]).unwrap();
    assert_eq!(t.life(P0), 16);
    t.resolve_all();
    assert!(t.obj_now(lich).tapped);
    assert!(t.obj_now(lich).has_keyword(KeywordKind::Indestructible));
}

// ---------------------------------------------------------------------------------------
// Mornsong Aria
// ---------------------------------------------------------------------------------------

#[test]
fn mornsong_aria_the_rest_of_a_draw_or_gain_spell_still_happens() {
    cr!("101.2", "608.2c");
    ruling!(
        "Mornsong Aria",
        "Spells and abilities that cause players to draw cards or gain life still resolve while Mornsong Aria is on the battlefield. No player will draw cards or gain life, but any other effects of that spell or ability will still happen."
    );
    supported("Mornsong Aria");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mornsong Aria");
    // Sign in Blood: "Target player draws two cards and loses 2 life."
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Sign in Blood", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 18);
    // Sephiroth's Intervention: "Destroy target creature. You gain 2 life."
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Sephiroth's Intervention", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 18);
}

// ---------------------------------------------------------------------------------------
// Biotransference
// ---------------------------------------------------------------------------------------

#[test]
fn biotransference_creatures_entering_get_artifact_entering_replacements() {
    cr!("614.12", "613.1d");
    ruling!(
        "Biotransference",
        "Replacement effects that modify artifacts as they enter the battlefield will apply after you apply Biotransference's effect. For example, if you control a Biotransference and there is an effect that says artifacts enter the battlefield tapped, creatures you control will enter the battlefield tapped."
    );
    supported("Biotransference");
    supported("Root Maze");
    // Root Maze: "Artifacts and lands enter tapped." Grizzly Bears cast and resolving, and
    // a creature card put onto the battlefield by an effect.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Biotransference");
    t.battlefield(P1, "Root Maze");
    // (Bears is an artifact spell: Biotransference's last ability triggers too.)
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(t.obj_now(bears).tapped);
    let giant = t.enter(P0, "Hill Giant");
    assert!(t.obj_now(giant).tapped);
    // An opponent's creature isn't an artifact: it enters untapped.
    let goblin = t.enter(P1, "Raging Goblin");
    assert!(!t.obj_now(goblin).tapped);
}

// ---------------------------------------------------------------------------------------
// Infernal Darkness
// ---------------------------------------------------------------------------------------

#[test]
fn infernal_darkness_changes_the_type_of_mana_not_the_amount() {
    cr!("106.12b", "106.3");
    ruling!(
        "Infernal Darkness",
        "This effect changes the type of mana produced, but not the amount. For example, if a land that's tapped for mana would add {W}{W}, it adds {B}{B} instead."
    );
    // Mana Reflection doubles the Plains's mana: {W}{W} becomes {B}{B}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Infernal Darkness");
    t.battlefield(P0, "Mana Reflection");
    let plains = t.battlefield(P0, "Plains");
    t.activate(P0, plains, 0, &[]).unwrap();
    let pool: Vec<ManaType> = t.g.player(P0).mana_pool.mana.iter().map(|m| m.ty).collect();
    assert_eq!(pool, vec![ManaType::B, ManaType::B]);
}

// ---------------------------------------------------------------------------------------
// Purgatory
// ---------------------------------------------------------------------------------------

#[test]
fn purgatory_creature_goes_to_the_graveyard_before_it_is_exiled() {
    cr!("603.6c", "700.4");
    ruling!(
        "Purgatory",
        "This has been restored to a triggered ability that triggers when a nontoken creature is put into a graveyard from the battlefield, rather than a replacement effect. The creature does go to the graveyard, and will trigger any other abilities that look for this event."
    );
    supported("Purgatory");
    // Zulaport Cutthroat: "Whenever this creature or another creature you control dies,
    // each opponent loses 1 life and you gain 1 life."
    let mut t = TestGame::new(2);
    let purgatory = t.battlefield(P0, "Purgatory");
    let cutthroat = t.battlefield(P0, "Zulaport Cutthroat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(abilities_from(&t, purgatory), 1);
    assert_eq!(abilities_from(&t, cutthroat), 1);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

// ---------------------------------------------------------------------------------------
// Aether Storm
// ---------------------------------------------------------------------------------------

#[test]
fn aether_storm_doesnt_stop_creatures_being_put_onto_the_battlefield() {
    cr!("101.2", "601.2");
    ruling!(
        "Aether Storm",
        "This does not stop a creature card from being put directly onto the battlefield by a spell or ability."
    );
    supported("Aether Storm");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Aether Storm");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // Zombify: "Return target creature card from your graveyard to the battlefield."
    let giant = t.graveyard(P0, "Hill Giant");
    cast_new(&mut t, P0, "Zombify", &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(!t.named_on_battlefield("Hill Giant").is_empty());
}

// ---------------------------------------------------------------------------------------
// Transcendence
// ---------------------------------------------------------------------------------------

#[test]
fn transcendence_with_platinum_angel_is_a_loop_that_draws_the_game() {
    cr!("104.4b", "603.8", "104.3b");
    ruling!(
        "Transcendence",
        "If your life total is 20 or more and you control an effect that states “you can’t lose the game”, then an infinite loop is formed. If neither player wants to end the loop by removing one of these two cards from the battlefield, the game ends in a draw."
    );
    supported("Transcendence");
    // P0 at 20 life with Transcendence and Platinum Angel ("You can't lose the game and
    // your opponents can't win the game."); nobody can do anything else.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Transcendence");
    t.battlefield(P0, "Platinum Angel");
    let ok = t.g.run_until(5_000, |g| g.result.is_some());
    assert!(ok, "the loop didn't end the game");
    assert_eq!(t.g.result, Some(GameResult::Draw));
}
