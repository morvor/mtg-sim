//! Rulings batch P224 — storm (CR 702.40): what it counts, what the copies are, and how
//! they're countered, retargeted and trigger other abilities.

use crate::r_p224_common::*;
use crate::r_s01_common::*;
use crate::r_s11_common::{spells_copied, triggered_from};
use mtg_engine::decision::Answer;
use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn copies_of(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).kind == ObjKind::SpellCopy && t.g.obj(*id).chars.name == name)
        .collect()
}

#[test]
fn chatterstorm_copies_are_countered_individually() {
    cr!("702.40a", "701.6a");
    ruling!(
        "Chatterstorm",
        " A copy of a spell can be countered like any other spell, but it must be countered individually. Countering a spell with storm won't affect the copies."
    );
    supported("Chatterstorm");
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, Entity::Player(P1));
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    // Chatterstorm: "Create a 1/1 green Squirrel creature token." Storm: two copies.
    t.lands(P0, "Forest", 2);
    let cs = t.hand(P0, "Chatterstorm");
    let original = t.cast(P0, cs).go();
    t.resolve();
    let copies = copies_of(&t, "Chatterstorm");
    assert_eq!(copies.len(), 2);
    // P1 counters one copy, then the original: the other copy still resolves.
    counterspell(&mut t, P1, copies[1]);
    t.resolve();
    counterspell(&mut t, P1, original);
    t.resolve();
    assert_eq!(copies_of(&t, "Chatterstorm").len(), 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Squirrel").len(), 1);
    assert!(t.in_graveyard(P0, "Chatterstorm"));
}

#[test]
fn all_of_history_copies_are_countered_individually() {
    cr!("702.40a", "701.6a", "701.56a");
    ruling!(
        "All of History, All at Once",
        "A copy of a spell can be countered like any other spell, but each copy must be countered individually. Countering a spell with storm won't affect the copies."
    );
    supported("All of History, All at Once");
    let mut t = TestGame::new(2);
    // A permanent with a time counter on it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::TIME, 2, None);
    t.g.recompute();
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    // All of History, All at Once: "Time travel." Storm: one copy. P0 adds a time counter
    // each time it time travels.
    for _ in 0..2 {
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
    }
    t.lands(P0, "Island", 4);
    let c = t.hand(P0, "All of History, All at Once");
    let original = t.cast(P0, c).go();
    t.resolve();
    assert_eq!(copies_of(&t, "All of History, All at Once").len(), 1);
    // The original is countered: the copy still resolves.
    counterspell(&mut t, P1, original);
    t.resolve();
    assert!(t.in_graveyard(P0, "All of History, All at Once"));
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::TIME), 3);
}

#[test]
fn chatterstorm_storm_counts_spells_from_other_zones_and_ones_that_failed_to_resolve() {
    cr!("702.40a", "707.10c");
    ruling!(
        "Chatterstorm",
        " Spells cast from zones other than a player's hand and spells that were countered or otherwise failed to resolve are counted by the storm ability."
    );
    let mut t = TestGame::new(2);
    spells_that_didnt_resolve_normally(&mut t);
    t.lands(P0, "Forest", 2);
    let cs = t.hand(P0, "Chatterstorm");
    t.cast(P0, cs).go();
    t.resolve();
    assert_eq!(copies_of(&t, "Chatterstorm").len(), 4);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Squirrel").len(), 5);
}

#[test]
fn chatterstorm_storm_copies_of_a_targeted_spell_may_get_new_targets() {
    cr!("702.40a", "707.10c");
    ruling!(
        "Chatterstorm",
        " If a spell with storm has targets, you may choose new targets for any of the copies. You can make different choices for each copy."
    );
    supported("Grapeshot");
    // Chatterstorm has no target; Grapeshot ("Grapeshot deals 1 damage to any target.
    // Storm") does. Two copies: one gets a new target, the other keeps the original's.
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, Entity::Player(P1));
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let gs = t.hand(P0, "Grapeshot");
    t.cast(P0, gs).target(P1).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(copies_of(&t, "Grapeshot").len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 6 - 2);
    assert_eq!(t.obj_now(bears).damage, 1);
}

/// P0 casts a Lightning Bolt, then `name` (a storm spell with no target, given the mana
/// for it): returns the number of copies its storm ability created, then resolves
/// everything and casts Weather the Storm, returning how many copies that one's storm
/// ability created (the copies of `name` aren't counted).
fn copies_then_later_storm(name: &str) -> (usize, usize) {
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    give_mana_for(&mut t, P0, name);
    let c = t.hand(P0, name);
    t.cast(P0, c).go();
    t.resolve();
    let first = copies_of(&t, name).len();
    t.resolve_all();
    // Two spells cast this turn: Lightning Bolt and `name`.
    assert_eq!(t.g.history.spells_cast.len(), 2);
    t.lands(P0, "Forest", 2);
    let wts = t.hand(P0, "Weather the Storm");
    t.cast(P0, wts).go();
    t.resolve();
    let later = copies_of(&t, "Weather the Storm").len();
    t.resolve_all();
    (first, later)
}

#[test]
fn storm_copies_arent_cast_and_arent_counted_by_later_storm_spells() {
    cr!("702.40a", "707.10");
    ruling!(
        "Chatterstorm",
        " The copies are put directly onto the stack. They aren't cast and won't be counted by other spells with storm cast later in the turn."
    );
    ruling!(
        "Elemental Eruption",
        "The copies of Elemental Eruption created by its storm ability are put directly onto the stack. They aren’t cast and won’t be counted by other spells with storm cast later in the turn."
    );
    ruling!(
        "Radstorm",
        "The copies of Radstorm created by its storm ability are put directly onto the stack. They aren’t cast and won’t be counted by other spells with storm cast later in the turn."
    );
    supported("Radstorm");
    supported("Elemental Eruption");
    for name in ["Chatterstorm", "Elemental Eruption", "Radstorm"] {
        assert_eq!(copies_then_later_storm(name), (1, 2), "{name}");
    }
}

#[test]
fn weather_the_storm_copies_dont_trigger_cast_triggers() {
    cr!("702.40a", "707.10", "603.2");
    ruling!(
        "Weather the Storm",
        "The copies storm creates are created on the stack, so they're not cast. Abilities that trigger when a player casts a spell (such as storm) won't trigger."
    );
    supported("Weather the Storm");
    supported("Young Pyromancer");
    let mut t = TestGame::new(2);
    // Young Pyromancer: "Whenever you cast an instant or sorcery spell, create a 1/1 red
    // Elemental creature token."
    t.battlefield(P0, "Young Pyromancer");
    bolt(&mut t, P0, Entity::Player(P1));
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    t.lands(P0, "Forest", 2);
    let wts = t.hand(P0, "Weather the Storm");
    t.cast(P0, wts).go();
    t.resolve_all();
    // Two copies and the original: 9 life. Three Elementals: none for the copies.
    assert_eq!(t.life(P0), 29);
    assert_eq!(with_subtype(&t, P0, "Elemental").len(), 3);
    assert_eq!(t.g.history.spells_cast.len(), 3);
}

#[test]
fn weather_the_storm_doesnt_count_spells_cast_after_it() {
    cr!("702.40a");
    ruling!(
        "Weather the Storm",
        "Storm counts spells cast before the spell with storm was cast. Spells cast after the spell with storm was cast but before the storm ability resolves aren't counted."
    );
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    t.lands(P0, "Forest", 2);
    let wts = t.hand(P0, "Weather the Storm");
    t.cast(P0, wts).go();
    t.settle();
    // In response to the storm trigger, P0 casts another Lightning Bolt.
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve(); // the second Lightning Bolt
    t.resolve(); // the storm trigger
    assert_eq!(copies_of(&t, "Weather the Storm").len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn each_fiery_encore_storm_copy_triggers_magecraft() {
    cr!("702.40a", "707.10", "603.2");
    ruling!(
        "Fiery Encore",
        "Each copy you create because of storm will cause your magecraft abilities to trigger."
    );
    supported("Fiery Encore");
    supported("Archmage Emeritus");
    // Archmage Emeritus: "Magecraft — Whenever you cast or copy an instant or sorcery
    // spell, draw a card."
    let mut t = TestGame::new(2);
    let emeritus = t.battlefield(P0, "Archmage Emeritus");
    bolt(&mut t, P0, Entity::Player(P1));
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    // Fiery Encore: "Discard a card, then draw a card. ..." Storm: two copies.
    for _ in 0..3 {
        t.hand(P0, "Forest");
    }
    t.lands(P0, "Mountain", 5);
    let fe = t.hand(P0, "Fiery Encore");
    t.cast(P0, fe).go();
    t.resolve_all();
    assert_eq!(spells_copied(&t), 2);
    // Five magecraft triggers: two Lightning Bolts, Fiery Encore, and its two copies.
    assert_eq!(triggered_from(&t, emeritus), 5);
}

#[test]
fn haze_of_rage_storm_triggers_each_time_its_cast_counting_only_spells_cast() {
    cr!("702.40a", "702.27a");
    ruling!(
        "Haze of Rage",
        "Each time you cast Haze of Rage in a turn, its storm ability will trigger. Each spell you cast before it will be considered when determining how many copies are created, but previous times you created copies of a spell will not."
    );
    supported("Haze of Rage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    // Haze of Rage ({1}{R}, buyback {2}): "Creatures you control get +1/+0 until end of
    // turn." Storm. First cast with buyback: one copy.
    t.lands(P0, "Mountain", 4);
    let haze = t.hand(P0, "Haze of Rage");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, haze).go();
    t.resolve();
    assert_eq!(copies_of(&t, "Haze of Rage").len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(t.in_hand(P0, "Haze of Rage"));
    // Second cast: Lightning Bolt and the first Haze of Rage are counted (two copies), not
    // the first one's copy.
    let haze =
        t.g.find_in_zone(mtg_engine::object::Zone::Hand(P0), "Haze of Rage")[0];
    t.lands(P0, "Mountain", 2);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, haze).go();
    t.resolve();
    assert_eq!(copies_of(&t, "Haze of Rage").len(), 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 2));
}
