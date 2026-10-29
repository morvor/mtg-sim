//! Durations, grammar V (cards4-triggers-durations): durations ending at a step of a later
//! turn (Orcish Farmer, Halfdane, Xenic Poltergeist) and "for as long as" conditions about
//! the objects an instruction names (Zygon Infiltrator, Old Man of the Sea).

use crate::combat2_common::supported;
use crate::td_next_turn_durations::mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn untapped(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].tapped = false;
    t.g.objects[id.0 as usize].summoning_sick = false;
}

#[test]
fn orcish_farmer_swamp_lasts_until_the_lands_controllers_next_untap_step() {
    cr!("611.2a", "500.4", "305.7");
    ruling!("Orcish Farmer", "This ability changes the targeted land’s land types.");
    interp!("until-its-controllers-next-step");
    supported("Orcish Farmer");
    let mut t = TestGame::new(2);
    let farmer = t.battlefield(P0, "Orcish Farmer");
    untapped(&mut t, farmer);
    let forest = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Forest");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, farmer, 0, &[Entity::Object(forest)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(forest).chars.has_subtype("Swamp"));
    assert!(!t.obj_now(forest).chars.has_subtype("Forest"));
    // Through the rest of P0's turn; it ends as P1's untap step begins.
    t.advance_to(P0, Step::End);
    assert!(t.obj_now(forest).chars.has_subtype("Swamp"));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(forest).chars.has_subtype("Forest"));
    assert!(!t.obj_now(forest).chars.has_subtype("Swamp"));
    // A land P0 controls stays a Swamp through P1's turn, until P0's next untap step.
    let farmer = t.g.current(farmer);
    untapped(&mut t, farmer);
    t.activate(P0, farmer, 0, &[Entity::Object(mine)]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::End);
    assert!(t.obj_now(mine).chars.has_subtype("Swamp"));
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(mine).chars.has_subtype("Forest"));
}

#[test]
fn xenic_poltergeist_animation_wears_off_as_your_next_upkeep_begins() {
    cr!("611.2a", "500.4");
    ruling!("Xenic Poltergeist", "Wears off at the beginning of the upkeep");
    supported("Xenic Poltergeist");
    let mut t = TestGame::new(2);
    let ghost = t.battlefield(P0, "Xenic Poltergeist");
    untapped(&mut t, ghost);
    let ring = t.battlefield(P1, "Sol Ring");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, ghost, 0, &[Entity::Object(ring)]).unwrap();
    t.resolve_all();
    // An artifact creature with power and toughness equal to its mana value.
    assert!(t.obj_now(ring).chars.card_types.contains(CardType::Creature));
    assert_eq!(t.pt(ring), (1, 1));
    t.advance_to(P1, Step::End);
    assert!(t.obj_now(ring).chars.card_types.contains(CardType::Creature));
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(ring).chars.card_types.contains(CardType::Creature));
}

#[test]
fn halfdane_copies_power_and_toughness_until_the_end_of_your_next_upkeep() {
    cr!("611.2a", "500.5", "613.4b", "608.2h");
    ruling!(
        "Halfdane",
        "the effect from the previous turn's ability will continue to work"
    );
    supported("Halfdane");
    let mut t = TestGame::new(2);
    let halfdane = t.battlefield(P0, "Halfdane");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(halfdane), (2, 2));
    // Fixed as the effect began: a counter on the Bears doesn't change it.
    t.g.objects[bears.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    t.g.recompute();
    assert_eq!(t.pt(halfdane), (2, 2));
    // Not over as this upkeep ends: through the opponent's turn.
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.pt(halfdane), (2, 2));
    t.advance_to(P1, Step::End);
    assert_eq!(t.pt(halfdane), (2, 2));
    // P0's next upkeep: the new trigger has no legal target (the Bears are gone), so the
    // old effect lasts until that upkeep ends.
    t.g.destroy(bears, None);
    t.g.settle();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.pt(halfdane), (2, 2));
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.pt(halfdane), (3, 3));
}

#[test]
fn zygon_infiltrator_is_a_copy_for_as_long_as_that_creature_remains_tapped() {
    cr!("611.2b", "707.2");
    ruling!(
        "Zygon Infiltrator",
        "When Zygon Infiltrator becomes a copy of a creature, it's neither entering nor leaving"
    );
    supported("Zygon Infiltrator");
    let mut t = TestGame::new(2);
    let zygon = t.battlefield(P0, "Zygon Infiltrator");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0);
    t.activate(P0, zygon, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(t.counters(giant, "stun"), 1);
    assert_eq!(t.obj_now(zygon).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(zygon), (3, 3));
    // Untapping the creature ends the effect for good, even if it's tapped again.
    t.g.objects[giant.0 as usize].tapped = false;
    t.g.recompute();
    assert_eq!(t.obj_now(zygon).chars.name.as_str(), "Zygon Infiltrator");
    t.g.objects[giant.0 as usize].tapped = true;
    t.g.recompute();
    assert_eq!(t.obj_now(zygon).chars.name.as_str(), "Zygon Infiltrator");
}

#[test]
fn zygon_infiltrator_copy_ends_when_that_creature_leaves() {
    cr!("611.2b", "400.7");
    let mut t = TestGame::new(2);
    let zygon = t.battlefield(P0, "Zygon Infiltrator");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0);
    t.activate(P0, zygon, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(zygon).chars.name.as_str(), "Hill Giant");
    t.g.destroy(giant, None);
    t.resolve_all();
    assert_eq!(t.obj_now(zygon).chars.name.as_str(), "Zygon Infiltrator");
}

#[test]
fn old_man_of_the_sea_keeps_control_while_tapped_and_bigger() {
    cr!("611.2b");
    ruling!(
        "Old Man of the Sea",
        "The ability doesn’t care whether Old Man of the Sea remains under your control"
    );
    supported("Old Man of the Sea");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Old Man of the Sea");
    untapped(&mut t, old);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, old, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    // The Bears' power grows past the Old Man's (2): control returns, for good.
    t.g.objects[bears.0 as usize]
        .counters
        .insert("+1/+1".into(), 1);
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P1);
    t.g.objects[bears.0 as usize].counters.clear();
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn old_man_of_the_sea_control_ends_when_it_untaps() {
    cr!("611.2b");
    ruling!(
        "Old Man of the Sea",
        "You lose control of the creature if the Old Man leaves the battlefield"
    );
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Old Man of the Sea");
    untapped(&mut t, old);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, old, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bears).controller, P0, "still tapped");
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(old).tapped);
    assert_eq!(t.obj_now(bears).controller, P1);
    // Leaving the battlefield ends it too.
    let old = t.g.current(old);
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, old, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.g.destroy(old, None);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
}
