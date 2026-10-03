//! "If [condition], [effect] instead." (pattern in
//! `src/oracle/patterns/damage_removal_instead.rs`).

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn instead_cards_compile() {
    assert_compiles(&[
        "Stomped by the Foot",
        "Roil Eruption",
        "Might of Murasa",
        "Hypnotic Cloud",
    ]);
}

#[test]
fn roil_eruption_deals_more_damage_to_the_same_target_when_kicked() {
    cr!("608.2c", "702.33d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 9);
    let r = t.hand(P0, "Roil Eruption");
    t.cast(P0, r).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    let r2 = t.hand(P0, "Roil Eruption");
    t.cast(P0, r2).target(P1).kicked(true).go();
    t.resolve();
    assert_eq!(t.life(P1), 12);
}

#[test]
fn stomped_by_the_foot_gives_minus_five_when_kicked() {
    cr!("608.2c", "702.33d");
    let mut t = TestGame::new(2);
    let mastodon = t.battlefield(P1, "Siege Mastodon");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    let s = t.hand(P0, "Stomped by the Foot");
    t.cast(P0, s).target(mastodon).go();
    t.resolve();
    assert_eq!(t.pt(mastodon), (1, 3), "-2/-2 on a 3/5");
    let s2 = t.hand(P0, "Stomped by the Foot");
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.cast(P0, s2).target(mastodon).kicked(true).go();
    t.resolve();
    assert!(!t.on_battlefield(mastodon), "-5/-5 more");
}

#[test]
fn leafkin_druid_is_still_a_mana_ability_with_an_instead_clause() {
    cr!("605.1a", "106.4");
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Leafkin Druid");
    let r = t.activate(P0, druid, 0, &[]).unwrap();
    assert!(r.is_none(), "mana abilities don't use the stack");
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::G), 1);
    // With four creatures: {G}{G} instead.
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Leafkin Druid");
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    t.activate(P0, druid, 0, &[]).unwrap();
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::G), 2);
    assert_eq!(t.stack_len(), 0);
}
