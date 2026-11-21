//! Durations ending at other points: "until end of combat on your next turn" (Brazen
//! Cannonade), "until the next end step" (Niko, Light of Hope), "until target enchantment
//! you control leaves the battlefield" (Calix, Destiny's Hand).

use crate::combat2_common::*;
use crate::td_next_turn_durations::{mana, next_combat};
use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether casting `card` is among `p`'s legal actions now.
pub fn may_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

/// Brazen Cannonade: P0 attacks, and its raid trigger exiles `top` in P0's postcombat main
/// phase. Returns the exiled card.
fn cannonade_exiles(t: &mut TestGame, top: &str) -> ObjectId {
    t.battlefield(P0, "Brazen Cannonade");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.library_top(P0, top);
    declare(t, P0, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), mtg_engine::object::Zone::Exile);
    exiled
}

#[test]
fn brazen_cannonade_card_is_playable_until_end_of_combat_on_your_next_turn() {
    cr!("611.2a", "500.5a");
    ruling!("Brazen Cannonade", "If you don't play the exiled card, it remains in exile.");
    supported("Brazen Cannonade");
    let mut t = TestGame::new(2);
    let bolt = cannonade_exiles(&mut t, "Lightning Bolt");
    mana(&mut t, P0);
    assert!(may_cast(&mut t, P0, bolt));
    // During the opponent's turn too, after their combat.
    t.advance_to(P1, Step::Upkeep);
    mana(&mut t, P0);
    assert!(may_cast(&mut t, P0, bolt));
    t.advance_to(P1, Step::PostcombatMain);
    mana(&mut t, P0);
    assert!(may_cast(&mut t, P0, bolt));
    // P0's next turn, through its combat.
    t.advance_to(P0, Step::DeclareAttackers);
    mana(&mut t, P0);
    assert!(may_cast(&mut t, P0, bolt));
    // Not after that combat ends.
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0);
    assert!(!may_cast(&mut t, P0, bolt));
    assert_eq!(t.zone(bolt), mtg_engine::object::Zone::Exile);
}

#[test]
fn brazen_cannonade_permission_ends_with_a_next_turn_that_has_no_combat() {
    cr!("611.2a");
    interp!("end-of-combat-on-your-next-turn-without-combat");
    let mut t = TestGame::new(2);
    let bolt = cannonade_exiles(&mut t, "Lightning Bolt");
    // P1 makes P0 skip all combat phases of their next turn.
    t.advance_to(P1, Step::PrecombatMain);
    let ruse = t.hand(P1, "Empty City Ruse");
    mana(&mut t, P1);
    t.cast_with(P1, ruse, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    mana(&mut t, P0);
    assert!(may_cast(&mut t, P0, bolt), "still P0's next turn");
    t.advance_to(P1, Step::Upkeep);
    mana(&mut t, P0);
    assert!(!may_cast(&mut t, P0, bolt));
}

#[test]
fn niko_shards_copy_the_creature_until_the_next_end_step_then_it_returns() {
    cr!("500.4", "603.7a", "707.2");
    ruling!(
        "Niko, Light of Hope",
        "they'll use the copiable values of the creature as it last existed on the battlefield"
    );
    supported("Niko, Light of Hope");
    let mut t = TestGame::new(2);
    let niko = t.enter(P0, "Niko, Light of Hope");
    t.resolve_all();
    let shards: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).chars.name.as_str() == "Shard Token")
        .collect();
    assert_eq!(shards.len(), 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize]
        .counters
        .insert("+1/+1".into(), 1);
    t.g.objects[niko.0 as usize].summoning_sick = false;
    mana(&mut t, P0);
    t.activate(P0, niko, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Exile);
    for s in &shards {
        assert_eq!(t.obj_now(*s).chars.name.as_str(), "Grizzly Bears");
        // Copiable values only: not the counter.
        assert_eq!(t.pt(*s), (2, 2));
    }
    // The next end step is this turn's. As it begins, the copy effect ends; then the
    // delayed trigger returns the Bears.
    t.advance_to(P0, Step::End);
    for s in &shards {
        assert_eq!(t.obj_now(*s).chars.name.as_str(), "Shard Token");
    }
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Once: nothing more happens at the following end step.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    for s in &shards {
        assert_eq!(t.obj_now(*s).chars.name.as_str(), "Shard Token");
    }
}

#[test]
fn niko_activated_during_an_end_step_lasts_until_the_next_turns_end_step() {
    cr!("500.4");
    let mut t = TestGame::new(2);
    let niko = t.enter(P0, "Niko, Light of Hope");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[niko.0 as usize].summoning_sick = false;
    t.advance_to(P0, Step::End);
    mana(&mut t, P0);
    t.activate(P0, niko, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    let shard = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|o| t.g.obj(*o).chars.name.as_str() == "Grizzly Bears")
        .expect("a Shard copying the Bears");
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.obj_now(shard).chars.name.as_str(), "Grizzly Bears");
    t.advance_to(P1, Step::End);
    assert_eq!(t.obj_now(shard).chars.name.as_str(), "Shard Token");
}

#[test]
fn calix_exiles_until_the_target_enchantment_leaves() {
    cr!("610.3", "610.3c");
    supported("Calix, Destiny's Hand");
    let mut t = TestGame::new(2);
    let calix = t.battlefield(P0, "Calix, Destiny's Hand");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, calix, 1, &[Entity::Object(bears), Entity::Object(anthem)])
        .unwrap();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    // Calix leaving doesn't matter.
    t.g.destroy(calix, None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    t.g.destroy(anthem, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    // Under its owner's control.
    assert_eq!(t.obj_now(back[0]).controller, P1);
}

#[test]
fn calix_exiles_nothing_if_the_enchantment_left_first() {
    cr!("610.3a");
    ruling!(
        "Calix, Destiny's Hand",
        "the target permanent you don’t control won’t be exiled"
    );
    let mut t = TestGame::new(2);
    let calix = t.battlefield(P0, "Calix, Destiny's Hand");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, calix, 1, &[Entity::Object(bears), Entity::Object(anthem)])
        .unwrap();
    // In response, the enchantment leaves.
    t.g.destroy(anthem, None);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}
