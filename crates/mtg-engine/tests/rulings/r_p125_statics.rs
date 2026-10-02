//! Rulings batch P125 — static abilities that protect permanents (protection, shroud,
//! hexproof, deathtouch, landwalk): whose permanents they affect, and when they apply
//! (CR 611.3, 604.2); characteristic-defining abilities in every zone (CR 604.3); looking
//! at the top card of your library any time (CR 401.5).

use crate::r_p125_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn absolute_grace_and_law_affect_all_creatures() {
    cr!("702.16b", "611.3a");
    ruling!("Absolute Grace", "This affects all creatures, not just your own.");
    ruling!("Absolute Law", "This affects all creatures, not just your own.");
    supported("Absolute Grace");
    supported("Absolute Law");
    for (ench, spell) in [("Absolute Grace", "Doom Blade"), ("Absolute Law", "Shock")] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, ench);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let card = t.hand(P1, spell);
        t.g.recompute();
        assert!(t.g.protected_from(mine, card), "{ench}");
        assert!(t.g.protected_from(theirs, card), "{ench}");
    }
}

#[test]
fn skyshroud_blessing_affects_every_land() {
    cr!("702.18a", "611.2c");
    ruling!("Skyshroud Blessing", "This affects your lands and your opponent’s.");
    supported("Skyshroud Blessing");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Island");
    let theirs = t.battlefield(P1, "Island");
    cast_new(&mut t, P0, "Skyshroud Blessing", &[]);
    t.resolve_all();
    assert!(has(&t, mine, KeywordKind::Shroud));
    assert!(has(&t, theirs, KeywordKind::Shroud));
}

#[test]
fn eladamri_forestwalk_for_elf_creatures_shroud_for_all_elves() {
    cr!("702.18a", "702.14c", "611.3a");
    ruling!(
        "Eladamri, Lord of Leaves",
        "This card only gives forestwalk to other Elf creatures. It gives shroud to all other Elf permanents, including non-creatures like Prowess of the Fair."
    );
    supported("Eladamri, Lord of Leaves");
    let mut t = TestGame::new(2);
    let eladamri = t.battlefield(P0, "Eladamri, Lord of Leaves");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    let prowess = t.battlefield(P0, "Prowess of the Fair");
    t.g.recompute();
    for e in [elves, theirs] {
        assert!(has(&t, e, KeywordKind::Landwalk));
        assert!(has(&t, e, KeywordKind::Shroud));
    }
    assert!(has(&t, prowess, KeywordKind::Shroud));
    assert!(!has(&t, prowess, KeywordKind::Landwalk));
    assert!(!has(&t, eladamri, KeywordKind::Shroud));
    assert!(!has(&t, eladamri, KeywordKind::Landwalk));
}

#[test]
fn crystalline_sliver_works_only_on_the_battlefield() {
    cr!("611.3a", "113.6");
    ruling!("Crystalline Sliver", "The ability only applies while this card is on the battlefield.");
    supported("Crystalline Sliver");
    supported("Metallic Sliver");
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P1, "Metallic Sliver");
    t.hand(P0, "Crystalline Sliver");
    t.graveyard(P0, "Crystalline Sliver");
    t.g.recompute();
    assert!(!has(&t, sliver, KeywordKind::Shroud));
    t.battlefield(P0, "Crystalline Sliver");
    t.g.recompute();
    assert!(has(&t, sliver, KeywordKind::Shroud));
}

#[test]
fn saryth_works_tapped_or_untapped() {
    cr!("611.3a");
    ruling!(
        "Saryth, the Viper's Fang",
        "Saryth's first ability doesn't care whether Saryth is tapped or untapped."
    );
    supported("Saryth, the Viper's Fang");
    for saryth_tapped in [false, true] {
        let mut t = TestGame::new(2);
        let saryth = t.battlefield(P0, "Saryth, the Viper's Fang");
        let tapped = t.battlefield(P0, "Grizzly Bears");
        let untapped = t.battlefield(P0, "Hill Giant");
        t.g.tap(tapped);
        if saryth_tapped {
            t.g.tap(saryth);
        }
        t.g.recompute();
        assert!(has(&t, tapped, KeywordKind::Deathtouch));
        assert!(!has(&t, tapped, KeywordKind::Hexproof));
        assert!(hexproof(&t, untapped));
        assert!(!has(&t, untapped, KeywordKind::Deathtouch));
        assert!(!hexproof(&t, saryth) && !has(&t, saryth, KeywordKind::Deathtouch));
    }
}

#[test]
fn sigarda_lets_you_look_at_the_top_card_any_time() {
    cr!("401.5", "116.1");
    ruling!(
        "Sigarda, Font of Blessings",
        "Sigarda lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority."
    );
    supported("Sigarda, Font of Blessings");
    use mtg_engine::facedown::can_look_at;
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    assert!(!can_look_at(&t.g, P0, top));
    t.battlefield(P0, "Sigarda, Font of Blessings");
    t.g.recompute();
    // During P1's turn while P1 has priority.
    t.set_step(P1, Step::DeclareAttackers);
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn bronze_guardians_power_is_defined_in_every_zone() {
    cr!("604.3", "604.3a");
    ruling!(
        "Bronze Guardian",
        "The ability that defines Bronze Guardian's power applies in all zones, not only while it is on the battlefield."
    );
    supported("Bronze Guardian");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Darksteel Relic");
    t.battlefield(P1, "Ornithopter");
    let in_hand = t.hand(P0, "Bronze Guardian");
    let in_yard = t.graveyard(P0, "Bronze Guardian");
    t.g.recompute();
    assert_eq!(t.obj_now(in_hand).power(), 2);
    assert_eq!(t.obj_now(in_yard).power(), 2);
    let on_bf = t.battlefield(P0, "Bronze Guardian");
    t.g.recompute();
    assert_eq!(t.obj_now(on_bf).power(), 3, "it counts itself on the battlefield");
}
