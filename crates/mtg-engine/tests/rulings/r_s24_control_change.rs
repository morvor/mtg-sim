//! Rulings batch S24 — control-changing effects (CR 613.1b, 301.5d, 303.4e): gaining
//! control of a permanent doesn't give you control of what's attached to it, Auras you put
//! on others' permanents stay yours, control effects end when their player leaves the
//! game, effects stay on a permanent across control changes, and "activate only once each
//! turn" counts activations by any player.

use crate::r_s01_common::supported;
use crate::r_s02_common::can_activate;
use crate::r_s06_common::{activate_containing, attach_new, attached_to, give_control};
use crate::r_s24_common::*;
use mtg_engine::decision::Action;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn gaining_control_of_a_creature_doesnt_give_you_its_auras_or_equipment() {
    cr!("613.1b", "301.5d", "303.4e");
    ruling!(
        "Act of Treason",
        "Gaining control of a creature doesn't cause you to gain control of any Auras or Equipment attached to it."
    );
    supported("Act of Treason");
    // P1's Bears wears P1's Bonesplitter ("Equipped creature gets +2/+0. Equip {1}") and
    // P1's Firebreathing ("{R}: Enchanted creature gets +1/+0 until end of turn.").
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let splitter = attach_new(&mut t, P1, "Bonesplitter", bears);
    let firebreathing = attach_new(&mut t, P1, "Firebreathing", bears);
    t.lands(P0, "Mountain", 4);
    t.lands(P1, "Mountain", 1);
    let treason = t.hand(P0, "Act of Treason");
    t.cast(P0, treason).target(bears).go();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    // The Aura and the Equipment are still P1's, and still attached.
    assert_eq!(controller(&mut t, splitter), P1);
    assert_eq!(controller(&mut t, firebreathing), P1);
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(bears)));
    assert_eq!(attached_to(&t, firebreathing), Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (4, 2));
    // Only P1 may activate their abilities.
    assert!(!can_activate(&mut t, P0, firebreathing));
    assert!(can_activate(&mut t, P1, firebreathing));
    activate_containing(&mut t, P1, firebreathing, "+1/+0").expect("P1 pumps it");
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 2));
}

#[test]
fn exchanging_control_of_creatures_doesnt_exchange_their_equipment() {
    cr!("301.5d", "701.12a");
    ruling!(
        "Switcheroo",
        "Gaining control of a creature doesn’t cause you to gain control of any Auras or Equipment attached to it."
    );
    supported("Switcheroo");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 6);
    let switcheroo = t.hand(P0, "Switcheroo");
    t.cast(P0, switcheroo)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P1);
    assert_eq!(controller(&mut t, giant), P0);
    // P0 still controls the Bonesplitter on P1's new Bears, and may move it.
    assert_eq!(controller(&mut t, splitter), P0);
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (4, 2));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, splitter, "Equip").expect("equip the Giant");
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(giant)));
    assert_eq!(t.pt(giant), (5, 3));
}

#[test]
fn a_stolen_creatures_auras_and_equipment_keep_working_for_their_controller() {
    cr!("303.4e", "301.5d", "702.6a", "109.5");
    ruling!(
        "Lay Claim",
        "They’ll remain attached, but an Aura’s effect that affects “you” still affects its controller rather than you, the controller of an Equipment can move it during their next main phase, and so on."
    );
    supported("Lay Claim");
    supported("All That Glitters");
    // P1's Bears wears P1's Bonesplitter and P1's All That Glitters ("Enchanted creature
    // gets +1/+1 for each artifact and/or enchantment you control").
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let splitter = attach_new(&mut t, P1, "Bonesplitter", bears);
    let glitters = attach_new(&mut t, P1, "All That Glitters", bears);
    // Bonesplitter and All That Glitters: +2/+2 and +2/+0.
    assert_eq!(t.pt(bears), (6, 4));
    // P0 has more artifacts; they don't count for P1's Aura.
    t.lands(P0, "Sol Ring", 3);
    t.lands(P0, "Island", 7);
    let claim = t.hand(P0, "Lay Claim");
    t.cast(P0, claim).target(bears).go();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    assert_eq!(controller(&mut t, splitter), P1);
    assert_eq!(controller(&mut t, glitters), P1);
    assert_eq!(t.pt(bears), (6, 4));
    // During P1's main phase, P1 moves the Bonesplitter to one of their creatures.
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Mountain", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P1, &[Entity::Object(giant)]);
    activate_containing(&mut t, P1, splitter, "Equip").expect("P1 equips the Giant");
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(giant)));
    // P1's Aura still counts P1's two permanents (not P0's Sol Rings and Lay Claim).
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn exchanging_control_of_permanents_doesnt_exchange_what_is_attached() {
    cr!("701.12a", "303.4e");
    ruling!(
        "Shifting Grift",
        "Gaining control of a permanent doesn’t cause you to gain control of any Auras or Equipment attached to it."
    );
    supported("Shifting Grift");
    // Spree: "+ {2} — Exchange control of two target creatures."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rancor = attach_new(&mut t, P0, "Rancor", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 4);
    let grift = t.hand(P0, "Shifting Grift");
    t.cast(P0, grift)
        .modes(&[0])
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P1);
    assert_eq!(controller(&mut t, giant), P0);
    assert_eq!(controller(&mut t, rancor), P0);
    assert_eq!(attached_to(&t, rancor), Some(Entity::Object(bears)));
}

#[test]
fn you_control_an_aura_you_put_on_another_players_permanent() {
    cr!("303.4e", "109.5");
    ruling!(
        "All That Glitters",
        "You still control Auras that you put onto the battlefield attached to a permanent you don't control."
    );
    // "Enchanted creature gets +1/+1 for each artifact and/or enchantment you control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Sol Ring");
    t.battlefield(P1, "Sol Ring");
    t.battlefield(P0, "Sol Ring");
    t.lands(P0, "Plains", 2);
    let glitters = t.hand(P0, "All That Glitters");
    t.cast(P0, glitters).target(bears).go();
    t.resolve_all();
    let aura = t.g.current(glitters);
    assert_eq!(controller(&mut t, aura), P0);
    // P0's Sol Ring and the Aura itself: +2/+2.
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn a_player_leaving_takes_their_cards_and_ends_their_control_effects() {
    cr!("800.4a");
    ruling!(
        "Mass Manipulation",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well, and any effects that give the player control of permanents immediately end."
    );
    // "Gain control of X target creatures and/or planeswalkers."
    let mut t = TestGame::with_config(3, GameConfig::default());
    let bears = t.battlefield(P2, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    give_control(&mut t, giant, P1);
    t.lands(P0, "Island", 6);
    let manipulation = t.hand(P0, "Mass Manipulation");
    t.cast(P0, manipulation)
        .x(1)
        .target(Entity::Object(bears))
        .go();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    t.g.turn.priority = Some(P0);
    t.g.take_action(P0, Action::Concede);
    t.g.recompute();
    assert!(t.has_lost(P0));
    // The Bears is P2's again, and P0's Hill Giant left the game with P0.
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P2);
    assert!(!t.on_battlefield(giant));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}

#[test]
fn a_doesnt_untap_effect_follows_the_creature_to_a_new_controller() {
    cr!("502.3", "611.2c");
    ruling!(
        "Icefall Regent",
        "The ability stopping the creature from untapping will continue to apply to it even if the creature changes controllers."
    );
    // "When this creature enters, tap target creature an opponent controls. That creature
    // doesn't untap during its controller's untap step for as long as you control this
    // creature."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Icefall Regent");
    t.resolve_all();
    assert!(tapped(&t, bears));
    // P0 gains control of the Bears: it doesn't untap during P0's untap step either.
    give_control(&mut t, bears, P0);
    let lions = t.battlefield(P0, "Savannah Lions");
    t.g.objects[lions.0 as usize].tapped = true;
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!tapped(&t, lions));
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(tapped(&t, bears));
}

#[test]
fn a_stolen_permanent_that_phased_out_phases_in_under_its_owner_at_your_untap_step() {
    cr!("702.26a", "702.26f");
    ruling!(
        "Teferi's Protection",
        "If you gain control of another player's permanent and it phases out, if the duration of the control-change effect expires before it phases in, that permanent phases in under that other player's control as your next untap step begins."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Plains", 3);
    let treason = t.hand(P0, "Act of Treason");
    t.cast(P0, treason).target(bears).go();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    // "All permanents you control phase out."
    let protection = t.hand(P0, "Teferi's Protection");
    t.cast(P0, protection).go();
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
    // The control effect ends this turn; the Bears stays phased out during P1's turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).phased_out);
    // It phases in as P0's next untap step begins, under P1's control.
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(bears).phased_out);
    assert_eq!(controller(&mut t, bears), P1);
}

#[test]
fn once_each_turn_counts_activations_by_a_previous_controller() {
    cr!("602.5b");
    ruling!(
        "Rootwalla",
        "If this card's ability is activated by one player, then another player takes control of it on the same turn, the second player can't activate its ability that turn."
    );
    supported("Rootwalla");
    // "{1}{G}: This creature gets +2/+2 until end of turn. Activate only once each turn."
    let mut t = TestGame::new(2);
    let rootwalla = t.battlefield(P0, "Rootwalla");
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Forest", 4);
    t.activate(P0, rootwalla, 0, &[]).expect("P0 activates it");
    t.resolve_all();
    assert_eq!(t.pt(rootwalla), (4, 4));
    give_control(&mut t, rootwalla, P1);
    assert!(!can_activate(&mut t, P1, rootwalla));
    assert!(t.activate(P1, rootwalla, 0, &[]).is_err());
    // Next turn, it can.
    t.advance_to(P1, Step::Upkeep);
    assert!(can_activate(&mut t, P1, rootwalla));
}
