//! CR 702.182 Job select (`src/kw/job_select.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn job_select_cards_compile() {
    assert_supported(&[
        "Monk's Fist",
        "Samurai's Katana",
        "Warrior's Sword",
        "Dragoon's Lance",
        "Bard's Bow",
        "Dark Knight's Greatsword",
    ]);
}

fn heroes(t: &TestGame) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Hero"))
        .map(|o| o.id)
        .collect()
}

#[test]
fn creates_a_hero_token_and_attaches_the_equipment_to_it() {
    cr!("702.182a");
    ruling!(
        "Monk's Fist",
        "The Hero token enters as a 1/1 creature, then the Equipment becomes attached to it."
    );
    // "Job select. Equipped creature gets +1/+0 and is a Monk in addition to its other
    // types. Equip {2}"
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let fist = t.hand(P0, "Monk's Fist");
    t.cast(P0, fist).go();
    t.resolve();
    // The trigger is on the stack.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let hs = heroes(&t);
    assert_eq!(hs.len(), 1);
    let hero = t.obj_now(hs[0]);
    assert!(hero.chars.colors.is_colorless());
    assert!(hero.is_creature());
    assert!(hero.chars.has_subtype("Monk"));
    assert_eq!(t.pt(hs[0]), (2, 1));
    let fist = t.named_on_battlefield("Monk's Fist")[0];
    assert_eq!(t.obj_now(fist).attached_to, Some(Entity::Object(hs[0])));
}

#[test]
fn the_equipment_can_move_and_stays_if_the_hero_dies() {
    cr!("702.182a");
    ruling!(
        "Monk's Fist",
        "You may pay the Equipment's equip cost as normal to move it from the Hero token"
    );
    ruling!(
        "Monk's Fist",
        "If the Hero token is destroyed, the Equipment stays on the battlefield."
    );
    let mut t = TestGame::new(2);
    let fist = t.enter(P0, "Monk's Fist");
    t.resolve_all();
    let hero = heroes(&t)[0];
    assert_eq!(t.obj_now(fist).attached_to, Some(Entity::Object(hero)));
    // Equip {2} to another creature.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.activate(P0, fist, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(fist).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(hero), (1, 1));
    // The Hero dies; the Equipment (now on the Bears) stays.
    t.g.destroy(hero, None);
    t.resolve_all();
    assert!(heroes(&t).is_empty());
    assert!(t.on_battlefield(fist));
}

#[test]
fn if_the_equipment_left_the_token_is_still_created() {
    cr!("702.182a");
    let mut t = TestGame::new(2);
    let fist = t.enter(P0, "Monk's Fist");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(fist, None);
    t.resolve_all();
    let hs = heroes(&t);
    assert_eq!(hs.len(), 1);
    assert_eq!(t.pt(hs[0]), (1, 1));
    assert!(t.in_graveyard(P0, "Monk's Fist"));
}

#[test]
fn with_two_heroes_the_equipment_is_attached_to_one() {
    cr!("702.182a");
    ruling!(
        "Monk's Fist",
        "the Equipment becomes attached to only one of them"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Parallel Lives");
    let fist = t.enter(P0, "Monk's Fist");
    t.resolve_all();
    let hs = heroes(&t);
    assert_eq!(hs.len(), 2);
    let attached: Vec<&ObjectId> = hs
        .iter()
        .filter(|h| t.obj_now(fist).attached_to == Some(Entity::Object(**h)))
        .collect();
    assert_eq!(attached.len(), 1);
}

#[test]
fn the_hero_gets_the_equipments_quoted_ability_and_job() {
    cr!("702.182a");
    // Thief's Knife ({2}{U} Equipment): "Job select. Equipped creature gets +1/+1, has
    // "Whenever this creature deals combat damage to a player, draw a card," and is a
    // Rogue in addition to its other types. Equip {4}"
    assert_supported(&["Thief's Knife", "White Mage's Staff", "Black Mage's Rod"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let knife = t.hand(P0, "Thief's Knife");
    t.cast(P0, knife).go();
    t.resolve_all();
    let hs = heroes(&t);
    assert_eq!(hs.len(), 1);
    let hero = hs[0];
    assert_eq!(t.pt(hero), (2, 2));
    assert!(t.obj_now(hero).chars.has_subtype("Rogue"));
    assert!(t.obj_now(hero).chars.has_subtype("Hero"));
    // It's been under its controller's control since their most recent turn began.
    t.g.objects[hero.0 as usize].summoning_sick = false;
    let hand = t.hand_size(P0);
    t.attack(&[(hero, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand + 1);
}
