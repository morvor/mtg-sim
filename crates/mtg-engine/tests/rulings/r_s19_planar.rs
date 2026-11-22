//! Rulings on plane cards (CR 311, 901): "you" is the planar controller, copies made by a
//! plane's ability keep the X and the mode, and a chaos ability's effect is fixed as it
//! resolves.

use crate::r_s01_common::*;
use crate::r_s06_common::give_control;
use crate::r_s07_common::chosen_modes;
use crate::r_s19_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::ObjKind;
use mtg_engine::planechase;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The spell copies on the stack.
fn copies_on_stack(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).kind == ObjKind::SpellCopy)
        .collect()
}

#[test]
fn you_on_a_plane_is_its_current_planar_controller() {
    cr!("311.5", "901.6");
    ruling!(
        "The Fourth Sphere",
        "If an ability of a plane refers to \"you,\" it's referring to whoever the plane's controller is at the time, not to the player that started the game with that plane card in their deck."
    );
    supported("The Fourth Sphere");
    // The Fourth Sphere (from P0's planar deck): "At the beginning of your upkeep,
    // sacrifice a nonblack creature. Whenever chaos ensues, create a 2/2 black Zombie
    // creature token."
    let mut t = planechase_game(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    start_planar_deck(&mut t, P0, &["The Fourth Sphere", "Llanowar"]);
    // In P1's turn P1 controls the plane: at P1's upkeep, P1 sacrifices.
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(planechase::planar_controller(&t.g), Some(P1));
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(mine));
    // Chaos in P1's turn: P1 creates the Zombie.
    chaos(&mut t, P1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P1, "Zombie").len(), 1);
    assert!(with_subtype(&t, P0, "Zombie").is_empty());
    // At P0's upkeep, P0 sacrifices.
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
}

#[test]
fn a_copy_of_a_spell_with_x_has_the_same_x() {
    cr!("707.10", "901.6");
    ruling!(
        "Izzet Steam Maze",
        "If the spell that's copied has an X whose value was determined as it was cast (like Earthquake does), the copy has the same value of X."
    );
    supported("Izzet Steam Maze");
    supported("Earthquake");
    // Izzet Steam Maze: "Whenever a player casts an instant or sorcery spell, that player
    // copies it. The player may choose new targets for the copy." Earthquake: "Earthquake
    // deals X damage to each creature without flying and each player."
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Izzet Steam Maze", "Llanowar"]);
    t.lands(P0, "Mountain", 3);
    let eq = t.hand(P0, "Earthquake");
    t.cast(P0, eq).x(2).go();
    t.settle();
    t.resolve();
    let copies = copies_on_stack(&t);
    assert_eq!(copies.len(), 1);
    let x_asked = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseX { .. }))
        .count();
    assert_eq!(x_asked, 1, "X is chosen only for the spell cast");
    assert_eq!(t.g.obj(copies[0]).stack.as_ref().unwrap().x, Some(2));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (16, 16));
    // The caster copies it, even when another player controls the plane: P1's Stroke of
    // Genius ("Target player draws X cards.") in P0's turn.
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Izzet Steam Maze", "Llanowar"]);
    t.lands(P1, "Island", 5);
    let stroke = t.hand(P1, "Stroke of Genius");
    let hand = t.hand_size(P1) - 1;
    t.cast(P1, stroke).x(2).target(P1).go();
    t.settle();
    t.resolve();
    let copies = copies_on_stack(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.g.obj(copies[0]).controller, P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 4);
}

#[test]
fn a_copy_of_a_modal_spell_has_the_same_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Izzet Steam Maze",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. Its controller can't choose a different one."
    );
    supported("Izzet Charm");
    // Izzet Charm, choosing "Draw two cards, then discard two cards."
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Izzet Steam Maze", "Llanowar"]);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let charm = t.hand(P0, "Izzet Charm");
    let spell = t.cast(P0, charm).modes(&[2]).go();
    t.settle();
    t.resolve();
    let copies = copies_on_stack(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(chosen_modes(&t, spell), vec![2]);
    assert_eq!(chosen_modes(&t, copies[0]), vec![2]);
    let modes_asked = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .count();
    assert_eq!(modes_asked, 1);
    let library = t.library_size(P0);
    t.resolve_all();
    assert_eq!(t.library_size(P0), library - 4);
}

#[test]
fn a_chaos_abilitys_bonus_applies_only_to_the_creatures_controlled_as_it_resolves() {
    cr!("611.2c", "311.7");
    ruling!(
        "Onakke Catacomb",
        "Creatures that you gain control of after the chaos ability resolves won’t get any of that ability’s bonuses."
    );
    supported("Onakke Catacomb");
    // Onakke Catacomb: "Whenever chaos ensues, creatures you control get +1/+0 and gain
    // first strike until end of turn."
    let mut t = planechase_game(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    start_planar_deck(&mut t, P0, &["Onakke Catacomb", "Llanowar"]);
    chaos(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.pt(mine), (3, 2));
    assert!(t.obj(mine).chars.has_keyword(KeywordKind::FirstStrike));
    // P0 gains control of P1's Bears afterwards: no bonus.
    give_control(&mut t, theirs, P0);
    assert_eq!(t.obj_now(theirs).controller, P0);
    assert_eq!(t.pt(theirs), (2, 2));
    assert!(!t.obj_now(theirs).chars.has_keyword(KeywordKind::FirstStrike));
}
