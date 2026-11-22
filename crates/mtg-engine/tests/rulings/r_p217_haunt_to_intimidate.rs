//! Rulings batch P217 — haunt (CR 702.55), heroic, hideaway (CR 702.75), inspired and
//! intimidate (CR 702.13).

use crate::r_s01_common::{stack_library, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s04_common::{add_mana, next_upkeep};
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s09_common::declare;
use crate::r_s17_common::color_set;
use crate::r_s21_common::legal_blocks;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::zones;
use mtg_engine::*;

#[test]
fn the_haunted_creature_you_own_can_be_returned_by_the_haunt_trigger() {
    cr!("702.55c", "115.1", "400.7");
    ruling!(
        "Exhumer Thrull",
        "If you own the creature Exhumer Thrull was haunting, that creature will generally be a legal target for the haunt trigger to return to your hand."
    );
    supported("Exhumer Thrull");
    let mut t = TestGame::new(2);
    let thrull = t.battlefield(P0, "Exhumer Thrull");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Exhumer Thrull dies and haunts P0's own Grizzly Bears.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    destroy(&mut t, thrull);
    t.resolve_all();
    assert!(t.in_exile("Exhumer Thrull"));
    // The Bears die: "When ... the creature it haunts dies, return target creature card
    // from your graveyard to your hand." The Bears card is in P0's graveyard by then, so
    // it's a legal target.
    destroy(&mut t, bears);
    let card = t.g.current(bears);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    assert_eq!(triggers_on_stack(&t, "the creature it haunts dies"), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn heroic_bonuses_go_only_to_creatures_you_control_as_the_ability_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Anax and Cymede",
        "Only creatures you control when the heroic ability resolves will get the bonuses. Creatures that come under your control later in the turn will not."
    );
    supported("Anax and Cymede");
    let mut t = TestGame::new(2);
    let anax = t.battlefield(P0, "Anax and Cymede");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(anax).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "targets"), 1);
    // A creature that enters before the heroic ability resolves gets the bonus.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(elves), (2, 2));
    assert!(t.obj_now(elves).chars.has_keyword(KeywordKind::Trample));
    t.resolve_all();
    // One that comes under P0's control afterwards doesn't.
    let giant = t.battlefield(P0, "Hill Giant");
    let stolen = t.battlefield(P1, "Grizzly Bears");
    give_control(&mut t, stolen, P0);
    t.g.recompute();
    assert_eq!(t.pt(giant), (3, 3));
    assert!(!t.obj_now(giant).chars.has_keyword(KeywordKind::Trample));
    assert_eq!(t.pt(stolen), (2, 2));
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn whoever_has_controlled_the_hideaway_permanent_may_look_at_the_exiled_card() {
    cr!("702.75a", "406.3");
    ruling!(
        "Cemetery Tampering",
        "Any player who has controlled a permanent with a hideaway ability since a card was exiled with it may look at that card."
    );
    supported("Cemetery Tampering");
    let mut t = TestGame::new(2);
    let cards = stack_library(
        &mut t,
        P0,
        &["Lightning Bolt", "Island", "Island", "Island", "Island"],
    );
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    let tampering = t.enter(P0, "Cemetery Tampering");
    t.resolve_all();
    let exiled = t.g.current(cards[0]);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(t.obj_now(exiled).face_down);
    assert!(zones::may_look(&t.g, P0, exiled));
    assert!(!zones::may_look(&t.g, P1, exiled));
    // P1 gains control of Cemetery Tampering: now both players have controlled it.
    give_control(&mut t, tampering, P1);
    assert!(zones::may_look(&t.g, P1, exiled));
    assert!(zones::may_look(&t.g, P0, exiled));
}

#[test]
fn a_creature_taps_when_its_regeneration_shield_is_used_not_when_created() {
    cr!("701.19a", "701.19c");
    ruling!(
        "Servant of Tymaret",
        "A creature taps when its regeneration shield is used, not when the shield is created."
    );
    supported("Servant of Tymaret");
    let mut t = TestGame::new(2);
    let servant = t.battlefield(P0, "Servant of Tymaret");
    // "{2}{B}: Regenerate this creature." creates a shield; the Servant stays untapped,
    // so it doesn't untap (and its inspired ability doesn't trigger) next turn.
    add_mana(&mut t, P0, ManaType::B, 3);
    activate_containing(&mut t, P0, servant, "Regenerate").expect("activate");
    t.resolve_all();
    assert!(!t.obj_now(servant).tapped);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "becomes untapped"), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // When the shield is used, the Servant taps; then its untapping triggers inspired.
    add_mana(&mut t, P0, ManaType::B, 3);
    activate_containing(&mut t, P0, servant, "Regenerate").expect("activate");
    t.resolve_all();
    destroy(&mut t, servant);
    assert!(t.on_battlefield(servant));
    assert!(t.obj_now(servant).tapped);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "becomes untapped"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn intimidate_looks_at_the_current_colors_of_the_creature() {
    cr!("702.13b", "509.1b");
    ruling!(
        "Halo Hunter",
        "Intimidate looks at the current colors of a creature that has it. Normally, Halo Hunter can't be blocked except by artifact creatures and/or black creatures. If it's turned white, then it can't be blocked except by artifact creatures and/or white creatures."
    );
    supported("Halo Hunter");
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Halo Hunter");
    // A black, a white, a green and an artifact creature.
    let black = t.battlefield(P1, "Drudge Skeletons");
    let white = t.battlefield(P1, "Savannah Lions");
    let green = t.battlefield(P1, "Grizzly Bears");
    let artifact = t.battlefield(P1, "Ornithopter");
    declare(&mut t, P0, &[(hunter, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(black, hunter)]));
    assert!(legal_blocks(&mut t, P1, &[(artifact, hunter)]));
    assert!(!legal_blocks(&mut t, P1, &[(white, hunter)]));
    assert!(!legal_blocks(&mut t, P1, &[(green, hunter)]));
    // Turned white.
    modify_until_eot(
        &mut t,
        hunter,
        vec![Modification::SetColors(color_set(&[Color::White]))],
    );
    assert!(legal_blocks(&mut t, P1, &[(white, hunter)]));
    assert!(legal_blocks(&mut t, P1, &[(artifact, hunter)]));
    assert!(!legal_blocks(&mut t, P1, &[(black, hunter)]));
    assert!(!legal_blocks(&mut t, P1, &[(green, hunter)]));
}
