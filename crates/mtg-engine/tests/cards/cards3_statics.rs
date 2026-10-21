//! Characteristic statics IV (CR 702.11d, 702.16): hexproof from colors for players (Veil
//! of Summer), protection from each color / mana value among objects (Pledge of Loyalty,
//! Rebbec), protection from each mana value but the chosen one (Haktos the Unscarred).

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn veil_of_summer_gives_you_and_your_permanents_hexproof_from_blue_and_black() {
    cr!("702.11d", "702.11f");
    supported("Veil of Summer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let black = t.battlefield(P1, "Hypnotic Specter");
    let blue = t.battlefield(P1, "Wind Drake");
    let red = t.battlefield(P1, "Goblin Piker");
    let veil = t.hand(P0, "Veil of Summer");
    t.lands(P0, "Forest", 1);
    t.set_step(P0, Step::PrecombatMain);
    assert!(!t.g.player_untargetable(P0, P1, Some(black)));
    t.cast(P0, veil).go();
    t.resolve_all();
    t.g.recompute();
    assert!(t.g.player_untargetable(P0, P1, Some(black)));
    assert!(t.g.player_untargetable(P0, P1, Some(blue)));
    assert!(!t.g.player_untargetable(P0, P1, Some(red)));
    // P0's own black or blue sources can still target P0.
    let mine = t.battlefield(P0, "Hypnotic Specter");
    assert!(!t.g.player_untargetable(P0, P0, Some(mine)));
    assert!(t.g.object_untargetable(bears, P1, Some(black)));
    assert!(!t.g.object_untargetable(bears, P1, Some(red)));
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.g.player_untargetable(P0, P1, Some(black)));
}

#[test]
fn rebbec_artifacts_have_protection_from_each_mana_value_among_artifacts() {
    cr!("702.16a", "702.16h");
    supported("Rebbec, Architect of Ascension");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rebbec, Architect of Ascension");
    let ring = t.battlefield(P0, "Sol Ring");
    let lions = t.battlefield(P1, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rebbec_mv4 = t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    // Artifacts: Sol Ring (1).
    assert!(t.g.protected_from(ring, lions));
    assert!(!t.g.protected_from(ring, bears));
    // An artifact with mana value 2 adds that quality.
    t.battlefield(P0, "Mind Stone");
    t.g.recompute();
    assert!(t.g.protected_from(ring, bears));
    assert!(!t.g.protected_from(ring, rebbec_mv4));
    // Rebbec isn't an artifact: no protection.
    let r = t.named_on_battlefield("Rebbec, Architect of Ascension")[0];
    assert!(!t.g.protected_from(r, lions));
}

#[test]
fn pledge_of_loyalty_protection_from_your_colors_without_removing_itself() {
    cr!("702.16c", "702.16h", "702.16n");
    supported("Pledge of Loyalty");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pledge = t.battlefield(P0, "Pledge of Loyalty");
    t.g.attach(pledge, Entity::Object(bears));
    let red = t.battlefield(P1, "Goblin Piker");
    let white = t.battlefield(P1, "Savannah Lions");
    let blue = t.battlefield(P1, "Wind Drake");
    t.settle();
    // P0 controls green (bears) and white (Pledge) permanents.
    assert!(t.g.protected_from(bears, white));
    assert!(!t.g.protected_from(bears, red));
    assert_eq!(t.g.obj(pledge).attached_to, Some(Entity::Object(bears)));
    // A red permanent of P0's adds red.
    t.battlefield(P0, "Goblin Piker");
    t.settle();
    assert!(t.g.protected_from(bears, red));
    assert!(!t.g.protected_from(bears, blue));
    assert!(t.on_battlefield(pledge));
}

#[test]
fn pledge_of_loyalty_on_an_opponents_creature_uses_your_permanents() {
    cr!("702.16h");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Wind Drake");
    let pledge = t.battlefield(P0, "Pledge of Loyalty");
    t.g.attach(pledge, Entity::Object(theirs));
    // P0 controls white (Pledge), blue and green permanents; P1 controls blue and red.
    let blue = t.battlefield(P0, "Wind Drake");
    let green = t.battlefield(P0, "Grizzly Bears");
    let piker = t.battlefield(P1, "Goblin Piker");
    t.settle();
    assert!(t.g.protected_from(theirs, blue));
    assert!(t.g.protected_from(theirs, green));
    assert!(!t.g.protected_from(theirs, piker));
}

#[test]
fn haktos_has_protection_from_every_mana_value_but_the_chosen_one() {
    cr!("702.16a");
    ruling!(
        "Haktos the Unscarred",
        "protection from each possible mana value other than the chosen value"
    );
    supported("Haktos the Unscarred");
    let mut t = TestGame::new(2);
    let haktos = t.enter(P0, "Haktos the Unscarred");
    let mvs: Vec<ObjectId> = ["Savannah Lions", "Grizzly Bears", "Gray Ogre", "Hill Giant", "Craw Wurm"]
        .iter()
        .map(|n| t.battlefield(P1, n))
        .collect();
    t.g.recompute();
    let unprotected: Vec<usize> = (0..mvs.len())
        .filter(|i| !t.g.protected_from(haktos, mvs[*i]))
        .collect();
    assert_eq!(unprotected.len(), 1, "{unprotected:?}");
    assert!((1..=3).contains(&unprotected[0]));
}
