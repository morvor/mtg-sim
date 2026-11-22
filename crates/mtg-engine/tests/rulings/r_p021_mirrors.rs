//! Rulings batch P021 — Mirage Mirror and Mirror of the Forebears, noncreature artifacts
//! that become copies until end of turn: the legend rule (CR 704.5j), an Aura or
//! Equipment copy (CR 704.5m, 704.5n), summoning sickness (CR 302.6), and choices made
//! for the copied object's linked abilities (CR 607.2d, 707.2).

use crate::r_p021_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_attack;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s18_common::lands_for;
use crate::r_s24_common::choose_creature_type;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s Mirage Mirror (summoning sick when `sick`) becomes a copy of `what`.
fn mirage_mirror_copies(t: &mut TestGame, p: PlayerId, what: ObjectId, sick: bool) -> ObjectId {
    supported("Mirage Mirror");
    let mirror = if sick {
        t.battlefield_sick(p, "Mirage Mirror")
    } else {
        t.battlefield(p, "Mirage Mirror")
    };
    lands_for(t, p, "{2}");
    t.activate(p, mirror, 0, &[obj(what)]).expect("activate");
    t.resolve_all();
    mirror
}

/// `p`'s Mirror of the Forebears (choosing `ty`; summoning sick when `sick`) becomes a
/// copy of `what`.
fn forebears_copies(
    t: &mut TestGame,
    p: PlayerId,
    ty: &str,
    what: ObjectId,
    sick: bool,
) -> ObjectId {
    supported("Mirror of the Forebears");
    choose_creature_type(t, p, ty);
    let mirror = t.enter(p, "Mirror of the Forebears");
    t.settle();
    if !sick {
        crate::r_p023_common::unsick(t, mirror);
    }
    lands_for(t, p, "{1}");
    t.activate(p, mirror, 0, &[obj(what)]).expect("activate");
    t.resolve_all();
    mirror
}

fn legend_rule_leaves_one(t: &TestGame, a: ObjectId, b: ObjectId) {
    let left = [a, b].iter().filter(|id| t.on_battlefield(**id)).count();
    assert_eq!(left, 1);
    assert!(
        t.in_graveyard(P0, "Isamaru, Hound of Konda")
            || t.in_graveyard(P0, "Mirage Mirror")
            || t.in_graveyard(P0, "Mirror of the Forebears")
    );
}

#[test]
fn mirage_mirror_copying_your_legendary_permanent_and_the_legend_rule() {
    cr!("704.5j", "707.2");
    ruling!(
        "Mirage Mirror",
        "If Mirage Mirror becomes a copy of a legendary permanent you control, you'll put one of them into your graveyard."
    );
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let mirror = mirage_mirror_copies(&mut t, P0, isamaru, false);
    legend_rule_leaves_one(&t, isamaru, mirror);
}

#[test]
fn mirror_of_the_forebears_copying_your_legendary_creature_and_the_legend_rule() {
    cr!("704.5j", "707.2");
    ruling!(
        "Mirror of the Forebears",
        "If Mirror of the Forebears becomes a copy of a legendary creature you control, you’ll put one of them into your graveyard."
    );
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let mirror = forebears_copies(&mut t, P0, "Dog", isamaru, false);
    legend_rule_leaves_one(&t, isamaru, mirror);
}

#[test]
fn mirage_mirror_copying_an_aura_or_an_equipment() {
    cr!("704.5m", "704.5n");
    ruling!(
        "Mirage Mirror",
        "If Mirage Mirror becomes a copy of an Aura, it's put into its owner's graveyard unless it's somehow attached to an appropriate object or player already."
    );
    // An Aura: it isn't attached to anything, so it's put into the graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pacifism = attach_new(&mut t, P0, "Pacifism", bears);
    let mirror = mirage_mirror_copies(&mut t, P0, pacifism, false);
    assert!(!t.on_battlefield(mirror));
    assert!(t.in_graveyard(P0, "Mirage Mirror"));
    // An Equipment: equipped to a creature, it becomes unattached when the copy effect
    // ends and it's a non-Equipment artifact again.
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mirror = mirage_mirror_copies(&mut t, P0, splitter, false);
    assert!(has_subtype(&t, mirror, "Equipment"));
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, mirror, "Equip").expect("equip");
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(mirror).attached_to, Some(obj(bears)));
    assert_eq!(t.pt(bears), (4, 2));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(mirror));
    assert_eq!(t.obj_now(mirror).chars.name, "Mirage Mirror");
    assert_eq!(t.obj_now(mirror).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
}

/// The Mirror, which entered this turn and is now a copy of Llanowar Elves (or of
/// Raging Goblin, which has haste), can't attack or use its {T} ability without haste.
fn summoning_sick_copy(make: fn(&mut TestGame, ObjectId) -> ObjectId) {
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let mirror = make(&mut t, elves);
    assert!(t.obj_now(mirror).is(CardType::Creature));
    assert_eq!(t.obj_now(mirror).chars.name, "Llanowar Elves");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, mirror));
    t.set_step(P0, Step::PrecombatMain);
    assert!(activate_containing(&mut t, P0, mirror, "Add {G}").is_err());
    assert!(activate_containing(&mut t, P0, elves, "Add {G}").is_ok());
    // With haste it can attack.
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let mirror = make(&mut t, goblin);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, mirror));
}

#[test]
fn mirage_mirror_becoming_a_creature_the_turn_it_enters_is_summoning_sick() {
    cr!("302.6", "702.10b");
    ruling!(
        "Mirage Mirror",
        "If Mirage Mirror becomes a creature the same turn it enters the battlefield, you can't attack with it or use any of its {T} abilities (if it gains any) unless it has haste."
    );
    summoning_sick_copy(|t, what| mirage_mirror_copies(t, P0, what, true));
}

#[test]
fn mirror_of_the_forebears_becoming_a_creature_the_turn_it_enters_is_summoning_sick() {
    cr!("302.6", "702.10b");
    ruling!(
        "Mirror of the Forebears",
        "If Mirror of the Forebears becomes a creature the same turn it enters the battlefield, you can’t attack with it or use any of its {T} abilities (if it gains any) unless it has haste."
    );
    summoning_sick_copy(|t, what| {
        let ty = if t.obj_now(what).chars.has_subtype("Elf") {
            "Elf"
        } else {
            "Goblin"
        };
        forebears_copies(t, P0, ty, what, true)
    });
}

#[test]
fn mirror_of_the_forebears_copying_metallic_mimic_has_no_chosen_type_for_it() {
    cr!("607.2d", "707.2");
    ruling!(
        "Mirror of the Forebears",
        "If Mirror of the Forebears becomes a copy of a creature that itself has an ability that asks you to choose a creature type as it enters the battlefield"
    );
    supported("Metallic Mimic");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Elf");
    let mimic = t.enter(P0, "Metallic Mimic");
    t.settle();
    assert!(has_subtype(&t, mimic, "Elf"));
    // The Mirror chooses Shapeshifter (Metallic Mimic is one) and becomes a copy of it.
    let mirror = forebears_copies(&mut t, P0, "Shapeshifter", mimic, false);
    assert_eq!(t.obj_now(mirror).chars.name, "Metallic Mimic");
    // It isn't an Elf (no choice was made for its copied abilities) ...
    assert!(!has_subtype(&t, mirror, "Elf"));
    // ... and its "chosen type" isn't Shapeshifter: a Shapeshifter entering gets no
    // counter from it; an Elf gets one, from the real Metallic Mimic only.
    let dopp = t.enter(P0, "Renegade Doppelganger");
    t.resolve_all();
    assert_eq!(t.counters(dopp, counters::PLUS1), 0);
    let elves = t.enter(P0, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(t.counters(elves, counters::PLUS1), 1);
}
