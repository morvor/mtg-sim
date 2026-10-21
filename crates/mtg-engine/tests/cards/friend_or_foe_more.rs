//! Friend-or-foe follow-ups (`oracle/patterns/friend_or_foe.rs`): any instruction for
//! "each player", performed by (or to) the friends or the foes. In a two-player game P0
//! calls itself a friend and P1 a foe (the default choice).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

fn cast(t: &mut TestGame, name: &str, land: &str) {
    t.lands(P0, land, 5);
    let spell = t.hand(P0, name);
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
}

#[test]
fn zndrsplts_judgment_friends_copy_foes_bounce() {
    cr!("608.2e");
    ruling!(
        "Zndrsplt's Judgment",
        "Friends perform their specified actions before foes"
    );
    compiles("Zndrsplt's Judgment");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    cast(&mut t, "Zndrsplt's Judgment", "Island");
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2, "a copy for P0");
    assert!(!t.on_battlefield(lions) && t.in_hand(P1, "Savannah Lions"));
}

#[test]
fn khorvaths_fury_friends_wheel_foes_take_damage_by_hand_size() {
    cr!("608.2e");
    ruling!(
        "Khorvath's Fury",
        "If a friend has zero cards in hand, that player discards nothing and draws one card"
    );
    compiles("Khorvath's Fury");
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Savannah Lions");
    for _ in 0..4 {
        t.hand(P1, "Forest");
    }
    cast(&mut t, "Khorvath's Fury", "Mountain");
    assert_eq!(t.hand_size(P0), 3, "discarded 2, drew 3");
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 16, "four cards in hand");
    assert_eq!(t.life(P0), 20);
}

#[test]
fn regnas_sanction_friends_get_counters_foes_tap_all_but_one() {
    cr!("608.2e");
    compiles("Regna's Sanction");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Savannah Lions");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(a)]);
    cast(&mut t, "Regna's Sanction", "Plains");
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(a, "+1/+1"), 0);
    assert!(!t.obj_now(a).tapped, "the one chosen");
    assert!(t.obj_now(b).tapped, "the rest");
}

#[test]
fn virtuss_maneuver_friends_regrow_foes_sacrifice() {
    cr!("608.2e");
    compiles("Virtus's Maneuver");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    cast(&mut t, "Virtus's Maneuver", "Swamp");
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.on_battlefield(lions));
}
