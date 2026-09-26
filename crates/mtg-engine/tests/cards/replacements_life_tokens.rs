//! "Damage that would reduce your life total to less than 1 reduces it to 1 instead"
//! (`src/oracle/patterns/replacements_life.rs`) and "instead create those tokens plus an
//! additional Treasure token" (`src/oracle/patterns/replacements_tokens.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn life_floor_and_extra_token_cards_compile() {
    assert_compiles(&[
        "Ali from Cairo",
        "Fortune Thief",
        "Sustaining Spirit",
        "Worship",
        "Angel's Grace",
        "Angel of Grace",
        "Serra the Benevolent",
        "Xorn",
    ]);
}

fn set_life(t: &mut TestGame, p: PlayerId, life: i32) {
    t.g.players[p.idx()].life = life;
}

#[test]
fn ali_from_cairo_changes_the_result_of_damage_not_the_damage() {
    cr!("614.1a", "120.3a");
    ruling!(
        "Ali from Cairo",
        "This effect does not prevent damage, it prevents the damage from turning into loss of life."
    );
    ruling!(
        "Ali from Cairo",
        "This effect does not apply to effects which reduce your life without doing damage."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ali from Cairo");
    set_life(&mut t, P0, 5);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 2);
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 1);
    // The full damage was dealt.
    assert_eq!(t.g.history.damage_dealt_to_players.get(&P0), Some(&6));
    t.settle();
    assert!(!t.has_lost(P0));
    // Life loss that isn't from damage isn't affected.
    t.g.lose_life(P0, 1);
    assert_eq!(t.life(P0), 0);
}

#[test]
fn fortune_thief_lets_lifelink_see_the_full_damage_and_stops_unpreventable_damage() {
    cr!("614.1a", "120.3a", "702.15b");
    ruling!(
        "Fortune Thief",
        "Fortune Thief doesn't change how much damage is dealt; it just changes how much life that damage makes you lose. Abilities such as lifelink will see the full amount of damage being dealt."
    );
    ruling!(
        "Fortune Thief",
        "Fortune Thief's effect is not a prevention effect. It stops unpreventable damage from reducing your life total below 1."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fortune Thief");
    set_life(&mut t, P0, 2);
    let vamp = t.battlefield(P1, "Vampire Nighthawk");
    t.g.deal_damage(vamp, Entity::Player(P0), 2, true);
    assert_eq!(t.life(P0), 1);
    assert_eq!(t.life(P1), 22);
    // Skullcrack: the damage can't be prevented, but the life total still stops at 1.
    t.lands(P1, "Mountain", 2);
    let crack = t.hand(P1, "Skullcrack");
    t.cast(P1, crack).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 1);
    assert!(!t.has_lost(P0));
}

#[test]
fn sustaining_spirit_doesnt_help_at_zero_life() {
    cr!("614.1a");
    ruling!(
        "Sustaining Spirit",
        "Does not affect damage if you are already at zero or negative life. You still take it all."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sustaining Spirit");
    set_life(&mut t, P0, 0);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), -3);
}

#[test]
fn worship_needs_a_creature() {
    cr!("614.1a");
    ruling!("Worship", "It reduces your life total to 1, not the damage to 1.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Worship");
    set_life(&mut t, P0, 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(giant, Entity::Player(P0), 10, true);
    assert_eq!(t.life(P0), 1);
    // Without a creature, damage is dealt as normal.
    t.g.destroy(bears, None);
    t.settle();
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), -2);
}

#[test]
fn angels_grace_lasts_until_end_of_turn() {
    cr!("614.1a", "611.2a");
    ruling!(
        "Angel's Grace",
        "If you have less than 1 life, damage dealt to you reduces your life total further below 0 (as normal)."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let grace = t.hand(P0, "Angel's Grace");
    t.cast(P0, grace).go();
    t.resolve();
    set_life(&mut t, P0, 4);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P0), 10, true);
    assert_eq!(t.life(P0), 1);
    // Losing life isn't affected; below 1, damage reduces the total further.
    t.g.lose_life(P0, 3);
    assert_eq!(t.life(P0), -2);
    t.g.deal_damage(giant, Entity::Player(P0), 1, true);
    assert_eq!(t.life(P0), -3);
}

#[test]
fn xorn_adds_a_treasure_and_copies_stack() {
    cr!("614.1a", "614.5");
    ruling!(
        "Xorn",
        "If you control multiple copies of Xorn, you'll create that many additional Treasure tokens."
    );
    let treasures = |t: &TestGame| t.named_on_battlefield("Treasure Token").len();
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Xorn");
    t.lands(P0, "Mountain", 2);
    let s = t.hand(P0, "Strike It Rich");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(treasures(&t), 2);
    t.battlefield(P0, "Xorn");
    let s = t.hand(P0, "Strike It Rich");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(treasures(&t), 5);
    // An opponent's Treasures aren't affected.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let s = t.hand(P1, "Strike It Rich");
    t.cast(P1, s).go();
    t.resolve();
    assert_eq!(treasures(&t), 6);
}
