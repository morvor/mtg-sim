//! Item `rulings-residue-type-change` — interaction rulings about layers: effects that
//! set base power and toughness (layer 7b) against characteristic-defining abilities
//! (7a), modifications and counters (7c), and each other (timestamps, CR 613.7);
//! type-changing effects (layer 4) against changelings and objects that stop being
//! creatures; and effects that change a spell on the stack (CR 400.7a).

use crate::r_residue_type_change_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn the_master_base_pt_under_counters_and_pumps() {
    cr!("613.4b", "613.4c");
    ruling!(
        "The Master, Transcendent",
        "Effects that modify a creature's power and/or toughness will still apply to the creature as normal. The same is true for any counters that change its power and/or toughness."
    );
    supported("The Master, Transcendent");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let m = t.battlefield(P0, "The Master, Transcendent");
    let angel = milled(&mut t, "Serra Angel");
    t.activate(P0, m, 0, &[obj(angel)]).unwrap();
    t.resolve_all();
    let angel = t.g.current(angel);
    assert_eq!(pt_now(&mut t, angel), (3, 3));
    give_plus1(&mut t, angel, 1);
    giant_growth(&mut t, angel);
    assert_eq!(pt_now(&mut t, angel), (7, 7));
}

#[test]
fn the_master_overrides_a_pt_defining_ability() {
    cr!("613.4a", "613.4b", "604.3");
    ruling!(
        "The Master, Transcendent",
        "The Master's last ability will override any characteristic-defining abilities the creature has that set its power and/or toughness."
    );
    supported("The Master, Transcendent");
    supported("Robobrain War Mind");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let m = t.battlefield(P0, "The Master, Transcendent");
    let robo = milled(&mut t, "Robobrain War Mind");
    t.activate(P0, m, 0, &[obj(robo)]).unwrap();
    t.resolve_all();
    let robo = t.g.current(robo);
    assert_eq!(t.zone(robo), Zone::Battlefield);
    assert_eq!(pt_now(&mut t, robo), (3, 3));
    // The number of cards in P0's hand doesn't affect its power at all.
    for _ in 0..4 {
        t.hand(P0, "Grizzly Bears");
    }
    assert_eq!(pt_now(&mut t, robo), (3, 3));
}

#[test]
fn curse_of_conformity_base_pt_under_earlier_pumps_and_counters() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Curse of Conformity",
        "Effects that modify a creature's power and/or toughness, such as the effect of Titanic Growth, will apply to the affected creatures no matter when they started to take effect."
    );
    supported("Curse of Conformity");
    supported("Titanic Growth");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Titanic Growth (+4/+4) and a +1/+1 counter, both before the Curse.
    cast_resolve(&mut t, P0, "Titanic Growth", &[obj(bears)]);
    give_plus1(&mut t, bears, 1);
    assert_eq!(pt_now(&mut t, bears), (7, 7));
    cast_resolve(&mut t, P0, "Curse of Conformity", &[Entity::Player(P1)]);
    assert_eq!(pt_now(&mut t, bears), (8, 8));
    assert!(creature_types(&mut t, bears).is_empty());
}

#[test]
fn curse_of_conformity_changeling_loses_all_creature_types() {
    cr!("613.3", "613.1d", "702.73a", "205.1b");
    ruling!(
        "Curse of Conformity",
        "Creatures with the changeling ability will still lose all creature types even though they don't lose changeling."
    );
    supported("Curse of Conformity");
    supported("Changeling Outcast");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let outcast = t.battlefield(P1, "Changeling Outcast");
    assert!(subtype(&mut t, outcast, "Elf"));
    cast_resolve(&mut t, P0, "Curse of Conformity", &[Entity::Player(P1)]);
    assert!(creature_types(&mut t, outcast).is_empty());
    assert!(has_kw(&mut t, outcast, KeywordKind::Changeling));
    assert_eq!(pt_now(&mut t, outcast), (3, 3));
}

#[test]
fn shadow_puppeteers_base_pt_under_pumps_and_counters() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Shadow Puppeteers",
        "Effects that modify a creature's power and/or toughness, such as the one created by Squeak By or a +1/+1 counter, will apply to the creature no matter when they started to take effect."
    );
    supported("Shadow Puppeteers");
    supported("Cheeky House-Mouse // Squeak By");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow Puppeteers");
    let crow = t.battlefield(P0, "Storm Crow");
    t.set_step(P0, Step::PrecombatMain);
    // Squeak By (the Adventure: +1/+1 until end of turn) and a +1/+1 counter first.
    t.lands(P0, "Plains", 1);
    let mouse = t.hand(P0, "Cheeky House-Mouse // Squeak By");
    t.cast(P0, mouse)
        .target(crow)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    t.resolve_all();
    give_plus1(&mut t, crow, 1);
    assert_eq!(pt_now(&mut t, crow), (3, 4));
    puppeteers_attack(&mut t, crow);
    t.resolve_all();
    assert_eq!(pt_now(&mut t, crow), (6, 6));
}

#[test]
fn shadow_puppeteers_overwrites_earlier_setting_and_later_setting_overwrites_it() {
    cr!("613.4b", "613.7b", "613.7");
    ruling!(
        "Shadow Puppeteers",
        "Shadow Puppeteers's last ability overwrites any previous effects that set the creature's power and/or toughness to specific values."
    );
    supported("Shadow Puppeteers");
    supported("Diminish");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow Puppeteers");
    let crow = t.battlefield(P0, "Storm Crow");
    t.set_step(P0, Step::PrecombatMain);
    diminish(&mut t, crow);
    assert_eq!(pt_now(&mut t, crow), (1, 1));
    puppeteers_attack(&mut t, crow);
    t.resolve_all();
    assert_eq!(pt_now(&mut t, crow), (4, 4));
    // A setting effect that starts later overwrites it.
    diminish(&mut t, crow);
    assert_eq!(pt_now(&mut t, crow), (1, 1));
    assert!(subtype(&mut t, crow, "Dragon"));
}

#[test]
fn shadow_puppeteers_doesnt_remove_abilities() {
    cr!("613.1d", "613.1f", "205.1b");
    ruling!(
        "Shadow Puppeteers",
        "Shadow Puppeteers's last ability doesn't remove any abilities the creature has."
    );
    supported("Shadow Puppeteers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow Puppeteers");
    let angel = t.battlefield(P0, "Serra Angel");
    puppeteers_attack(&mut t, angel);
    t.resolve_all();
    assert!(subtype(&mut t, angel, "Dragon") && subtype(&mut t, angel, "Angel"));
    assert!(has_kw(&mut t, angel, KeywordKind::Flying));
    assert!(has_kw(&mut t, angel, KeywordKind::Vigilance));
    assert!(!tapped(&t, angel));
}

#[test]
fn shadow_puppeteers_on_a_creature_that_stopped_being_a_creature() {
    cr!("205.1b", "205.3d", "611.2c", "613.4b", "208.3", "208.3a");
    interp!("subtype-gained-only-if-it-corresponds-as-the-effect-begins");
    interp!("base-pt-set-on-a-noncreature-applies-once-it-is-a-creature");
    ruling!(
        "Shadow Puppeteers",
        "If the creature stops being a creature before Shadow Puppeteers's last ability resolves, it will become red until end of turn in addition to its other colors."
    );
    supported("Shadow Puppeteers");
    supported("Swift Reconfiguration");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow Puppeteers");
    let crow = t.battlefield(P0, "Storm Crow");
    let giants = [t.battlefield(P0, "Hill Giant"), t.battlefield(P0, "Hill Giant")];
    puppeteers_attack(&mut t, crow);
    // Swift Reconfiguration (flash) makes the Crow a Vehicle artifact, not a creature.
    cast_new(&mut t, P0, "Swift Reconfiguration", &[obj(crow)]);
    t.resolve();
    assert!(!is_type(&mut t, crow, CardType::Creature));
    t.resolve_all();
    assert!(has_color(&mut t, crow, Color::Red) && has_color(&mut t, crow, Color::Blue));
    assert!(!subtype(&mut t, crow, "Dragon"));
    assert!(subtype(&mut t, crow, "Vehicle"));
    // A noncreature permanent has no power or toughness: the base 4/4 does nothing ...
    t.g.recompute();
    assert_eq!(t.obj_now(crow).chars.power, None);
    assert_eq!(t.obj_now(crow).chars.toughness, None);
    // ... unless it becomes a creature this turn (crewed: the Storm Crow is 1/2), CR 208.3a.
    assert!(crate::r_s04_common::crew(&mut t, P0, crow, &giants));
    t.resolve_all();
    assert!(is_type(&mut t, crow, CardType::Creature));
    assert_eq!(pt_now(&mut t, crow), (4, 4));
    // It never gains the Dragon subtype, even as a creature: it couldn't as the effect began
    // (CR 205.3d; decided by the user, Nov 19). (Whether crewing brings back its Bird type,
    // which Swift Reconfiguration removed with its creature type, is a separate open question
    // of CR 205.1a: `docs/agents/items/rulings-residue.md`; not asserted here.)
    assert!(!subtype(&mut t, crow, "Dragon"));
    assert!(has_color(&mut t, crow, Color::Red));
}

#[test]
fn phantasmal_form_base_pt_under_pumps_and_counters() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Phantasmal Form",
        "Effects that modify the target creatures' power or toughness (such as the effects of Force of Virtue or Giant Growth) will apply no matter when they started to take effect."
    );
    supported("Phantasmal Form");
    supported("Force of Virtue");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Force of Virtue");
    giant_growth(&mut t, bears);
    give_plus1(&mut t, elves, 1);
    assert_eq!(pt_now(&mut t, bears), (6, 6));
    assert_eq!(pt_now(&mut t, elves), (3, 3));
    lands_for_cost(&mut t, P0, "Phantasmal Form");
    let form = t.hand(P0, "Phantasmal Form");
    t.cast(P0, form).targets(&[obj(bears), obj(elves)]).go();
    t.resolve_all();
    // Base 3/3, +3/+3 (Giant Growth), +1/+1 (Force of Virtue); 3/3 +1/+1 counter +1/+1.
    assert_eq!(pt_now(&mut t, bears), (7, 7));
    assert_eq!(pt_now(&mut t, elves), (5, 5));
    assert!(subtype(&mut t, bears, "Illusion") && subtype(&mut t, bears, "Bear"));
    assert!(has_color(&mut t, elves, Color::Blue) && has_color(&mut t, elves, Color::Green));
}

#[test]
fn minsc_overwrites_earlier_setting_keeps_modifications_and_counters() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Minsc, Beloved Ranger",
        "The activated ability will overwrite any previous effects that set the creature's power and toughness to specific values. Effects that otherwise modify the target creature's power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Minsc, Beloved Ranger");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let minsc = t.battlefield(P0, "Minsc, Beloved Ranger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    giant_growth(&mut t, bears);
    diminish(&mut t, bears);
    give_plus1(&mut t, bears, 1);
    assert_eq!(pt_now(&mut t, bears), (5, 5));
    activate_x(&mut t, minsc, 0, 4, &[obj(bears)]);
    t.resolve_all();
    // Base 4/4 (overwriting Diminish's 1/1), +3/+3, +1/+1 counter.
    assert_eq!(pt_now(&mut t, bears), (8, 8));
    assert!(subtype(&mut t, bears, "Giant") && subtype(&mut t, bears, "Bear"));
}

#[test]
fn werewolf_pack_leader_overwrites_earlier_setting_keeps_modifications() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Werewolf Pack Leader",
        "For example, if you cast a spell that gives Werewolf Pack Leader +1/+1, after that resolves it is a 4/4 creature. If you then activate its last ability, it will be a 6/4 creature."
    );
    supported("Werewolf Pack Leader");
    supported("Ranger's Guile");
    // The ruling's example: a spell gives it +1/+1, then the ability: 6/4.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wolf = t.battlefield(P0, "Werewolf Pack Leader");
    cast_resolve(&mut t, P0, "Ranger's Guile", &[obj(wolf)]);
    assert_eq!(pt_now(&mut t, wolf), (4, 4));
    t.lands(P0, "Forest", 4);
    t.activate(P0, wolf, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(pt_now(&mut t, wolf), (6, 4));
    // An earlier effect setting it to 1/1 is overwritten; a +1/+1 counter still applies.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wolf = t.battlefield(P0, "Werewolf Pack Leader");
    diminish(&mut t, wolf);
    give_plus1(&mut t, wolf, 1);
    assert_eq!(pt_now(&mut t, wolf), (2, 2));
    t.lands(P0, "Forest", 4);
    t.activate(P0, wolf, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(pt_now(&mut t, wolf), (6, 4));
}
