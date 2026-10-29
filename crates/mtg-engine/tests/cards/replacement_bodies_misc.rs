//! Replacement effects with instructions (CR 614.1a, 614.6, 614.12, 614.15): a countered
//! spell exiled "then you may cast it without paying its mana cost", a life total that
//! would be reduced to 0 or less ("instead transform ~ and your life total becomes 3"), and
//! a permanent that enters only if a cost is paid ("If ~ would enter, sacrifice two
//! untapped lands instead. If you do, put ~ onto the battlefield. If you don't, put it into
//! its owner's graveyard.").

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn transcendent_dragon_exiles_the_countered_spell_and_casts_it() {
    cr!("614.15", "701.6a", "608.2g");
    ruling!(
        "Transcendent Dragon",
        "as part of the resolution of Transcendent Dragon’s last ability"
    );
    assert_supported("Transcendent Dragon");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    // P0 flashes in the Dragon in response.
    t.lands(P0, "Island", 6);
    let dragon = t.hand(P0, "Transcendent Dragon");
    t.cast(P0, dragon).go();
    t.resolve();
    // The enters trigger targets the Bears spell; P0 casts it.
    t.answer_targets(P0, &[Entity::Object(spell)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    let mine = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(mine.len(), 1);
    assert_eq!(t.g.obj(mine[0]).controller, P0);
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn enduring_angel_transforms_instead_of_its_controller_losing() {
    cr!("614.1a", "119.5", "701.27a");
    ruling!(
        "Enduring Angel // Angelic Enforcer",
        "It doesn't use the stack and can't be responded to"
    );
    assert_supported("Enduring Angel // Angelic Enforcer");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Enduring Angel // Angelic Enforcer");
    t.g.player_mut(P0).life = 2;
    t.set_step(P1, Step::PrecombatMain);
    // "You have hexproof": damage that doesn't target (Flame Rift deals 4 to each player).
    t.lands(P1, "Mountain", 2);
    let rift = t.hand(P1, "Flame Rift");
    t.cast(P1, rift).go();
    t.resolve_all();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 3);
    assert_eq!(t.named_on_battlefield("Angelic Enforcer"), vec![angel]);
    // Damage that leaves the life total above 0 isn't replaced.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enduring Angel // Angelic Enforcer");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    let rift = t.hand(P1, "Flame Rift");
    t.cast(P1, rift).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.named_on_battlefield("Enduring Angel").len(), 1);
}

#[test]
fn lotus_vale_enters_only_if_two_untapped_lands_are_sacrificed() {
    cr!("614.12", "614.5", "305.1");
    ruling!("Lotus Vale", "Lotus Vale never enters");
    assert_supported("Lotus Vale");
    // Sacrificing two untapped lands.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let vale = t.hand(P0, "Lotus Vale");
    t.answer_yes(P0, true);
    t.play_land(P0, vale).unwrap();
    t.settle();
    assert_eq!(t.named_on_battlefield("Lotus Vale").len(), 1);
    assert_eq!(t.named_on_battlefield("Forest").len(), 0);
    // "Sacrifice two untapped lands instead" isn't an offer: with two untapped lands, they
    // are sacrificed whatever the player would answer (roundtrip-pairs-4; see
    // `tests/cards/roundtrip_pairs_4.rs`).
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let vale = t.hand(P0, "Lotus Vale");
    t.answer_yes(P0, false);
    t.play_land(P0, vale).unwrap();
    t.settle();
    assert_eq!(t.named_on_battlefield("Lotus Vale").len(), 1);
    assert_eq!(t.named_on_battlefield("Forest").len(), 0);
    // Tapped lands don't count: it can't be paid.
    let mut t = TestGame::new(2);
    let forests = t.lands(P0, "Forest", 2);
    for f in forests {
        t.g.tap(f);
    }
    let vale = t.hand(P0, "Lotus Vale");
    t.answer_yes(P0, true);
    t.play_land(P0, vale).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Lotus Vale"));
}
