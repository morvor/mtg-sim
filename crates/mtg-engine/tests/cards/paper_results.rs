//! Results of earlier instructions in older wordings (cards4-paper-effects, family
//! `results`): "If you sacrifice an Island this way", "If ~ deals damage to you this way",
//! "If ~ is destroyed this way", "If those permanents are exchanged this way", "If you
//! don't sacrifice a Forest".

use crate::combat2_common::supported;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn serendib_djinn_hurts_only_when_an_island_is_sacrificed() {
    cr!("701.21a", "608.2c");
    ruling!(
        "Serendib Djinn",
        "The sacrifice of the land is done during the resolution of the triggered ability. You also choose the land at that time."
    );
    supported("Serendib Djinn");
    for (land, dmg) in [("Island", 3), ("Mountain", 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Serendib Djinn");
        let l = t.battlefield(P0, land);
        t.battlefield(P0, if land == "Island" { "Mountain" } else { "Island" });
        t.answer_choose(P0, &[Entity::Object(l)]);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert!(t.in_graveyard(P0, land), "{land}");
        assert_eq!(t.life(P0), 20 - dmg, "{land}");
    }
}

#[test]
fn gargantuan_gorilla_snow_forest_trample_or_sacrifice_and_damage() {
    cr!("701.21a", "608.2c");
    supported("Gargantuan Gorilla");
    // A snow Forest: trample, the Gorilla stays.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gargantuan Gorilla");
    let snow = t.battlefield(P0, "Snow-Covered Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(snow)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(g));
    assert!(t.obj_now(g).chars.has_keyword(KeywordKind::Trample));
    assert_eq!(t.life(P0), 20);
    // A plain Forest: no trample, but the Gorilla stays and deals no damage.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gargantuan Gorilla");
    let forest = t.battlefield(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.on_battlefield(g));
    assert!(!t.obj_now(g).chars.has_keyword(KeywordKind::Trample));
    assert_eq!(t.life(P0), 20);
    // No Forest sacrificed: the Gorilla is sacrificed and deals 7 damage to you.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gargantuan Gorilla");
    t.battlefield(P0, "Forest");
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(g));
    assert!(t.in_graveyard(P0, "Gargantuan Gorilla"));
    assert!(!t.in_graveyard(P0, "Forest"));
    assert_eq!(t.life(P0), 13);
}

#[test]
fn minion_of_leshrac_taps_only_if_it_dealt_damage() {
    cr!("120.4b", "608.2c");
    ruling!(
        "Minion of Leshrac",
        "but you somehow prevent all the damage Minion of Leshrac would deal to you, then you don’t tap Minion of Leshrac."
    );
    supported("Minion of Leshrac");
    // Not sacrificing: 5 damage, then it taps.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Minion of Leshrac");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 15);
    assert!(t.obj_now(m).tapped);
    // Sacrificing another creature: no damage, stays untapped.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Minion of Leshrac");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.obj_now(m).tapped);
}

#[test]
fn cosmic_horror_deals_damage_only_if_actually_destroyed() {
    cr!("701.8a", "702.12b", "608.2c");
    ruling!(
        "Cosmic Horror",
        "it deals 7 damage to you only if it’s actually destroyed. If it regenerates or an effect has given it indestructible, it deals no damage."
    );
    supported("Cosmic Horror");
    // Not paying: destroyed, 7 damage.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Cosmic Horror");
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(h));
    assert!(t.in_graveyard(P0, "Cosmic Horror"));
    assert_eq!(t.life(P0), 13);
    // Indestructible (Avacyn, Angel of Hope): not destroyed, no damage.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Cosmic Horror");
    t.battlefield(P0, "Avacyn, Angel of Hope");
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(h));
    assert_eq!(t.life(P0), 20);
    // Paying: nothing happens.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Cosmic Horror");
    t.lands(P0, "Swamp", 6);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(h));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn gauntlets_of_chaos_destroys_auras_only_after_an_exchange() {
    cr!("701.12a", "701.12b");
    ruling!(
        "Gauntlets of Chaos",
        "Since the exchange doesn’t happen, no Auras are destroyed."
    );
    supported("Gauntlets of Chaos");
    for legal in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let gl = t.battlefield(P0, "Gauntlets of Chaos");
        t.lands(P0, "Plains", 5);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Hill Giant");
        let aura = t.battlefield(P0, "Holy Strength");
        t.g.attach(aura, Entity::Object(mine));
        t.activate(
            P0,
            gl,
            0,
            &[Entity::Object(mine), Entity::Object(theirs)],
        )
        .unwrap();
        if !legal {
            // The opponent's creature leaves before the ability resolves.
            t.g.exile_object(theirs, None);
        }
        t.resolve();
        if legal {
            assert_eq!(t.obj_now(mine).controller, P1);
            assert_eq!(t.obj_now(theirs).controller, P0);
            assert!(t.in_graveyard(P0, "Holy Strength"));
        } else {
            assert_eq!(t.obj_now(mine).controller, P0);
            assert!(!t.in_graveyard(P0, "Holy Strength"));
        }
    }
}
