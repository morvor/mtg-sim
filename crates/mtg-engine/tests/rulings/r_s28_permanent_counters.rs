//! Rulings batch S28 — counters on permanents: optional upkeep counters (Lotus Blossom),
//! storage lands (their upkeep trigger's intervening "if", CR 603.4; counters that wait
//! while the land loses its abilities; tapping by other effects), phasing with counters
//! and Auras (CR 702.26g), defense counters on an indestructible battle (CR 310.12b), and
//! pawprint modes, which aren't a resource (CR 107.18).

use crate::r_s01_common::{supported, tokens};
use crate::r_s04_common::next_upkeep;
use crate::r_s06_common::damage;
use crate::r_s28_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::*;

const STORAGE: &str = "storage";

#[test]
fn lotus_blossoms_upkeep_counter_is_optional() {
    cr!("603.5", "122.6");
    ruling!(
        "Lotus Blossom",
        "Adding a counter is optional. If you forget to add one during your upkeep, you cannot back up and add one later."
    );
    supported("Lotus Blossom");
    // "At the beginning of your upkeep, you may put a petal counter on this artifact."
    let mut t = TestGame::new(2);
    let blossom = t.battlefield(P0, "Lotus Blossom");
    t.answer_yes(P0, false);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(blossom, "petal"), 0);
    // Later in the turn there's no way to add it.
    t.advance_to(P0, mtg_engine::turn::Step::End);
    assert_eq!(t.counters(blossom, "petal"), 0);
    t.answer_yes(P0, true);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(blossom, "petal"), 1);
}

/// P0's Sand Silos ("{T}, Remove any number of storage counters from this land: Add {U}
/// for each storage counter removed this way.") with `n` storage counters.
fn silos(t: &mut TestGame, n: u32) -> ObjectId {
    supported("Sand Silos");
    let silos = t.battlefield(P0, "Sand Silos");
    t.g.add_counters(Entity::Object(silos), STORAGE, n, None);
    silos
}

#[test]
fn a_storage_land_tapped_by_another_effect_keeps_its_counters() {
    cr!("122.1", "118.3");
    ruling!(
        "Sand Silos",
        "If the land is tapped by some external effect, no counters are removed from it."
    );
    supported("Icy Manipulator");
    // Icy Manipulator: "{1}, {T}: Tap target artifact, creature, or land."
    let mut t = TestGame::new(2);
    let silos = silos(&mut t, 2);
    let icy = t.battlefield(P1, "Icy Manipulator");
    t.lands(P1, "Wastes", 1);
    t.activate(P1, icy, 0, &[Entity::Object(silos)])
        .expect("tap the Silos");
    t.resolve_all();
    assert!(t.obj_now(silos).tapped);
    assert_eq!(t.counters(silos, STORAGE), 2);
    assert!(t.g.player(P0).mana_pool.is_empty());
}

#[test]
fn a_storage_lands_counters_wait_while_its_land_type_is_changed() {
    cr!("305.7", "122.1");
    ruling!(
        "Sand Silos",
        "Counters are not lost if the land is changed to another land type. They wait around for the land to change back."
    );
    supported("Evil Presence");
    // Evil Presence: "Enchanted land is a Swamp." It loses its own abilities (CR 305.7).
    let mut t = TestGame::new(2);
    let silos = silos(&mut t, 2);
    t.answer_targets(P0, &[Entity::Object(silos)]);
    let presence = cast_card(&mut t, P0, "Evil Presence");
    t.resolve_all();
    assert!(t.obj_now(silos).chars.has_subtype("Swamp"));
    assert!(!t
        .obj_now(silos)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("storage")));
    assert_eq!(t.counters(silos, STORAGE), 2);
    // The Aura leaves: the counters can be used again.
    crate::r_s02_common::destroy(&mut t, presence);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, silos, 0, &[]).expect("remove two storage counters");
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    assert_eq!(t.counters(silos, STORAGE), 0);
}

#[test]
fn a_storage_land_gets_a_counter_only_if_tapped_at_upkeep_and_on_resolution() {
    cr!("603.4", "502.3");
    ruling!(
        "Sand Silos",
        "Whether or not it is tapped is checked at the beginning of upkeep. If it is not tapped, the ability does not trigger. It also checks during resolution and you only get a counter if it is still tapped then."
    );
    // "You may choose not to untap this land during your untap step. At the beginning of
    // your upkeep, if this land is tapped, put a storage counter on it."
    let mut t = TestGame::new(2);
    let silos = silos(&mut t, 0);
    // Untapped at the beginning of the upkeep: no trigger.
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    // Kept tapped: it triggers, but it's untapped before the ability resolves.
    t.g.tap(silos);
    let follower = t.battlefield(P0, "Kiora's Follower");
    t.answer_yes(P0, false);
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    assert!(t.obj_now(silos).tapped);
    t.activate(P0, follower, 0, &[Entity::Object(silos)])
        .expect("untap the Silos");
    t.resolve_all();
    assert!(!t.obj_now(silos).tapped);
    assert_eq!(t.counters(silos, STORAGE), 0);
    // Kept tapped through the resolution: a counter.
    t.g.tap(silos);
    t.answer_yes(P0, false);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(silos, STORAGE), 1);
}

#[test]
fn a_phased_out_creature_phases_in_with_its_counters_and_auras() {
    cr!("702.26b", "702.26g", "702.26c");
    ruling!(
        "The Moment",
        "Each Aura and Equipment attached to a permanent that's phasing out also phases out. They will phase in with that permanent and still be attached to it. Similarly, permanents that phase out with counters phase in with those counters."
    );
    supported("The Moment");
    // "{2}, {T}: Untap target creature you control. It phases out until The Moment leaves
    // the battlefield."
    let mut t = TestGame::new(2);
    let moment = t.battlefield(P0, "The Moment");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let rancor = cast_card(&mut t, P0, "Rancor");
    t.resolve_all();
    let rancor = t.g.current(rancor);
    assert_eq!(t.pt(bears), (5, 3));
    t.lands(P0, "Wastes", 2);
    t.activate(P0, moment, 0, &[Entity::Object(bears)])
        .expect("phase the Bears out");
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
    assert!(t.obj_now(rancor).phased_out);
    crate::r_s02_common::destroy(&mut t, moment);
    t.g.recompute();
    assert!(!t.obj_now(bears).phased_out);
    assert!(!t.obj_now(rancor).phased_out);
    assert_eq!(t.obj_now(rancor).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (5, 3));
}

#[test]
fn an_indestructible_siege_still_loses_defense_counters_and_is_defeated() {
    cr!("310.12b", "702.12b", "120.3h");
    ruling!(
        "The Walls of Ba Sing Se",
        "A battle with indestructible still loses defense counters as it's dealt damage. If it's a Siege, it will still be exiled when the last defense counter is removed from it, and its controller may still cast it transformed without paying its mana cost."
    );
    supported("The Walls of Ba Sing Se");
    // "Other permanents you control have indestructible." Invasion of Theros is a Siege
    // with defense 4 // Ephara, Ever-Sheltering.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Walls of Ba Sing Se");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let siege = t.enter(P0, "Invasion of Theros // Ephara, Ever-Sheltering");
    t.resolve_all();
    assert!(t
        .obj_now(siege)
        .has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
    let source = t.battlefield(P1, "Hill Giant");
    damage(&mut t, source, 3, siege);
    assert_eq!(t.counters(siege, "defense"), 1);
    t.answer_yes(P0, true);
    damage(&mut t, source, 1, siege);
    t.resolve_all();
    let ephara = t.named_on_battlefield("Ephara, Ever-Sheltering");
    assert_eq!(ephara.len(), 1);
    assert_eq!(t.obj(ephara[0]).face, FaceState::Back);
}

#[test]
fn pawprints_cant_be_saved_up_for_a_later_season() {
    cr!("107.18", "700.2i");
    ruling!(
        "Season of the Burrow",
        "The pawprint symbol does not represent a cost, mana, counters, or any kind of persistent resource. You can't \"save up\" pawprint symbols from one Season spell to use on a future one, mostly because there isn't anything concrete to save up. They're just pawprints."
    );
    supported("Season of the Burrow");
    // "Choose up to five {P} worth of modes. ... {P} — Create a 1/1 white Rabbit creature
    // token."
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 0, 0]));
    cast_card(&mut t, P0, "Season of the Burrow");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 3);
    assert!(t.g.player(P0).counters.is_empty());
    // The next Season still has only five {P} worth: seven (five plus the two "left
    // over") isn't a legal choice.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0; 7]));
    let spell = cast_card(&mut t, P0, "Season of the Burrow");
    let chosen = crate::r_s07_common::chosen_modes(&t, spell);
    assert!(!chosen.is_empty() && chosen.len() <= 5);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 3 + chosen.len());
    // Five is.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0; 5]));
    let spell = cast_card(&mut t, P0, "Season of the Burrow");
    assert_eq!(crate::r_s07_common::chosen_modes(&t, spell), vec![0; 5]);
}
