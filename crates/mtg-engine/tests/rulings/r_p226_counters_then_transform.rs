//! Rulings batch P226 — "Then if there are three or more [kind] counters on it, remove
//! those counters, transform it, and ..." (CR 608.2c): Treasure Map and Hostile Hostel,
//! with every ability on both faces.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::activate_containing;
use crate::r_s08_common::is_tapped;
use crate::r_s13_common::add;
use crate::r_s17_common::name_of;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const MAP: &str = "Treasure Map // Treasure Cove";
const HOSTEL: &str = "Hostile Hostel // Creeping Inn";

fn treasures(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    with_subtype(t, p, "Treasure")
}

/// Activates Treasure Map's ability ({1} from a fresh Wastes) and resolves it.
fn use_map(t: &mut TestGame, map: ObjectId) {
    t.lands(P0, "Wastes", 1);
    let now = t.g.current(map);
    t.g.objects[now.0 as usize].tapped = false;
    activate_containing(t, P0, map, "landmark").unwrap();
    t.resolve_all();
}

#[test]
fn treasure_map_becomes_treasure_cove_on_its_third_landmark_counter() {
    cr!("608.2c", "701.22a", "701.27a", "111.10a");
    supported(MAP);
    // "{1}, {T}: Scry 1. Put a landmark counter on this artifact. Then if there are three
    // or more landmark counters on it, remove those counters, transform this artifact,
    // and create three Treasure tokens."
    let mut t = TestGame::new(2);
    let map = t.battlefield(P0, MAP);
    let from = t.asked().len();
    use_map(&mut t, map);
    assert!(asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Scry { .. })));
    assert_eq!(t.counters(map, "landmark"), 1);
    use_map(&mut t, map);
    assert_eq!(t.counters(map, "landmark"), 2);
    assert!(treasures(&t, P0).is_empty());
    use_map(&mut t, map);
    assert_eq!(name_of(&t, map), "Treasure Cove");
    assert_eq!(t.counters(map, "landmark"), 0);
    assert_eq!(treasures(&t, P0).len(), 3);
    // Treasure Cove: "{T}: Add {C}." and "{T}, Sacrifice a Treasure: Draw a card."
    t.g.objects[map.0 as usize].tapped = false;
    t.activate(P0, map, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    t.g.objects[map.0 as usize].tapped = false;
    activate_containing(&mut t, P0, map, "Draw a card").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(treasures(&t, P0).len(), 2);
}

#[test]
fn a_treasure_map_that_left_the_battlefield_still_counts_its_last_landmark_counters() {
    cr!("608.2h", "113.7a", "400.7");
    ruling!(
        "Treasure Map // Treasure Cove",
        "If Treasure Map leaves the battlefield before its ability resolves, you can't put a landmark counter on it. However, if it somehow already had three landmark counters on it before it left the battlefield, you'll get three Treasures."
    );
    // One counter, then it's destroyed in response: no counter, no Treasures.
    let mut t = TestGame::new(2);
    let map = t.battlefield(P0, MAP);
    add(&mut t, map, "landmark", 1);
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, map, "landmark").unwrap();
    destroy(&mut t, map);
    t.resolve_all();
    assert_eq!(t.zone(map), Zone::Graveyard(P0));
    assert!(treasures(&t, P0).is_empty());
    // Three counters already, then it's destroyed in response: three Treasures.
    let mut t = TestGame::new(2);
    let map = t.battlefield(P0, MAP);
    add(&mut t, map, "landmark", 3);
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, map, "landmark").unwrap();
    destroy(&mut t, map);
    t.resolve_all();
    assert_eq!(t.zone(map), Zone::Graveyard(P0));
    assert_eq!(treasures(&t, P0).len(), 3);
}

#[test]
fn hostile_hostel_gathers_souls_and_becomes_creeping_inn() {
    cr!("608.2c", "701.27a", "702.26a", "607.2a");
    supported(HOSTEL);
    // "{T}: Add {C}." / "{1}, {T}, Sacrifice a creature: Put a soul counter on this land.
    // Then if there are three or more soul counters on it, remove those counters,
    // transform it, then untap it. Activate only as a sorcery."
    let mut t = TestGame::new(2);
    let hostel = t.battlefield(P0, HOSTEL);
    t.activate(P0, hostel, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    t.g.players[0].mana_pool = Default::default();
    t.g.objects[hostel.0 as usize].tapped = false;
    add(&mut t, hostel, "soul", 1);
    t.lands(P0, "Wastes", 2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, hostel, "soul counter").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(hostel, "soul"), 2);
    assert!(is_tapped(&t, hostel));
    // Not as an instant.
    t.g.objects[hostel.0 as usize].tapped = false;
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(activate_containing(&mut t, P0, hostel, "soul counter").is_err());
    t.set_step(P0, Step::PostcombatMain);
    activate_containing(&mut t, P0, hostel, "soul counter").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, hostel), "Creeping Inn");
    assert_eq!(t.counters(hostel, "soul"), 0);
    assert!(!is_tapped(&t, hostel));
    assert_eq!(t.pt(hostel), (3, 7));
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn creeping_inn_drains_for_each_creature_card_exiled_with_it_and_phases_out() {
    cr!("607.2a", "702.26b", "603.5");
    // "Whenever this creature attacks, you may exile a creature card from your
    // graveyard. If you do, each opponent loses X life and you gain X life, where X is
    // the number of creature cards exiled with this creature." / "{4}: This creature
    // phases out."
    let mut t = TestGame::new(2);
    let inn = crate::r_s17_common::enter_transformed(&mut t, P0, HOSTEL);
    t.g.objects[inn.0 as usize].summoning_sick = false;
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(inn, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!((t.life(P1), t.life(P0)), (19, 21));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
    // The next attack: two cards exiled with it.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(b)]);
    attack_with(&mut t, &[(inn, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P0)), (14, 23));
    // Declining: nothing.
    block_and_finish(&mut t, P1, &[]);
    t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(inn, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    block_and_finish(&mut t, P1, &[]);
    // "{4}: This creature phases out."
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Wastes", 4);
    activate_containing(&mut t, P0, inn, "phases out").unwrap();
    t.resolve_all();
    assert!(t.obj_now(inn).phased_out);
}
