//! Toggo, Goblin Weaponsmith: "Landfall — Whenever a land you control enters, create a
//! colorless Equipment artifact token named Rock with "Equipped creature has '{1}, {T},
//! Sacrifice Rock: This creature deals 2 damage to any target'" and equip {1}." The
//! token's given name refers to that token, also in the ability it grants (CR 111.4,
//! 201.5, 201.5a).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Toggo's landfall makes a Rock; it's attached to a Grizzly Bears. Returns (bears, rock).
fn rock_on_bears(t: &mut TestGame) -> (ObjectId, ObjectId) {
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Toggo, Goblin Weaponsmith");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let land = t.hand(P0, "Mountain");
    t.play_land(P0, land).expect("land");
    t.settle();
    t.resolve_all();
    let rocks = t.named_on_battlefield("Rock");
    assert_eq!(rocks.len(), 1, "landfall created a Rock");
    let rock = rocks[0];
    assert_eq!(t.obj_now(rock).controller, P0);
    assert!(t.g.attach(rock, Entity::Object(bears)));
    (bears, rock)
}

#[test]
fn the_equipped_creature_sacrifices_rock_to_deal_2_damage() {
    cr!("111.4", "201.5a", "113.7");
    let mut t = TestGame::new(2);
    let (bears, rock) = rock_on_bears(&mut t);
    // The Mountain played pays {1}.
    t.activate(P0, bears, 0, &[Entity::Player(P1)])
        .expect("activates");
    // Rock (not the creature) was sacrificed as a cost.
    assert!(!t.on_battlefield(rock));
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(bears));
}

#[test]
fn the_equipped_creature_is_the_source_of_the_damage() {
    cr!("113.7", "702.16b");
    ruling!(
        "Toggo, Goblin Weaponsmith",
        "The creature equipped by Rock is the source of the activated ability, not Rock or Toggo."
    );
    let mut t = TestGame::new(2);
    let (bears, _rock) = rock_on_bears(&mut t);
    // Protection from artifacts doesn't stop a green creature's ability.
    let chosen = t.battlefield(P1, "Tel-Jilad Chosen");
    t.activate(P0, bears, 0, &[Entity::Object(chosen)])
        .expect("a creature with protection from artifacts can be targeted");
    t.resolve_all();
    assert!(!t.on_battlefield(chosen));
}
