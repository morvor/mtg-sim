//! Object references in effects: "tap enchanted permanent" / "destroy enchanted land" (the
//! object an Aura is attached to, CR 303.4), "untap that artifact" (the target named
//! earlier), and "target artifact creature card" (both card types).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn stuck_in_summoners_sanctum_taps_enchanted_artifact() {
    cr!("303.4a", "303.4b");
    assert_supported("Stuck in Summoner's Sanctum");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let sable = t.battlefield(P1, "Bronze Sable");
    let aura = t.hand(P0, "Stuck in Summoner's Sanctum");
    t.cast(P0, aura).target(sable).go();
    t.resolve_all();
    assert!(t.obj_now(sable).tapped, "{}", t.dump_log());
}

#[test]
fn silken_strength_untaps_enchanted_creature() {
    cr!("303.4b");
    assert_supported("Silken Strength");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    let aura = t.hand(P0, "Silken Strength");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped, "{}", t.dump_log());
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn pooling_venom_destroys_enchanted_land() {
    cr!("303.4b");
    assert_supported("Pooling Venom");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let forest = t.battlefield(P1, "Forest");
    let aura = t.hand(P0, "Pooling Venom");
    t.cast(P0, aura).target(forest).go();
    t.resolve_all();
    let venom = t.named_on_battlefield("Pooling Venom")[0];
    t.activate(P0, venom, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Forest"), "{}", t.dump_log());
}

#[test]
fn metallic_mastery_untaps_that_artifact() {
    cr!("608.2c");
    assert_supported("Metallic Mastery");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let sable = t.battlefield(P1, "Bronze Sable");
    t.g.objects[sable.0 as usize].tapped = true;
    let mm = t.hand(P0, "Metallic Mastery");
    t.cast(P0, mm).target(sable).go();
    t.resolve();
    let s = t.obj_now(sable);
    assert_eq!(s.controller, P0);
    assert!(!s.tapped);
    assert!(s.has_keyword(KeywordKind::Haste));
}

#[test]
fn scarecrone_returns_only_artifact_creature_cards() {
    cr!("115.1", "205.2a");
    assert_supported("Scarecrone");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let crone = t.battlefield(P0, "Scarecrone");
    let sable = t.graveyard(P0, "Bronze Sable");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Sol Ring");
    t.activate(P0, crone, 1, &[Entity::Object(sable)]).unwrap();
    // Neither a creature card that isn't an artifact nor a noncreature artifact card was a
    // legal target.
    let offered = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap();
    assert_eq!(offered, vec![Entity::Object(sable)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bronze Sable").len(), 1, "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Sol Ring"));
}
