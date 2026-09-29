//! Rulings batch S28 — power and toughness with counters (CR 613.4): effects that set
//! power and toughness apply in timestamp order in layer 7b (a later one overwrites an
//! earlier one, but not effects created after it), while effects and counters that modify
//! them (7c) and switches (7d) apply whenever they started. Also counters on a permanent
//! that isn't a creature, enters triggers that put counters, and "enters with an
//! additional counter".

use crate::r_s01_common::supported;
use crate::r_s04_common::next_upkeep;
use crate::r_s28_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p` casts `name` targeting `target` (lands for it are added) and it resolves.
fn cast_at(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId) {
    t.answer_targets(p, &[Entity::Object(target)]);
    cast_card(t, p, name);
    t.resolve_all();
}

fn plus1(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.add_counters(Entity::Object(id), "+1/+1", n, None);
    t.g.recompute();
}

/// Activates the land's (or artifact's) "becomes a creature" ability (its `index`th
/// activated ability), with lands for its cost, and resolves it.
fn animate(t: &mut TestGame, id: ObjectId, index: usize, lands: &[(&str, usize)]) {
    for (name, n) in lands {
        t.lands(P0, name, *n);
    }
    t.activate(P0, id, index, &[]).expect("animate");
    t.resolve_all();
}

#[test]
fn reanimating_a_manland_overrides_an_earlier_pt_setting_effect() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Faerie Conclave",
        "Activating the ability that turns it into a creature while it's already a creature will override any effects that set its power and/or toughness to a specific number. However, any effect that raises or lowers power and/or toughness (such as the effect created by Giant Growth, Glorious Anthem, or a +1/+1 counter) will continue to apply."
    );
    supported("Faerie Conclave");
    // "{1}{U}: This land becomes a 2/1 blue Faerie creature with flying until end of turn."
    let mut t = TestGame::new(2);
    let conclave = t.battlefield(P0, "Faerie Conclave");
    animate(&mut t, conclave, 1, &[("Island", 1), ("Wastes", 1)]);
    assert_eq!(t.pt(conclave), (2, 1));
    t.battlefield(P0, "Glorious Anthem");
    plus1(&mut t, conclave, 1);
    cast_at(&mut t, P0, "Giant Growth", conclave);
    assert_eq!(t.pt(conclave), (7, 6));
    // Wings of Velis Vel: base power and toughness 4/4.
    cast_at(&mut t, P0, "Wings of Velis Vel", conclave);
    assert_eq!(t.pt(conclave), (9, 9));
    // Activating it again: 2/1 again, with the anthem, counter, and Giant Growth.
    animate(&mut t, conclave, 1, &[("Island", 1), ("Wastes", 1)]);
    assert_eq!(t.pt(conclave), (7, 6));
}

#[test]
fn reanimating_forbidding_watchtower_overrides_an_earlier_pt_setting_effect() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Forbidding Watchtower",
        "Activating the ability that turns it into a creature while it’s already a creature will override any effects that set its power and/or toughness to a specific number. However, any effect that raises or lowers power and/or toughness (such as the effect created by Giant Growth, Glorious Anthem, or a +1/+1 counter) will continue to apply."
    );
    supported("Forbidding Watchtower");
    // "{1}{W}: This land becomes a 1/5 white Soldier creature until end of turn."
    let mut t = TestGame::new(2);
    let tower = t.battlefield(P0, "Forbidding Watchtower");
    animate(&mut t, tower, 1, &[("Plains", 1), ("Wastes", 1)]);
    plus1(&mut t, tower, 1);
    assert_eq!(t.pt(tower), (2, 6));
    cast_at(&mut t, P0, "Wings of Velis Vel", tower);
    assert_eq!(t.pt(tower), (5, 5));
    animate(&mut t, tower, 1, &[("Plains", 1), ("Wastes", 1)]);
    assert_eq!(t.pt(tower), (2, 6));
}

#[test]
fn battlegate_mimics_pt_overwrites_only_earlier_setting_effects() {
    cr!("613.4b", "613.4c", "613.4d", "613.7");
    ruling!(
        "Battlegate Mimic",
        "The effect from the ability overwrites other effects that set power and/or toughness if and only if those effects existed before the ability resolved. It will not overwrite effects that modify power or toughness (whether from a static ability, counters, or a resolved spell or ability), nor will it overwrite effects that set power and toughness which come into existence after the ability resolves. Effects that switch the creature’s power and toughness are always applied after any other power or toughness changing effects, including this one, regardless of the order in which they are created."
    );
    supported("Battlegate Mimic");
    // "Whenever you cast a spell that's both red and white, this creature has base power
    // and toughness 4/2 until end of turn and gains first strike until end of turn."
    let mut t = TestGame::new(2);
    let mimic = t.battlefield(P0, "Battlegate Mimic");
    plus1(&mut t, mimic, 1);
    assert_eq!(t.pt(mimic), (3, 2));
    // An earlier setting effect (4/4) and an earlier switch.
    cast_at(&mut t, P0, "Wings of Velis Vel", mimic);
    assert_eq!(t.pt(mimic), (5, 5));
    cast_at(&mut t, P0, "Twisted Image", mimic);
    // The Mimic's ability: base 4/2 overwrites the 4/4; the counter still applies, and the
    // earlier switch is applied after it.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_card(&mut t, P0, "Lightning Helix");
    t.resolve_all();
    assert_eq!(t.pt(mimic), (3, 5));
    // A later setting effect isn't overwritten: Snakeform makes it a 1/1.
    cast_at(&mut t, P0, "Snakeform", mimic);
    assert_eq!(t.pt(mimic), (2, 2));
}

#[test]
fn a_pt_setting_spell_overwrites_only_earlier_setting_effects() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Wings of Velis Vel",
        "The power/toughness-setting effect overwrites other effects that set power and toughness only if those effects existed before this spell resolved. It will not overwrite effects that modify power or toughness (whether from a static ability, counters, or a resolved spell or ability), nor will it overwrite effects that set power and toughness which come into existence after this spell resolves. Effects that switch the creature's power and toughness are always applied after any other power or toughness changing effects, including this one, regardless of the order in which they are created."
    );
    supported("Wings of Velis Vel");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    plus1(&mut t, giant, 1);
    // Sure Strike: +3/+0. Twisted Image switches.
    cast_at(&mut t, P0, "Sure Strike", giant);
    cast_at(&mut t, P0, "Snakeform", giant);
    assert_eq!(t.pt(giant), (5, 2));
    cast_at(&mut t, P0, "Twisted Image", giant);
    assert_eq!(t.pt(giant), (2, 5));
    // Wings: base 4/4 overwrites Snakeform's 1/1; the switch still applies last.
    cast_at(&mut t, P0, "Wings of Velis Vel", giant);
    assert_eq!(t.pt(giant), (5, 8));
    // A later Turn to Frog (base 1/1) isn't overwritten by Wings.
    cast_at(&mut t, P0, "Turn to Frog", giant);
    assert_eq!(t.pt(giant), (2, 5));
}

#[test]
fn modifying_effects_counters_and_switches_apply_whenever_they_started() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Turn to Frog",
        "Effects that modify a creature’s power and/or toughness, such as the effect of Titanic Growth, will apply to the creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    supported("Turn to Frog");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    plus1(&mut t, giant, 1);
    cast_at(&mut t, P0, "Sure Strike", giant);
    cast_at(&mut t, P0, "Twisted Image", giant);
    assert_eq!(t.pt(giant), (4, 7));
    // Turn to Frog: base 1/1, with the earlier counter, +3/+0, and switch.
    cast_at(&mut t, P0, "Turn to Frog", giant);
    assert_eq!(t.pt(giant), (2, 5));
    // A later Titanic Growth (+4/+4) applies too.
    cast_at(&mut t, P0, "Titanic Growth", giant);
    assert_eq!(t.pt(giant), (6, 9));
}

#[test]
fn kenriths_transformation_keeps_counters_and_pumps() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Kenrith's Transformation",
        "Effects that modify a creature's power and/or toughness, such as the effect of Festive Funeral, will apply to the creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness."
    );
    supported("Kenrith's Transformation");
    // "Enchanted creature loses all abilities and is a green Elk creature with base power
    // and toughness 3/3."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    plus1(&mut t, bears, 1);
    cast_at(&mut t, P1, "Sure Strike", bears);
    assert_eq!(t.pt(bears), (6, 3));
    cast_at(&mut t, P0, "Kenrith's Transformation", bears);
    assert_eq!(t.pt(bears), (7, 4));
    cast_at(&mut t, P1, "Giant Growth", bears);
    assert_eq!(t.pt(bears), (10, 7));
}

#[test]
fn an_animated_artifacts_counters_pumps_and_switches_apply() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Firdoch Core",
        "Effects that modify a creature's power and/or toughness, such as the effect of Appeal to Eirdu, will apply to the creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    supported("Firdoch Core");
    // "{4}: This artifact becomes a 4/4 artifact creature until end of turn."
    let mut t = TestGame::new(2);
    let core = t.battlefield(P0, "Firdoch Core");
    // A counter put on it before it's a creature.
    plus1(&mut t, core, 1);
    animate(&mut t, core, 1, &[("Wastes", 4)]);
    assert_eq!(t.pt(core), (5, 5));
    cast_at(&mut t, P0, "Sure Strike", core);
    assert_eq!(t.pt(core), (8, 5));
    cast_at(&mut t, P0, "Twisted Image", core);
    assert_eq!(t.pt(core), (5, 8));
}

#[test]
fn a_monuments_counters_stay_while_it_isnt_a_creature() {
    cr!("122.1", "613.4c");
    ruling!(
        "Atarka Monument",
        "If a Monument has any +1/+1 counters on it, those counters will remain on the permanent after it stops being a creature. Those counters will have no effect as long as the Monument isn't a creature, but they will apply again if the Monument later becomes a creature."
    );
    supported("Atarka Monument");
    // "{4}{R}{G}: This artifact becomes a 4/4 red and green Dragon artifact creature with
    // flying until end of turn."
    let mut t = TestGame::new(2);
    let monument = t.battlefield(P0, "Atarka Monument");
    let lands = [("Mountain", 1), ("Forest", 1), ("Wastes", 4)];
    animate(&mut t, monument, 1, &lands);
    plus1(&mut t, monument, 1);
    assert_eq!(t.pt(monument), (5, 5));
    next_upkeep(&mut t, P1);
    assert!(!t.obj_now(monument).is(mtg_engine::types::CardType::Creature));
    assert_eq!(t.counters(monument, "+1/+1"), 1);
    next_upkeep(&mut t, P0);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    animate(&mut t, monument, 1, &lands);
    assert_eq!(t.pt(monument), (5, 5));
}

#[test]
fn an_enters_trigger_puts_the_counter_after_the_creature_enters() {
    cr!("603.2", "603.6a");
    ruling!(
        "Kazuul Warlord",
        "This ability triggers on this creature entering, so it will enter with its unmodified power and toughness, and then receive a +1/+1 counter a short time later. It does not enter with the +1/+1 counter already on it."
    );
    supported("Kazuul Warlord");
    // "Whenever this creature or another Ally you control enters, you may put a +1/+1
    // counter on each Ally creature you control."
    let mut t = TestGame::new(2);
    let warlord = t.enter(P0, "Kazuul Warlord");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.counters(warlord, "+1/+1"), 0);
    assert_eq!(t.pt(warlord), (3, 3));
    // In response, 3 damage kills it before it gets the counter.
    t.answer_targets(P1, &[Entity::Object(warlord)]);
    cast_card(&mut t, P1, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Kazuul Warlord"));
    // Unanswered, it gets the counter.
    let mut t = TestGame::new(2);
    let warlord = t.enter(P0, "Kazuul Warlord");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(warlord), (4, 4));
}

#[test]
fn an_additional_counter_applies_to_any_creature_entering_under_your_control() {
    cr!("614.1c", "122.6", "110.2");
    ruling!(
        "Bramblewood Paragon",
        "The creature gets the counter if it would enter under your control. It doesn't matter who owns the creature or what zone it enters from (such as your opponent's graveyard, for example)."
    );
    supported("Bramblewood Paragon");
    supported("Reanimate");
    // "Each other Warrior creature you control enters with an additional +1/+1 counter on
    // it." Goblin Piker is a Goblin Warrior.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bramblewood Paragon");
    let theirs = t.graveyard(P1, "Goblin Piker");
    cast_at(&mut t, P0, "Reanimate", theirs);
    let piker = t.g.current(theirs);
    assert!(t.on_battlefield(piker));
    assert_eq!(t.obj_now(piker).controller, P0);
    assert_eq!(t.counters(piker, "+1/+1"), 1);
    assert_eq!(t.pt(piker), (3, 2));
    // A Warrior entering under P1's control doesn't get one.
    let other = t.enter(P1, "Goblin Piker");
    assert_eq!(t.counters(other, "+1/+1"), 0);
}
