//! "Counters remain on ~ as it moves to any zone other than a player's hand or library."
//! (Skullbriar, the Walking Grave; Me, the Immortal): an exception to CR 122.2 and 400.7
//! (`rule_statics::counters_remain`).

use mtg_engine::testing::*;
use mtg_engine::object::Zone;
use mtg_engine::*;

fn with_counters(t: &mut TestGame, name: &str, n: u32) -> ObjectId {
    let id = t.battlefield(P0, name);
    t.g.add_counters(Entity::Object(id), "+1/+1", n, None);
    t.g.recompute();
    id
}

fn murder(t: &mut TestGame, target: ObjectId) {
    t.lands(P0, "Swamp", 3);
    let m = t.hand(P0, "Murder");
    t.cast(P0, m).target(target).go();
    t.resolve();
}

#[test]
fn counters_follow_it_to_the_graveyard_and_back_without_being_put() {
    cr!("122.2", "400.7", "122.1a");
    ruling!("Skullbriar, the Walking Grave", "aren't \"placed\" on Skullbriar");
    ruling!("Skullbriar, the Walking Grave", "affect Skullbriar's power and/or toughness in zones other than the battlefield");
    let mut t = TestGame::new(2);
    let sk = with_counters(&mut t, "Skullbriar, the Walking Grave", 2);
    murder(&mut t, sk);
    let in_gy = t.g.current(sk);
    assert!(matches!(t.zone(in_gy), Zone::Graveyard(_)));
    assert_eq!(t.counters(in_gy, "+1/+1"), 2);
    assert_eq!(t.pt(in_gy), (3, 3));
    // Returned to the battlefield with Doubling Season out: the counters aren't put on
    // it, so they aren't doubled.
    t.battlefield(P0, "Doubling Season");
    let back = t
        .g
        .move_object(in_gy, Zone::Battlefield, events::MoveCause::Effect, Some(P0))
        .expect("returned");
    t.settle();
    assert_eq!(t.counters(back, "+1/+1"), 2);
    assert_eq!(t.pt(back), (3, 3));
}

#[test]
fn counters_are_lost_going_to_a_hand_or_library() {
    cr!("122.2", "400.7");
    let mut t = TestGame::new(2);
    let sk = with_counters(&mut t, "Skullbriar, the Walking Grave", 1);
    let in_hand = t
        .g
        .move_object(sk, Zone::Hand(P0), events::MoveCause::Effect, None)
        .unwrap();
    assert_eq!(t.counters(in_hand, "+1/+1"), 0);
    let me = with_counters(&mut t, "Me, the Immortal", 1);
    let in_lib = t
        .g
        .move_object(me, Zone::Library(P0), events::MoveCause::Effect, None)
        .unwrap();
    assert_eq!(t.counters(in_lib, "+1/+1"), 0);
}

#[test]
fn counters_remain_in_the_command_zone_and_exile() {
    cr!("122.2", "122.1a");
    ruling!("Me, the Immortal", "a Me in the command zone with a +1/+1 counter on it will be 4/4");
    ruling!("Me, the Immortal", "aren't \"put\" on that card");
    let mut t = TestGame::new(2);
    let me = with_counters(&mut t, "Me, the Immortal", 1);
    let cz = t
        .g
        .move_object(me, Zone::Command, events::MoveCause::Effect, None)
        .unwrap();
    t.g.recompute();
    assert_eq!(t.counters(cz, "+1/+1"), 1);
    assert_eq!(t.pt(cz), (4, 4));
    let ex = t
        .g
        .move_object(cz, Zone::Exile, events::MoveCause::Effect, None)
        .unwrap();
    assert_eq!(t.counters(ex, "+1/+1"), 1);
}

#[test]
fn the_ability_must_function_in_the_zone_it_moves_from() {
    cr!("122.2", "400.7");
    ruling!("Skullbriar, the Walking Grave", "only works if it has that ability in the zone it's moving from");
    ruling!("Me, the Immortal", "only works if it has that ability in the zone it's moving from");
    let mut t = TestGame::new(2);
    let sk = with_counters(&mut t, "Skullbriar, the Walking Grave", 2);
    murder(&mut t, sk);
    let in_gy = t.g.current(sk);
    assert_eq!(t.counters(in_gy, "+1/+1"), 2);
    // Cards in graveyards lose all abilities: it loses its counters as it leaves.
    t.battlefield(P1, "Yixlid Jailer");
    t.g.recompute();
    let back = t
        .g
        .move_object(in_gy, Zone::Battlefield, events::MoveCause::Effect, Some(P0))
        .unwrap();
    assert_eq!(t.counters(back, "+1/+1"), 0);
}

#[test]
fn all_kinds_of_counters_remain() {
    cr!("122.2");
    ruling!("Skullbriar, the Walking Grave", "Skullbriar retains all counters, not just +1/+1 counters.");
    ruling!("Me, the Immortal", "retains all counters, not just those granted by the first ability");
    let mut t = TestGame::new(2);
    let me = t.battlefield(P0, "Me, the Immortal");
    t.g.add_counters(Entity::Object(me), "menace", 1, None);
    t.g.add_counters(Entity::Object(me), "-1/-1", 1, None);
    let ex = t
        .g
        .move_object(me, Zone::Exile, events::MoveCause::Effect, None)
        .unwrap();
    t.g.recompute();
    assert_eq!(t.counters(ex, "menace"), 1);
    assert_eq!(t.counters(ex, "-1/-1"), 1);
    assert!(t.obj_now(ex).has_keyword(keywords::KeywordKind::Menace));
}

#[test]
fn a_copy_keeps_its_counters_only_while_its_a_copy() {
    cr!("122.2", "707.2");
    ruling!("Skullbriar, the Walking Grave", "If a card becomes a copy of Skullbriar, counters will remain on that card");
    ruling!("Me, the Immortal", "If a card becomes a copy of Me, counters will remain on that card");
    let mut t = TestGame::new(2);
    let sk = t.battlefield(P0, "Skullbriar, the Walking Grave");
    // P1's Clone copies it (another player's, so the legend rule doesn't apply).
    t.lands(P1, "Island", 4);
    let clone = t.hand(P1, "Clone");
    t.answer_choose(P1, &[Entity::Object(sk)]);
    t.set_step(P1, turn::Step::PrecombatMain);
    t.cast(P1, clone).go();
    t.resolve();
    let clone = t.g.current(clone);
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Skullbriar, the Walking Grave");
    t.g.add_counters(Entity::Object(clone), "+1/+1", 2, None);
    // It leaves the battlefield as a copy of Skullbriar: the counters remain on the card.
    let in_gy = t
        .g
        .move_object(clone, Zone::Graveyard(P1), events::MoveCause::Effect, None)
        .unwrap();
    t.g.recompute();
    assert_eq!(t.obj_now(in_gy).chars.name.as_str(), "Clone");
    assert_eq!(t.counters(in_gy, "+1/+1"), 2);
    // In the graveyard it's Clone, without the ability: they cease to exist as it moves.
    let ex = t
        .g
        .move_object(in_gy, Zone::Exile, events::MoveCause::Effect, None)
        .unwrap();
    assert_eq!(t.counters(ex, "+1/+1"), 0);
}
