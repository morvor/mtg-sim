//! CR 702.8 Flash.

use super::k702_001_010_common::*;
use mtg_engine::ability::{Duration, Effect, Modification, Sel};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn flash_creature_can_be_cast_any_time_you_could_cast_an_instant() {
    cr!("702.8a");
    assert_eq!(printed_keywords("Ambush Viper", KeywordKind::Flash).len(), 1);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let viper = t.hand(P0, "Ambush Viper");
    let bears = t.hand(P0, "Grizzly Bears");
    // The opponent's end step, with P0 holding priority.
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, bears).try_go().is_err(), "no flash: sorcery timing");
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let spell = t.cast(P0, viper).go();
    assert_eq!(t.stack_len(), 1);
    // And in response to another spell.
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, bears).try_go().is_err());
    t.resolve();
    assert!(!t.g.is_live(spell));
    assert_eq!(t.named_on_battlefield("Ambush Viper").len(), 1);
}

#[test]
fn flash_creature_ambushes_an_attacker() {
    cr!("702.8a", "702.2b");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Forest", 2);
    let viper = t.hand(P0, "Ambush Viper");
    t.set_step(P1, Step::BeginningOfCombat);
    declare(&mut t, &[(wurm, Entity::Player(P0))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.g.turn.priority = Some(P0);
    t.cast(P0, viper).go();
    t.resolve();
    let viper = t.named_on_battlefield("Ambush Viper")[0];
    block(&mut t, P0, &[(viper, wurm)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_flash_creature_is_not_on_the_battlefield_while_being_cast() {
    cr!("702.8a");
    ruling!(
        "Brineborn Cutthroat",
        "Notably, the ability won't trigger as you cast Brineborn Cutthroat during an opponent's turn."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let cutthroat = t.hand(P0, "Brineborn Cutthroat");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    t.cast(P0, cutthroat).go();
    t.resolve_all();
    let c = t.named_on_battlefield("Brineborn Cutthroat")[0];
    assert_eq!(t.counters(c, "+1/+1"), 0);
    // Another spell cast during the opponent's turn does trigger it.
    let jump = t.hand(P0, "Jump");
    t.g.turn.priority = Some(P0);
    t.cast(P0, jump).target(c).go();
    t.resolve_all();
    assert_eq!(t.counters(c, "+1/+1"), 1);
}

#[test]
fn flash_works_from_any_zone_the_card_could_be_cast_from() {
    cr!("702.8a");
    ruling!(
        "Vizier of the Menagerie",
        "If that creature card has flash, you'll be able to cast it any time you could cast an instant, even on an opponent's turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of the Menagerie");
    t.lands(P0, "Forest", 2);
    // Ambush Viper on top of the library.
    let viper = t.library_top(P0, "Ambush Viper");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    t.cast(P0, viper).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Ambush Viper").len(), 1);
}

#[test]
fn a_creature_without_flash_on_top_of_the_library_needs_sorcery_timing() {
    cr!("702.8a");
    ruling!(
        "Vizier of the Menagerie",
        "Normally, Vizier of the Menagerie allows you to cast the top card of your library if it's a creature card, it's your main phase, and the stack is empty."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of the Menagerie");
    t.lands(P0, "Forest", 2);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, bears).try_go().is_err());
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, bears).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn conditional_flash_functions_in_the_hand() {
    cr!("702.8a", "601.3d");
    assert_supported("Colossal Rattlewurm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let wurm = t.hand(P0, "Colossal Rattlewurm");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    assert!(!t.obj(wurm).has_keyword(KeywordKind::Flash));
    assert!(t.cast(P0, wurm).try_go().is_err());
    t.battlefield(P0, "Desert");
    t.g.recompute();
    assert!(t.obj(wurm).has_keyword(KeywordKind::Flash));
    t.g.turn.priority = Some(P0);
    t.cast(P0, wurm).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Colossal Rattlewurm").len(), 1);
}

#[test]
fn multiple_instances_of_flash_are_redundant() {
    cr!("702.8b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let viper = t.hand(P0, "Ambush Viper");
    // An effect gives the card in hand flash again.
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Flash))],
            duration: Duration::EndOfTurn,
        },
        &[viper],
    );
    assert_eq!(instances(&t, viper, KeywordKind::Flash), 2);
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    t.cast(P0, viper).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Ambush Viper").len(), 1);
    assert_eq!(t.stack_len(), 0);
}
