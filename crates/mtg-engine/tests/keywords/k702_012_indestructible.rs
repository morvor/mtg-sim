//! CR 702.12 Indestructible.

use crate::common_k702_011_017::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn indestructible_permanents_arent_destroyed_by_destroy_effects() {
    cr!("702.12", "702.12a", "702.12b");
    assert_supported("Darksteel Myr");
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P0, "Darksteel Myr");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Targeted destruction.
    t.lands(P1, "Swamp", 3);
    let murder = t.hand(P1, "Murder");
    t.cast(P1, murder).target(myr).go();
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    // "Destroy all creatures."
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn indestructible_ignores_lethal_damage_and_deathtouch() {
    cr!("702.12b", "704.5g", "704.5h");
    let mut t = TestGame::new(2);
    // Darksteel Colossus: 11/11 trample, indestructible.
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    let myr = t.battlefield(P0, "Darksteel Myr");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(myr).go();
    t.resolve_all();
    // The damage stays marked, but the SBA for lethal damage doesn't destroy it.
    assert!(t.on_battlefield(myr));
    assert_eq!(t.obj_now(myr).damage, 3);
    // Deathtouch damage doesn't destroy it either.
    let rats = t.battlefield(P1, "Typhoid Rats");
    t.set_step(P0, turn::Step::BeginningOfCombat);
    t.attack(&[(colossus, Entity::Player(P1))], &[(rats, colossus)]);
    assert!(t.on_battlefield(colossus));
    assert_eq!(t.obj_now(colossus).damage, 1);
    assert!(!t.on_battlefield(rats));
}

#[test]
fn indestructible_doesnt_stop_zero_toughness_or_sacrifice() {
    cr!("702.12b", "704.5f");
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P0, "Darksteel Myr");
    // Toughness 0 or less isn't destruction.
    t.lands(P1, "Swamp", 1);
    let slip = t.hand(P1, "Tragic Slip");
    t.cast(P1, slip).target(myr).go();
    t.resolve_all();
    assert!(!t.on_battlefield(myr));
    assert!(t.in_graveyard(P0, "Darksteel Myr"));
    // Sacrificing isn't destruction.
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    t.lands(P1, "Swamp", 2);
    let edict = t.hand(P1, "Diabolic Edict");
    t.cast(P1, edict).target(P0).go();
    t.resolve_all();
    assert!(!t.on_battlefield(colossus));
}

#[test]
fn indestructible_planeswalker_still_dies_with_no_loyalty() {
    cr!("702.12b", "704.5i");
    ruling!(
        "Heroic Intervention",
        "A planeswalker with indestructible still loses loyalty counters as it's dealt damage"
    );
    assert_supported("Heroic Intervention");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let hi = t.hand(P0, "Heroic Intervention");
    t.cast(P0, hi).go();
    t.resolve_all();
    assert!(t.obj_now(jace).has_keyword(KeywordKind::Indestructible));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    // Jace Beleren has 3 loyalty; damage (here from a creature an opponent controls, as
    // hexproof stops targeting) removes loyalty counters, and with none left it's put
    // into the graveyard even though it's indestructible.
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Object(jace), 2, false);
    t.settle();
    assert!(t.on_battlefield(jace));
    assert_eq!(t.obj_now(jace).loyalty(), 1);
    t.g.deal_damage(giant, Entity::Object(jace), 1, false);
    t.settle();
    assert!(!t.on_battlefield(jace));
    // The creature with lethal damage survives.
    t.g.deal_damage(giant, Entity::Object(bears), 5, false);
    t.settle();
    assert!(t.on_battlefield(bears));
}

#[test]
fn losing_indestructible_lets_marked_damage_destroy_it() {
    cr!("702.12b", "704.5g");
    ruling!(
        "Shadowspear",
        "damage previously dealt to a creature with indestructible may cause it to be destroyed if Shadowspear's second ability resolves during that turn"
    );
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P0, "Darksteel Myr");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(myr).go();
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    let spear = t.battlefield(P1, "Shadowspear");
    t.lands(P1, "Wastes", 1);
    t.activate(P1, spear, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(myr));
}

#[test]
fn multiple_instances_of_indestructible_are_redundant() {
    cr!("702.12c");
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P0, "Darksteel Myr");
    t.lands(P0, "Forest", 2);
    let hi = t.hand(P0, "Heroic Intervention");
    t.cast(P0, hi).go();
    t.resolve_all();
    assert_eq!(keyword_count(&t, myr, KeywordKind::Indestructible), 2);
    // Still just indestructible.
    t.lands(P1, "Plains", 4);
    let wrath = t.hand(P1, "Wrath of God");
    t.set_step(P1, turn::Step::PrecombatMain);
    t.cast(P1, wrath).go();
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    // Losing indestructible removes every instance at once.
    let spear = t.battlefield(P1, "Shadowspear");
    t.lands(P1, "Wastes", 1);
    t.activate(P1, spear, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(keyword_count(&t, myr, KeywordKind::Indestructible), 0);
}
