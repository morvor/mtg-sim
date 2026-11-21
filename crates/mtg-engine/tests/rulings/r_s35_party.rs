//! Rulings batch S35 — the number of creatures in your party (CR 700.8): a number from
//! zero to four, determined without choosing creatures, as the ability resolves.

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use mtg_engine::decision::Decision;
use mtg_engine::game_terms::party_size;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts a full party and more onto `p`'s battlefield: two Clerics (one a Cleric Wizard),
/// a Rogue, a Warrior and a Wizard's worth of creatures.
fn big_party(t: &mut TestGame, p: PlayerId) {
    t.battlefield(p, "Keepers of the Faith");
    t.battlefield(p, "Shieldmage Elder");
    t.battlefield(p, "Bane Alley Blackguard");
    t.battlefield(p, "Oreskos Swiftclaw");
}

/// Whether `p` was asked to choose objects since decision `from`.
fn chose_objects(t: &TestGame, p: PlayerId, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(q, d)| *q == p && matches!(d, Decision::ChooseEntities { .. }))
}

#[test]
fn the_party_is_zero_to_four_and_never_chosen() {
    cr!("700.8", "700.8a");
    ruling!(
        "Malakir Blood-Priest",
        "An ability referring to the number of creatures in your party gets a number from zero to four. Such abilities never ask which creatures are in your party, and you never have to designate specific creatures as being in your party. You can't choose to exclude creatures from this count to lower the number."
    );
    supported("Malakir Blood-Priest");
    // "When this creature enters, each opponent loses X life and you gain X life, where X
    // is the number of creatures in your party." With five party creatures (and the
    // Blood-Priest, a Cleric), X is four.
    let mut t = TestGame::new(2);
    assert_eq!(party_size(&t.g, P0), 0);
    big_party(&mut t, P0);
    let from = t.asked().len();
    enter(&mut t, P0, "Malakir Blood-Priest");
    t.resolve_all();
    assert_eq!(party_size(&t.g, P0), 4);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
    assert!(!chose_objects(&t, P0, from));
}

#[test]
fn a_full_party_cant_be_counted_lower() {
    cr!("700.8", "700.8a", "118.12a");
    ruling!(
        "Concerted Defense",
        "An ability referring to the number of creatures in your party gets a number from zero to four. Such abilities never ask which creatures are in your party, and you never have to designate specific creatures as being in your party. You can’t choose to exclude creatures from this count to lower the number."
    );
    supported("Concerted Defense");
    // "Counter target noncreature spell unless its controller pays {1} plus an additional
    // {1} for each creature in your party." P1's party is four (not five, not fewer): P0
    // must pay {5}.
    for (spare, countered) in [(4, true), (5, false)] {
        let mut t = TestGame::new(2);
        big_party(&mut t, P1);
        t.battlefield(P1, "Malakir Blood-Priest");
        t.lands(P1, "Island", 1);
        t.lands(P0, "Forest", 1 + spare);
        let growth = t.hand(P0, "Giant Growth");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let spell = t.cast(P0, growth).target(bears).go();
        let defense = t.hand(P1, "Concerted Defense");
        let from = t.asked().len();
        t.answer_yes(P0, true);
        t.cast(P1, defense).target(Entity::Object(spell)).go();
        t.resolve_all();
        assert_eq!(t.in_graveyard(P0, "Giant Growth"), true);
        assert_eq!(t.pt(bears) == (2, 2), countered, "spare {spare}");
        assert!(!chose_objects(&t, P1, from));
    }
}

#[test]
fn the_party_is_counted_as_the_ability_resolves() {
    cr!("700.8", "608.2h");
    ruling!(
        "Thundering Sparkmage",
        "If an ability of a creature counts the number of creatures in your party, that number is counted as the ability resolves. If that creature is still on the battlefield when the ability resolves, it’ll be counted if appropriate."
    );
    supported("Thundering Sparkmage");
    // "When this creature enters, it deals X damage to target creature or planeswalker,
    // where X is the number of creatures in your party." Thundering Sparkmage is a Wizard.
    // With a Cleric: X is 2 if it's still there as the ability resolves...
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keepers of the Faith");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.answer_targets(P0, &[Entity::Object(wall)]);
    enter(&mut t, P0, "Thundering Sparkmage");
    t.resolve_all();
    assert_eq!(t.obj_now(wall).damage, 2);
    // ... 1 if it has left the battlefield by then ...
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keepers of the Faith");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.answer_targets(P0, &[Entity::Object(wall)]);
    let mage = enter(&mut t, P0, "Thundering Sparkmage");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, mage);
    t.resolve_all();
    assert_eq!(t.obj_now(wall).damage, 1);
    // ... and 3 if a Rogue entered before it resolved.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keepers of the Faith");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.answer_targets(P0, &[Entity::Object(wall)]);
    enter(&mut t, P0, "Thundering Sparkmage");
    t.battlefield(P0, "Bane Alley Blackguard");
    t.resolve_all();
    assert_eq!(t.obj_now(wall).damage, 3);
}
