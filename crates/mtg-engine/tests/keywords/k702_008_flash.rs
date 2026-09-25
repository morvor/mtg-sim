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

#[test]
fn quick_sliver_has_flash_and_lets_any_player_cast_slivers_as_though_they_had_flash() {
    cr!("702.8a", "601.3b");
    ruling!(
        "Quick Sliver",
        "The first ability applies when this card is not on the battlefield. The second ability applies when this card is on the battlefield."
    );
    assert_supported("Quick Sliver");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let quick = t.hand(P0, "Quick Sliver");
    let metallic = t.hand(P0, "Metallic Sliver");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    // Metallic Sliver has no flash and Quick Sliver isn't on the battlefield yet.
    assert!(t.cast(P0, metallic).try_go().is_err());
    // Quick Sliver's own flash works from the hand.
    t.cast(P0, quick).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Quick Sliver").len(), 1);
    // Now Sliver spells can be cast as though they had flash, by any player.
    t.g.turn.priority = Some(P0);
    t.cast(P0, metallic).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Metallic Sliver").len(), 1);
    t.lands(P1, "Forest", 1);
    let theirs = t.hand(P1, "Metallic Sliver");
    t.set_step(P0, Step::End);
    t.g.turn.priority = Some(P1);
    t.cast(P1, theirs).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Metallic Sliver").len(), 2);
}

#[test]
fn casting_as_though_it_had_flash_doesnt_change_sorcery_speed_abilities() {
    cr!("601.3b", "702.6a");
    ruling!(
        "Vedalken Orrery",
        "This applies only to casting spells. It does not, for example, change when you may activate abilities that can only be activated \"any time you could cast a sorcery\"."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vedalken Orrery");
    let sword = t.battlefield(P0, "Short Sword");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    let giant = t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    t.cast(P0, giant).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    t.g.turn.priority = Some(P0);
    assert!(t.activate(P0, sword, 0, &[Entity::Object(bears)]).is_err());
}

#[test]
fn a_sorcery_may_be_cast_as_though_it_had_flash_for_an_additional_cost() {
    cr!("601.3c");
    assert_supported("Rout");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    let rout = t.hand(P0, "Rout");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    // Normally it can't be cast at instant speed.
    assert!(t.cast(P0, rout).try_go().is_err());
    let uid = t.obj(rout).chars.abilities[0].uid;
    let method = mtg_engine::object::CastMethod::Alternative(uid);
    // {3}{W}{W} isn't enough: it costs {2} more.
    assert!(t.cast(P0, rout).method(method.clone()).try_go().is_err());
    t.lands(P0, "Plains", 2);
    t.g.turn.priority = Some(P0);
    t.cast(P0, rout).method(method).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
}
